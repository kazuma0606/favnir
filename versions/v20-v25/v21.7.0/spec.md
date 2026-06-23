# v21.7.0 仕様書 — `fav doc` サイト生成（docsite）

## 概要

`fav doc` コマンドに `--format site` / `--serve` を追加し、`///` コメントから
mdBook ライクな静的 HTML ドキュメントサイトを自動生成する。

ロードマップ v21.7 の機能:
- `fav doc --format site src/ --out docs/` → `docs/index.html` + per-module HTML
- `fav doc --serve src/` → ローカルプレビューサーバー（http://localhost:8080）

**スコープ外（v21.7 では実装しない）:**
- Markdown 以外の形式（PDF、EPUB）
- 検索機能（全文検索インデックス）
- i18n / 多言語対応
- 外部 CSS フレームワーク（Tailwind 等）の利用 — インライン CSS のみ
- GitHub Pages / Netlify への自動デプロイ
- `fav publish` との統合（将来バージョンで `cmd_doc_site` API を活用予定）

---

## アーキテクチャ

### 既存の `fav doc` との関係

| コマンド | 動作 | 変更 |
|---|---|---|
| `fav doc src/ --out docs/` | `.md` ファイル生成（既存） | **変更なし** |
| `fav doc --format site src/ --out docs/` | 静的 HTML サイト生成 | **新規追加** |
| `fav doc --serve src/` | ローカルプレビューサーバー | **新規追加** |

既存の `cmd_doc()` + `doc_source_str()` は変更しない。
`--format site` のみ新しいコードパスを通る。

### HTML サイト構造

```
docs/
  index.html          ← モジュール一覧 + サイドバー付きトップページ
  pipeline.html       ← pipeline.fav のドキュメントページ
  transform.html      ← transform.fav のドキュメントページ
  ...
```

すべての HTML は自己完結（外部リソースなし）。インライン CSS で統一。

### ページ構成

```
┌─ sidebar ─────────┬─ content ────────────────────────┐
│ Favnir Docs       │  # pipeline                       │
│                   │                                   │
│ • index           │  ## stage Double                  │
│ • pipeline  ←     │  Double: Int -> Int               │
│ • transform       │  2倍にする stage                  │
│                   │                                   │
│                   │  ## stage AddOne                  │
│                   │  AddOne: Int -> Int               │
└───────────────────┴───────────────────────────────────┘
```

### Markdown → HTML 変換（インライン実装）

外部クレート不使用。`md_to_html(md: &str) -> String` を `driver.rs` に実装。

| Markdown 記法 | HTML 変換 |
|---|---|
| `# Heading` | `<h1>Heading</h1>` |
| `## Heading` | `<h2>Heading</h2>` |
| ` ```...``` ` | `<pre><code>...</code></pre>` |
| `` `inline` `` | `<code>inline</code>` |
| `**bold**` | `<strong>bold</strong>` |
| それ以外の行 | `<p>...</p>` |

---

## 変更ファイル一覧

### fav（Rust）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/main.rs` | 更新 | `fav doc` CLI に `--format` / `--serve` / `--port` フラグ追加 |
| `fav/src/driver.rs` | 更新 | `cmd_doc_site` / `cmd_doc_serve` / `md_to_html` / `build_docsite_html` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "21.6.0"` → `"21.7.0"` |
| `fav/src/driver.rs` | 更新 | `v217000_tests` 追加（8件） |

### CHANGELOG / docs

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `CHANGELOG.md` | 更新 | v21.7.0 エントリ追加 |
| `site/content/docs/tools/doc-site.mdx` | 新規 | `fav doc --format site` / `--serve` ドキュメント |

---

## API 設計

### `cmd_doc_site(path: &str, out_dir: &str)`

```rust
pub fn cmd_doc_site(path: &str, out_dir: &str)
```

1. `path` のソースファイルを収集
2. 各ファイルに `doc_source_str()` を適用して Markdown 取得
3. `md_to_html()` で HTML コンテンツに変換
4. `build_page_html(title, nav_html, content_html) -> String` でページ HTML 生成
5. `docs/index.html` + `docs/<stem>.html` を書き出し

### `cmd_doc_serve(path: &str, port: u16)`

```rust
pub fn cmd_doc_serve(path: &str, port: u16)
```

1. サイト HTML をメモリ上に構築
2. `TcpListener::bind(format!("127.0.0.1:{port}"))` でサーバー起動
3. `GET /` → `index.html` を返す
4. `GET /<stem>` → `<stem>.html` を返す（`.html` 拡張子なしも受け付ける）
5. その他 → 404
6. `open::that(url)` でブラウザを自動オープン（`--no-open` フラグで抑制）

### `md_to_html(md: &str) -> String`

```rust
fn md_to_html(md: &str) -> String
```

行単位で処理。コードブロック（` ``` `）はまとめて `<pre><code>` タグに変換。

### `build_page_html(title: &str, nav_html: &str, content_html: &str) -> String`

```rust
fn build_page_html(title: &str, nav_html: &str, content_html: &str) -> String
```

自己完結 HTML テンプレートを返す。インライン CSS でダークテーマ。

---

## CLI 仕様

### `fav doc --format site src/ --out docs/`

```
フラグ:
  --format <markdown|site>  出力形式。デフォルト: markdown（既存動作）
  --out <dir>               出力ディレクトリ。デフォルト: docs/
  <path>                    入力ファイルまたはディレクトリ。デフォルト: .

例:
  fav doc --format site src/ --out docs/
  fav doc --format site src/pipeline.fav --out docs/
```

### `fav doc --serve src/`

```
フラグ:
  --serve                   プレビューサーバーを起動
  --port <num>              ポート番号。デフォルト: 8080
  --no-open                 ブラウザを自動オープンしない
  <path>                    入力ファイルまたはディレクトリ。デフォルト: .

例:
  fav doc --serve src/
  fav doc --serve src/ --port 9000 --no-open
```

---

## HTML デザイン仕様

### インライン CSS（ダークテーマ）

```css
body      { background: #0d1117; color: #c9d1d9; font-family: sans-serif; margin: 0; }
.sidebar  { width: 220px; background: #161b22; position: fixed; top: 0; bottom: 0;
            overflow-y: auto; padding: 16px; }
.content  { margin-left: 236px; padding: 32px; max-width: 860px; }
a         { color: #58a6ff; }
pre       { background: #161b22; padding: 12px; border-radius: 6px; overflow-x: auto; }
code      { font-family: monospace; font-size: 0.9em; }
h1        { border-bottom: 1px solid #30363d; padding-bottom: 8px; }
```

### サイドバーナビゲーション HTML

```html
<nav class="sidebar">
  <div class="sidebar-title">Favnir Docs</div>
  <ul>
    <li><a href="index.html">index</a></li>
    <li><a href="pipeline.html">pipeline</a></li>
    <li><a href="transform.html">transform</a></li>
  </ul>
</nav>
```

---

## テスト一覧（v217000_tests、8件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_7_0` | Cargo.toml に `"21.7.0"` が含まれる |
| `doc_site_generates_index_html` | `cmd_doc_site` が `index.html` を生成する |
| `doc_site_generates_module_html` | `cmd_doc_site` が `<stem>.html` を生成する |
| `doc_site_html_has_nav_sidebar` | 生成 HTML に `class="sidebar"` が含まれる |
| `doc_site_html_has_fn_doc` | `///` コメントのある fn のドキュメントが HTML に含まれる |
| `doc_serve_cmd_exists` | `cmd_doc_serve` 関数が存在する（コンパイル確認） |
| `changelog_has_v21_7_0` | `CHANGELOG.md` に `[v21.7.0]` が含まれる |
| `doc_site_mdx_exists` | `site/content/docs/tools/doc-site.mdx` が存在する |

---

## 完了条件

- [ ] `fav doc --format site src/ --out docs/` が `docs/index.html` + per-module HTML を生成する
- [ ] 生成 HTML が自己完結（外部リソース参照なし）
- [ ] `md_to_html` が `#`/`##`/コードブロック/インラインコードを正しく変換する
- [ ] `fav doc --serve src/` が `http://localhost:8080` でサーバーを起動する（手動確認 — 自動テスト対象外）
- [ ] `--format` なしの既存 `fav doc` 動作が変わらない（リグレッションなし）
- [ ] `cargo test v217000` — 8/8 PASS
- [ ] `cargo test` — リグレッションなし（1824 件以上合格）
- [ ] `CHANGELOG.md` に v21.7.0 エントリ
- [ ] `site/content/docs/tools/doc-site.mdx` 作成済み
