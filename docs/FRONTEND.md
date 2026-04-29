# Frontend Architecture (React/TypeScript)

The frontend of monClips is built with React and TypeScript, bundled via Vite. It acts as a reactive view layer that interfaces with the Rust backend.

## Tech Stack

- **React 18**
- **TypeScript**
- **Tailwind CSS** (for styling)
- **@tauri-apps/api** (for IPC communication)

---

## Core Principles

1. **Stateless UI (Mostly):** The React frontend does not maintain a complex local cache of the clipboard items. The source of truth is always the Rust backend. React fetches, displays, and delegates actions.
2. **Reactivity:** The UI automatically refreshes without user intervention when new items are copied outside the app.

## 1. The `useClipboard` Hook (`src/hooks/useClipboard.ts`)

This is the central nervous system of the frontend. It abstracts all Tauri IPC calls into a simple React hook.

### Responsibilities:

- **State Management:** Holds the `items` array and the `searchQuery` string.
- **Fetching:** `fetchItems(query)` calls `invoke('get_clipboard_items')`.
- **Event Listening:** It sets up a listener for the `"clipboard_updated"` event emitted by Rust. When received, it automatically triggers a re-fetch.
- **Mutations:** Provides wrapper functions for `togglePin`, `deleteItem`, and `copyToClipboard`.
- **OS Integration:** Provides `openUrl` which uses `@tauri-apps/plugin-opener` to securely open links in the user's default web browser instead of navigating the WebView.

## 2. Main Application (`src/App.tsx`)

The UI is intentionally minimal and native-feeling.

### Layout

- **Sticky Header:** Contains the search input. Modifying the search input updates the hook's state, which triggers a re-fetch with the query parameter.
- **Scrollable List:** Renders the `ClipItem` array.

### Interactions

- **URL Detection:** A simple Regex (`/^https?:\/\//i`) checks if a clip starts with a web protocol. If it does, clicking the item opens the browser. If not, it copies the text to the clipboard.
- **Hover States:** Action buttons (Pin, Delete) use Tailwind's `group` and `group-hover` utilities to only appear when the user hovers over a specific clip row, keeping the UI clean.
