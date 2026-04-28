# monClips Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a desktop clipboard manager with a Rust backend (Tauri) and React/TypeScript frontend.

**Architecture:** "Thick backend" where Rust manages the SQLite database, OS clipboard events, and a TTL cleanup routine. React frontend acts as a view layer reacting to Tauri events.

**Tech Stack:** Tauri, Rust, React, TypeScript, Vite, Tailwind CSS, shadcn/ui, rusqlite, arboard, clipboard-master.

---

### Task 1: Project Scaffolding & Setup

**Files:**
- Create: `package.json`, `src-tauri/Cargo.toml`, basic Vite + React structure.

- [ ] **Step 1: Scaffold Tauri Project**
Run the create-tauri-app CLI. We will use `pnpm` (or npm/yarn if preferred, assuming `npm` for universal compatibility here, but adapt if needed).

```bash
npm create tauri-app@latest . -- --manager npm --template react-ts --yes
```

- [ ] **Step 2: Install Frontend Dependencies (Tailwind & shadcn)**
```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
npm install lucide-react clsx tailwind-merge
```

- [ ] **Step 3: Configure Tailwind**
Modify `tailwind.config.js`:
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

Modify `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 4: Add Backend Dependencies**
Modify `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.31", features = ["bundled"] }
clipboard-master = "4.0"
arboard = "3.3"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 5: Verify build works**
Run: `npm run tauri build -- --debug` (Wait for it to complete)
Expected: Builds successfully.

- [ ] **Step 6: Commit**
```bash
git add .
git commit -m "chore: scaffold tauri + react project with tailwind and rust deps"
```

---

### Task 2: Database Setup & Model (Rust)

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Define DB initialization**
Create `src-tauri/src/db.rs`:
```rust
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
```

- [ ] **Step 2: Wire up DB in main**
Modify `src-tauri/src/main.rs`:
```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let _db_conn = db::init(&app_dir).expect("Failed to init DB");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Test compilation**
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Passes.

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat(backend): initialize sqlite database"
```

---

### Task 3: Data Access Layer (Rust)

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Define Structs and Queries**
Append to `src-tauri/src/db.rs`:
```rust
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
```

- [ ] **Step 2: Expose Tauri Commands**
Modify `src-tauri/src/main.rs` to manage state and expose commands:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::State;

mod db;
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let db_conn = db::init(&app_dir).expect("Failed to init DB");
            // Run initial cleanup
            db::cleanup_expired(&db_conn).expect("Failed cleanup");
            
            app.manage(AppState {
                db: Mutex::new(db_conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_items,
            toggle_item_pin,
            delete_clipboard_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Test compilation**
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Passes.

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat(backend): add data access layer and tauri commands"
```

---

### Task 4: Clipboard Integration (Rust)

**Files:**
- Create: `src-tauri/src/clipboard.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create Clipboard Listener**
Create `src-tauri/src/clipboard.rs`:
```rust
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager};
use rusqlite::Connection;
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
                        let _ = app_clone.emit_all("clipboard_updated", ());
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
        let mut master = Master::new(handler);
        let _ = master.run();
    });
}
```

- [ ] **Step 2: Add Manual Write & Wire Monitor**
Modify `src-tauri/src/main.rs`:
```rust
// ... existing imports ...
use tauri::Manager;

mod db;
mod clipboard; // Add this
use db::ClipItem;

// ... AppState definition ...

// ... existing commands ...

#[tauri::command]
fn write_to_clipboard(content: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(content).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let db_conn = db::init(&app_dir).expect("Failed to init DB");
            db::cleanup_expired(&db_conn).expect("Failed cleanup");
            
            app.manage(AppState {
                db: Mutex::new(db_conn),
            });
            
            // Spawn monitor
            clipboard::spawn_monitor(app.handle());
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_items,
            toggle_item_pin,
            delete_clipboard_item,
            write_to_clipboard // Add this
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Test compilation**
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Passes.

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/clipboard.rs src-tauri/src/main.rs
git commit -m "feat(backend): implement clipboard monitoring and writing"
```

---

### Task 5: Frontend Core & Hooks

**Files:**
- Create: `src/hooks/useClipboard.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create Types and Hooks**
Create `src/hooks/useClipboard.ts`:
```typescript
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface ClipItem {
  id: number;
  content: string;
  created_at: number;
  pinned: boolean;
}

export function useClipboard() {
  const [items, setItems] = useState<ClipItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');

  const fetchItems = useCallback(async (query?: string) => {
    try {
      const result = await invoke<ClipItem[]>('get_clipboard_items', { 
        searchQuery: query || null 
      });
      setItems(result);
    } catch (e) {
      console.error("Failed to fetch items", e);
    }
  }, []);

  useEffect(() => {
    fetchItems(searchQuery);
  }, [searchQuery, fetchItems]);

  useEffect(() => {
    const unlisten = listen('clipboard_updated', () => {
      fetchItems(searchQuery);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, [searchQuery, fetchItems]);

  const togglePin = async (id: number) => {
    await invoke('toggle_item_pin', { id });
    await fetchItems(searchQuery);
  };

  const deleteItem = async (id: number) => {
    await invoke('delete_clipboard_item', { id });
    await fetchItems(searchQuery);
  };

  const copyToClipboard = async (content: string) => {
    await invoke('write_to_clipboard', { content });
  };
  
  const openUrl = async (url: string) => {
      // Using tauri shell api via dynamic import to avoid setup issues if testing in regular browser
      const { open } = await import('@tauri-apps/api/shell');
      await open(url);
  }

  return {
    items,
    searchQuery,
    setSearchQuery,
    togglePin,
    deleteItem,
    copyToClipboard,
    openUrl
  };
}
```

- [ ] **Step 2: Basic UI Skeleton**
Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import { useClipboard } from './hooks/useClipboard';

function App() {
  const { items, searchQuery, setSearchQuery, togglePin, deleteItem, copyToClipboard, openUrl } = useClipboard();

  const isUrl = (str: string) => /^https?:\/\//i.test(str);

  return (
    <div className="flex flex-col h-screen bg-gray-50 text-gray-900">
      <header className="p-4 bg-white border-b sticky top-0 z-10">
        <input 
          type="text" 
          placeholder="Search clips..." 
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full p-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </header>
      
      <main className="flex-1 overflow-y-auto p-4 space-y-2">
        {items.length === 0 ? (
          <p className="text-center text-gray-500 mt-10">No clips found.</p>
        ) : (
          items.map(item => (
            <div key={item.id} className="group flex items-start p-3 bg-white border rounded-md shadow-sm hover:border-blue-300 relative pr-16">
               <div 
                  className="flex-1 overflow-hidden cursor-pointer"
                  onClick={() => isUrl(item.content) ? openUrl(item.content) : copyToClipboard(item.content)}
                >
                  <p className="whitespace-pre-wrap break-words text-sm">{item.content}</p>
               </div>
               
               <div className="absolute right-2 top-2 flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button 
                    onClick={() => togglePin(item.id)}
                    className={`p-1 rounded hover:bg-gray-100 ${item.pinned ? 'text-blue-500' : 'text-gray-400'}`}
                  >
                    {item.pinned ? '★' : '☆'}
                  </button>
                  <button 
                    onClick={() => deleteItem(item.id)}
                    className="p-1 rounded text-red-400 hover:bg-red-50 hover:text-red-600"
                  >
                    ✕
                  </button>
               </div>
            </div>
          ))
        )}
      </main>
    </div>
  );
}

export default App;
```

- [ ] **Step 3: Verify TypeScript builds**
Run: `npm run tsc` (Add `tsc` script to package.json if missing: `"tsc": "tsc --noEmit"`)
Expected: No errors.

- [ ] **Step 4: Commit**
```bash
git add src/hooks/useClipboard.ts src/App.tsx
git commit -m "feat(frontend): implement react ui and tauri hooks"
```
