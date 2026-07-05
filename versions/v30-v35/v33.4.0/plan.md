# v33.4.0 — Plan: Arrow 列指向統合 確認

## アプローチ

「確認・記録」パターン（v33.1〜v33.3 踏襲）。
v19.5.0 で実装済みの Arrow 列指向統合を `v334000_tests`（4 件）で記録する。
新規実装なし。

---

## 実装手順

### Step 1: 前提確認
- `Cargo.toml` version = `33.3.0` であること
- `cargo test` が 2508 passed, 0 failed であること
- `mod v334000_tests` が `driver.rs` に存在しないこと
- `v195000_tests` のテスト名（`arrow_batch_from_list` 等）と重複しないこと

### Step 2: バージョン更新
- `fav/Cargo.toml`: `33.3.0` → `33.4.0`
- `fav/src/driver.rs`: `cargo_toml_version_is_33_3_0` を空スタブ化

### Step 3: テスト追加
- `fav/src/driver.rs` に `v334000_tests`（4 件）を追加
- 挿入位置: `v333000_tests` 閉じ括弧の直後、`// ── v31.7.0 tests` の前
- `use super::*` なし。`use crate::frontend::parser::Parser;` のみ

### Step 4: ドキュメント更新
- `CHANGELOG.md` 先頭に `[v33.4.0]` セクション追加
- `benchmarks/v33.4.0.json` 新規作成（暫定 2512）
- `versions/current.md` 更新

### Step 5: テスト確認
- `cargo test --bin fav v334000` — 4/4 PASS
- `cargo test` — 全件 PASS（2512 passed 想定）

### Step 6: 完了処理
- `benchmarks/v33.4.0.json` `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新

---

## リスク

| リスク | 対策 |
|---|---|
| v195000_tests テスト名との重複 | 事前に grep で確認。逆ケース（false/独立性）を選ぶ |
| `TrfDef.stateful` フィールド名が変わっている | ast.rs で確認済み（`pub stateful: bool`）|
| パースが失敗する Favnir 構文 | spec.md 記載の構文は v19.x 時点で動作確認済み |

---

## 差分見積もり

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version 1 行 |
| `fav/src/driver.rs` | スタブ 1 件 + テスト 4 件（約 50 行）|
| `CHANGELOG.md` | セクション 1 つ（約 15 行）|
| `benchmarks/v33.4.0.json` | 新規（8 行）|
| `versions/current.md` | 2〜3 行更新 |
