# v15.2.0 Tasks — GCP BigQuery Rune（`!Gcp` エフェクト）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"15.2.0"` に変更

---

## Phase B — テスト追加

- [x] B-1: `fav/src/driver.rs` に `v152000_tests` モジュール追加（5 テスト）
  - `version_is_15_2_0`
  - `bigquery_query_raw_primitive_exists`
  - `gcp_effect_in_ast`
  - `bigquery_rune_exists`
  - `bigquery_e2e_demo_structure`

- [x] B-2: `cargo test v152000` → 5/5 パス確認（実装前は 3/5 が FAIL、A + B-1 完了後は 1/5 パス）

---

## Phase C — `Effect::Gcp` 型システム

- [x] C-1: `fav/src/ast.rs` に `Effect::Gcp` 追加（Display / from_str も対応）

- [x] C-2: `fav/src/middle/checker.rs` 更新
  - `BUILTIN_EFFECTS` に `"Gcp"` 追加
  - `BUILTIN_NAMESPACES` に `"BigQuery"` 追加
  - `require_gcp_effect` ヘルパー追加
  - `builtin_ret_ty` に `BigQuery.query_raw` / `execute_raw` / `infer_table_raw` 追加
  - E0318 エラーコード: `!Gcp` エフェクトなしで BigQuery.* を呼んだ場合

- [x] C-3: `fav/src/lineage.rs` 更新
  - `EffectKind::GcpRead` / `GcpWrite` 追加
  - `collect_gcp_call_kinds` 関数追加

- [x] C-4: `cargo test` → リグレッションなし確認

---

## Phase D — `BigQuery.*` VM Primitive

- [x] D-1: `fav/src/backend/vm.rs` に `gcp_get_access_token` ヘルパー追加
  - `GOOGLE_APPLICATION_CREDENTIALS` からサービスアカウント JSON 読み込み
  - RS256 JWT 生成（`jsonwebtoken` クレート流用）
  - `https://oauth2.googleapis.com/token` で Bearer token 取得

- [x] D-2: `BigQuery.query_raw` primitive 追加
  - `(project_id, dataset, sql, params_json) -> Result<String, String>`
  - `POST /bigquery/v2/projects/{project_id}/queries`（同期クエリ）
  - 成功: rows + schema を JSON 文字列で返す
  - 失敗: `err("...")` を返す

- [x] D-3: `BigQuery.execute_raw` primitive 追加
  - `(project_id, dataset, sql, params_json) -> Result<Int, String>`
  - `POST /bigquery/v2/projects/{project_id}/jobs`（DML ジョブ）
  - Jobs API で polling → `numDmlAffectedRows` を返す
  - `BQ_LOCATION` 環境変数で asia-northeast1 等のリージョン指定対応

- [x] D-4: `BigQuery.infer_table_raw` primitive 追加
  - `(project_id, dataset, table) -> Result<String, String>`
  - `INFORMATION_SCHEMA.COLUMNS` クエリ → スキーマ JSON を返す

- [x] D-5: `cargo test` → リグレッションなし確認

---

## Phase E — checker.fav 更新

- [x] E-1: `bigquery_fn` スキーム追加（compiler.fav または inline）
  - `query_raw` → `"Result<String, String>"`
  - `execute_raw` → `"Result<Int, String>"`
  - `infer_table_raw` → `"Result<String, String>"`

- [x] E-2: `ns_to_effect` に `"BigQuery"` → `"Gcp"` 追加

- [x] E-3: `builtin_ret_ty` に BigQuery ブランチ追加（checker.fav 側）

---

## Phase F — BigQuery Rune

- [x] F-1: `runes/bigquery/` ディレクトリ作成

- [x] F-2: `runes/bigquery/bigquery.fav` 作成
  - `query<T>(project_id, dataset, sql) -> Result<List<T>, String> !Gcp`
  - `execute(project_id, dataset, sql) -> Result<Int, String> !Gcp`

---

## Phase G — `fav.toml [gcp]` セクション

- [x] G-1: `fav/src/toml.rs` に `GcpTomlConfig` 構造体追加

- [x] G-2: `FavToml` に `gcp: Option<GcpTomlConfig>` フィールド追加
  - `driver.rs` / `checker.rs` / `resolver.rs` の全 FavToml struct literal に `gcp: None` 追加

- [x] G-3: `inject_gcp_config` 関数追加（`GCP_PROJECT_ID` / `GOOGLE_APPLICATION_CREDENTIALS` を env に設定）

- [x] G-4: `cmd_run` の toml 読み込みブロックに `inject_gcp_config` 呼び出し追加

- [ ] G-5: `fav new` テンプレートの `fav.toml` に `[gcp]` セクション追記（コメントアウト）
  ※ スコープ外として未実施

---

## Phase H — `fav infer --from bigquery`

- [x] H-1: `fav/src/driver.rs` の `cmd_infer` に `--from bigquery` ブランチ追加
  - `BigQuery.infer_table_raw` を呼ぶ一時 Favnir プログラムを生成して実行
  - BigQuery 型 → Favnir 型のマッピングテーブルを適用してスキーマを出力

---

## Phase I — E2E デモ

- [x] I-1: `infra/e2e-demo/bigquery/` ディレクトリ構造作成

- [x] I-2: `src/demo.fav` 作成（LoadCsv |> TransformRows |> BigQueryInsert |> QuerySummary）

- [x] I-3: `terraform/gcp/main.tf` 作成（`google_bigquery_dataset` + `google_bigquery_table`）

- [x] I-4: `scripts/seed.sh` / `run.sh` / `verify.sh` 作成

- [x] I-5: `README.md` 作成（前提条件・実行手順・期待結果）

---

## Phase J — E2E 実行（GCP 環境）

- [x] J-1: GCP サービスアカウント JSON キー準備（`GOOGLE_APPLICATION_CREDENTIALS` 設定）

- [x] J-2: `terraform init && terraform apply`

- [x] J-3: `bash scripts/seed.sh` → ハードコードデータ（3 件）準備
  ※ Windows 環境のため `/tmp/` 非対応。ハードコードデータを demo.fav に直接埋め込み

- [x] J-4: `bash scripts/run.sh <gcp_project_id>` → `fav run --legacy demo.fav` 実行
  - LoadCsv → TransformRows → BigQueryInsert（3 件 INSERT）→ QuerySummary → EXIT 0

- [x] J-5: `bash scripts/verify.sh` → COUNT = 3 確認（PASS=1 FAIL=0）

- [x] J-6: `fav infer --from bigquery --table users` → スキーマ出力確認

- [x] J-7: `terraform destroy`（E2E 完了後）
  - 証跡: `infra/e2e-demo/bigquery/trail/run-output.txt` に保存済み

---

## Phase K — コミット

- [x] K-1: `cargo test v152000` → 5/5 パス最終確認

- [x] K-2: `cargo test` → 全件パス（リグレッションなし）確認

- [x] K-3: コミット（commit: d7824f0）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.2.0"` | [x] |
| `cargo test v152000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `BigQuery.query_raw` primitive が vm.rs に存在する | [x] |
| `BigQuery.execute_raw` primitive が vm.rs に存在する | [x] |
| `BigQuery.infer_table_raw` primitive が vm.rs に存在する | [x] |
| `Effect::Gcp` が ast.rs に存在する | [x] |
| `runes/bigquery/bigquery.fav` が存在する | [x] |
| `fav infer --from bigquery --table <name>` でスキーマ出力 | [x] |
| E2E: BigQueryInsert 3 件 → verify.sh PASS=1 FAIL=0 | [x] |
| terraform destroy 完了 | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.2.0/spec.md` | 仕様・スコープ |
| `versions/v15.2.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v15.1-v16.0.md` | v15.2.0 セクション |
| `versions/v10.x/` | Snowflake 統合（同パターン） |
| `infra/e2e-demo/snowflake/` | E2E デモ参考実装 |
