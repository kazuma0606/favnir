# v39.4.0 実装計画 — Secret Rune 強化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/secret/secret.fav` | 新規作成 | `get_aws` / `get_vault` / `get_gcp` / `get_env` スタブ実装 |
| `runes/secret/rune.toml` | 新規作成 | Secret Rune メタデータ（name/version/description/entry/effects/dependencies）|
| `fav/src/driver.rs` | 変更 | `v39300_tests::cargo_toml_version_is_39_3_0` スタブ化 / `v39400_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "39.3.0"` → `"39.4.0"` |
| `CHANGELOG.md` | 追記 | `[v39.4.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.4.0 を完了済みにマーク（✅） |
| `versions/current.md` | 更新 | 最新安定版 v39.4.0、次に切る版 v39.5.0 |
| `versions/v36-v40/v39.4.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T8 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v39.4.0] エントリ追加

`## [v39.3.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.4.0] — YYYY-MM-DD

### Added
- `runes/secret/secret.fav` — `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` 追加
- `runes/secret/rune.toml` — Secret Rune メタデータ
- `fav.toml` `[secrets] backend` 宣言スキーマ（"aws"/"vault"/"gcp"/"env"）
- `v39400_tests` 3 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `runes/secret/` ディレクトリ + ファイル新規作成

1. `mkdir runes/secret/`（存在しない場合）
2. `runes/secret/secret.fav` を spec.md §1 の内容で作成
3. `runes/secret/rune.toml` を spec.md §2 の内容で作成

`secret.fav` の関数一覧:
- `fn get_aws(ctx: AppCtx, name: String) -> Result<String, String> !Http`
- `fn get_vault(ctx: AppCtx, path: String) -> Result<String, String> !Http`
- `fn get_gcp(ctx: AppCtx, name: String) -> Result<String, String> !Http`
- `fn get_env(ctx: AppCtx, name: String) -> Result<String, String>` ← `!Http` なし

`rune.toml` の必須フィールド: `name` / `version` / `description` / `entry` / `effects` / `[dependencies]`

### Step 3: `driver.rs` — `v39300_tests::cargo_toml_version_is_39_3_0` スタブ化

Grep で `cargo_toml_version_is_39_3_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.4.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_3_0` / `policy_rs_exists` はスタブ化しない。

### Step 4: `driver.rs` — `v39400_tests` モジュール追加（Step 1・2 完了後）

`v39300_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_39_4_0` / `changelog_has_v39_4_0` / `secret_rune_exists`

`secret_rune_exists` は `include_str!("../../runes/secret/secret.fav")` を使用 — Step 2 完了後に追加すること。

### Step 5: Cargo.toml バージョン更新

Step 1〜4 完了後に `39.3.0` → `39.4.0` に更新。

### Step 6: `cargo test` 実行・全通過確認

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2797 passed, 0 failed

### Step 7: ドキュメント更新

- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.4.0 を ✅ にマーク
- `versions/current.md` を v39.4.0（最新安定版）・v39.5.0（次に切る版）に更新
- `versions/v36-v40/v39.4.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 4 (driver tests, changelog_has_v39_4_0)
Step 2 (secret.fav + rune.toml) ─────────────────► Step 4 (driver tests, secret_rune_exists)
Step 3 (stub v39300) ────────────────────────────► Step 4 (driver tests)
Step 4 (v39400_tests) ───────────────────────────► Step 5 (Cargo.toml bump)
Step 5 (Cargo.toml) ─────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ───────────────────────────────► Step 7 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `get_env` に `!Http` を誤付与 | spec 通り `!Http` なしシグネチャを厳守 |
| `rune.toml` の `entry` / `effects` フィールド漏れ | `audit/rune.toml` を参照して形式を統一 |
| `include_str!` パス誤り | `driver.rs` は `fav/src/` 配下 → `../../runes/secret/secret.fav` |
| `gen` 予約語（Rust 2024）| v39.4.0 テストでは `secret` 系変数のみ使用 — 問題なし |
