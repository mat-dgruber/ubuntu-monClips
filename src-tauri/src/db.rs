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
    pub category: String,
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
            pinned INTEGER NOT NULL DEFAULT 0,
            category TEXT NOT NULL DEFAULT 'Text'
        )",
        (),
    )?;

    // Lightweight migration for existing databases
    let _ = conn.execute("ALTER TABLE clipboard_items ADD COLUMN category TEXT NOT NULL DEFAULT 'Text'", ());

    // Optimization: Add indices for faster search and sorting
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clipboard_content ON clipboard_items (content)",
        (),
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clipboard_pinned_created ON clipboard_items (pinned, created_at)",
        (),
    )?;

    Ok(conn)
}

pub fn detect_category(content: &str) -> &'static str {
    let content = content.trim();
    if content.is_empty() {
        return "Text";
    }

    // URL detection
    if content.starts_with("http://") || content.starts_with("https://") {
        return "URL";
    }

    // Color detection (hex)
    if content.starts_with('#') && (content.len() == 4 || content.len() == 7 || content.len() == 9) && content[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return "Color";
    }

    // Markdown detection
    if content.starts_with("# ") || content.starts_with("## ") || content.starts_with("### ") 
        || (content.contains("```") && content.len() > 6)
        || (content.contains("* ") || content.contains("- ")) && content.contains('\n')
        || (content.contains("[") && content.contains("](") && content.contains(")"))
    {
        return "Markdown";
    }
    
    // Improved code detection heuristics
    let code_keywords = [
        "fn ", "function ", "const ", "let ", "class ", "import ", 
        "pub ", "use ", "static ", "struct ", "enum ", "interface ", 
        "include ", "#include", "std::", "package ", "public class ", 
        "private ", "protected ", "void ", "int ", "bool ", "float ", 
        "async ", "await ", "return ", "if (", "while (", "for (", 
        "printf(", "println!", "console.log(", "fmt.Println(",
        "<?php", "module.exports", "export default"
    ];

    // Structural indicators of code
    let has_braces = content.contains('{') && content.contains('}');
    let has_semicolon_end = content.ends_with(';') || content.contains(");") || content.contains("];");
    let has_arrow = content.contains("=>") || content.contains("->");
    
    // Count how many keywords we find
    let keyword_count = code_keywords.iter().filter(|&&k| content.contains(k)).count();

    if (keyword_count >= 2) 
        || (keyword_count >= 1 && (has_braces || has_semicolon_end || has_arrow))
        || (has_braces && has_semicolon_end)
        || content.contains("```") // Redundant but safe
    {
        return "Code";
    }
    
    "Text"
}

pub fn insert_item(conn: &Connection, content: &str) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let category = detect_category(content);
    
    // Check if item already exists
    let mut stmt = conn.prepare("SELECT id FROM clipboard_items WHERE content = ?1")?;
    let mut rows = stmt.query(params![content])?;
    
    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        // Update timestamp and category of existing item
        conn.execute(
            "UPDATE clipboard_items SET created_at = ?1, category = ?2 WHERE id = ?3",
            params![now, category, id],
        )?;
    } else {
        // Insert new item
        conn.execute(
            "INSERT INTO clipboard_items (content, created_at, pinned, category) VALUES (?1, ?2, 0, ?3)",
            params![content, now, category],
        )?;
    }
    Ok(())
}

pub fn get_items(conn: &Connection, query: Option<String>, limit: i64, offset: i64) -> Result<Vec<ClipItem>> {
    let mut sql = String::from("SELECT id, content, created_at, pinned, category FROM clipboard_items");
    let mut stmt;

    if let Some(q) = query {
        if !q.is_empty() {
            sql.push_str(" WHERE content LIKE ?1");
            sql.push_str(" ORDER BY pinned DESC, created_at DESC LIMIT ?2 OFFSET ?3");
            stmt = conn.prepare(&sql)?;
            let search_term = format!("%{}%", q);

            let items = stmt.query_map(params![search_term, limit, offset], |row| {
                Ok(ClipItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    created_at: row.get(2)?,
                    pinned: row.get::<_, i64>(3)? == 1,
                    category: row.get(4)?,
                })
            })?.filter_map(Result::ok).collect();
            return Ok(items);
        }
    }

    sql.push_str(" ORDER BY pinned DESC, created_at DESC LIMIT ?1 OFFSET ?2");
    stmt = conn.prepare(&sql)?;

    let items = stmt.query_map(params![limit, offset], |row| {
        Ok(ClipItem {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            pinned: row.get::<_, i64>(3)? == 1,
            category: row.get(4)?,
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
