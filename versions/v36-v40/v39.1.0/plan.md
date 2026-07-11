# v39.1.0 実装計画 — RBAC Rune

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/auth/auth.fav` | 新規作成 | `require_role` / `check_permission` / `verify_jwt` 実装 |
| `runes/auth/rune.toml` | 新規作成 | Rune 設定ファイル |
| `fav/src/driver.rs` | 変更 | `v39000_tests::cargo_toml_version_is_39_0_0` スタブ化 / `v39100_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "39.0.0"` → `"39.1.0"` |
| `CHANGELOG.md` | 追記 | `[v39.1.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.1.0 を完了済みにマーク（✅）・テスト件数を 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v39.1.0、次バージョン v39.2.0 |
| `versions/v36-v40/v39.1.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T8 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v39.1.0] エントリ追加

`## [v39.0.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.1.0] — YYYY-MM-DD

### Added
- `runes/auth/auth.fav` — RBAC Rune（`require_role` / `check_permission` / `verify_jwt`）
- `runes/auth/rune.toml` — Rune 設定ファイル
- `v39100_tests` 3 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `runes/auth/auth.fav` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `fn require_role(ctx: AppCtx, role: String) -> Result<Unit, String> !Http`
- `fn check_permission(ctx: AppCtx, permission: String) -> Result<Unit, String> !Http`
- `fn verify_jwt(ctx: AppCtx, token: String) -> Result<String, String> !Http`

**注意**: `runes/auth/` ディレクトリは新規作成が必要（`fav/` と同じ親ディレクトリ下の `runes/`）。

### Step 3: `runes/auth/rune.toml` 新規作成

spec.md §2 の内容で作成。

### Step 4: `driver.rs` — `v39000_tests::cargo_toml_version_is_39_0_0` スタブ化

Grep で `cargo_toml_version_is_39_0_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.1.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_0_0` / `milestone_has_intelligence_and_assistance` / `readme_mentions_intelligence_assistance` はスタブ化しない。

### Step 5: `driver.rs` — `v39100_tests` モジュール追加（Step 1・2 完了後）

`v39000_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_39_1_0` / `changelog_has_v39_1_0` / `auth_rune_exists`

`auth_rune_exists` は `include_str!("../../runes/auth/auth.fav")` を使用 — Step 2 完了後に追加すること。

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `39.0.0` → `39.1.0` に更新。

### Step 7: `cargo test` 実行・全通過確認

`cargo test 2>&1 | grep "test result"`

期待: ≥ 2788 passed, 0 failed

### Step 8: ドキュメント更新

- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.1.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v39.1.0（最新安定版）・v39.2.0（次バージョン）に更新
- `versions/v36-v40/v39.1.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 5 (driver tests, changelog_has_v39_1_0)
Step 2 (auth.fav) ───────────────────────────────► Step 5 (driver tests, auth_rune_exists)
Step 3 (rune.toml) ──────────────────────────────► Step 7 (cargo test)
Step 4 (stub v39000) ────────────────────────────► Step 5 (driver tests)
Step 5 (v39100_tests) ───────────────────────────► Step 6 (Cargo.toml bump)
Step 5 (v39100_tests) ───────────────────────────► Step 7 (cargo test)
Step 6 (Cargo.toml) ─────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ───────────────────────────────► Step 8 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `runes/auth/` ディレクトリが存在しない | Write ツールで `runes/auth/auth.fav` を作成すると自動的に親ディレクトリが作られる（Bash で mkdir は不要）|
| `include_str!` パスのミス | `../../runes/auth/auth.fav` — `fav/src/` から 2 階層上の `favnir/` ルートからのパスを確認 |
| auth.fav の Favnir 構文エラー | 型チェックは driver.rs テストでは行わず、`include_str!` でファイル内容のみ検証するため問題なし |
