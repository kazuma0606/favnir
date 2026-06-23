# v21.7.0 実装計画 — `fav doc` サイト生成（docsite）

## 実装方針

既存の `cmd_doc()` / `doc_source_str()` を**一切変更しない**。
`--format site` / `--serve` の新コードパスを追加するのみ。

HTML 生成は外部クレート不使用。`md_to_html()` + `build_page_html()` をインライン実装。

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `md_to_html` 実装（driver.rs） | なし |
| T2 | `build_page_html` / `build_nav_html` 実装（driver.rs） | T1 |
| T3 | `cmd_doc_site` 実装（driver.rs） | T2 |
| T4 | `cmd_doc_serve` 実装（driver.rs） | T2 |
| T5 | `main.rs` CLI フラグ追加（`--format` / `--serve` / `--port` / `--no-open`） | T3, T4 |
| T6 | Cargo.toml バージョン更新 + `v216000_tests` に `#[ignore]` | なし |
| T7 | `v217000_tests` 追加（driver.rs） | T3 |
| T8 | CHANGELOG + `site/content/docs/tools/doc-site.mdx` | T7 |

---

## T1: `md_to_html(md: &str) -> String`

`driver.rs` に実装。行単位ステートマシン（コードブロック inside/outside フラグ）。

```rust
fn md_to_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    for line in md.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }
        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }
        // headings
        if let Some(text) = line.strip_prefix("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", inline_md(text)));
        } else if let Some(text) = line.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", inline_md(text)));
        } else if let Some(text) = line.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", inline_md(text)));
        } else if line.trim().is_empty() {
            // blank line — skip
        } else {
            html.push_str(&format!("<p>{}</p>\n", inline_md(line)));
        }
    }
    if in_code_block {
        html.push_str("</code></pre>\n"); // 閉じ忘れ対策
    }
    html
}

// インライン要素変換（`code` → <code>、**bold** → <strong>）
fn inline_md(text: &str) -> String { ... }

// < > & をエスケープ
fn html_escape(s: &str) -> String { ... }
```

**注意:** `inline_md` は正規表現不使用。単純な文字列走査（バッファ状態遷移）で実装。
`` ` `` → `<code>...</code>`、`**` → `<strong>...</strong>`。
- `html_escape` はテキストセグメント（タグ以外の部分）にのみ適用し、生成した `<code>`/`<strong>` タグ文字列には適用しない。
- 未クローズの `` ` `` / `**` を含むテキストは変換せずテキストとして出力（パニック不可）。

---

## T2: `build_page_html` / `build_nav_html`

```rust
fn build_nav_html(modules: &[String], current: Option<&str>) -> String
```
- `modules`: ファイル stem のリスト（例: `["pipeline", "transform"]`）
- `current`: アクティブなモジュール名（`class="active"` 付与）

```rust
fn build_page_html(title: &str, nav_html: &str, content_html: &str) -> String
```
- 自己完結 HTML（`<!DOCTYPE html>...`）
- インライン CSS（ダークテーマ）を `<style>` タグに埋め込む
- `<nav class="sidebar">` + `<main class="content">` の 2 カラムレイアウト

**CSS の文字列リテラル**:

```rust
const DOCSITE_CSS: &str = r#"
  body { background:#0d1117; color:#c9d1d9; font-family:sans-serif; margin:0; }
  .sidebar { width:220px; background:#161b22; position:fixed; top:0; bottom:0;
             overflow-y:auto; padding:16px; }
  .sidebar-title { font-weight:bold; font-size:1.1em; margin-bottom:12px; }
  .sidebar ul { list-style:none; padding:0; margin:0; }
  .sidebar li a { color:#58a6ff; text-decoration:none; display:block; padding:4px 0; }
  .sidebar li a:hover { text-decoration:underline; }
  .sidebar li a.active { color:#f0f6fc; font-weight:bold; }
  .content { margin-left:236px; padding:32px; max-width:860px; }
  h1 { border-bottom:1px solid #30363d; padding-bottom:8px; }
  pre { background:#161b22; padding:12px; border-radius:6px; overflow-x:auto; }
  code { font-family:monospace; font-size:0.9em; }
  p { line-height:1.6; }
"#;
```

### index.html

`title = "Favnir Docs"` で、コンテンツ部分にモジュール一覧のリンクを出力:

```html
<h1>Favnir Docs</h1>
<ul>
  <li><a href="pipeline.html">pipeline</a></li>
  <li><a href="transform.html">transform</a></li>
</ul>
```

---

## T3: `cmd_doc_site(path: &str, out_dir: &str)`

```rust
pub fn cmd_doc_site(path: &str, out_dir: &str) {
    let entries = collect_fav_files_recursive(Path::new(path));   // 既存ヘルパー流用
    let mut modules: Vec<(String, String)> = vec![];  // (stem, md_content)

    for entry in &entries {
        let src = std::fs::read_to_string(entry).unwrap_or_default();
        let md = crate::compiler_fav_runner::doc_source_str(&src).unwrap_or_default();
        let stem = entry.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "out".to_string());
        modules.push((stem, md));
    }

    let stems: Vec<String> = modules.iter().map(|(s, _)| s.clone()).collect();

    std::fs::create_dir_all(out_dir)...;

    // index.html
    let nav = build_nav_html(&stems, None);
    let index_content = build_index_content_html(&stems);
    let index_html = build_page_html("Favnir Docs", &nav, &index_content);
    write(out_dir/index.html, index_html);

    // per-module pages
    for (stem, md) in &modules {
        let nav = build_nav_html(&stems, Some(stem));
        let content = md_to_html(md);
        let page = build_page_html(stem, &nav, &content);
        write(out_dir/<stem>.html, page);
        println!("doc: {} → {}/{}.html", stem, out_dir, stem);
    }
}
```

---

## T4: `cmd_doc_serve(path: &str, port: u16, no_open: bool)`

```rust
pub fn cmd_doc_serve(path: &str, port: u16, no_open: bool) {
    // サイト HTML をメモリ上に構築
    let entries = collect_entries(path);
    let mut pages: HashMap<String, String> = HashMap::new();  // stem → html

    // ... (cmd_doc_site と同様のロジックで HashMap に格納)

    // bind を先に行い、成功確認後にブラウザを開く（ポート競合時にブラウザが空ページを開かないようにする）
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))...;

    let url = format!("http://localhost:{}", port);
    println!("Favnir doc server running at {}", url);
    if !no_open { let _ = open::that(&url); }
    for stream in listener.incoming() {
        let stream = stream?;
        handle_doc_serve_request(stream, &pages);
    }
}

fn handle_doc_serve_request(stream: TcpStream, pages: &HashMap<String, String>) {
    // GET / → pages["index"]
    // GET /index or GET /index.html → pages["index"]
    // GET /<stem> or GET /<stem>.html → pages[stem]（.html 拡張子を除去してキー参照）
    // その他 → 404
    // HTTP/1.0 レスポンス（Content-Type: text/html; charset=utf-8）
}
```

**`open` クレート**: 既存の `cmd_docs` で使われている `open = "3"` を流用（Cargo.toml 変更不要）。

---

## T5: `main.rs` CLI フラグ追加

`Some("doc")` ブランチを拡張:

```rust
Some("doc") => {
    // --builtins (既存、変更なし)
    if args.iter().any(|a| a == "--builtins") { ... }

    // --serve (新規)
    if args.iter().any(|a| a == "--serve") {
        let port = args.windows(2)
            .find(|w| w[0] == "--port")
            .and_then(|w| w[1].parse::<u16>().ok())
            .unwrap_or(8080);
        let no_open = args.iter().any(|a| a == "--no-open");
        let mut path = ".".to_string();
        // while ループで --port の値トークンをスキップしてから位置引数を取得
        let mut i = 2usize;
        while i < args.len() {
            match args[i].as_str() {
                "--serve" | "--no-open" => { i += 1; }
                "--port" => { i += 2; }  // 値トークンもスキップ
                other if !other.starts_with("--") => { path = other.to_string(); break; }
                _ => { i += 1; }
            }
        }
        cmd_doc_serve(&path, port, no_open);
        return;
    }

    // --format site (新規) / --format markdown (既存と同等)
    let format = args.windows(2)
        .find(|w| w[0] == "--format")
        .map(|w| w[1].as_str())
        .unwrap_or("markdown");
    let mut path = ".".to_string();
    let mut out_dir = "docs".to_string();
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => { i += 2; }  // 消費済みスキップ
            "--out" => { out_dir = args[i+1].clone(); i += 2; }
            other if !other.starts_with("--") => { path = other.to_string(); i += 1; }
            _ => { i += 1; }
        }
    }
    match format {
        "site" => cmd_doc_site(&path, &out_dir),
        _      => cmd_doc(&path, &out_dir),   // "markdown" + デフォルト（既存動作）
    }
}
```

---

## T6: Cargo.toml バージョン更新

```toml
version = "21.7.0"
```

`driver.rs` の `v216000_tests` モジュールの `version_is_21_6_0` テストに `#[ignore]` を追加。

---

## T7: `v217000_tests` — 8 件

```rust
#[cfg(test)]
mod v217000_tests {
    use super::*;
    use std::path::Path;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
    }

    #[test]
    fn version_is_21_7_0() {
        let cargo = std::fs::read_to_string(
            root().join("fav/Cargo.toml")
        ).unwrap();
        assert!(cargo.contains("\"21.7.0\""), "Cargo.toml should contain 21.7.0");
    }

    #[test]
    fn doc_site_generates_index_html() {
        let dir = tempdir();
        // 単純な .fav ソースを tmp に書いてから cmd_doc_site 呼び出し
        // → dir/index.html が存在することを確認
        ...
        assert!(Path::new(&dir).join("index.html").exists());
    }

    #[test]
    fn doc_site_generates_module_html() { ... }

    #[test]
    fn doc_site_html_has_nav_sidebar() {
        // 生成 HTML に class="sidebar" が含まれる
        ...
        assert!(html.contains("sidebar"));
    }

    #[test]
    fn doc_site_html_has_fn_doc() {
        // /// コメントのある fn → HTML に doc テキストが含まれる
        ...
    }

    #[test]
    fn doc_serve_cmd_exists() {
        // コンパイル確認のみ（関数シグネチャのアサーション）
        let _: fn(&str, u16, bool) = cmd_doc_serve;
    }

    #[test]
    fn changelog_has_v21_7_0() {
        let cl = std::fs::read_to_string(root().join("CHANGELOG.md")).unwrap();
        assert!(cl.contains("[v21.7.0]"));
    }

    #[test]
    fn doc_site_mdx_exists() {
        assert!(root().join("site/content/docs/tools/doc-site.mdx").exists());
    }
}
```

**注意:** `doc_site_generates_*` テストは `tempfile` クレートを使わず、
`std::env::temp_dir()` + ランダムサフィックスで一時ディレクトリを作成する。
`tempfile` クレートはプロジェクトの依存に含まれていないため。

```rust
struct TempDir(std::path::PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}
fn make_temp_dir(prefix: &str) -> TempDir {
    let dir = std::env::temp_dir().join(format!("{}{}", prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
    std::fs::create_dir_all(&dir).unwrap();
    TempDir(dir)
}
// 使用例: let tmp = make_temp_dir("fav_docsite_test_"); cmd_doc_site(src, tmp.0.to_str().unwrap());
```

---

## T8: CHANGELOG + MDX

### CHANGELOG.md エントリ

```markdown
## [v21.7.0] — 2026-06-20

### Added
- `fav doc --format site src/ --out docs/` — `///` コメントから静的 HTML ドキュメントサイトを生成
- `fav doc --serve src/` — ローカルプレビューサーバー（デフォルト: http://localhost:8080）
- `md_to_html()` — Markdown → HTML 変換（外部クレートなし）
- `--port <num>` / `--no-open` フラグ
```

### `site/content/docs/tools/doc-site.mdx`

Playground / LSP と同様の形式でドキュメントページを作成。
コマンド例・HTML 出力構造・ローカルプレビューの使い方を記載。

---

## 実装上の注意点

### `doc_source_str` の戻り値

`doc_source_str` が `Err` を返す場合は空文字列 `""` として扱う（`unwrap_or_default()`）。
ソース解析エラーを docsite 生成の致命的エラーとしない。

### HTML エスケープ

`md_to_html` のコードブロック内では `<`, `>`, `&` をエスケープ必須。
ユーザーのソースコードが直接 HTML に埋め込まれるため XSS を防ぐ。

```rust
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}
```

### `inline_md` の `<` エスケープ

インライン MD 変換でも型シグネチャ（`List<Int>` 等）が含まれるため、
`<` → `&lt;` のエスケープを先に適用してから `` ` `` / `**` 変換する。

### `--serve` のブロッキング

`cmd_doc_serve` は `for stream in listener.incoming()` でブロック。
Ctrl-C でプロセスを終了する設計で良い（シグナルハンドラ不要）。

### `open` クレートの条件付きコンパイル

`cmd_docs`（既存）が使っている `open = "3"` を流用。
`TcpListener` / `fs::write` / `open::that` はいずれも WASM 非対応のため、
`cmd_doc_serve` と `cmd_doc_site` 全体を `#[cfg(not(target_arch = "wasm32"))]` で囲む（既存パターンに合わせる）。

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `inline_md` の `` ` `` / `**` ネストで無限ループ | 各 pass を独立した前後 split で実装。再帰不使用 |
| `html_escape` を忘れてコードブロック内に `<T>` → XSS | `in_code_block = true` 時は必ず `html_escape` を通す |
| `--serve` 起動時に `path` が空ディレクトリ → modules が空 | `modules.is_empty()` 時は index.html のみ生成して警告表示 |
| `open::that` が CI 環境で失敗 | 戻り値を `let _ = ...` で無視（既存 `cmd_docs` と同様） |
| テスト用一時ディレクトリが残留 | `drop` 時に削除するガード構造体か、テスト後に `fs::remove_dir_all` |
