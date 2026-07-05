# v33.7.0 — Plan: エフェクトシステム移行準備 確認

## アプローチ

「確認・記録」パターン（v33.1〜v33.6 踏襲）。
v13.10.0 で実装済みのエフェクト移行ツールを `v337000_tests`（4 件）で記録する。
新規実装なし。

---

## 実装手順

### Step 1: 前提確認
- `Cargo.toml` version = `33.6.0` であること
- `cargo test` が 2520 passed, 0 failed であること
- `mod v337000_tests` が `driver.rs` に存在しないこと
- `v13100_tests` のテスト名と重複しないこと

### Step 2: バージョン更新
- `fav/Cargo.toml`: `33.6.0` → `33.7.0`
- `fav/src/driver.rs`: `cargo_toml_version_is_33_6_0` を空スタブ化

### Step 3: テスト追加
- `fav/src/driver.rs` に `v337000_tests`（4 件）を追加
- 挿入位置: `v336000_tests` 閉じ括弧の直後、`// ── v31.7.0 tests` の前
- `use super::*` なし
- `use crate::driver::{migrate_effects_in_source, resolve_use_effects};` を明示 import

### Step 4: ドキュメント更新
- `CHANGELOG.md` 先頭に `[v33.7.0]` セクション追加
- `benchmarks/v33.7.0.json` 新規作成（暫定 2524）
- `versions/current.md` 更新

### Step 5: テスト確認
- `cargo test --bin fav v337000` — 4/4 PASS
- `cargo test` — 全件 PASS（2524 passed 想定）

### Step 6: 完了処理
- `benchmarks/v33.7.0.json` `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新

---

## リスク

| リスク | 対策 |
|---|---|
| v13100_tests テスト名との重複 | 冪等性・バージョン判定という異なる観点を選択 |
| `migrate_effects_in_source` が 2 回目で別の変換をする | spec 冪等性の説明から安全と判断（移行後は `!Effect` を含まない）|
| `resolve_use_effects(Some("v12"), false)` が予期せず true を返す | driver.rs l.15991 で `v13` / `13` のみ true と確認済み |

---

## 差分見積もり

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version 1 行 |
| `fav/src/driver.rs` | スタブ 1 件 + テスト 4 件（約 40 行）|
| `CHANGELOG.md` | セクション 1 つ（約 15 行）|
| `benchmarks/v33.7.0.json` | 新規（8 行）|
| `versions/current.md` | 2〜3 行更新 |
