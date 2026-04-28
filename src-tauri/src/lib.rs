// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::{Manager, State};

pub mod clipboard;
pub mod db;
use db::ClipItem;

struct AppState {
    db: Mutex<Connection>,
}

#[tauri::command]
fn get_clipboard_items(search_query: Option<String>, state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    let conn = state.db.lock().unwrap();
    db::get_items(&conn, search_query).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_item_pin(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::toggle_pin(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_clipboard_item(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_item(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_to_clipboard(content: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap();
            let db_conn = db::init(&app_dir).expect("Failed to init DB");

            // Clean up expired items on startup
            db::cleanup_expired(&db_conn).expect("Failed to cleanup expired items");

            app.manage(AppState {
                db: Mutex::new(db_conn),
            });

            // Spawn clipboard monitor
            clipboard::spawn_monitor(app.handle().clone());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_clipboard_items,
            toggle_item_pin,
            delete_clipboard_item,
            write_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
