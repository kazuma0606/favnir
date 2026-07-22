# Plan: v50.9.0 — 安定化・コードフリーズ（DX 3.0 前調整）

## 作業ステップ

### Step 1: `dx3-overview.mdx` 作成

**ファイルパス**: `site/content/docs/dx3-overview.mdx`

既存 `site/content/docs/tools/diagnostics.mdx` のスタイルに合わせて日本語ドキュメントとして作成。

**構成:**

```markdown
# Developer Experience 3.0 — 概要

DX 3.0 の目標・背景

---

## 何が変わったか

機能概要テーブル（バージョン × 機能 × リンク）

---

## 診断の統一（v50.1〜v50.3）
- suggestion / JSON / LSP / fav explain
リンク: diagnostics.mdx

## LSP インレイヒント（v50.4〜v50.5）
- bind 型ヒント / fn 戻り型ヒント / pipeline stage 型

## LSP ホバー強化（v50.6）
- Rune メソッドシグネチャ

## Trace & Watch（v50.7〜v50.8）
- fav run --trace 構造化ログ / --watch
リンク: trace-watch.mdx

---

## 関連ドキュメント
```

**テスト用キーワード**: `DX 3.0`, `Developer Experience`, `dx3` のいずれかを含める。

### Step 2: `driver.rs` — `v509000_tests` 追加

**対象**: `fav/src/driver.rs`（`v508000_tests` モジュールの直前）

3 件:

1. `cargo_toml_version_is_50_9_0`:
```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"50.9.0\""),
    "Cargo.toml version should be 50.9.0");
```

2. `dx3_overview_doc_exists`:
```rust
let content = include_str!("../../site/content/docs/dx3-overview.mdx");
assert!(content.len() >= 300, "dx3-overview.mdx is too short: {} bytes", content.len());
assert!(
    content.contains("DX 3.0") || content.contains("Developer Experience") || content.contains("dx3"),
    "dx3-overview.mdx must mention DX 3.0"
);
```

3. `code_freeze_v50_9_0`:
```rust
// v50.9.0 コードフリーズ宣言テスト — バージョンが 50.9.0 に固定されていることを assert
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"50.9.0\""), "code freeze: version must be 50.9.0");
```

`v508000_tests::cargo_toml_version_is_50_8_0` を削除（他 2 件は保持）。

### Step 3: `Cargo.toml` バージョン更新

`fav/Cargo.toml`: `version = "50.8.0"` → `version = "50.9.0"`

### Step 4: テスト・Lint 確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -5
```

期待: 3109 tests passed, 0 failed、clippy 警告 0

### Step 5: CHANGELOG・current.md 更新

- `CHANGELOG.md` に v50.9.0 エントリ追加
- `versions/current.md` を v50.9.0（3109 tests）に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.9.0 実績欄を更新
- `versions/v50-v55/v50.9.0/tasks.md` を COMPLETE に更新

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/dx3-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | v509000_tests 追加 + cargo_toml_version_is_50_8_0 削除 |
| `fav/Cargo.toml` | version → `50.9.0` |
| `CHANGELOG.md` | v50.9.0 エントリ追加 |
| `versions/current.md` | v50.9.0 更新 |
| `versions/roadmap/roadmap-v50.1-v51.0.md` | v50.9.0 実績欄更新 |
| `versions/v50-v55/v50.9.0/tasks.md` | COMPLETE に更新 |

---

## リスク・注意点

- `include_str!("../../site/content/docs/dx3-overview.mdx")` のパスは `fav/src/driver.rs` 起点で `../../` = `favnir/` ルート。`site/` は `fav/` と並列ディレクトリなので `../../site/...` が正しい。
- v50.9.0 は「コードフリーズ前調整」なので新機能追加は行わない。ドキュメント作成と安定化のみ。
- `code_freeze_v50_9_0` テストは Cargo.toml のバージョン確認のみに留める（実行時 clippy は CI で担保）。
