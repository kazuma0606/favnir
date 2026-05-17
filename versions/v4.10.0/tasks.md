# Favnir v4.10.0 タスクリスト — Notebook

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.10.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.10.0` に更新

---

## Phase 1: データ型とファイル形式

- [x] `fav/src/notebook/mod.rs` 作成（`pub mod notebook;` を `main.rs` に追加）
- [x] `Notebook` 構造体（version / title / cells）— `serde::Serialize + Deserialize`
- [x] `Cell` 構造体（id / cell_type / content / output）
- [x] `CellType` enum: `Markdown` / `Code`（`#[serde(rename_all = "lowercase")]`）
- [x] `CellOutput` 構造体（kind / text）
- [x] `OutputKind` enum: `Value` / `Table` / `Error` / `None`
- [x] `Notebook::load(path)` — JSON 読み込み
- [x] `Notebook::save(path)` — `serde_json::to_string_pretty` で書き込み
- [x] `Notebook::new_notebook(title)` — デフォルト 2 セル（markdown + code）
- [x] `Notebook::add_cell(cell_type, content)` — ID 自動採番して追加
- [x] `Notebook::remove_cell(id)` — ID でセルを削除
- [x] `Notebook::update_cell_content(id, content)` — コンテンツ更新

---

## Phase 2: コンテキスト実行エンジン

- [x] `notebook/executor.rs` 作成（または `mod.rs` 内に含める）
- [x] `NotebookExecutor` 構造体（`&mut Notebook` 参照保持）
- [x] `run_cell(cell_id)` — 指定セルを実行して `CellOutput` を返す + 保存
- [x] `run_all()` — 全セルを順次実行、`Vec<(String, CellOutput)>` を返す
- [x] `build_context_source(up_to_idx)` — 前セルのコードを結合する
- [x] セル結果ラッパー生成: コード末尾に `fn $cell_{id}_result() = ...` を追加（最後の式を取り出す）
- [x] `value_to_output(val)` — `Value` → `CellOutput` 変換
- [x] `format_table(items)` — `List<Map>` → Markdown テーブル文字列
- [x] `is_list_of_records(items)` — テーブル判定ヘルパー
- [x] セルが定義のみ（fn/type）の場合は `OutputKind::None` を返す

---

## Phase 3: 型チェック

- [x] `check_notebook(notebook)` — 全コードセルを順次型チェック、`Vec<(String, Vec<String>)>` を返す
- [x] ParseError → `(cell_id, ["E0500: ..."])` に変換
- [x] TypeError → `(cell_id, errors)` に変換
- [x] コンテキスト累積（前セルのコードを結合）

---

## Phase 4: Markdown エクスポート

- [x] `export_markdown(notebook)` → `String`
- [x] Markdown セル → そのまま出力
- [x] Code セル → ` ```favnir ... ``` ` ブロック
- [x] output `value` → ` ```\n...\n``` ` ブロック
- [x] output `table` → Markdown テーブル文字列をそのまま出力
- [x] output `error` → `> Error: ...` blockquote

---

## Phase 5: HTTP サーバー

- [x] `notebook/server.rs` 作成（または `mod.rs` 内に含める）
- [x] `serve_notebook(path, port, no_open)` — `tiny_http::Server` 起動
- [x] `GET /` → UI HTML を返す
- [x] `GET /api/notebook` → ノートブック JSON を返す
- [x] `GET /api/export` → Markdown テキストを返す（Content-Disposition: attachment）
- [x] `POST /api/run/{cell_id}` → セル実行 → JSON レスポンス
- [x] `POST /api/run-all` → 全セル実行 → JSON レスポンス
- [x] `POST /api/update/{cell_id}` → セル内容更新 → 保存
- [x] `POST /api/add-cell` → セル追加（type/content を body から受取）
- [x] `DELETE /api/cell/{cell_id}` → セル削除 → 保存
- [x] `NOTEBOOK_UI_HTML` 定数（inline HTML + CSS + JS）
- [x] `--no-open` フラグ対応（`open::that` 呼び出しを抑制）

---

## Phase 6: ドライバー関数

- [x] `cmd_notebook_new(name)` — `Notebook::new_notebook` → `.fav.nb` ファイル保存 → "created <name>.fav.nb" を表示
- [x] `cmd_notebook_run(path, no_cache)` — ロード → 全セル実行 → 保存 → サマリ表示
- [x] `cmd_notebook_serve(path, port, no_open)` — `serve_notebook` 呼び出し
- [x] `cmd_notebook_export(path, out)` — ロード → `export_markdown` → 出力
- [x] `cmd_notebook_check(path)` — ロード → `check_notebook` → エラー表示（exit 1 if any）

---

## Phase 7: CLI 配線

- [x] `fav/src/main.rs` に `mod notebook;` 追加
- [x] `Some("notebook")` アーム追加:
  - [x] `new <name>` → `cmd_notebook_new`
  - [x] `run [--no-cache] <file>` → `cmd_notebook_run`
  - [x] `serve [--port <n>] [--no-open] <file>` → `cmd_notebook_serve`
  - [x] `export [--out <path>] <file>` → `cmd_notebook_export`
  - [x] `check <file>` → `cmd_notebook_check`
- [x] HELP テキストに `notebook` コマンド群を記載

---

## Phase 8: テスト（目標 13 件）

### ユニットテスト（`notebook/mod.rs` 内）

| テスト | 内容 |
|--------|------|
| `notebook_parse_valid_json` | 正しい JSON をパースできる |
| `notebook_serialize_roundtrip` | パース → シリアライズが一致する |
| `notebook_new_creates_default_cells` | `new_notebook` が markdown + code の 2 セルを作成 |
| `run_cell_returns_int_value` | `42` を返すセルが `Value` output を返す |
| `run_cell_returns_error_on_type_error` | 型エラーが `Error` output になる |
| `run_cell_uses_context_from_previous` | 前セルの fn を参照できる |
| `run_cell_markdown_returns_none` | Markdown セルが `None` output を返す |
| `run_all_returns_all_outputs` | 全セル実行で output リストが返る |
| `export_markdown_has_favnir_blocks` | ` ```favnir ` ブロックが含まれる |
| `export_markdown_includes_output` | 実行済み出力が ` ``` ` ブロックに含まれる |

### 統合テスト（`driver.rs` — `notebook_tests` モジュール）

| テスト | 内容 |
|--------|------|
| `notebook_run_saves_output_to_file` | `cmd_notebook_run` が `.fav.nb` を更新する |
| `notebook_check_detects_type_error` | `cmd_notebook_check` が型エラーを出力する |
| `notebook_export_produces_markdown` | `cmd_notebook_export` が Markdown を生成する |

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存テスト（893 件）が全て pass
- [x] Notebook 13 件のテストが pass
- [x] `fav notebook new demo` が `demo.fav.nb` を作成する
- [x] `fav notebook run demo.fav.nb` が出力を計算してファイルに書き戻す
- [x] `fav notebook check demo.fav.nb` が型エラーを検出する
- [x] `fav notebook export demo.fav.nb` が Markdown を出力する
- [x] `fav notebook serve demo.fav.nb` が HTTP サーバーを起動してブラウザで開く
- [x] セル間コンテキスト共有（前セルの fn を後セルで呼べる）

---

## 実装メモ

- **セル結果ラッパー戦略**: コードが `fn`/`type`/`stage`/`seq` 定義で終わる場合は `OutputKind::None`。それ以外は末尾を式として `fn $cell_{id}_result() -> _ { <last_expr> }` でラップ。`_` の推論はチェッカーに任せる
- **セルコンテキストの型エラー伝播**: 前セルにエラーがあっても後セルのチェックを止めない（コンテキストにエラーセルのコードは含めない）
- **`tiny_http` リクエストの消費**: `request.respond(...)` は所有権を消費する→クローン不要
- **テーブルフォーマット**: ヘッダー行 `| key1 | key2 |` + セパレータ `|---|---|` + データ行
- **ID 採番の重複防止**: `add_cell` では `cells.iter().map(|c| &c.id).max()` + increment ではなく `cells.len() + 1` をゼロパディング（既存 ID が欠番でも単純増加で OK）
