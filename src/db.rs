use rusqlite::{params, Connection, Result as SqlResult};
use std::sync::Mutex;
use uuid::Uuid;

use crate::models::Schedule;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Open (or create) the SQLite database and ensure the schema exists.
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        db.seed_schedules()?;
        Ok(db)
    }

    fn migrate(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS bulb_state (
                id          INTEGER PRIMARY KEY CHECK (id = 1),
                is_on       INTEGER NOT NULL DEFAULT 0,
                updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            INSERT OR IGNORE INTO bulb_state (id, is_on) VALUES (1, 0);

            CREATE TABLE IF NOT EXISTS schedules (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                cron_expr  TEXT NOT NULL,
                action     TEXT NOT NULL CHECK (action IN ('on', 'off')),
                enabled    INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    /// Insert default schedules if the table is empty.
    /// - ON  at 17:40 WIB (UTC+7) → 10:40 UTC → cron `0 40 10 * * *`
    /// - OFF at 05:00 WIB (UTC+7) → 22:00 UTC → cron `0 0 22 * * *`
    fn seed_schedules(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM schedules", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(());
        }

        let seeds = [
            (
                Uuid::new_v4().to_string(),
                "Evening ON (17:40 WIB)",
                "0 40 10 * * *",
                "on",
            ),
            (
                Uuid::new_v4().to_string(),
                "Morning OFF (05:00 WIB)",
                "0 0 22 * * *",
                "off",
            ),
        ];

        for (id, name, cron_expr, action) in &seeds {
            conn.execute(
                "INSERT INTO schedules (id, name, cron_expr, action) VALUES (?1, ?2, ?3, ?4)",
                params![id, name, cron_expr, action],
            )?;
        }
        Ok(())
    }

    // ── Bulb state ──────────────────────────────────────────────────

    /// Returns (is_on, updated_at).
    pub fn get_state(&self) -> SqlResult<(bool, String)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT is_on, updated_at FROM bulb_state WHERE id = 1")?;
        stmt.query_row([], |row| {
            let is_on: i32 = row.get(0)?;
            let updated_at: String = row.get(1)?;
            Ok((is_on != 0, updated_at))
        })
    }

    /// Set the bulb on (true) or off (false). Returns the new state.
    pub fn set_state(&self, on: bool) -> SqlResult<(bool, String)> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE bulb_state SET is_on = ?, updated_at = datetime('now') WHERE id = 1",
            [on as i32],
        )?;
        drop(conn);
        self.get_state()
    }

    // ── Schedule CRUD ───────────────────────────────────────────────

    pub fn create_schedule(
        &self,
        name: &str,
        cron_expr: &str,
        action: &str,
    ) -> SqlResult<Schedule> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO schedules (id, name, cron_expr, action) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, cron_expr, action],
        )?;
        drop(conn);
        self.get_schedule(&id)
    }

    pub fn get_schedule(&self, id: &str) -> SqlResult<Schedule> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, cron_expr, action, enabled, created_at, updated_at
             FROM schedules WHERE id = ?1",
            params![id],
            |row| {
                let enabled_int: i32 = row.get(4)?;
                Ok(Schedule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cron_expr: row.get(2)?,
                    action: row.get(3)?,
                    enabled: enabled_int != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
    }

    pub fn list_schedules(&self) -> SqlResult<Vec<Schedule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, cron_expr, action, enabled, created_at, updated_at
             FROM schedules ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let enabled_int: i32 = row.get(4)?;
            Ok(Schedule {
                id: row.get(0)?,
                name: row.get(1)?,
                cron_expr: row.get(2)?,
                action: row.get(3)?,
                enabled: enabled_int != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_schedule(
        &self,
        id: &str,
        name: Option<&str>,
        cron_expr: Option<&str>,
        action: Option<&str>,
        enabled: Option<bool>,
    ) -> SqlResult<Schedule> {
        let conn = self.conn.lock().unwrap();

        if let Some(v) = name {
            conn.execute(
                "UPDATE schedules SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![v, id],
            )?;
        }
        if let Some(v) = cron_expr {
            conn.execute(
                "UPDATE schedules SET cron_expr = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![v, id],
            )?;
        }
        if let Some(v) = action {
            conn.execute(
                "UPDATE schedules SET action = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![v, id],
            )?;
        }
        if let Some(v) = enabled {
            conn.execute(
                "UPDATE schedules SET enabled = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![v as i32, id],
            )?;
        }

        drop(conn);
        self.get_schedule(id)
    }

    pub fn delete_schedule(&self, id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Return all enabled schedules — used at startup for crash recovery.
    pub fn get_enabled_schedules(&self) -> SqlResult<Vec<Schedule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, cron_expr, action, enabled, created_at, updated_at
             FROM schedules WHERE enabled = 1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let enabled_int: i32 = row.get(4)?;
            Ok(Schedule {
                id: row.get(0)?,
                name: row.get(1)?,
                cron_expr: row.get(2)?,
                action: row.get(3)?,
                enabled: enabled_int != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }
}
