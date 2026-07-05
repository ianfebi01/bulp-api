use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing;

use crate::db::Db;
use crate::models::Schedule;

/// Manages cron jobs that turn the bulb on/off according to schedules stored
/// in the database.  Keeps a map of schedule-DB-id → cron-job-UUID so jobs can
/// be removed or replaced when a schedule is updated or deleted.
pub struct ScheduleRunner {
    scheduler: JobScheduler,
    /// schedule DB id → cron job UUID
    jobs: Mutex<HashMap<String, uuid::Uuid>>,
    db: Arc<Db>,
}

impl ScheduleRunner {
    /// Create a new runner (does **not** start the scheduler yet).
    pub async fn new(db: Arc<Db>) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            scheduler,
            jobs: Mutex::new(HashMap::new()),
            db,
        })
    }

    /// Start the background scheduler loop.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.start().await?;
        Ok(())
    }

    /// Register a cron job for the given schedule.  If the schedule is
    /// disabled this is a no-op.
    pub async fn add_job(&self, schedule: &Schedule) -> Result<(), Box<dyn std::error::Error>> {
        if !schedule.enabled {
            return Ok(());
        }

        let action_on = schedule.action == "on";
        let db = Arc::clone(&self.db);
        let schedule_name = schedule.name.clone();
        let schedule_id = schedule.id.clone();

        let job = Job::new_async(schedule.cron_expr.as_str(), move |_uuid, _lock| {
            let db = Arc::clone(&db);
            let name = schedule_name.clone();
            let sid = schedule_id.clone();
            Box::pin(async move {
                let label = if action_on { "ON" } else { "OFF" };
                tracing::info!(schedule_id = %sid, name = %name, "⏰ Cron fired — turning bulb {}", label);
                match db.set_state(action_on) {
                    Ok((is_on, updated_at)) => {
                        tracing::info!(is_on, updated_at = %updated_at, "Bulb state updated");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to set bulb state from cron job");
                    }
                }
            })
        })?;

        let job_id = job.guid();
        self.scheduler.add(job).await?;

        let mut map = self.jobs.lock().await;
        map.insert(schedule.id.clone(), job_id);

        tracing::info!(
            schedule_id = %schedule.id,
            name = %schedule.name,
            cron = %schedule.cron_expr,
            action = %schedule.action,
            "Registered cron job"
        );

        Ok(())
    }

    /// Remove the cron job associated with the given schedule DB id (if any).
    pub async fn remove_job(&self, schedule_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut map = self.jobs.lock().await;
        if let Some(job_id) = map.remove(schedule_id) {
            self.scheduler.remove(&job_id).await?;
            tracing::info!(schedule_id, "Removed cron job");
        }
        Ok(())
    }

    /// Replace the cron job for a schedule (remove old, add new).
    pub async fn reload_job(&self, schedule: &Schedule) -> Result<(), Box<dyn std::error::Error>> {
        self.remove_job(&schedule.id).await?;
        self.add_job(schedule).await?;
        Ok(())
    }

    /// Load all enabled schedules from the database and register cron jobs.
    /// This is the **crash-recovery** entry point — called once on startup.
    pub async fn load_all_from_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let schedules = self.db.get_enabled_schedules()?;
        tracing::info!(count = schedules.len(), "Loading schedules from database");
        for s in &schedules {
            if let Err(e) = self.add_job(s).await {
                tracing::error!(
                    schedule_id = %s.id,
                    name = %s.name,
                    error = %e,
                    "Failed to register schedule — skipping"
                );
            }
        }
        Ok(())
    }
}
