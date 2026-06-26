use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;

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

            INSERT OR IGNORE INTO bulb_state (id, is_on) VALUES (1, 0);",
        )?;
        Ok(())
    }

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
}
