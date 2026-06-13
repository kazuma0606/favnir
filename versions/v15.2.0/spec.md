# v15.2.0 Spec — GCP BigQuery Rune（`!Gcp` エフェクト）

Date: 2026-06-14
Branch: master

---

## テーマ

Snowflake 統合（v10.x）と同じパターンで **GCP BigQuery** を追加する。
AWS / Azure に続く 3 クラウド目のデータソースサポート。

認証は **Service Account JSON + RS256 JWT → OAuth2 Bearer token** で実現する（`jsonwebtoken` クレートは既存依存のため追加 Cargo 依存なし）。

---

## スコープ

### A: `BigQuery.*` VM Primitive

| Primitive | シグネチャ | 説明 |
|---|---|---|
| `BigQuery.query_raw` | `(project_id, dataset, sql, params_json) -> Result<String, String>` | SELECT クエリ → JSON 文字列（rows 配列）を返す |
| `BigQuery.execute_raw` | `(project_id, dataset, sql, params_json) -> Result<Int, String>` | DML（INSERT/UPDATE/DELETE）→ 影響行数を返す |
| `BigQuery.infer_table_raw` | `(project_id, dataset, table) -> Result<String, String>` | INFORMATION_SCHEMA.COLUMNS クエリ → スキーマ JSON |

認証フロー（3 ステップ）:
1. `GOOGLE_APPLICATION_CREDENTIALS` 環境変数からサービスアカウント JSON キーファイルを読む
2. RS256 JWT を生成し `https://oauth2.googleapis.com/token` でアクセストークンを取得
3. BigQuery REST API を `Bearer <token>` で呼び出す

BigQuery REST エンドポイント:
- `query_raw` / `infer_table_raw` → `POST /bigquery/v2/projects/{project_id}/queries`（同期クエリ）
- `execute_raw` → `POST /bigquery/v2/projects/{project_id}/jobs`（非同期 DML → polling）

### B: `Effect::Gcp` 追加（型システム）

- `fav/src/ast.rs`: `Effect::Gcp` 追加
- `fav/src/middle/checker.rs`:
  - `BUILTIN_EFFECTS` に `"Gcp"` 追加
  - `builtin_ret_ty` に `BigQuery.*` 追加（`require_gcp_effect` チェック）
  - E0318: `!Gcp` エフェクトなしで `BigQuery.*` を呼んだ場合のエラー
- `fav/src/lineage.rs`: `EffectKind::GcpRead` / `GcpWrite` 追加

### C: checker.fav 更新

- `bigquery_fn` スキーム追加（`query_raw`, `execute_raw`, `infer_table_raw`）
- `ns_to_effect` に `"BigQuery"` → `"Gcp"` 追加
- `builtin_ret_ty` の BigQuery ブランチ追加

### D: BigQuery Rune

`runes/bigquery/bigquery.fav`:
- `query<T>(project_id, dataset, sql) -> Result<List<T>, String> !Gcp`
- `execute(project_id, dataset, sql) -> Result<Int, String> !Gcp`

### E: `fav.toml [gcp]` セクション

```toml
[gcp]
project_id = "my-gcp-project"
credentials_file = "/path/to/service-account.json"
dataset = "my_dataset"
```

`inject_gcp_config` で環境変数 `GCP_PROJECT_ID` / `GOOGLE_APPLICATION_CREDENTIALS` を自動設定。

### F: `fav infer --from bigquery`

```bash
fav infer --from bigquery --table users
```

`BigQuery.infer_table_raw` primitive を使い INFORMATION_SCHEMA.COLUMNS を照会して Favnir スキーマを生成。

BigQuery 型 → Favnir 型マッピング:

| BigQuery 型 | Favnir 型 |
|---|---|
| STRING | String |
| INT64 / INTEGER | Int |
| FLOAT64 / FLOAT | Float |
| BOOL / BOOLEAN | Bool |
| TIMESTAMP / DATETIME / DATE | String |
| BYTES | String |
| ARRAY | List |
| STRUCT / RECORD | Map |

### G: E2E デモ

`infra/e2e-demo/bigquery/` — CSV → BigQuery の 4 ステージパイプライン:

```
LoadCsv |> TransformRows |> BigQueryInsert |> QuerySummary
```

Terraform:
- `google_bigquery_dataset` — demo データセット
- `google_bigquery_table` — users テーブル

### H: テスト（v152000_tests — 5 件）

1. `version_is_15_2_0`
2. `bigquery_query_raw_primitive_exists`（vm.rs に `BigQuery.query_raw` が含まれる）
3. `gcp_effect_in_ast`（ast.rs に `Gcp` が含まれる）
4. `bigquery_rune_exists`（`runes/bigquery/bigquery.fav` が存在する）
5. `bigquery_e2e_demo_structure`（`infra/e2e-demo/bigquery/` の主要ファイルが存在する）

---

## 完了条件

1. `cargo test v152000` → 5/5 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "15.2.0"`
4. E2E デモ: CSV を BigQuery に INSERT → クエリで件数確認
5. `fav infer --from bigquery --table <name>` でスキーマ出力
6. 改ざんデータや認証エラーが適切に `Result.err` を返す

---

## 新規 Cargo 依存

なし（`ureq` + `serde_json` + `jsonwebtoken` + `sha2` はすべて既存依存）。

---

## 既知の制約・スコープ外

- BigQuery Streaming Insert（Storage Write API）は対象外。REST API の標準 DML のみ。
- GCS → BigQuery 直接ロードは対象外（CSV ファイルを Favnir 側で読んで INSERT）
- BigQuery → Snowflake の 3 クラウド横断デモは v16.x 以降
- GCP Terraform（Terraform GCP プロバイダー）の tfstate は `infra/e2e-demo/bigquery/terraform/` に独立管理
- fav.toml [gcp] の inject は `fav run` 時のみ（checker.fav / compiler.fav のパスには影響しない）

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v15.2.0 セクション
- `versions/v10.x/` — Snowflake 統合（同パターン）
- `infra/e2e-demo/snowflake/` — E2E デモ参考実装
- `runes/snowflake/snowflake.fav` — Rune 参考実装
