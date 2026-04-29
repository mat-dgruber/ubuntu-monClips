# Backend Architecture (Rust/Tauri)

The backend of monClips is written in Rust using the Tauri framework. It follows a "Thick Backend" pattern, meaning all business logic, data persistence, and OS-level integrations are handled securely here, treating the frontend purely as a view layer.

## Core Responsibilities

1. **Database Management (SQLite)**
2. **OS Clipboard Monitoring**
3. **TTL (Time-To-Live) Cleanup**
4. **IPC (Inter-Process Communication) API**

---

## 1. Database Management (`src/db.rs`)

We use the `rusqlite` crate with the `"bundled"` feature to embed SQLite directly into the binary, removing the need for external database dependencies on the host OS.

### Schema: `clipboard_items`
- `id` (INTEGER PRIMARY KEY)
- `content` (TEXT NOT NULL)
- `created_at` (INTEGER NOT NULL): Stored as Unix timestamp.
- `pinned` (INTEGER NOT NULL DEFAULT 0): Handled as a boolean (0 or 1).

### State Management
The database connection is created during Tauri's `setup` hook and wrapped in an `Arc<Mutex<Connection>>` (stored inside `AppState`). This allows thread-safe access from both the background clipboard listener and incoming IPC command requests.

## 2. Clipboard Monitoring (`src/clipboard.rs`)

Unlike basic clipboard managers that use interval polling (which drains battery and CPU), monClips uses OS-level event hooks via the `clipboard-master` crate.

- **Thread Segregation:** A dedicated background thread is spawned on startup (`spawn_monitor`).
- **Event Hook:** It registers a callback with the OS. When the clipboard changes, the OS wakes the thread.
- **Deduplication:** We keep an `Arc<Mutex<String>>` of the `last_content`. If the new clip matches the previous one, we ignore it to prevent spamming the database.
- **Event Emission:** Upon saving a valid new clip, Rust emits a `clipboard_updated` event to the Tauri frontend via `app.emit()`.

## 3. TTL Cleanup

Unpinned items should only live for 24 hours.
- This logic is executed via `db::cleanup_expired(&conn)`.
- It calculates a `cutoff` timestamp (`now - 24 hours`) and deletes all rows where `created_at < cutoff AND pinned = 0`.
- **Trigger:** Currently, this routine runs immediately upon application startup.

## 4. IPC Commands (Tauri API)

These are the `#[tauri::command]` functions exposed in `lib.rs` that the React frontend invokes.

- `get_clipboard_items(search_query: Option<String>)`: Returns a sorted vector of `ClipItem` structs. Pinned items first, then by date descending.
- `toggle_item_pin(id: i64)`: Flips the pin boolean.
- `delete_clipboard_item(id: i64)`: Deletes specific row.
- `write_to_clipboard(content: String)`: Uses the `arboard` crate to manually overwrite the user's system clipboard (used when they click an item in the UI).
