# v16.0.0 Plan — "Production Multi-Cloud" マイルストーン宣言

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

### A-1: `fav/Cargo.toml` version 更新

```toml
version = "16.0.0"
```

---

## Phase B — CHANGELOG.md 更新

### B-1: v15.1.0〜v15.5.0 エントリを先頭に追加

既存フォーマット（`## [vX.Y.Z] — YYYY-MM-DD` / `### Added` / `### Fixed`）に従い追記:

```markdown
## [v15.5.0] — 2026-06-14

### Added
- `fav deploy` 完成（AWS Lambda デプロイ CLI）
- `DeployConfig` に `target` / `function_name` フィールド追加
- `scripts/build-lambda-layer.sh`（cross-compile + zip パッケージング）
- `site/content/docs/deploy.mdx` 新規作成

---

## [v15.4.0] — 2026-06-14

### Added
- Kafka / MSK Rune（`!Stream` エフェクト）
- `Kafka.produce_raw` / `Kafka.consume_one_raw` VM プリミティブ（rskafka 0.6）
- E0319: `!Stream` エフェクト欠如エラー
- `runes/kafka/kafka.fav`
- `infra/e2e-demo/kafka/`（4-stage pipeline + Terraform AWS MSK）

---

## [v15.3.0] — 2026-06-14

### Added
- `fav test` DSL（`test "..."` ブロック、`assert_ok` / `assert_err` / `assert_true` プリミティブ）
- `cmd_test`（Bool(false) → FAIL 判定）
- `site/content/docs/language/testing.mdx` 新規作成

---

## [v15.2.0] — 2026-06-14

### Added
- GCP BigQuery Rune（`!Gcp` エフェクト）
- `BigQuery.query_raw` / `execute_raw` / `infer_table_raw` VM プリミティブ（RS256 JWT 認証）
- E0318: `!Gcp` エフェクト欠如エラー
- `runes/bigquery/bigquery.fav`
- `infra/e2e-demo/bigquery/`（4-stage pipeline + Terraform GCP BigQuery）

---

## [v15.1.5] — 2026-06-14

### Added
- CrossCloud 認証層セキュア版（KMS ECDSA P-256）
- Lambda verifier_v2（KMS `GetPublicKey` + `cryptography` ECDSA 検証）
- `infra/e2e-demo/crosscloud/lambda/verifier_v2/`
- E2E: 改ざんリクエスト → 401 PASS

---

## [v15.1.0] — 2026-06-14

### Added
- CrossCloud 認証層（HMAC-SHA256 + Cognito + Lambda verifier）
- `AWS.dynamo_put_item_cond_raw`（nonce 管理）/ `AWS.ecs_run_task_raw` VM プリミティブ
- `infra/e2e-demo/crosscloud/lambda/verifier/`（Favnir コンテナ）
- E2E: reject_cases.sh PASS=5 FAIL=0
```

---

## Phase C — README.md 更新

### C-1: 「現在の状態」セクション更新

v15.5.0 → v16.0.0 への記述を追加:

```markdown
v15.1.0〜v15.5.0（2026-06-14）で、Production Multi-Cloud 機能群を整備しました。
CrossCloud 認証（HMAC / KMS ECDSA）、GCP BigQuery、Kafka/MSK、`fav test`、`fav deploy` が揃い、
v16.0.0 で「Production Multi-Cloud」マイルストーンを宣言します。
```

### C-2: 対応クラウド一覧表を追加

```markdown
| クラウド | サービス | Rune / Effect |
|---|---|---|
| AWS | S3 / SQS / Lambda / MSK | `!AWS` / `!Stream` |
| Azure | PostgreSQL / Blob Storage | `!AzureDb` / `!AzureStorage` |
| GCP | BigQuery | `!Gcp` |
| Snowflake | Data Warehouse | `!Snowflake` |
```

### C-3: 機能一覧に `fav test` / `fav deploy` を追加

---

## Phase D — サイトドキュメント追加

### D-1: `site/content/docs/runes/bigquery.mdx` 新規作成

内容:
- `BigQuery.query_raw` / `execute_raw` / `infer_table_raw` 関数リファレンス
- `fav.toml [gcp]` 設定例
- `fav infer --from bigquery --table <name>` の使い方
- `GOOGLE_APPLICATION_CREDENTIALS` 設定方法

### D-2: `site/content/docs/runes/kafka.mdx` 新規作成

内容:
- `Kafka.produce_raw` / `Kafka.consume_one_raw` 関数リファレンス
- `fav.toml [kafka]` 設定例
- `KAFKA_BOOTSTRAP_BROKERS` / `KAFKA_SASL_*` 環境変数
- AWS MSK との接続設定例

---

## Phase E — v160000_tests 追加（driver.rs）

### E-1: `v160000_tests` モジュール追加

```rust
// ── v160000_tests (v16.0.0) — Production Multi-Cloud マイルストーン ─────────
#[cfg(test)]
mod v160000_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn version_is_16_0_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"16.0.0\""), ...);
    }

    #[test]
    fn changelog_has_v15_entries() {
        let changelog = fs::read_to_string("../CHANGELOG.md").unwrap();
        assert!(changelog.contains("[v15."), ...);
    }

    #[test]
    fn readme_mentions_bigquery() {
        let readme = fs::read_to_string("../README.md").unwrap();
        assert!(readme.contains("BigQuery"), ...);
    }

    #[test]
    fn readme_mentions_kafka() {
        let readme = fs::read_to_string("../README.md").unwrap();
        assert!(readme.contains("Kafka"), ...);
    }

    #[test]
    fn all_e2e_demo_dirs_exist() {
        for dir in &["airgap", "fav2py", "snowflake", "crosscloud", "bigquery", "kafka"] {
            assert!(
                Path::new(&format!("../infra/e2e-demo/{}", dir)).exists(),
                "infra/e2e-demo/{} must exist",
                dir
            );
        }
    }
}
```

---

## Phase F — テスト・コミット

### F-1: `cargo test v160000` → 5/5 パス最終確認

### F-2: `cargo test` → 全件パス（リグレッションなし）確認

### F-3: コミット

```
feat: v16.0.0 — Production Multi-Cloud マイルストーン宣言
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version 16.0.0 |
| `fav/src/driver.rs` | 更新 | v160000_tests 追加 |
| `CHANGELOG.md` | 更新 | v15.1.0〜v15.5.0 エントリ追加 |
| `README.md` | 更新 | v16.0.0 状態・対応クラウド表・機能一覧 |
| `site/content/docs/runes/bigquery.mdx` | 新規 | BigQuery Rune リファレンス |
| `site/content/docs/runes/kafka.mdx` | 新規 | Kafka Rune リファレンス |
| `versions/v16.0.0/spec.md` | 新規 | 仕様書 |
| `versions/v16.0.0/plan.md` | 新規 | 実装計画 |
| `versions/v16.0.0/tasks.md` | 新規 | タスクリスト |

---

## 実装上の注意点

1. **CHANGELOG.md のフォーマット**: 既存エントリは `## [vX.Y.Z] — YYYY-MM` 形式（月のみ）と `## [vX.Y.Z] — YYYY-MM-DD` 形式が混在している。新規エントリは `YYYY-MM-DD` 形式で統一する。

2. **README.md の更新箇所**: 「v15.x.0（日付）で〜しました。」の形式で末尾に追記。既存の文体・敬体（「〜しました」「〜します」）を維持する。

3. **site/content/docs/runes/ ディレクトリ**: 既存の `runes/` サブディレクトリが存在するか確認してから作成。

4. **v160000_tests のパス**: CHANGELOG.md / README.md は `fav/` の 1 つ上（`../CHANGELOG.md`）にある。`infra/e2e-demo/` も `../infra/e2e-demo/` でアクセス。
