//! Cron runner — keeps the in-memory cron scheduler in step with the
//! `schedules` table.
//!
//! Exactly one `ScheduleRunner` exists per process: `main` builds it, stores it
//! in [`crate::state::AppState`], and every request borrows it through an
//! `Arc`. Handlers must never build their own — a `JobScheduler` dropped at the
//! end of a request takes all of its jobs with it, so the schedule would be
//! persisted but never fire.
//!
//! The database is the source of truth; the jobs are a cache of it that lives
//! only in memory. [`ScheduleRunner::load_from_db`] rebuilds that cache at
//! startup, which is what makes a restart or a crash invisible.
//!
//! # Timezone
//!
//! Cron expressions are read in one process-wide timezone, taken from
//! `SCHEDULE_TZ` and defaulting to `Asia/Jakarta`. So `0 53 13 * * *` means
//! 13:53 local, not 13:53 UTC — which is what you almost always mean when
//! scheduling a light. Stored timestamps remain `TIMESTAMPTZ` (UTC); only the
//! cron fields are read locally.

use std::collections::HashMap;
use std::sync::Arc;

use chrono_tz::Tz;
use deadpool_postgres::Pool;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use super::dto::{Action, Schedule};
use super::repo;
use crate::api::bulb::repo as bulb_repo;
use crate::error::AppError;

/// Env var naming the timezone every cron expression is read in.
const TZ_ENV: &str = "SCHEDULE_TZ";

/// Used when `SCHEDULE_TZ` is unset. Fixed UTC+07:00, no DST.
const DEFAULT_TZ: Tz = Tz::Asia__Jakarta;

pub struct ScheduleRunner {
    scheduler: JobScheduler,
    pool: Pool,
    /// The timezone cron expressions are read in. Resolved once at startup so
    /// every job agrees, and so a typo in the env var stops the process at boot
    /// rather than silently scheduling things in the wrong zone.
    tz: Tz,
    /// `schedules.id` → the id `tokio-cron-scheduler` handed back for that job.
    /// We need this mapping because the two ids are unrelated, and removing a
    /// job requires the scheduler's own id.
    ///
    /// A `tokio::sync::Mutex`, not a `std::sync::Mutex`: its guard is `Send`,
    /// so it can be held across an `.await`. That is what makes "remove the old
    /// job, then add the new one" a single atomic step — two concurrent
    /// requests for the same schedule cannot interleave and leave a stale job
    /// firing.
    jobs: Mutex<HashMap<Uuid, Uuid>>,
}

impl ScheduleRunner {
    /// The timezone in effect, for logging.
    pub fn timezone(&self) -> Tz {
        self.tz
    }

    /// Read `SCHEDULE_TZ`, falling back to [`DEFAULT_TZ`].
    ///
    /// An unrecognised name is an error rather than a fallback: quietly running
    /// every schedule in the wrong timezone is the exact bug this feature
    /// exists to prevent.
    fn resolve_tz() -> Result<Tz, AppError> {
        match std::env::var(TZ_ENV) {
            Err(_) => Ok(DEFAULT_TZ),
            Ok(name) => name.parse().map_err(|_| {
                AppError::Internal(format!(
                    "{TZ_ENV}={name:?} is not an IANA timezone name \
                     (expected e.g. \"Asia/Jakarta\")"
                ))
            }),
        }
    }

    /// Build the runner and start its background tick loop.
    pub async fn start(pool: Pool) -> Result<Arc<Self>, AppError> {
        let tz = Self::resolve_tz()?;
        let scheduler = JobScheduler::new().await?;
        // Spawns a Tokio task that ticks every 500ms. Without this call, jobs
        // are registered but nothing ever runs them.
        scheduler.start().await?;

        Ok(Arc::new(Self {
            scheduler,
            pool,
            tz,
            jobs: Mutex::new(HashMap::new()),
        }))
    }

    /// Install a job for every enabled schedule in the database.
    /// Returns how many jobs were installed.
    pub async fn load_from_db(&self) -> Result<usize, AppError> {
        let schedules = repo::list_enabled(&self.pool).await?;
        for schedule in &schedules {
            self.sync(schedule).await?;
        }
        Ok(schedules.len())
    }

    /// Make the scheduler match this row: install its job, or drop the job if
    /// the schedule is disabled.
    ///
    /// Safe to call repeatedly. Any previous job for the same schedule id is
    /// removed first, so an update can never leave a second job firing on the
    /// old cron expression.
    pub async fn sync(&self, schedule: &Schedule) -> Result<(), AppError> {
        let mut jobs = self.jobs.lock().await;

        if let Some(previous) = jobs.remove(&schedule.id) {
            self.scheduler.remove(&previous).await?;
        }

        if !schedule.enabled {
            return Ok(());
        }

        let job_id = self.scheduler.add(self.build_job(schedule)?).await?;
        jobs.insert(schedule.id, job_id);
        Ok(())
    }

    /// Stop and forget a schedule's job. A schedule with no job is a no-op.
    pub async fn remove(&self, schedule_id: Uuid) -> Result<(), AppError> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job_id) = jobs.remove(&schedule_id) {
            self.scheduler.remove(&job_id).await?;
        }
        Ok(())
    }

    /// Check a cron expression without installing anything.
    ///
    /// Call this *before* writing to the database, so a typo comes back as a
    /// 400 instead of persisting a row we could never schedule. Building a
    /// throwaway job is deliberate: it runs the exact same parser [`sync`] will
    /// use, so validation and execution can never drift apart.
    ///
    /// No timezone needed here — the timezone changes *when* an expression
    /// fires, never whether it parses.
    ///
    /// [`sync`]: ScheduleRunner::sync
    pub fn validate_cron(cron_expr: &str) -> Result<(), AppError> {
        Job::new_async(cron_expr, |_id, _scheduler| Box::pin(async {}))?;
        Ok(())
    }

    fn build_job(&self, schedule: &Schedule) -> Result<Job, AppError> {
        // Parsed once, when the job is built, rather than on every tick: a bad
        // action becomes an error the caller can report instead of a surprise
        // at 3am. The DB CHECK constraint makes this unreachable in practice.
        let is_on = Action::parse(&schedule.action)?.is_on();

        let pool = self.pool.clone();
        let name = schedule.name.clone();
        let id = schedule.id;

        // `new_async_tz`, not `new_async`: the latter reads the expression as
        // UTC. Caveat worth knowing — the library converts `tz` into a fixed
        // UTC offset once, here, and reuses it for every later tick. For a
        // fixed-offset zone like Asia/Jakarta (UTC+7, no DST) that is exactly
        // right; under a DST-observing zone a job would drift by an hour after
        // a transition, until the process restarts and rebuilds it.
        let job = Job::new_async_tz(&schedule.cron_expr, self.tz, move |_job_id, _scheduler| {
            // Cloned per tick, not once. The closure is `FnMut` — it runs again
            // on the next tick — but the `async move` block below takes
            // ownership of everything it captures, so it has to be handed
            // fresh clones each time.
            let pool = pool.clone();
            let name = name.clone();

            Box::pin(async move {
                match bulb_repo::set(&pool, is_on).await {
                    Ok(state) => {
                        tracing::info!(%id, %name, is_on = state.is_on, "schedule fired")
                    }
                    // A failed tick must never propagate: the scheduler has
                    // nowhere to return an error to. Log it and let the next
                    // tick try again.
                    Err(err) => tracing::error!(%id, %name, "schedule failed: {err:?}"),
                }
            })
        })?;

        Ok(job)
    }
}
