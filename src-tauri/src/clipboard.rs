use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager};
use std::time::Duration;

struct Handler {
    app: AppHandle,
    last_content: Arc<Mutex<String>>,
}

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        // Debounce/delay slightly to let the clipboard settle
        thread::sleep(Duration::from_millis(50));

        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return CallbackResult::Next,
        };

        if let Ok(text) = clipboard.get_text() {
            let text = text.trim().to_string();
            if text.is_empty() {
                return CallbackResult::Next;
            }

            let mut last = self.last_content.lock().unwrap();
            if *last != text {
                *last = text.clone();

                // We need to access DB to insert
                let app_clone = self.app.clone();
                let text_clone = text.clone();

                tauri::async_runtime::spawn(async move {
                    let state = app_clone.state::<crate::AppState>();
                    let conn = state.db.lock().unwrap();
                    if let Ok(_) = crate::db::insert_item(&conn, &text_clone) {
                        // In Tauri v2, emitting events uses Emitter trait which is in scope via tauri::Manager
                        use tauri::Emitter;
                        let _ = app_clone.emit("clipboard_updated", ());
                    }
                });
            }
        }
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, _error: std::io::Error) -> CallbackResult {
        CallbackResult::Next
    }
}

pub fn spawn_monitor(app: AppHandle) {
    let last_content = Arc::new(Mutex::new(String::new()));

    // Initial read to set last_content
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            *last_content.lock().unwrap() = text.trim().to_string();
        }
    }

    thread::spawn(move || {
        let handler = Handler { app, last_content };
        if let Ok(mut master) = Master::new(handler) {
            let _ = master.run();
        }
    });
}
