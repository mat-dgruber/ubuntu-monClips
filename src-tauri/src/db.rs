use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct ClipItem {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
    pub pinned: bool,
}

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

pub fn insert_item(conn: &Connection, content: &str) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    conn.execute(
        "INSERT INTO clipboard_items (content, created_at, pinned) VALUES (?1, ?2, 0)",
        params![content, now],
    )?;
    Ok(())
}

pub fn get_items(conn: &Connection, query: Option<String>) -> Result<Vec<ClipItem>> {
    let mut sql = String::from("SELECT id, content, created_at, pinned FROM clipboard_items");
    let mut stmt;

    if let Some(q) = query {
        if !q.is_empty() {
            sql.push_str(" WHERE content LIKE ?1");
            sql.push_str(" ORDER BY pinned DESC, created_at DESC");
            stmt = conn.prepare(&sql)?;
            let search_term = format!("%{}%", q);

            let items = stmt.query_map(params![search_term], |row| {
                Ok(ClipItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    created_at: row.get(2)?,
                    pinned: row.get::<_, i64>(3)? == 1,
                })
            })?.filter_map(Result::ok).collect();
            return Ok(items);
        }
    }

    sql.push_str(" ORDER BY pinned DESC, created_at DESC");
    stmt = conn.prepare(&sql)?;

    let items = stmt.query_map([], |row| {
        Ok(ClipItem {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            pinned: row.get::<_, i64>(3)? == 1,
        })
    })?.filter_map(Result::ok).collect();

    Ok(items)
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard_items SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn cleanup_expired(conn: &Connection) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let twenty_four_hours = 24 * 60 * 60;
    let cutoff = now - twenty_four_hours;

    conn.execute(
        "DELETE FROM clipboard_items WHERE pinned = 0 AND created_at < ?1",
        params![cutoff],
    )?;
    Ok(())
}
