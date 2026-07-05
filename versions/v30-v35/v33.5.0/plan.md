# v33.5.0 — Plan: fav run --precompiled 確認

## アプローチ

「確認・記録」パターン（v33.1〜v33.4 踏襲）。
v19.7.0 で実装済みの事前コンパイルキャッシュを `v335000_tests`（4 件）で記録する。
新規実装なし。

---

## 実装手順

### Step 1: 前提確認
- `Cargo.toml` version = `33.4.0` であること
- `cargo test` が 2512 passed, 0 failed であること
- `mod v335000_tests` が `driver.rs` に存在しないこと
- `v197000_tests` のテスト名（`compile_produces_favc` 等）と重複しないこと

### Step 2: バージョン更新
- `fav/Cargo.toml`: `33.4.0` → `33.5.0`
- `fav/src/driver.rs`: `cargo_toml_version_is_33_4_0` を空スタブ化

### Step 3: テスト追加
- `fav/src/driver.rs` に `v335000_tests`（4 件）を追加
- 挿入位置: `v334000_tests` 閉じ括弧の直後、`// ── v31.7.0 tests` の前
- `use super::*` なし
- `use crate::driver::cmd_compile_to_bytes;` / `use crate::backend::artifact::FvcArtifact;` を明示 import

### Step 4: ドキュメント更新
- `CHANGELOG.md` 先頭に `[v33.5.0]` セクション追加
- `benchmarks/v33.5.0.json` 新規作成（暫定 2516）
- `versions/current.md` 更新

### Step 5: テスト確認
- `cargo test --bin fav v335000` — 4/4 PASS
- `cargo test` — 全件 PASS（2516 passed 想定）

### Step 6: 完了処理
- `benchmarks/v33.5.0.json` `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新

---

## リスク

| リスク | 対策 |
|---|---|
| v197000_tests テスト名との重複 | 逆ケース（META / 差分）を選ぶ |
| `FvcArtifact::meta` が `None` になる可能性 | `cmd_compile_to_bytes` は常に FavcMeta を埋め込む（driver.rs l.32987 確認済み）|
| 異なるソースが同一バイトになる可能性 | 実数値が違えばコンスタントプールが異なるため発生しない |

---

## 差分見積もり

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version 1 行 |
| `fav/src/driver.rs` | スタブ 1 件 + テスト 4 件（約 45 行）|
| `CHANGELOG.md` | セクション 1 つ（約 15 行）|
| `benchmarks/v33.5.0.json` | 新規（8 行）|
| `versions/current.md` | 2〜3 行更新 |
