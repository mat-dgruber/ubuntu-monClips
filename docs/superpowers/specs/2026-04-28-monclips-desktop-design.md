# monClips Design Specification

## Overview
monClips is a desktop clipboard manager built with Tauri (Rust backend) and React/TypeScript (frontend). It captures copied text, stores it locally, allows users to pin important items, and automatically deletes unpinned items after 24 hours.

## Architecture & Technology Stack
*   **Framework:** Tauri
*   **Backend:** Rust
*   **Frontend:** React (TypeScript) + Vite
*   **UI/Styling:** Tailwind CSS + shadcn/ui
*   **Database:** SQLite (`rusqlite`)
*   **Clipboard Management:** Rust OS-level event listeners (fallback to polling if necessary) and `arboard` for manual get/set.

## Data Model (SQLite)
Database stored in the app's local data directory.

**Table: `clipboard_items`**
*   `id`: INTEGER PRIMARY KEY AUTOINCREMENT
*   `content`: TEXT NOT NULL
*   `created_at`: INTEGER NOT NULL (Unix timestamp)
*   `pinned`: INTEGER NOT NULL DEFAULT 0 (Boolean representation)

## Backend Logic (Rust)
1.  **Clipboard Monitor:** A background thread listening for OS clipboard events. Upon detecting new text, it saves it to the database and emits a `clipboard_updated` event to the frontend. It ignores immediate duplicates.
2.  **TTL Cleanup:** A routine that runs on app startup and periodically (e.g., every 15 minutes) to execute: `DELETE FROM clipboard_items WHERE pinned = 0 AND created_at < [Now - 24h]`.

## Tauri Commands (API Bridge)
*   `get_clipboard_items(search_query: Option<String>) -> Vec<ClipItem>`: Returns items sorted by `pinned DESC, created_at DESC`.
*   `toggle_item_pin(id: i64)`: Inverts the pinned status.
*   `delete_clipboard_item(id: i64)`: Deletes a specific item.
*   `write_to_clipboard(content: String)`: Writes text to the OS clipboard.
*   `open_in_browser(url: String)`: Opens a URL in the default browser using Tauri shell API.

## Frontend UI/UX
*   **Layout:** Simple list view with a fixed search bar at the top.
*   **Search:** Input changes trigger `get_clipboard_items` with a ~200ms debounce.
*   **List Items:**
    *   Clicking plain text: Calls `write_to_clipboard`, shows "Copied!" feedback.
    *   Clicking URLs: Calls `open_in_browser`.
    *   Hovering reveals Pin and Delete action buttons.
*   **Reactivity:** Listens for the `clipboard_updated` Tauri event to auto-refresh the list when new items are copied externally.
