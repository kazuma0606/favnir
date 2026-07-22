# Plan: v50.8.0 — ドキュメントサイト DX 3.0 記事

## 作業ステップ

### Step 1: `diagnostics.mdx` 作成

**ファイルパス**: `site/content/docs/tools/diagnostics.mdx`

既存の `tools/lsp.mdx`・`tools/profiling.mdx` の構造を参考に作成。
日本語ドキュメントスタイル（lsp.mdx に合わせる）。

**構成:**

```markdown
# 診断出力 (Diagnostics)

Favnir ...

## エラーコードと修正提案 (suggestion)
...
## fav check --json
...出力例...
## LSP 診断 (textDocument/publishDiagnostics)
...
## fav explain --error
...使い方...
## エラーコード一覧
...
```

**テスト用キーワード**: `diagnostics`, `fav explain`, `suggestion` のいずれかを含める。

### Step 2: `trace-watch.mdx` 作成

**ファイルパス**: `site/content/docs/tools/trace-watch.mdx`

**構成:**

```markdown
# トレース & ウォッチ (Trace & Watch)

...
## fav run --trace
...出力例...
## fav run --debug (DAP) との違い
...
## --watch フィールド追跡（将来対応）
...
## ユースケース: パイプラインデバッグ
...
```

**テスト用キーワード**: `--trace`, `[trace]`, `trace-watch` のいずれかを含める。

### Step 3: `driver.rs` — `v508000_tests` 追加

**対象**: `fav/src/driver.rs`（`v507000_tests` モジュールの直前 = 最新バージョンテストが先頭に来る慣例順）

3 件:

1. `cargo_toml_version_is_50_8_0`:
```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"50.8.0\""));
```

2. `docs_diagnostics_page_exists`:
```rust
let content = include_str!("../../site/content/docs/tools/diagnostics.mdx");
assert!(content.len() >= 300, "diagnostics.mdx is too short: {} bytes", content.len());
assert!(
    content.contains("diagnostics") || content.contains("fav explain") || content.contains("suggestion"),
    "diagnostics.mdx must contain docs keywords"
);
```

3. `docs_trace_watch_page_exists`:
```rust
let content = include_str!("../../site/content/docs/tools/trace-watch.mdx");
assert!(content.len() >= 300, "trace-watch.mdx is too short: {} bytes", content.len());
assert!(
    content.contains("--trace") || content.contains("[trace]") || content.contains("trace-watch"),
    "trace-watch.mdx must contain trace docs keywords"
);
```

`v507000_tests::cargo_toml_version_is_50_7_0` を削除（他 2 件は保持）。

### Step 4: `Cargo.toml` バージョン更新

`fav/Cargo.toml`: `version = "50.7.0"` → `version = "50.8.0"`

### Step 5: テスト・Lint 確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -5
```

期待: 3107 tests passed, 0 failed

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/tools/diagnostics.mdx` | 新規作成 |
| `site/content/docs/tools/trace-watch.mdx` | 新規作成 |
| `fav/src/driver.rs` | v508000_tests 追加 + cargo_toml_version_is_50_7_0 削除 |
| `fav/Cargo.toml` | version → `50.8.0` |
| `CHANGELOG.md` | v50.8.0 エントリ追加 |

---

## リスク・注意点

- `include_str!("../../site/content/docs/tools/diagnostics.mdx")` のパスは `fav/src/driver.rs` 起点で `../../` = `favnir/` ルート。`site/` は `fav/` と並列ディレクトリなので `../../site/...` が正しい（v24.7.0 で確立済みパターン）。
- MDX ファイルの内容は `cargo test` でコンパイル時に組み込まれるため、存在しない場合はコンパイルエラーになる（テスト実行前に検出可能）。
- `--watch` CLI フラグの説明は「v50.8.0 時点では API のみ、CLI フラグは将来対応」と明示し、誤解を招かないようにする。
