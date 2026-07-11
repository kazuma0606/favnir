# v39.5.0 実装計画 — マルチテナント対応

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/tenant/tenant.fav` | 新規作成 | `db_schema` / `s3_prefix` / `validate_tenant` スタブ実装 |
| `runes/tenant/rune.toml` | 新規作成 | Multi-tenant Rune メタデータ（name/version/description/entry/effects/dependencies）|
| `fav/src/driver.rs` | 変更 | `v39400_tests::cargo_toml_version_is_39_4_0` スタブ化 / `v39500_tests` 追加（4 テスト）|
| `fav/Cargo.toml` | 更新 | `version = "39.4.0"` → `"39.5.0"` |
| `CHANGELOG.md` | 追記 | `[v39.5.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.5.0 を完了済みにマーク（✅）|
| `versions/current.md` | 更新 | 最新安定版 v39.5.0、次に切る版 v39.6.0 |
| `versions/v36-v40/v39.5.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T7 全チェック）|

## 注記: Step 番号と tasks.md T 番号の対応

| plan.md Step | tasks.md T 番号 | 内容 |
|---|---|---|
| Step 1 | T1 | CHANGELOG 追加 |
| Step 2 | T2 | runes/tenant/ 作成 |
| Step 3 | T3 | driver.rs スタブ化 |
| Step 4 | T4 | driver.rs v39500_tests 追加 |
| Step 5 | T5 | Cargo.toml バージョン更新 |
| Step 6 | T6 | cargo test 実行 |
| Step 7 | T7 | ドキュメント更新 |

tasks.md には T0（事前確認）が先頭に挿入されているため、Step N は T(N) に対応する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.5.0] エントリ追加（tasks.md: T1）

`## [v39.4.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.5.0] — YYYY-MM-DD

### Added
- `runes/tenant/tenant.fav` — `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` 追加
- `runes/tenant/rune.toml` — Multi-tenant Rune メタデータ
- `ctx.tenant_id` ベースの DB スキーマ切り替え・S3 prefix 分離スタブ実装
- `v39500_tests` 4 テスト追加（meta 2 + テナント分離 E2E 2）

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `runes/tenant/` ディレクトリ + ファイル新規作成（tasks.md: T2）

1. `mkdir runes/tenant/`
2. `runes/tenant/tenant.fav` を spec.md §1 の内容で作成（3 関数: `db_schema` / `s3_prefix` / `validate_tenant`）
3. `runes/tenant/rune.toml` を spec.md §2 の内容で作成（`effects = []`）

**注意**:
- `effects = []` — 全関数がスタブのため HTTP/DB エフェクトなし
- `validate_tenant` の `allowed` パラメータ型は `List<String>`

### Step 3: `driver.rs` — `v39400_tests::cargo_toml_version_is_39_4_0` スタブ化（tasks.md: T3）

Grep で `cargo_toml_version_is_39_4_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.5.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_4_0` / `secret_rune_exists` はスタブ化しない。

### Step 4: `driver.rs` — `v39500_tests` モジュール追加（tasks.md: T4、Step 1・2 完了後）

`v39400_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 4 テスト: `cargo_toml_version_is_39_5_0` / `changelog_has_v39_5_0` / `tenant_rune_db_schema` / `tenant_rune_s3_prefix`
- `tenant_rune_db_schema` と `tenant_rune_s3_prefix` は同一ファイル（`../../runes/tenant/tenant.fav`）を参照

**注意**: `include_str!` パスは `../../runes/tenant/tenant.fav`（`driver.rs` は `fav/src/` 配下）。

### Step 5: Cargo.toml バージョン更新（tasks.md: T5）

Step 1〜4 完了後に `39.4.0` → `39.5.0` に更新。

### Step 6: `cargo test` 実行・全通過確認（tasks.md: T6）

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2801 passed, 0 failed

### Step 7: ドキュメント更新（tasks.md: T7）

- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.5.0 を ✅ にマーク
- `versions/current.md` を v39.5.0（最新安定版）・v39.6.0（次に切る版）に更新
- `versions/v36-v40/v39.5.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 4 (driver tests, changelog_has_v39_5_0)
Step 2 (tenant.fav + rune.toml) ─────────────────► Step 4 (driver tests, tenant_rune_*)
Step 3 (stub v39400) ────────────────────────────► Step 4 (driver tests)
Step 4 (v39500_tests) ───────────────────────────► Step 5 (Cargo.toml bump)
Step 5 (Cargo.toml) ─────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ───────────────────────────────► Step 7 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `effects = []` の空リスト形式が toml パーサーで問題になる | `rune.toml` の `effects` フィールドは他 Rune と同形式（配列）で記述 |
| `List<String>` 型が `validate_tenant` で未解決エラー | `.fav` ファイルはビルド時に Rust コンパイルの対象外（`include_str!` のみ）— 問題なし |
| `gen` 予約語（Rust 2024）| v39.5.0 テストでは `tenant` 系変数のみ使用 — 問題なし |
| `include_str!` パス誤り | `driver.rs` は `fav/src/` 配下 → `../../runes/tenant/tenant.fav` |
