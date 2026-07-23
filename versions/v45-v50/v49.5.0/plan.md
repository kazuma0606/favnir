# Plan: v49.5.0 — cookbook 更新

## 作業順序

### Step 1: cookbook MDX ファイル 3 件作成

`site/content/cookbook/` ディレクトリは既存（他のレシピあり）。新規ファイルのみ追加。

#### `return-guard-pattern.mdx`
- frontmatter: `title: "Return Guard Pattern"` / `category: "クックブック"` / `description`
- 本文に英語 `"return"` と `"guard"` を含む（テスト assert 条件）
- `return expr if condition` のコード例を含む

#### `inline-testing.mdx`
- frontmatter: `title: "Inline Testing"` / `category: "クックブック"` / `description`
- 本文に `"#[test]"` と `"assert"` を含む（テスト assert 条件）
- `fav test` コマンドの実行例を含む

#### `modular-pipelines.mdx`
- frontmatter: `title: "Modular Pipelines"` / `category: "クックブック"` / `description`
- 新 import 構文のコード例を含む（テスト対象外だが作成必須）

### Step 2: `v495000_tests` 追加

`v494000_tests` の直前に挿入（2テスト）:
- `cookbook_return_guard_exists` — `return-guard-pattern.mdx` に `"return"` と `"guard"` が含まれる
- `cookbook_fav_test_exists` — `inline-testing.mdx` に `"#[test]"` と `"assert"` が含まれる

`include_str!` パス:
- `"../../site/content/cookbook/return-guard-pattern.mdx"`
- `"../../site/content/cookbook/inline-testing.mdx"`

### Step 3: `Cargo.toml` version 更新

`"49.4.0"` → `"49.5.0"`

### Step 4: 完了処理

- `cargo test` 3079 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.5.0 エントリ追加
- `versions/current.md` 更新（v49.5.0・3079 tests・進行中 v49.6.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.5.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `site/content/cookbook/return-guard-pattern.mdx` | 新規作成 |
| `site/content/cookbook/inline-testing.mdx` | 新規作成 |
| `site/content/cookbook/modular-pipelines.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v495000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.5.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.5.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `site/content/docs/syntax/return.mdx` | v49.4.0 で作成済み |
| `site/content/docs/modules/import.mdx` | v49.4.0 で作成済み |
| 既存 cookbook ファイル（70+ 件） | 本バージョンのスコープ外 |
