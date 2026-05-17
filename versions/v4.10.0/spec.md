# Favnir v4.10.0 仕様書 — Notebook

作成日: 2026-05-17

---

## 概要

Favnir Notebook は、データエンジニアリングのプロトタイピング・ドキュメント化・インタラクティブ実行を支援するドキュメント形式とツールチェーンを提供する。Jupyter Notebook に相当するが Favnir 専用に設計されており、型安全なコードセルと Markdown セルを組み合わせてパイプラインの検証・共有を行える。

**主な追加機能:**
- `.fav.nb` ファイル形式（JSON）— セルのリストで構成
- `fav notebook new <name>` — 新規ノートブック作成
- `fav notebook run <file>` — 全コードセルを順次実行し出力を表示
- `fav notebook serve <file> [--port <n>]` — ブラウザ UI でインタラクティブ実行
- `fav notebook export <file> [--out <path>]` — Markdown へエクスポート
- `fav notebook check <file>` — 全コードセルを型チェックのみ実行

---

## ファイル形式 (`.fav.nb`)

JSON 形式。拡張子は `.fav.nb`（または `.fnb`）。

```json
{
  "version": "1.0",
  "title": "My Pipeline Analysis",
  "cells": [
    {
      "id": "c001",
      "type": "markdown",
      "content": "# Overview\nThis notebook analyzes..."
    },
    {
      "id": "c002",
      "type": "code",
      "content": "fn double(n: Int) -> Int { n * 2 }",
      "output": null
    },
    {
      "id": "c003",
      "type": "code",
      "content": "double(21)",
      "output": {
        "kind": "value",
        "text": "42"
      }
    }
  ]
}
```

### セル型

| type | 説明 |
|------|------|
| `markdown` | Markdown テキスト（実行不可） |
| `code` | Favnir コードセル（実行・型チェック対象） |

### output フィールド

| kind | 説明 |
|------|------|
| `value` | 最後の式の評価値を文字列化 |
| `table` | `List<Map<String,String>>` をテーブル形式で表示 |
| `error` | 型エラーまたはランタイムエラー |
| `none` | 出力なし（Unit を返す式 等） |

---

## 実行モデル

### コンテキスト累積

コードセルは上から順番に実行される。後のセルは前のセルで定義した関数・型を参照できる。

実装方針：
1. セル 1〜N のコード内容を順番に結合してパース・型チェック
2. セル N の最後の式を「出力」として評価
3. 累積コンテキストは前のセルの関数定義のみ（副作用は各セルで独立）

```
cell_1_code + cell_2_code + ... + cell_N_code
  → Parser::parse_str
  → Checker::check_with_self
  → compile_program + codegen_program
  → VM::run("$cell:N")  ← セル N の最後の式を返す関数
```

### 出力変換

VM が返す `Value` を表示文字列に変換する：

| Value | 表示 |
|-------|------|
| `Int(n)` | `"42"` |
| `Float(f)` | `"3.14"` |
| `Bool(b)` | `"true"` / `"false"` |
| `Str(s)` | `"hello"` |
| `List([...])` | JSON 配列 または テーブル（Map のリストの場合） |
| `Record(...)` | JSON オブジェクト |
| `Unit` | `""` (空表示) |
| その他 | `debug_format` |

---

## コマンド

### `fav notebook new <name>`

新しいノートブック `<name>.fav.nb` を作成する。

**生成内容:**
```json
{
  "version": "1.0",
  "title": "<name>",
  "cells": [
    { "id": "c001", "type": "markdown", "content": "# <name>\n\nNotebook description here." },
    { "id": "c002", "type": "code",     "content": "// Start writing Favnir code here\n42", "output": null }
  ]
}
```

### `fav notebook run <file> [--no-cache]`

全コードセルを上から順に実行し、各セルの出力を stdout に表示する。実行済みの出力を `output` フィールドに書き戻してファイルを更新する。

`--no-cache`: 既存の `output` を無視して全セルを再実行する。

**出力例:**
```
[cell c001] markdown — skipped
[cell c002] fn double(n: Int) -> Int { n * 2 }
  → (no output)
[cell c003] double(21)
  → 42

3 cells, 2 code cells, 2 passed, 0 failed
```

### `fav notebook serve <file> [--port <n>] [--no-open]`

ローカル HTTP サーバーを起動してブラウザ UI を提供する。デフォルトポートは `8888`。

**エンドポイント:**

| Path | 説明 |
|------|------|
| `GET /` | ノートブック UI HTML |
| `GET /api/notebook` | ノートブック JSON を返す |
| `POST /api/run/{cell_id}` | 指定セルを実行して結果を返す |
| `POST /api/run-all` | 全コードセルを実行して結果を返す |
| `POST /api/update/{cell_id}` | セルの内容を更新してファイルに保存 |
| `POST /api/add-cell` | 新しいセルを追加 |
| `DELETE /api/cell/{cell_id}` | セルを削除 |

**ブラウザ UI:**
- 各セルを表示（コードエリア + 出力エリア）
- 「Run」ボタンでセル単体実行
- 「Run All」ボタンで全セル実行
- Markdown セルはレンダリング表示（または raw text）
- シンプルな HTML + JavaScript（外部 CDN 依存なし）

### `fav notebook export <file> [--out <path>]`

ノートブックを Markdown ファイルにエクスポートする。`--out` を省略した場合は stdout に出力。

**変換規則:**
- `markdown` セル → そのまま Markdown
- `code` セル → ` ```favnir ... ``` ` コードブロック
- output が `value` → ` ``` 42 ``` ` 結果ブロック
- output が `table` → Markdown テーブル

### `fav notebook check <file>`

全コードセルを型チェックのみ実行する（VM は走らせない）。

---

## ブラウザ UI 詳細

`tiny_http`（既存依存）を使用。外部 CDN 不使用。

### HTML 構成

```
┌─────────────────────────────────────────┐
│  Favnir Notebook: <title>               │
│  [Run All]                              │
├─────────────────────────────────────────┤
│  ▼ [markdown]                           │
│  # Overview                             │
│  This notebook...                       │
├─────────────────────────────────────────┤
│  ▼ [code]         [Run] [Delete]        │
│  ┌─ Favnir ──────────────────────────┐ │
│  │  fn double(n: Int) -> Int { n*2 } │ │
│  └───────────────────────────────────┘ │
│  Out: 42                                │
└─────────────────────────────────────────┘
```

### JavaScript API 呼び出し

```javascript
// Run single cell
fetch('/api/run/' + cellId, { method: 'POST' })
  .then(r => r.json())
  .then(result => updateOutput(cellId, result));

// Update cell content
fetch('/api/update/' + cellId, {
  method: 'POST',
  body: JSON.stringify({ content: newContent })
});
```

---

## 型チェック・実行の詳細実装

### コンテキスト管理

```rust
pub struct NotebookExecutor {
    notebook: Notebook,
}

impl NotebookExecutor {
    // Run cell N with context from cells 0..N
    pub fn run_cell(&self, idx: usize) -> CellOutput {
        let ctx_source = self.build_context_source(idx);
        // Parse + check + compile ctx_source + cell source
        // Extract the last expression as "$cell_N_result"
        // VM::run → Value → CellOutput
    }

    fn build_context_source(&self, up_to: usize) -> String {
        // Concatenate fn/type definitions from code cells 0..up_to
        // (exclude the last expression of each cell — only definitions)
    }
}
```

### セル関数の命名規則

コンパイル時に各コードセルの最後の式を関数として包む：

```
// セル c003 のコード: double(21)
// 生成されるラッパー:
fn $cell_c003_result() -> Int { double(21) }
```

型推論でリターン型を決定する。最後の式が `Unit` の場合は出力なし。

---

## データ型

```rust
pub struct Notebook {
    pub version: String,
    pub title: String,
    pub cells: Vec<Cell>,
}

pub struct Cell {
    pub id: String,
    pub cell_type: CellType,
    pub content: String,
    pub output: Option<CellOutput>,
}

pub enum CellType {
    Markdown,
    Code,
}

pub struct CellOutput {
    pub kind: OutputKind,
    pub text: String,
}

pub enum OutputKind {
    Value,
    Table,
    Error,
    None,
}
```

---

## テスト方針

### ユニットテスト（`fav/src/notebook/mod.rs`）

| テスト | 内容 |
|--------|------|
| `notebook_parse_valid_json` | 正しい JSON をパースできる |
| `notebook_serialize_roundtrip` | パース → シリアライズが一致する |
| `notebook_new_creates_default` | `new_notebook` が正しいデフォルトを生成 |
| `run_cell_returns_value` | コードセルが値を返す |
| `run_cell_returns_error_on_type_error` | 型エラーが error output になる |
| `run_cell_uses_context_from_previous` | 前セルの定義を参照できる |
| `run_all_cells_accumulates_outputs` | 全セル実行で出力が蓄積される |
| `export_markdown_has_code_blocks` | Markdown エクスポートに ```favnir ブロックが含まれる |
| `export_markdown_has_output` | 実行済みセルの出力が Markdown に含まれる |
| `cell_id_generation_is_unique` | 追加セルの ID がユニーク |

### 統合テスト（`fav/src/driver.rs`）

| テスト | 内容 |
|--------|------|
| `notebook_run_produces_outputs` | `cmd_notebook_run` が出力を書き戻す |
| `notebook_check_detects_type_error` | `cmd_notebook_check` が型エラーを報告する |
| `notebook_export_produces_markdown` | `cmd_notebook_export` が Markdown を返す |

---

## 既知の制約

- v4.10.0 は `!Io` / `!File` 効果付きセルの実行対応（DB 操作は副作用があるため制限しない）
- セル間の状態共有は「関数・型定義の共有」のみ（変数は共有しない）
- Markdown レンダリングはブラウザ UI でも plain text（HTML エスケープのみ）
- セルの並び替えは v4.10.0 では未対応（将来の UI 強化で対応）
- `table` 出力は `List<Map<String,String>>` のみ（他の構造は `value` として表示）
- `--port` は 1024〜65535 の範囲のみ対応
