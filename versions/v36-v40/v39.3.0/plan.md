# v39.3.0 実装計画 — `fav policy`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/policy.rs` | 新規作成 | `cmd_policy_check` / `load_policy_rules` / `check_rules` 実装 |
| `fav/src/main.rs` | 変更 | `mod policy;` 追加 + `Some("policy")` ディスパッチアーム追加 |
| `fav/src/driver.rs` | 変更 | `v39200_tests::cargo_toml_version_is_39_2_0` スタブ化 / `v39300_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "39.2.0"` → `"39.3.0"` |
| `CHANGELOG.md` | 追記 | `[v39.3.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.3.0 を完了済みにマーク（✅） |
| `versions/current.md` | 更新 | 最新安定版 v39.3.0、次に切る版 v39.4.0 |
| `versions/v36-v40/v39.3.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T8 全チェック）|

## 注記: タスク番号と Step 番号の対応

plan.md の Step 番号（Step 1〜8）は tasks.md の T 番号（T0〜T8）と 1 対 1 に対応していない。
tasks.md の T0 が「事前確認」として先頭に挿入されているため、Step N は概ね T(N) に相当する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.3.0] エントリ追加

`## [v39.2.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.3.0] — YYYY-MM-DD

### Added
- `fav/src/policy.rs` — `fav policy check` / `fav policy check --ci` コマンド追加
- `policy { deny_runes / require_schema / require_tests / max_pipeline_stages }` ブロック仕様
- `v39300_tests` 3 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `fav/src/policy.rs` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `pub fn cmd_policy_check(ci_mode: bool) -> Result<(), String>`
- `fn load_policy_rules() -> Result<Vec<String>, String>`（スタブ: デフォルトルール 3 件を返す）
- `fn check_rules(rules: &[String]) -> Vec<String>`（スタブ: 常に空 Vec を返す）

### Step 3: `fav/src/main.rs` — `mod policy;` + `Some("policy")` アーム追加

1. Read で `mod suggest;` の行番号を確認
2. `mod suggest;` の直後に `mod policy;` を追加
3. Read で `Some("suggest")` ディスパッチアームの行番号を確認
4. `Some("suggest")` アームの直後に `Some("policy")` アームを追加（spec.md §2 参照）

**注意**: Step 2（policy.rs 作成）完了後に Step 3 を実施すること（`mod policy;` 追加時に `policy.rs` が存在する必要がある）。

### Step 4: `driver.rs` — `v39200_tests::cargo_toml_version_is_39_2_0` スタブ化

Grep で `cargo_toml_version_is_39_2_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.3.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_2_0` / `audit_rune_exists` はスタブ化しない。

### Step 5: `driver.rs` — `v39300_tests` モジュール追加（Step 1・2 完了後）

`v39200_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_39_3_0` / `changelog_has_v39_3_0` / `policy_rs_exists`

`policy_rs_exists` は `include_str!("policy.rs")` を使用 — Step 2 完了後に追加すること。

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `39.2.0` → `39.3.0` に更新。

### Step 7: `cargo test` 実行・全通過確認

`cargo test 2>&1 | grep "test result"`

期待: ≥ 2794 passed, 0 failed

### Step 8: ドキュメント更新

- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.3.0 を ✅ にマーク
- `versions/current.md` を v39.3.0（最新安定版）・v39.4.0（次に切る版）に更新
- `versions/v36-v40/v39.3.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 5 (driver tests, changelog_has_v39_3_0)
Step 2 (policy.rs) ──────────────────────────────► Step 3 (main.rs mod + dispatch)
                   ──────────────────────────────► Step 5 (driver tests, policy_rs_exists)
Step 3 (main.rs) ────────────────────────────────► Step 7 (cargo test)
Step 4 (stub v39200) ────────────────────────────► Step 5 (driver tests)
Step 5 (v39300_tests) ───────────────────────────► Step 6 (Cargo.toml bump)
Step 5 (v39300_tests) ───────────────────────────► Step 7 (cargo test)
Step 6 (Cargo.toml) ─────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ───────────────────────────────► Step 8 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `mod policy;` を先に追加すると `policy.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
| `Some("policy")` アームの挿入位置がずれる | Read で `Some("suggest")` 前後を確認してから Edit |
| `gen` 予約語（Rust 2024） | v39.3.0 テストでは `policy` 系変数のみ使用 — 問題なし |
