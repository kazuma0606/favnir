# v21.7.0 — `fav doc` サイト生成（docsite）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `md_to_html` / `html_escape` / `inline_md` 実装（driver.rs）

- [x] **事前確認**: `grep -n "pub fn cmd_doc\b" fav/src/driver.rs` で挿入位置を確認
- [x] `html_escape(s: &str) -> String` を実装（`&` → `&amp;`、`<` → `&lt;`、`>` → `&gt;`）
- [x] `inline_md(text: &str) -> String` を実装:
  - **テキストセグメント（タグ以外の部分）にのみ `html_escape` を適用**。生成した `<code>`/`<strong>` タグ文字列には `html_escape` を通さない
  - `` ` `` で囲まれた部分 → `<code>html_escape(inner)</code>`
  - `**...**` で囲まれた部分 → `<strong>html_escape(inner)</strong>`
  - 未クローズの `` ` `` / `**` はタグ変換せずそのままテキストとして出力（パニック禁止）
- [x] 未クローズ `` ` `` / `**` で `inline_md` がパニックしないことをコードで確認
- [x] `md_to_html(md: &str) -> String` を実装:
  - コードブロック（` ``` ` 行）の inside/outside を `in_code_block: bool` で管理
  - コードブロック内: `html_escape` + 改行保持
  - `### ` → `<h3>`、`## ` → `<h2>`、`# ` → `<h1>`
  - 空行: スキップ（`<br>` 不要）
  - それ以外: `<p>...</p>`
  - 閉じ忘れコードブロック対策（ファイル末尾に `</code></pre>` を追加）
- [x] コードブロック内の `<` / `>` が `&lt;` / `&gt;` にエスケープされることを目視確認

---

### T2: `DOCSITE_CSS` / `build_nav_html` / `build_page_html` / `build_index_content_html` 実装

T1 完了後に着手。

- [x] `const DOCSITE_CSS: &str` をダークテーマ CSS 文字列として定義
- [x] `build_nav_html(modules: &[String], current: Option<&str>) -> String` を実装:
  - `index.html` へのリンク（常に先頭）
  - 各 module の `<stem>.html` へのリンク
  - `current` に一致する場合は `class="active"`
- [x] `build_page_html(title: &str, nav_html: &str, content_html: &str) -> String` を実装:
  - `<!DOCTYPE html>` から始まる自己完結 HTML
  - `<style>` タグに `DOCSITE_CSS` を埋め込む
  - `<nav class="sidebar">` + `<main class="content">`
- [x] `build_index_content_html(modules: &[String]) -> String` を実装:
  - `<h1>Favnir Docs</h1>` + モジュール一覧の `<ul>`
  - 各モジュールを `<a href="<stem>.html">` でリンク

---

### T3: `cmd_doc_site(path: &str, out_dir: &str)` 実装

T2 完了後に着手。

- [x] **事前確認**: `cmd_doc` 関数の直後に追加する位置を確認
- [x] ソースファイルを収集（既存 `collect_fav_files_recursive` ヘルパーを流用）
- [x] 各ファイルを `doc_source_str` でドキュメント Markdown に変換（エラーは `""`）
- [x] `stems` リストを構築
- [x] `out_dir` を `std::fs::create_dir_all` で作成
- [x] `index.html` を生成・書き出し
- [x] per-module HTML を生成・書き出し
- [x] 各ファイル書き出し時に `println!("doc: {} → {}/{}.html", stem, out_dir, stem)` を出力

---

### T4: `cmd_doc_serve(path: &str, port: u16, no_open: bool)` 実装

T2 完了後に着手（T3 と並列可）。

- [x] `cmd_doc_site` と同様のロジックで HTML を `HashMap<String, String>` に構築:
  - キー: `"index"` / `"<stem>"`
  - 値: 完全な HTML 文字列
- [x] `TcpListener::bind(format!("127.0.0.1:{}", port))` でサーバー起動（**`bind` を先に行い、成功確認後に `open::that` を呼ぶ**）
- [x] `println!("Favnir doc server running at http://localhost:{}", port)` を出力
- [x] `no_open` が false なら `open::that(&url)` を呼ぶ（戻り値は `let _ = ...` で無視）
- [x] リクエストハンドラ `handle_doc_serve_request(stream: TcpStream, pages: &HashMap<String, String>)` を実装:
  - `GET /` → `pages["index"]`
  - `GET /index` or `GET /index.html` → `pages["index"]`
  - `GET /<stem>` or `GET /<stem>.html` → `pages[stem]`（`.html` を除去してキー参照）
  - それ以外 → `HTTP/1.0 404 Not Found`
  - レスポンスヘッダ: `Content-Type: text/html; charset=utf-8`
- [x] `cmd_doc_serve` と `cmd_doc_site` 全体を `#[cfg(not(target_arch = "wasm32"))]` で囲む（`TcpListener` / `fs::write` は WASM 非対応）
- [x] `modules.is_empty()` 時は `eprintln!("warning: no .fav files found in '{}'", path)` して続行

---

### T5: `main.rs` — `fav doc` CLI フラグ追加

T3, T4 完了後に着手。

- [x] **事前確認**: 現在の `Some("doc")` ブランチを読んで変更前状態を確認
- [x] `--serve` フラグ検出ブランチを追加（`--builtins` ブランチの直後）:
  - `--port <num>` を解析（デフォルト: `8080`）
  - `--no-open` フラグを検出
  - `while` ループで `--port` の値トークンをスキップしながら位置引数を `path` として取得（`--port 9000 src/` の順で渡しても `"9000"` が `path` に入らないよう注意）
  - `cmd_doc_serve(&path, port, no_open)` を呼ぶ
- [x] `--format` フラグ解析を追加:
  - `"site"` → `cmd_doc_site(&path, &out_dir)`
  - それ以外 / 未指定 → `cmd_doc(&path, &out_dir)`（既存動作）
- [x] `--format` / `--serve` / `--port` / `--no-open` トークンが位置引数として `path` に入らないよう while ループの `match` アームを更新

---

### T6: Cargo.toml + `#[ignore]` 追加

- [x] `fav/Cargo.toml` の `version = "21.6.0"` → `"21.7.0"` に変更
- [x] `driver.rs` の `v216000_tests` モジュール内 `version_is_21_6_0` テストに `#[ignore]` を追加
- [x] 既存 `v216000_tests` モジュールの `#[cfg(test)]` 有無を確認し、`v217000_tests` でスタイルを統一する

---

### T7: `v217000_tests` — 8 件追加（driver.rs）

- [x] **事前確認**: `v216000_tests` モジュールの末尾を確認して追加位置を決める
- [x] `TempDir` struct（`Drop` で `fs::remove_dir_all`）と `make_temp_dir(prefix: &str) -> TempDir` ヘルパーを実装:
  - `std::env::temp_dir().join(format!("{}{}", prefix, subsec_nanos))`
  - `Drop` 実装で `fs::remove_dir_all` を呼ぶ（テスト終了時に自動クリーンアップ）
  - `tmp.0` でパスを参照: `cmd_doc_site(src_dir, tmp.0.to_str().unwrap())`
- [x] 以下 8 件のテストを実装:
  - `version_is_21_7_0` — `Cargo.toml` に `"21.7.0"` が含まれる
  - `doc_site_generates_index_html` — `cmd_doc_site` が `index.html` を生成する
  - `doc_site_generates_module_html` — `cmd_doc_site` が `<stem>.html` を生成する
  - `doc_site_html_has_nav_sidebar` — 生成 HTML に `"sidebar"` が含まれる
  - `doc_site_html_has_fn_doc` — `///` コメントのある fn のドキュメントが HTML に含まれる
  - `doc_serve_cmd_exists` — `let _: fn(&str, u16, bool) = cmd_doc_serve;` でコンパイル確認
  - `changelog_has_v21_7_0` — `CHANGELOG.md` に `[v21.7.0]` が含まれる
  - `doc_site_mdx_exists` — `site/content/docs/tools/doc-site.mdx` が存在する
- [x] `cargo test v217000` — 8/8 PASS を確認

---

### T8: CHANGELOG + `site/content/docs/tools/doc-site.mdx`

- [x] `CHANGELOG.md` の先頭に v21.7.0 エントリを追加:
  ```
  ## [v21.7.0] — 2026-06-20
  ### Added
  - `fav doc --format site src/ --out docs/` — 静的 HTML ドキュメントサイト生成
  - `fav doc --serve src/` — ローカルプレビューサーバー（デフォルト http://localhost:8080）
  - `--port <num>` / `--no-open` フラグ
  ```
- [x] `site/content/docs/tools/doc-site.mdx` を新規作成（以下のセクションを含む）:
  - 概要（`///` コメントから HTML サイト生成）
  - コマンドリファレンス（`--format site` / `--serve` / `--port` / `--no-open`）
  - 出力構造（`docs/index.html` + per-module HTML）
  - ローカルプレビューの使い方
  - 既存 `fav doc`（Markdown 出力）との違い

---

## テスト一覧（v217000_tests、8件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_7_0` | Cargo.toml に `"21.7.0"` が含まれる |
| `doc_site_generates_index_html` | `cmd_doc_site` が `index.html` を生成する |
| `doc_site_generates_module_html` | `cmd_doc_site` が `<stem>.html` を生成する |
| `doc_site_html_has_nav_sidebar` | 生成 HTML に `"sidebar"` が含まれる |
| `doc_site_html_has_fn_doc` | `///` コメント付き fn のドキュメントが HTML に含まれる |
| `doc_serve_cmd_exists` | `cmd_doc_serve` 関数が存在する（コンパイル確認） |
| `changelog_has_v21_7_0` | `CHANGELOG.md` に `[v21.7.0]` が含まれる |
| `doc_site_mdx_exists` | `site/content/docs/tools/doc-site.mdx` が存在する |

---

## テストカバレッジに関する注記

`cmd_doc_serve` のネットワーク動作（HTTP レスポンス）は Rust テストでは検証しない。
以下の手動確認で代替する:

1. `fav doc --serve src/` を起動して `http://localhost:8080` でブラウザ表示確認
2. `GET /pipeline` と `GET /pipeline.html` の両方でページが返ることを確認
3. 存在しない URL で 404 が返ることを確認（`curl http://localhost:8080/nonexistent`）

---

## 完了条件チェックリスト

- [x] `cargo test v217000` — 8/8 PASS
- [x] `cargo test` — リグレッションなし（1824 件以上合格）
- [x] `fav doc --format site src/ --out docs/` でブラウザ表示可能な HTML が生成される（手動確認）
- [x] `fav doc --serve src/` が `http://localhost:8080` でサーバーを起動する（手動確認 — 自動テスト対象外）
- [x] コードブロック内の `List<Int>` が `List&lt;Int&gt;` としてエスケープされる（目視確認）
- [x] `fav doc --format markdown src/`（`--format` 省略時と同等）が既存動作と変わらない
- [x] `CHANGELOG.md` に v21.7.0 エントリ
- [x] `fav/Cargo.toml` version が `21.7.0`
- [x] `site/content/docs/tools/doc-site.mdx` 作成済み

---

## 優先度

```
T1（md_to_html）        ← 最初。すべての基盤
T2（HTML ビルダー）     ← T1 後
T3（cmd_doc_site）      ← T2 後（T4 と並列可）
T4（cmd_doc_serve）     ← T2 後（T3 と並列可）
T5（main.rs CLI）       ← T3, T4 後
T6（Cargo.toml）        ← いつでも
T7（tests）             ← T3 完了後（HTML 生成が確定してから）
T8（CHANGELOG + MDX）   ← 最後
```
