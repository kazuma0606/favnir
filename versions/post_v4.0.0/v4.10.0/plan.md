# Favnir v4.10.0 実装計画 — Notebook

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.10.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.10.0` に更新

---

## Phase 1: データ型とファイル形式 (`fav/src/notebook/mod.rs`)

### 型定義

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub version: String,
    pub title: String,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(rename = "type")]
    pub cell_type: CellType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<CellOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Markdown,
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub kind: OutputKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputKind {
    Value,
    Table,
    Error,
    None,
}
```

### ファイル読み書き

```rust
impl Notebook {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {path}: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("invalid notebook JSON: {e}"))
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize error: {e}"))?;
        std::fs::write(path, json)
            .map_err(|e| format!("cannot write {path}: {e}"))
    }

    pub fn new_notebook(title: &str) -> Self { ... }

    pub fn add_cell(&mut self, cell_type: CellType, content: &str) -> String {
        // 生成する ID: "c" + zero-padded counter
        let id = format!("c{:03}", self.cells.len() + 1);
        self.cells.push(Cell { id: id.clone(), cell_type, content: content.to_string(), output: None });
        id
    }
}
```

### `new_notebook` の内容

```rust
Notebook {
    version: "1.0".to_string(),
    title: title.to_string(),
    cells: vec![
        Cell {
            id: "c001".to_string(),
            cell_type: CellType::Markdown,
            content: format!("# {}\n\nNotebook description here.", title),
            output: None,
        },
        Cell {
            id: "c002".to_string(),
            cell_type: CellType::Code,
            content: "// Start writing Favnir code here\n42".to_string(),
            output: None,
        },
    ],
}
```

---

## Phase 2: コンテキスト実行エンジン (`notebook/executor.rs`)

### NotebookExecutor

```rust
pub struct NotebookExecutor<'a> {
    notebook: &'a mut Notebook,
}

impl<'a> NotebookExecutor<'a> {
    pub fn new(notebook: &'a mut Notebook) -> Self { Self { notebook } }

    pub fn run_cell(&mut self, cell_id: &str) -> CellOutput {
        let idx = match self.notebook.cells.iter().position(|c| c.id == cell_id) {
            Some(i) => i,
            None => return CellOutput { kind: OutputKind::Error, text: format!("cell {} not found", cell_id) },
        };
        if self.notebook.cells[idx].cell_type == CellType::Markdown {
            return CellOutput { kind: OutputKind::None, text: String::new() };
        }
        let output = self.execute_code_cell(idx);
        self.notebook.cells[idx].output = Some(output.clone());
        output
    }

    pub fn run_all(&mut self) -> Vec<(String, CellOutput)> {
        let ids: Vec<String> = self.notebook.cells.iter().map(|c| c.id.clone()).collect();
        ids.into_iter().map(|id| {
            let out = self.run_cell(&id);
            (id, out)
        }).collect()
    }

    fn execute_code_cell(&self, idx: usize) -> CellOutput {
        // 1. コンテキスト（定義のみ）を前のコードセルから収集
        // 2. セル idx のコードを末尾に追加してパース・チェック
        // 3. コンパイル → VM 実行
        // 4. Value → CellOutput に変換
    }
}
```

### コンテキスト結合戦略

前のセル（0..idx）のコードを結合する際、各セルの **「最後の式」は除外**し、定義（fn, type, stage 等）のみを取り込む：

```rust
fn build_context_source(cells: &[Cell], up_to: usize) -> String {
    let mut parts = Vec::new();
    for cell in cells[..up_to].iter().filter(|c| c.cell_type == CellType::Code) {
        // セルのコードをそのまま追加（関数定義として扱う）
        parts.push(cell.content.as_str());
    }
    parts.join("\n")
}
```

> **実装ノート**: 最後の式の除外は難しいため v4.10.0 では全コードを結合する。つまりセル 1 に `42` があれば、セル 2 のコンテキストに `42` も含まれる。パーサーがこれを複数の item として扱えば問題ない。末尾の式だけ `$cell_result` 関数でラップする。

### 結果ラッパー

```rust
fn wrap_last_expr(source: &str, fn_name: &str) -> String {
    // source の末尾の式を fn_name() -> _ { expr } でラップする
    // 簡易実装: source の後に追記
    // ただしセル全体が fn/type 定義のみなら出力なし
    format!("{}\nfn {}() = do\n  {}", definitions, fn_name, last_expr)
}
```

> **v4.10.0 簡易実装**: セルのコードが式（fn 定義で終わらない）かどうかをヒューリスティックで判定。まず `fn_name()` ラッパーを試み、コンパイルエラーなら「定義のみ」と判断して `OutputKind::None` を返す。

### Value → CellOutput 変換

```rust
fn value_to_output(val: &Value) -> CellOutput {
    match val {
        Value::Int(n)   => CellOutput { kind: OutputKind::Value, text: n.to_string() },
        Value::Float(f) => CellOutput { kind: OutputKind::Value, text: format!("{}", f) },
        Value::Bool(b)  => CellOutput { kind: OutputKind::Value, text: b.to_string() },
        Value::Str(s)   => CellOutput { kind: OutputKind::Value, text: s.clone() },
        Value::Unit     => CellOutput { kind: OutputKind::None, text: String::new() },
        Value::List(items) if is_list_of_records(items)
                        => CellOutput { kind: OutputKind::Table, text: format_table(items) },
        other           => CellOutput { kind: OutputKind::Value, text: format!("{:?}", other) },
    }
}

fn format_table(items: &[Value]) -> String {
    // List<Map> → TSV 形式（または Markdown テーブル）
    // ヘッダーは最初の Map のキー
}
```

---

## Phase 3: 型チェックのみモード

```rust
pub fn check_notebook(notebook: &Notebook) -> Vec<(String, Vec<String>)> {
    // 各コードセルを型チェックし、(cell_id, errors) のリストを返す
    let mut errors = Vec::new();
    let mut ctx = String::new();
    for cell in &notebook.cells {
        if cell.cell_type != CellType::Code { continue; }
        let combined = format!("{}\n{}", ctx, cell.content);
        match Parser::parse_str(&combined, &cell.id) {
            Err(e) => errors.push((cell.id.clone(), vec![format!("E0500: {}", e.message)])),
            Ok(program) => {
                let mut checker = Checker::new();
                let (errs, _) = checker.check_with_self(&program);
                if !errs.is_empty() {
                    errors.push((
                        cell.id.clone(),
                        errs.iter().map(|e| format!("{}: {}", e.code, e.message)).collect(),
                    ));
                }
            }
        }
        ctx = format!("{}\n{}", ctx, cell.content);
    }
    errors
}
```

---

## Phase 4: Markdown エクスポート

```rust
pub fn export_markdown(notebook: &Notebook) -> String {
    let mut md = format!("# {}\n\n", notebook.title);
    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                md.push_str(&cell.content);
                md.push_str("\n\n");
            }
            CellType::Code => {
                md.push_str("```favnir\n");
                md.push_str(&cell.content);
                md.push_str("\n```\n\n");
                if let Some(output) = &cell.output {
                    if output.kind != OutputKind::None && !output.text.is_empty() {
                        if output.kind == OutputKind::Error {
                            md.push_str(&format!("> Error: {}\n\n", output.text));
                        } else if output.kind == OutputKind::Table {
                            md.push_str(&output.text); // already Markdown table
                            md.push_str("\n\n");
                        } else {
                            md.push_str(&format!("```\n{}\n```\n\n", output.text));
                        }
                    }
                }
            }
        }
    }
    md
}
```

---

## Phase 5: HTTP サーバー (`notebook/server.rs`)

`tiny_http`（既存依存）を使用。LSP と同様に外部 JS フレームワーク不使用。

### ルーティング

```rust
pub fn serve_notebook(path: &str, port: u16, no_open: bool) {
    let addr = format!("0.0.0.0:{}", port);
    let server = tiny_http::Server::http(&addr).expect("failed to start server");
    if !no_open {
        let _ = open::that(format!("http://localhost:{}", port));
    }
    eprintln!("[notebook] serving {} on http://localhost:{}", path, port);

    for request in server.incoming_requests() {
        handle_request(request, path);
    }
}
```

### エンドポイント実装

```rust
fn handle_request(req: tiny_http::Request, nb_path: &str) {
    let method = req.method().as_str().to_string();
    let url = req.url().to_string();

    match (method.as_str(), url.as_str()) {
        ("GET", "/")                 => serve_ui(req, nb_path),
        ("GET", "/api/notebook")     => serve_notebook_json(req, nb_path),
        ("POST", path) if path.starts_with("/api/run/") => {
            let cell_id = path.strip_prefix("/api/run/").unwrap_or("");
            run_cell_endpoint(req, nb_path, cell_id);
        }
        ("POST", "/api/run-all")     => run_all_endpoint(req, nb_path),
        ("POST", path) if path.starts_with("/api/update/") => {
            let cell_id = path.strip_prefix("/api/update/").unwrap_or("");
            update_cell_endpoint(req, nb_path, cell_id);
        }
        ("POST", "/api/add-cell")    => add_cell_endpoint(req, nb_path),
        ("DELETE", path) if path.starts_with("/api/cell/") => {
            let cell_id = path.strip_prefix("/api/cell/").unwrap_or("");
            delete_cell_endpoint(req, nb_path, cell_id);
        }
        _ => {
            let _ = req.respond(tiny_http::Response::from_string("Not Found")
                .with_status_code(404));
        }
    }
}
```

### UI HTML（インライン）

```rust
const NOTEBOOK_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Favnir Notebook</title>
<style>
  body { font-family: monospace; max-width: 900px; margin: 40px auto; padding: 0 20px; }
  .cell { border: 1px solid #ddd; margin: 12px 0; border-radius: 4px; }
  .cell-header { background: #f5f5f5; padding: 4px 8px; display: flex; justify-content: space-between; }
  .cell-type { font-size: 12px; color: #888; }
  textarea { width: 100%; font-family: monospace; padding: 8px; border: none; resize: vertical; box-sizing: border-box; }
  .output { padding: 8px 12px; background: #fafafa; border-top: 1px solid #eee; white-space: pre; }
  .output.error { color: #c00; }
  .output.table { overflow-x: auto; }
  button { padding: 4px 10px; cursor: pointer; }
  h1 { font-size: 20px; }
  .toolbar { margin-bottom: 16px; }
</style>
</head>
<body>
<h1 id="nb-title">Favnir Notebook</h1>
<div class="toolbar">
  <button onclick="runAll()">Run All</button>
  <button onclick="addCodeCell()">+ Code</button>
  <button onclick="addMarkdownCell()">+ Markdown</button>
  <button onclick="exportMarkdown()">Export MD</button>
</div>
<div id="cells"></div>
<script>
/* notebook JS — see full implementation in notebook/server.rs NOTEBOOK_UI_HTML */
</script>
</body>
</html>"#;
```

### JavaScript（ルーム内 inline）

```javascript
let notebook = null;

async function loadNotebook() {
  const res = await fetch('/api/notebook');
  notebook = await res.json();
  document.getElementById('nb-title').textContent = notebook.title;
  renderCells();
}

function renderCells() {
  const container = document.getElementById('cells');
  container.innerHTML = '';
  for (const cell of notebook.cells) {
    container.appendChild(renderCell(cell));
  }
}

function renderCell(cell) {
  const div = document.createElement('div');
  div.className = 'cell';
  div.id = 'cell-' + cell.id;

  const header = document.createElement('div');
  header.className = 'cell-header';
  header.innerHTML = `<span class="cell-type">[${cell.type}]</span>
    <span>
      <button onclick="runCell('${cell.id}')">Run</button>
      <button onclick="deleteCell('${cell.id}')">Del</button>
    </span>`;
  div.appendChild(header);

  const ta = document.createElement('textarea');
  ta.rows = Math.max(3, cell.content.split('\n').length + 1);
  ta.value = cell.content;
  ta.oninput = () => updateCell(cell.id, ta.value);
  div.appendChild(ta);

  if (cell.output) {
    const out = document.createElement('div');
    out.className = 'output' + (cell.output.kind === 'error' ? ' error' : '');
    out.textContent = cell.output.kind === 'none' ? '' : cell.output.text;
    div.appendChild(out);
  }
  return div;
}

async function runCell(id) {
  const res = await fetch('/api/run/' + id, { method: 'POST' });
  const result = await res.json();
  await loadNotebook();  // reload to reflect saved output
}

async function runAll() {
  await fetch('/api/run-all', { method: 'POST' });
  await loadNotebook();
}

async function updateCell(id, content) {
  await fetch('/api/update/' + id, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content })
  });
}

async function addCodeCell() {
  await fetch('/api/add-cell', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ type: 'code', content: '' })
  });
  await loadNotebook();
}

async function addMarkdownCell() {
  await fetch('/api/add-cell', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ type: 'markdown', content: '' })
  });
  await loadNotebook();
}

async function deleteCell(id) {
  await fetch('/api/cell/' + id, { method: 'DELETE' });
  await loadNotebook();
}

async function exportMarkdown() {
  const res = await fetch('/api/export');
  const text = await res.text();
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'text/markdown' }));
  a.download = 'notebook.md';
  a.click();
}

loadNotebook();
```

---

## Phase 6: ドライバー関数 (`driver.rs`)

```rust
pub fn cmd_notebook_new(name: &str) { ... }
pub fn cmd_notebook_run(path: &str, no_cache: bool) { ... }
pub fn cmd_notebook_serve(path: &str, port: u16, no_open: bool) { ... }
pub fn cmd_notebook_export(path: &str, out: Option<&str>) { ... }
pub fn cmd_notebook_check(path: &str) { ... }
```

---

## Phase 7: CLI 配線 (`main.rs`)

```rust
Some("notebook") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| { ... });
            cmd_notebook_new(name);
        }
        Some("run") => {
            let path = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| { ... });
            let no_cache = args.iter().any(|a| a == "--no-cache");
            cmd_notebook_run(path, no_cache);
        }
        Some("serve") => {
            let path = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| { ... });
            let port = parse_port_arg(&args, 8888);
            let no_open = args.iter().any(|a| a == "--no-open");
            cmd_notebook_serve(path, port, no_open);
        }
        Some("export") => {
            let path = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| { ... });
            let out = parse_out_arg(&args);
            cmd_notebook_export(path, out.as_deref());
        }
        Some("check") => {
            let path = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| { ... });
            cmd_notebook_check(path);
        }
        _ => { eprintln!("error: notebook requires new|run|serve|export|check"); process::exit(1); }
    }
}
```

HELP テキスト:
```
    notebook new <name>
                  Create a new notebook (<name>.fav.nb).
    notebook run [--no-cache] <file>
                  Execute all code cells and save outputs.
    notebook serve [--port <n>] [--no-open] <file>
                  Start interactive browser UI (default port 8888).
    notebook export [--out <path>] <file>
                  Export notebook to Markdown.
    notebook check <file>
                  Type-check all code cells without executing.
```

---

## Phase 8: テスト

### ユニットテスト（`fav/src/notebook/mod.rs`）

`#[cfg(test)]` モジュールに配置。tempfile クレート（既存 dev-dep）を活用。

### 統合テスト（`fav/src/driver.rs` の `notebook_tests` モジュール）

`cmd_notebook_run` / `cmd_notebook_check` / `cmd_notebook_export` を呼び出してファイル内容を検証。

---

## 実装メモ

- **セル ID 生成**: `format!("c{:03}", cells.len() + 1)` — 同一 ID の衝突を防ぐためカウンタ管理
- **コンテキスト結合の落とし穴**: セル間の型推論が累積するため、前セルの型エラーが後セルに伝播する。v4.10.0 では許容する（型エラーがある場合は早期終了）
- **テーブル出力検出**: `Value::List(items)` の先頭要素が `Value::Record(...)` であれば table 形式
- **`tiny_http` スレッド安全性**: リクエストハンドラはシングルスレッドで実行（ノートブックの状態を Mutex で保護するか、各リクエストでファイルを読み直す）
- **デバウンス不要**: `notebook serve` はリクエストごとにファイルを読む（ホットリロード相当）
- **`export` エンドポイント**: serve 中に `GET /api/export` を追加してブラウザから MD ダウンロードを可能にする
- **`serde_json::to_string_pretty`**: ノートブック保存時に使用（人間が読める差分）
