use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;

pub fn init(app_dir: &PathBuf) -> Result<Connection> {
    if !app_dir.exists() {
        fs::create_dir_all(app_dir).expect("Failed to create app dir");
    }

    let db_path = app_dir.join("monclips.db");
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            pinned INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )?;

    Ok(conn)
}
