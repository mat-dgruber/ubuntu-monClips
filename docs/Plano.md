Visão geral do novo plano ￼

Vamos recriar o monClips como um app desktop com Tauri + frontend web (React/TS, por exemplo), focado em macOS e Ubuntu. Toda a lógica de negócio que você descreveu vira:

- Backend Tauri (Rust): acesso ao clipboard, banco local, timers, TTL, atalhos globais (no macOS), abertura de links.

- Frontend web: lista, busca, fixar, deletar, copiar, feedback visual.

Vou organizar o plano por camadas e depois em etapas de desenvolvimento.

1. Modelo de dados e banco local ￼

Banco local (SQLite ou similar) gerenciado no lado Tauri:

Tabela ‎`clipboard_items`:

- ‎`id` (PK, auto‑incremento)

- ‎`content` (TEXT)

- ‎`created_at` (INTEGER – epoch/segundos)

- ‎`pinned` (BOOLEAN/INTEGER 0 ou 1)

Regras de negócio:

- Itens não fixados vivem 24h e depois são apagados na rotina de limpeza.

- Itens fixados:

▫ Nunca são apagados pela limpeza automática (imunes à TTL).

▫ Sempre aparecem no topo da lista.

Consultas padrão:

- Listagem: retorna todos (ou filtrados por busca), com ordenação:

▫ Primeiro ‎`pinned = 1`, depois ‎`pinned = 0`.

▫ Dentro de cada grupo, ‎`created_at` desc (mais recente primeiro).

- Busca: ‎`content` contém termo (case insensitive).

2. Monitoramento do clipboard (polling) ￼

Implementado em Rust dentro do Tauri:

- Timer em thread separada rodando a cada 2 segundos:

a. Executa rotina de limpeza (TTL).

b. Lê o conteúdo atual do clipboard (se for texto).

c. Se for texto novo (diferente do último capturado e não vazio):

⁃ Verifica se não é duplicata imediata.

⁃ Insere no banco com ‎`pinned = 0` e ‎`created_at = agora`.

Outros gatilhos além do timer:

- Ao abrir o app / trazer para frente:

▫ Rodar limpeza.

▫ Ler clipboard uma vez e aplicar mesma lógica acima.

- Opcional: botão “Capturar agora” na UI que dispara essa leitura manual.

Importante: encapsular acesso ao clipboard em uma função única para reutilizar em polling manual/automático.

3. Rotina de limpeza (TTL 24h) ￼

Função Rust chamada:

- A cada ciclo do timer (antes de ler o clipboard).

- Opcionalmente, ao abrir o app.

Lógica:

- ‎`now = epoch atual`

- ‎`limite = now - 24h`

- ‎`DELETE FROM clipboard_items WHERE pinned = 0 AND created_at < limite`

Essa função é totalmente no backend, transparente pro frontend.

4. API Tauri (comandos para o frontend) ￼

Definir comandos ‎`#[tauri::command]` que o React chama via ‎`invoke`:

- ‎`list_items(query: Option<String>) -> Vec<ClipItem>`

▫ Implementa:

⁃ Sem ‎`query`: lista tudo com ordenação (fixados no topo).

⁃ Com ‎`query`: filtra por ‎`content LIKE %query%` (case insensitive) mantendo a mesma ordenação.

- ‎`toggle_pin(id: i64)`

▫ Inverte o ‎`pinned` (0 ↔ 1).

- ‎`delete_item(id: i64)`

▫ Apaga item manualmente.

- ‎`copy_to_clipboard(content: String)`

▫ Escreve texto no clipboard.

- (Opcional) ‎`force_refresh_clipboard()`

▫ Roda uma “leitura única” do clipboard usando a mesma lógica do timer.

O frontend nunca acessa o banco direto, só através desses comandos.

5. Lógica de clique e detecção de links ￼

No frontend (React/TS):

- Para cada item renderizado:

▫ Detectar se é link:

⁃ Se começa com ‎`http` ou ‎`www`:

▪ Se começar com ‎`www`, converter para ‎`https://...` para abrir.

▫ Ao clicar:

⁃ Se link → chamar ‎`shell.open` do Tauri (ou comando Rust) para abrir no navegador padrão.

⁃ Se texto normal:

A. Chamar ‎`copy_to_clipboard` com o conteúdo.

B. Exibir feedback visual (ex.: ícone de check verde sobre o item por ~2s).

C. (Opcional) Limpar clipboard antes de escrever, se quiser seguir 100% o comportamento original.

Menu de contexto / ações extras:

- Long press / botão direito:

▫ Opções: copiar novamente, compartilhar (no desktop, isso pode ser:

⁃ copiar (de novo),

⁃ abrir em outra janela,

⁃ ou no futuro integrar com outros apps).

6. UI: lista, busca, fixar, deletar ￼

Tela principal React:

- Campo de busca no topo:

▫ OnChange → chama ‎`list_items(query)` com debounce curto (busca em tempo real).

- Lista de itens:

▫ Visualmente pode separar “Fixados” e “Recentes”, mas a ordenação já vem do backend.

▫ Mostra:

⁃ Conteúdo (talvez truncado).

⁃ Data/hora (formatada).

⁃ Ícones para:

▪ Fixar/desfixar (chama ‎`toggle_pin`).

▪ Deletar (chama ‎`delete_item`).

▪ Copiar (ou clique no card inteiro faz isso).

- Feedback:

▫ Estado de carregando, vazio, sem resultados na busca, etc.

▫ Indicação clara de item recém copiado (check).

7. Atalhos globais (macOS) e compatibilidade com Ubuntu ￼

- macOS:

▫ Usar suporte do Tauri a atalhos globais (ou plugin/biblioteca específica).

▫ Registrar, por exemplo, ‎`Ctrl + Option + V`:

⁃ Quando pressionado, o app:

▪ Ganha foco / traz para frente.

▪ (Opcional) foca automaticamente no campo de busca ou na lista.

- Ubuntu:

▫ Atalhos globais são mais complicados e variam por ambiente gráfico.

▫ Plano mínimo:

⁃ Primeiro release: sem atalho global no Ubuntu, só janela normal.

⁃ Versão futura: estudar hotkeys globais com libs específicas (X11/Wayland) se fizer sentido.

A função de “colar no app anterior simulando teclas” (tipo ‎`Cmd+V` no macOS) é avançada e dependente do SO; dá para manter como futura melhoria, principalmente em desktop macOS.

8. Desenvolvimento, testes e publicação ￼

Fluxo de dev:

1. Mac:

▫ Criar projeto Tauri + React/TS.

▫ Implementar comandos Tauri com SQLite e clipboard.

▫ Construir UI e testar fluxo básico (‎`npm run tauri dev`).

2. Ubuntu (VM):

▫ Clonar o repositório.

▫ Instalar Node + Rust.

▫ Testar:

⁃ Polling do clipboard,

⁃ TTL,

⁃ UI,

⁃ Abertura de links.

3. Build para distribuição:

▫ No Ubuntu, rodar ‎`tauri build` para gerar binário / ‎`.deb` (conforme config).

▫ Ajustar:

⁃ Nome do app,

⁃ Ícone,

⁃ Arquivo ‎`.desktop` para aparecer no menu de apps.

Se você quiser, no próximo passo posso transformar esse plano em uma estrutura de pastas + arquivos iniciais (código real de ‎`main.rs`, modelo ‎`ClipItem`, comandos, e um componente React básico com listagem e busca).

Vou montar um plano já com estrutura de pastas e primeiros arquivos, pensando em Tauri + React/TS, recriando exatamente o comportamento que você descreveu no texto do monClips.

1. Estrutura de pastas do projeto ￼

Depois de criar o app com ‎`npm create tauri-app` (escolhendo React + TS), vamos organizar assim:

- ‎`package.json`

- ‎`src/`

▫ ‎`main.tsx` (entry React)

▫ ‎`App.tsx` (tela principal)

▫ ‎`components/`

⁃ ‎`ClipboardList.tsx`

⁃ ‎`ClipboardItemCard.tsx`

⁃ ‎`SearchBar.tsx`

⁃ ‎`Toast.tsx` (feedback de “copiado”)

▫ ‎`types/`

⁃ ‎`clipboard.ts` (tipo ‎`ClipItem`)

▫ ‎`hooks/`

⁃ ‎`useClipboardItems.ts` (lógica de busca/listagem)

- ‎`src-tauri/`

▫ ‎`Cargo.toml`

▫ ‎`tauri.conf.json`

▫ ‎`src/`

⁃ ‎`main.rs`

⁃ ‎`db.rs` (tudo de SQLite)

⁃ ‎`clipboard.rs` (polling + copiar/colar)

⁃ ‎`models.rs` (struct ‎`ClipItem`)

⁃ ‎`cleanup.rs` (TTL de 24h)

⁃ ‎`hotkeys.rs` (atalhos globais, focado em macOS; opcional para Linux)

Assim você separa bem: banco, clipboard, regras de TTL, API para o frontend.

2. Modelo de dados e tipos ￼

Rust – ‎`models.rs` ￼

```rust
// src-tauri/src/models.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct ClipItem {
    pub id: i64,
    pub content: String,
    pub created_at: i64, // epoch seconds
    pub pinned: bool,
}

```

TypeScript – ‎`types/clipboard.ts` ￼

```ts
// src/types/clipboard.ts
export interface ClipItem {
  id: number;
  content: string;
  created_at: number; // epoch seconds
  pinned: boolean;
}
```

3. Banco de dados e TTL ￼

Rust – ‎`db.rs` ￼

```rust
// src-tauri/src/db.rs
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::ClipItem;

pub struct Db(pub Mutex<Connection>);

impl Db {
    pub fn init(db_path: PathBuf) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS clipboard_items (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              content TEXT NOT NULL,
              created_at INTEGER NOT NULL,
              pinned INTEGER NOT NULL DEFAULT 0
            );
            ",
        )?;
        Ok(Db(Mutex::new(conn)))
    }
}

pub fn insert_clip(db: &Db, content: &str, created_at: i64) -> anyhow::Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO clipboard_items (content, created_at, pinned) VALUES (?1, ?2, 0)",
        params![content, created_at],
    )?;
    Ok(())
}

pub fn list_clips(db: &Db, query: Option<String>) -> anyhow::Result<Vec<ClipItem>> {
    let conn = db.0.lock().unwrap();
    let mut sql = String::from(
        "SELECT id, content, created_at, pinned
         FROM clipboard_items",
    );

    let mut params_vec: Vec<String> = Vec::new();
    if let Some(q) = query {
        sql.push_str(" WHERE LOWER(content) LIKE '%' || ?1 || '%'");
        params_vec.push(q.to_lowercase());
    }

    sql.push_str(" ORDER BY pinned DESC, created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let items_iter = if params_vec.is_empty() {
        stmt.query_map([], |row| {
            Ok(ClipItem {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
            })
        })?
    } else {
        stmt.query_map([&params_vec[0]], |row| {
            Ok(ClipItem {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
            })
        })?
    };

    let mut result = Vec::new();
    for item in items_iter {
        result.push(item?);
    }
    Ok(result)
}

pub fn toggle_pin(db: &Db, id: i64) -> anyhow::Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE clipboard_items SET pinned = 1 - pinned WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn delete_clip(db: &Db, id: i64) -> anyhow::Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

// TTL: apagar não fixados com mais de 24 horas
pub fn cleanup_expired(db: &Db, cutoff: i64) -> anyhow::Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM clipboard_items WHERE pinned = 0 AND created_at < ?1",
        params![cutoff],
    )?;
    Ok(())
}

```

4. Clipboard + polling ￼

Rust – ‎`clipboard.rs` ￼

```rust
// src-tauri/src/clipboard.rs
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::db::{insert_clip, cleanup_expired, Db};

pub fn start_clipboard_watcher(db: Arc<Db>) {
    let last_value = Arc::new(Mutex::new(String::new()));

    let last_clone = last_value.clone();
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().unwrap();

        loop {
            // 1. limpeza TTL
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let cutoff = now - 24 * 60 * 60;
            let _ = cleanup_expired(&db, cutoff);

            // 2. leitura do clipboard
            if let Ok(text) = clipboard.get_text() {
                let mut last = last_clone.lock().unwrap();
                let trimmed = text.trim();

                if !trimmed.is_empty() && *last != trimmed {
                    // evitar duplicata imediata
                    *last = trimmed.to_string();
                    let _ = insert_clip(&db, trimmed, now);
                }
            }

            thread::sleep(Duration::from_secs(2));
        }
    });
}

pub fn copy_to_clipboard(content: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(content.to_string())?;
    Ok(())
}

```

5. Comandos Tauri e ‎`main.rs` ￼

Rust – ‎`main.rs` ￼

```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod clipboard;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::State;
use db::{Db, list_clips, toggle_pin as db_toggle_pin, delete_clip as db_delete_clip, cleanup_expired};
use clipboard::{start_clipboard_watcher, copy_to_clipboard as cb_copy};

#[tauri::command]
fn list_items(db: State<Arc<Db>>, query: Option<String>) -> Result<Vec<models::ClipItem>, String> {
    list_clips(&db, query).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_pin(db: State<Arc<Db>>, id: i64) -> Result<(), String> {
    db_toggle_pin(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_item(db: State<Arc<Db>>, id: i64) -> Result<(), String> {
    db_delete_clip(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn copy_item(content: String) -> Result<(), String> {
    cb_copy(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn force_refresh_clipboard(db: State<Arc<Db>>) -> Result<(), String> {
    // equivalente a um "pull manual" do clipboard + TTL
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let cutoff = now - 24 * 60 * 60;
    cleanup_expired(&db, cutoff).map_err(|e| e.to_string())?;
    // aqui você poderia reutilizar lógica similar à do watcher
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir).unwrap();

            let db_path = app_dir.join("clipboard.db");
            let db = Db::init(db_path).unwrap();
            let db_arc = Arc::new(db);

            // iniciar watcher de clipboard
            start_clipboard_watcher(db_arc.clone());

            app.manage(db_arc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_items,
            toggle_pin,
            delete_item,
            copy_item,
            force_refresh_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar app Tauri");
}

```

6. Frontend: tela principal ￼

React – ‎`hooks/useClipboardItems.ts` ￼

```ts
// src/hooks/useClipboardItems.ts
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import type { ClipItem } from "../types/clipboard";

export function useClipboardItems(query: string) {
  const [items, setItems] = useState<ClipItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function fetchItems() {
      setLoading(true);
      try {
        const result = await invoke<ClipItem[]>("list_items", {
          query: query || null,
        });
        if (!cancelled) {
          setItems(result);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    fetchItems();

    return () => {
      cancelled = true;
    };
  }, [query]);

  return { items, loading, setItems };
}
```

React – ‎`components/SearchBar.tsx` ￼

```tsx
// src/components/SearchBar.tsx
import { ChangeEvent } from "react";

interface Props {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBar({ value, onChange }: Props) {
  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    onChange(e.target.value);
  }

  return (
    <input
      className="search-input"
      placeholder="Pesquisar clipes..."
      value={value}
      onChange={handleChange}
    />
  );
}
```

React – ‎`components/ClipboardItemCard.tsx` ￼

```tsx
// src/components/ClipboardItemCard.tsx
import type { ClipItem } from "../types/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

interface Props {
  item: ClipItem;
  onChange: () => void; // para recarregar lista depois de ações
}

function isLink(text: string): string | null {
  const trimmed = text.trim();
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return trimmed;
  }
  if (trimmed.startsWith("www.")) {
    return `https://${trimmed}`;
  }
  return null;
}

export function ClipboardItemCard({ item, onChange }: Props) {
  const [copied, setCopied] = useState(false);

  async function handleClick() {
    const url = isLink(item.content);
    if (url) {
      // abrir no navegador padrão
      await invoke("open_link", { url }); // você pode criar esse comando depois
      return;
    }

    await invoke("copy_item", { content: item.content });
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleTogglePin(e: React.MouseEvent) {
    e.stopPropagation();
    await invoke("toggle_pin", { id: item.id });
    onChange();
  }

  async function handleDelete(e: React.MouseEvent) {
    e.stopPropagation();
    await invoke("delete_item", { id: item.id });
    onChange();
  }

  const date = new Date(item.created_at * 1000).toLocaleString();

  return (
    <div className="clip-card" onClick={handleClick}>
      <div className="clip-content">
        <p>{item.content}</p>
      </div>
      <div className="clip-footer">
        <span className="clip-date">{date}</span>
        <div className="clip-actions">
          <button onClick={handleTogglePin}>
            {item.pinned ? "Desfixar" : "Fixar"}
          </button>
          <button onClick={handleDelete}>Excluir</button>
        </div>
      </div>
      {copied && <div className="clip-copied">Copiado!</div>}
    </div>
  );
}
```

Você ainda precisa implementar o comando ‎`open_link` no Rust (bem simples, usando API de shell do Tauri), mas a estrutura já está pronta.

React – ‎`components/ClipboardList.tsx` ￼

```tsx
// src/components/ClipboardList.tsx
import type { ClipItem } from "../types/clipboard";
import { ClipboardItemCard } from "./ClipboardItemCard";

interface Props {
  items: ClipItem[];
  onChange: () => void;
}

export function ClipboardList({ items, onChange }: Props) {
  if (!items.length) {
    return <div className="empty-state">Nenhum clipe ainda.</div>;
  }

  return (
    <div className="clip-list">
      {items.map((item) => (
        <ClipboardItemCard key={item.id} item={item} onChange={onChange} />
      ))}
    </div>
  );
}
```

React – ‎`App.tsx` ￼

```tsx
// src/App.tsx
import { useState } from "react";
import { useClipboardItems } from "./hooks/useClipboardItems";
import { SearchBar } from "./components/SearchBar";
import { ClipboardList } from "./components/ClipboardList";

function App() {
  const [query, setQuery] = useState("");
  const { items, loading } = useClipboardItems(query);
  const [refreshKey, setRefreshKey] = useState(0);

  function forceRefresh() {
    setRefreshKey((prev) => prev + 1);
  }

  // reaproveita hook para quando quiser recarregar manualmente depois
  const { items: displayItems } = useClipboardItems(query + "|" + refreshKey);

  return (
    <div className="app-root">
      <header className="app-header">
        <h1>monClips Desktop</h1>
        <SearchBar value={query} onChange={setQuery} />
      </header>
      <main>
        {loading ? (
          <div>Carregando…</div>
        ) : (
          <ClipboardList items={displayItems} onChange={forceRefresh} />
        )}
      </main>
    </div>
  );
}

export default App;
```

(Depois você pode refinar esse hook para não duplicar chamada, mas a ideia de callback ‎`onChange` está aí.)

7. Desenvolvimento, teste e publicação ￼

1. Criar projeto com ‎`npm create tauri-app` no Mac.

1. Substituir/adicinar os arquivos acima nas pastas correspondentes.

1. Rodar no Mac: ‎`npm run tauri dev` para validar UI e fluxo básico.

1. Criar VM Ubuntu, clonar o repo, instalar Node + Rust, rodar também ‎`npm run tauri dev` para testar clipboard/TTL no Linux.

1. No Ubuntu, gerar build: ‎`npm run tauri build` e usar o ‎`.deb`/binário gerado.

Esse plano já te coloca bem próximo de um MVP funcional do monClips em Tauri, com a mesma lógica de negócios que você descreveu.
