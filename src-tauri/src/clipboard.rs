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
        let app_handle = self.app.clone();
        let last_content_arc = self.last_content.clone();

        // Spawn a task to handle the read without blocking the master thread too much
        tauri::async_runtime::spawn(async move {
            // Give the OS a moment to finish the clipboard write
            tokio::time::sleep(Duration::from_millis(150)).await;

            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(_) => return,
            };

            // Retry up to 3 times with small delays if it fails
            let mut text = None;
            for _ in 0..3 {
                if let Ok(t) = clipboard.get_text() {
                    text = Some(t);
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            if let Some(text) = text {
                let text = text.trim().to_string();
                if text.is_empty() {
                    return;
                }

                let mut last = last_content_arc.lock().unwrap();
                if *last != text {
                    *last = text.clone();

                    let state = app_handle.state::<crate::AppState>();
                    let conn = state.db.lock().unwrap();
                    if let Ok(_) = crate::db::insert_item(&conn, &text) {
                        use tauri::Emitter;
                        let _ = app_handle.emit("clipboard_updated", ());
                    }
                }
            }
        });

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
