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
fn get_clipboard_items(search_query: Option<String>, limit: i64, offset: i64, state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    let conn = state.db.lock().unwrap();
    db::get_items(&conn, search_query, limit, offset).map_err(|e| e.to_string())
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
async fn write_to_clipboard(content: String) -> Result<(), String> {
    // Retry up to 3 times with small delays
    for i in 0..3 {
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Ok(_) = cb.set_text(content.clone()) {
                    return Ok(());
                }
            }
            Err(e) if i == 2 => return Err(e.to_string()),
            Err(_) => {}
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err("Failed to write to clipboard after multiple attempts".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, _event| {
                let shortcut_str = shortcut.to_string().to_lowercase();
                println!("Shortcut pressed: {}", shortcut_str);
                
                // Be more flexible with the matching
                if shortcut_str.contains("alt") && shortcut_str.contains("c") {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                        println!("Window shown and focused");
                    } else {
                        println!("Main window not found");
                    }
                }
            })
            .build()
        )
        .setup(|app| {
            // Register shortcut
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
            
            // Try different ways to register if one fails or use a more explicit way
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyC);
            match app.global_shortcut().register(shortcut) {
                Ok(_) => println!("Registered Alt+C successfully"),
                Err(e) => println!("Failed to register Alt+C: {}", e),
            }

            let app_dir = app.path().app_data_dir().unwrap();
            let db_conn = db::init(&app_dir).expect("Failed to init DB");

            // Clean up expired items on startup
            db::cleanup_expired(&db_conn).expect("Failed to cleanup expired items");

            app.manage(AppState {
                db: Mutex::new(db_conn),
            });

            // Start monitoring clipboard
            clipboard::spawn_monitor(app.handle().clone());

            // Periodic cleanup every hour
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
                loop {
                    interval.tick().await;
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let conn = state.db.lock().unwrap();
                        let _ = db::cleanup_expired(&conn);
                        // Emit event to refresh UI in case items were deleted
                        use tauri::Emitter;
                        let _ = app_handle.emit("clipboard_updated", ());
                    }
                }
            });

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
