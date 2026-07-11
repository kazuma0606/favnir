# v39.8.0 実装計画 — Enterprise cookbook + ガバナンスドキュメント

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/docs/governance/rbac.mdx` | 新規作成 | RBAC ガバナンスドキュメント |
| `site/content/docs/governance/audit-log.mdx` | 新規作成 | Audit Log ガバナンスドキュメント |
| `site/content/docs/governance/policy.mdx` | 新規作成 | Policy ガバナンスドキュメント |
| `site/content/cookbook/multi-tenant-etl.mdx` | 新規作成 | マルチテナント ETL クックブック |
| `site/content/cookbook/secret-manager-vault.mdx` | 新規作成 | Secret Manager クックブック |
| `site/content/cookbook/ci-policy-gate.mdx` | 新規作成 | CI ポリシーゲートクックブック |
| `fav/src/driver.rs` | 変更 | `v39700_tests::cargo_toml_version_is_39_7_0` スタブ化 / `v39800_tests` 追加（2 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "39.7.0"` → `"39.8.0"` |
| `CHANGELOG.md` | 追記 | `[v39.8.0]` エントリ追加（`### Added` セクション使用） |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.8.0 を完了済みにマーク（✅） |
| `versions/current.md` | 更新 | 最新安定版 v39.8.0、次に切る版 v39.9.0 |
| `versions/v36-v40/v39.8.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T7 全チェック） |

> **Rust ソースファイル新規作成なし・main.rs 変更なし**: v39.8.0 は MDX 6 件と meta テスト 2 件のみ。

## 注記: Step 番号と tasks.md T 番号の対応

| plan.md Step | tasks.md T 番号 | 内容 |
|---|---|---|
| Step 1 | T1 | CHANGELOG 追加 |
| Step 2 | T2 | docs/governance/ MDX 3 ファイル作成 |
| Step 3 | T3 | cookbook/ MDX 3 ファイル作成 |
| Step 4 | T4 | driver.rs スタブ化 |
| Step 5 | T5 | driver.rs v39800_tests 追加 |
| Step 6 | T6 | Cargo.toml バージョン更新 |
| Step 7 | T7 | cargo test 実行 + ドキュメント更新 |

tasks.md には T0（事前確認）が先頭に挿入されているため、Step N は T(N) に対応する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.8.0] エントリ追加（tasks.md: T1）

`## [v39.7.0]` ヘッダ行の直前に挿入（`### Added` セクション使用）。

### Step 2: `site/content/docs/governance/` — ガバナンスドキュメント 3 ファイル作成（tasks.md: T2）

Write ツールで以下を作成（`governance/` ディレクトリが新規のため親も自動作成される）:

1. `rbac.mdx` — RBAC Rune（require_role / check_permission / verify_jwt）
2. `audit-log.mdx` — Audit Rune（Audit.log / start_trace / end_trace）+ `fav.toml` `[audit]` 設定例
3. `policy.mdx` — `fav policy check` / `fav policy check --ci` + policy ブロック記法

各ファイルは frontmatter + `# タイトル` + 概要 + `## コード例` + `## ポイント` の構造に従う。

### Step 3: `site/content/cookbook/` — クックブック 3 ファイル作成（tasks.md: T3）

Write ツールで以下を作成:

1. `multi-tenant-etl.mdx` — `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant`
2. `secret-manager-vault.mdx` — `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env`
3. `ci-policy-gate.mdx` — `fav policy check --ci` を CI ゲートとして使う例

### Step 4: `driver.rs` — `v39700_tests::cargo_toml_version_is_39_7_0` スタブ化（tasks.md: T4）

Grep で `cargo_toml_version_is_39_7_0` の行番号を確認 →
NOTE コメントとライブアサーションを:
```rust
// Stubbed: version bumped to 39.8.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_7_0` はスタブ化しない。

### Step 5: `driver.rs` — `v39800_tests` モジュール追加（tasks.md: T5、T1 および T4 完了後に実施）

`v39700_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §7 のコードブロックに従う:
- 3 テスト: `cargo_toml_version_is_39_8_0`（NOTE コメント付き）/ `changelog_has_v39_8_0` / `site_has_governance_docs`（6 MDX を `include_str!` 参照）

### Step 6: Cargo.toml バージョン更新（tasks.md: T6）

Step 1〜5 完了後に `39.7.0` → `39.8.0` に更新。

### Step 7: `cargo test` 実行 + ドキュメント更新（tasks.md: T7）

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2808 passed, 0 failed

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 5 (driver tests, changelog_has_v39_8_0)
Step 2 (governance MDX) ─────────────────────────► Step 7 (目視確認)
Step 3 (cookbook MDX) ───────────────────────────► Step 7 (目視確認)
Step 4 (stub v39700) ────────────────────────────► Step 5 (driver tests)
Step 5 (v39800_tests) ───────────────────────────► Step 6 (Cargo.toml bump)
Step 6 (Cargo.toml) ─────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ───────────────────────────────► docs 更新
```

## リスク

| リスク | 対処 |
|---|---|
| `governance/` ディレクトリが未存在 | Write ツールが親ディレクトリを自動作成するため問題なし |
| MDX フロントマター形式のズレ | 既存クックブック（jwt-auth.mdx 等）を Read で確認してから作成 |
| `include_str!` パスの誤り | Step 5 では `site/` 参照なし（driver.rs テストは include_str! で MDX 未参照）— MDX 存在確認は目視のみ |
| `gen` 予約語（Rust 2024）| v39.8.0 テストでは `cargo`/`src` 変数のみ使用 — 問題なし |
