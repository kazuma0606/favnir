# v39.2.0 実装計画 — Audit Log Rune

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/audit/audit.fav` | 新規作成 | `log` / `start_trace` / `end_trace` / `audit_config` / `emit_log` 実装 |
| `runes/audit/rune.toml` | 新規作成 | Rune 設定ファイル |
| `fav/src/driver.rs` | 変更 | `v39100_tests::cargo_toml_version_is_39_1_0` スタブ化 / `v39200_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "39.1.0"` → `"39.2.0"` |
| `CHANGELOG.md` | 追記 | `[v39.2.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.2.0 を完了済みにマーク（✅）・テスト件数を 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v39.2.0、次に切る版 v39.3.0 |
| `versions/v36-v40/v39.2.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T8 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v39.2.0] エントリ追加

`## [v39.1.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.2.0] — YYYY-MM-DD

### Added
- `runes/audit/audit.fav` — Audit Log Rune（`log` / `start_trace` / `end_trace`）
- `runes/audit/rune.toml` — Rune 設定ファイル
- `fav.toml` `[audit]` セクション仕様（`enabled` / `output = "file"/"webhook"`）
- `v39200_tests` 3 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `runes/audit/audit.fav` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `fn log(ctx: AppCtx, trace_id: String, message: String) -> Result<Unit, String> !Http`
- `fn start_trace(ctx: AppCtx, pipeline_name: String) -> Result<String, String> !Http`
- `fn end_trace(ctx: AppCtx, trace_id: String, status: String) -> Result<Unit, String> !Http`
- `fn audit_config(ctx: AppCtx) -> Result<AuditConfig, String> !Http`（内部ヘルパー）
- `fn emit_log(...)` — file / webhook 出力分岐（内部ヘルパー）

**注意**: `runes/audit/` ディレクトリが存在しない場合は Write ツールでファイルを作成すると自動作成される。

### Step 3: `runes/audit/rune.toml` 新規作成

spec.md §2 の内容で作成。

### Step 4: `driver.rs` — `v39100_tests::cargo_toml_version_is_39_1_0` スタブ化

Grep で `cargo_toml_version_is_39_1_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.2.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_1_0` / `auth_rune_exists` はスタブ化しない。

### Step 5: `driver.rs` — `v39200_tests` モジュール追加（Step 1・2 完了後）

`v39100_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_39_2_0` / `changelog_has_v39_2_0` / `audit_rune_exists`

`audit_rune_exists` は `include_str!("../../runes/audit/audit.fav")` を使用 — Step 2 完了後に追加すること。

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `39.1.0` → `39.2.0` に更新。

### Step 7: `cargo test` 実行・全通過確認

`cargo test 2>&1 | grep "test result"`

期待: ≥ 2791 passed, 0 failed

### Step 8: ドキュメント更新

- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.2.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v39.2.0（最新安定版）・v39.3.0（次に切る版）に更新
- `versions/v36-v40/v39.2.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 5 (driver tests, changelog_has_v39_2_0)
Step 2 (audit.fav) ──────────────────────────────► Step 5 (driver tests, audit_rune_exists)
Step 3 (rune.toml) ──────────────────────────────► Step 7 (cargo test)
Step 4 (stub v39100) ────────────────────────────► Step 5 (driver tests)
Step 5 (v39200_tests) ───────────────────────────► Step 6 (Cargo.toml bump)
Step 5 (v39200_tests) ───────────────────────────► Step 7 (cargo test)
Step 6 (Cargo.toml) ─────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ───────────────────────────────► Step 8 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `runes/audit/` ディレクトリが存在しない | Write ツールでファイル作成時に自動作成される |
| `audit.fav` 既存ファイルが存在する可能性 | T0 で事前確認し、存在する場合は内容を確認して `fn log` が含まれるか検証 |
| `include_str!` パスのミス | `../../runes/audit/audit.fav` — `fav/src/` から 2 階層上のルートからのパスを確認 |
