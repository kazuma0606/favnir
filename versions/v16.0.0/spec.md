# v16.0.0 Spec — "Production Multi-Cloud" マイルストーン宣言

Date: 2026-06-14
Branch: master

---

## テーマ

v15.x シリーズの集大成として **Production Multi-Cloud** マイルストーンを宣言する。
CrossCloud 認証・GCP BigQuery・Kafka/MSK・`fav test`・`fav deploy` が揃い、
AWS / Azure / GCP / Snowflake の 4 クラウドを型安全なパイプラインで統一的に扱える言語になった。

コードの変更は最小限（バージョン更新のみ）とし、ドキュメント・サイト・テストを整備して宣言する。

---

## スコープ

### A: Cargo.toml version 更新

```toml
version = "16.0.0"
```

### B: CHANGELOG.md 更新

v15.1.0〜v15.5.0 の全エントリを先頭に追加:

| バージョン | 内容 |
|---|---|
| v15.1.0 | CrossCloud 認証層（HMAC + Cognito + Lambda verifier）|
| v15.1.5 | CrossCloud 認証層セキュア版（KMS ECDSA P-256）|
| v15.2.0 | GCP BigQuery Rune（`!Gcp` エフェクト）|
| v15.3.0 | `fav test` DSL（ネイティブテストフレームワーク）|
| v15.4.0 | Kafka / MSK Rune（`!Stream` エフェクト）|
| v15.5.0 | `fav deploy`（AWS Lambda デプロイ CLI）|

### C: README.md 更新

1. 「現在の状態」を v16.0.0 に更新（v15.5.0 → v16.0.0）
2. 対応クラウド一覧表を追加:

| クラウド | サービス | Rune / Effect |
|---|---|---|
| AWS | S3 / SQS / Lambda / MSK | `!AWS` / `!Stream` |
| Azure | PostgreSQL / Blob Storage | `!AzureDb` / `!AzureStorage` |
| GCP | BigQuery | `!Gcp` |
| Snowflake | Data Warehouse | `!Snowflake` |

3. `fav test` / `fav deploy` を機能一覧に追加

### D: サイトドキュメント追加

- `site/content/docs/runes/bigquery.mdx` — BigQuery Rune リファレンス
- `site/content/docs/runes/kafka.mdx` — Kafka Rune リファレンス

### E: テスト（v160000_tests — 5 件）

1. `version_is_16_0_0`: Cargo.toml version == "16.0.0"
2. `changelog_has_v15_entries`: CHANGELOG.md に `[v15.` エントリが含まれる
3. `readme_mentions_bigquery`: README.md に `BigQuery` が含まれる
4. `readme_mentions_kafka`: README.md に `Kafka` が含まれる
5. `all_e2e_demo_dirs_exist`: 全 E2E デモディレクトリが存在する
   （`airgap` / `fav2py` / `snowflake` / `crosscloud` / `bigquery` / `kafka` の 6 件）

---

## 完了条件

1. `cargo test v160000` → 5/5 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "16.0.0"`
4. CHANGELOG.md に v15.1.0〜v15.5.0 エントリが含まれる
5. README.md に対応クラウド一覧・`fav test` / `fav deploy` の記載がある
6. `site/content/docs/runes/bigquery.mdx` / `kafka.mdx` が存在する

---

## 新規 Cargo 依存

なし。

---

## 既知の制約・スコープ外

- Azure Function デプロイ（`fav deploy --target azure-function`）は v16.x 以降
- Azure Event Hubs（Kafka 互換）は v16.x 以降
- Kafka オフセット管理・バッチ消費は v16.x 以降
- MSK IAM 認証は v16.x 以降

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v16.0.0 セクション
- `CHANGELOG.md` — 既存フォーマット確認用
- `README.md` — 既存フォーマット確認用
- `infra/e2e-demo/` — E2E デモ全ディレクトリ
