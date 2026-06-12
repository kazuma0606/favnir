# Roadmap v14.1.0 〜 v15.0.0 — CrossCloud E2E Demo

Date: 2026-06-12（最終更新）

## 目標

`infra/e2e-demo/crosscloud/plan.md` の企画を実現する。
AWS RDS Postgres（ソース）→ Favnir パイプライン → Azure DB for PostgreSQL（ターゲット）のクロスクラウドマイグレーションを E2E デモとして動作させる。

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Azure Postgres 認証 | パスワードベース接続文字列（`postgresql://user:password@host/db`） |
| 証跡ストレージ | Azure Blob Storage（モダンアプローチ。S3 ではなく Blob に保存） |
| CrossCloud 関数シグネチャ | `fn migrate(aws_ctx: AwsCtx, azure_ctx: AzureCtx)` を基本形。`CrossCloudCtx` 統合型もコメントで示す。 |
| AWS Rune | `runes/aws/` に型付きラッパーとして正式実装（VM ビルトイン直呼びより rich） |

---

## バージョン計画

### v14.1.0 — Azure PostgreSQL Rune ✅ COMPLETE

**テーマ**: Azure DB for PostgreSQL への接続・操作をサポート

**実装内容:**

- `fav/src/vm/builtins.rs`: Azure Postgres VM プリミティブ追加
  - `AzurePostgres.connect_raw(conn_str: String) -> Result<Unit, String>`
  - `AzurePostgres.execute_raw(conn_str, sql, params) -> Result<Int, String>`
  - `AzurePostgres.query_raw(conn_str, sql, params) -> Result<String, String>`
  - `AzurePostgres.close_raw(conn_str) -> Result<Unit, String>`
  - Crate: `tokio-postgres`（既存依存）+ `tokio-postgres-openssl` for TLS
  - 接続文字列形式: `postgresql://user:password@host:5432/db?sslmode=require`

- `fav/src/middle/checker.rs`: `AzurePostgres` namespace 追加（builtin_ret_ty / BUILTIN_EFFECTS）
  - `!AzureDb` エフェクト追加

- `fav/src/lineage.rs`: `!AzureDb(read/write)` 区別追加

- `runes/azure-postgres/rune.fav`: 型付きラッパー

- テスト: `v141000_tests`

---

### v14.2.0 — AzureCtx / AwsCtx + fav.toml [azure] ✅ COMPLETE

**テーマ**: クロスクラウド用 Context 型と設定ファイル拡張

**実装内容:**

- `fav/src/config.rs`: `fav.toml` に `[azure]` セクション追加
- `fav/src/vm/builtins.rs`: `Ctx.build_aws_raw` / `Ctx.build_azure_raw` / `Ctx.aws_get_field_raw` / `Ctx.azure_get_field_raw` プリミティブ追加
- テスト: `v142000_tests`

---

### v14.3.0 — Azure lineage + fav explain 出力改善 ✅ COMPLETE

**テーマ**: CrossCloud パイプラインのリネージ可視化

**実装内容:**

- `fav/src/lineage.rs`: `EffectKind::AzureDbRead/Write`, `EffectKind::AzureBlobRead/Write` 追加
- `ast::Effect::AzureStorage` 追加
- `BUILTIN_EFFECTS` に `"AzureStorage"` 追加
- テスト: `v143000_tests`

---

### v14.4.0 — AWS Rune 正式パッケージング (runes/aws/) ✅ COMPLETE

**テーマ**: AWS VM ビルトインを型付き Rune ラッパーとして正式公開

**実装内容:**

- `fav/src/backend/vm.rs`: `AWS.secrets_get_raw` プリミティブ追加（SigV4 + Secrets Manager）
- `fav/src/middle/checker.rs`: `AWS.secrets_get_raw` の builtin_ret_ty 追加
- `runes/aws/secrets.fav`: `secrets_get(ctx: String, secret_name: String)` ラッパー
- `runes/aws/s3.fav`: `s3_put/s3_get/s3_delete/s3_list` ctx-aware ラッパー追加
- `runes/aws/rune.toml`: メタデータ
- テスト: `v144000_tests`

**実装メモ（後続バージョンで参照）:**
- `import rune "ctx"` は使用不可（`runes/ctx/ctx.fav` 未存在）→ `ctx: String` で代替
- rune ファイル内で `let` 構文を使うとパースエラー → インライン化が必須

---

### v14.5.0 — Azure Blob Storage Rune ✅ COMPLETE

**テーマ**: Azure Blob Storage への証跡保存をサポート

**実装内容:**

- `fav/src/backend/vm.rs`: Azure Blob VM プリミティブ追加
  - `azure_blob_sign` ヘルパー（HMAC-SHA256 + base64 Shared Key 署名）
  - `AzureBlob.put_raw/get_raw/list_raw/delete_raw`
- `fav/src/middle/checker.rs`: `AzureBlob` namespace + `require_azure_storage_effect` (E0317)
- `runes/azure-blob/azure_blob.fav`: `put/get/list/delete` ラッパー（ctx: String）
- `runes/azure-blob/rune.toml`: メタデータ
- テスト: `v145000_tests`（4件全パス）

---

### v14.6.0 — ドキュメント整備（README + CHANGELOG）

**テーマ**: v14.1.0〜v14.5.0 の積み残しドキュメントを一括修正

**背景（2026-06-12 調査結果）:**
- CHANGELOG に v14.1.0〜v14.5.0 のエントリが丸ごと欠落
- README の「現在の状態」見出しが `v10.0.0` のまま
- README のロードマップ表が v14.0.0 で止まっている
- README のコード例が旧 `!Effect` スタイルで書かれており、v14.0.0 Capability Context と矛盾して見える

**実装内容:**

- `CHANGELOG.md`: v14.1.0〜v14.5.0 の各エントリを追加
  - v14.1.0: AzurePostgres VM + checker + lineage
  - v14.2.0: Ctx.build_aws_raw / Ctx.build_azure_raw / fav.toml [azure]
  - v14.3.0: AzureStorage effect + lineage AzureBlobRead/Write
  - v14.4.0: AWS.secrets_get_raw + runes/aws/ 正式パッケージング
  - v14.5.0: AzureBlob.put_raw/get_raw/list_raw/delete_raw + runes/azure-blob/

- `README.md`:
  - 「現在の状態」見出しを `v14.5.0` に更新
  - ロードマップ表に v14.1.0〜v14.5.0（完了）を追記
  - コード例の注記整理: `stage` ブロックの `!Io` / `!Db` 等は「--legacy モードでのみ有効」ラベルを追加、または v14.0.0 スタイルのサンプルに差し替え
  - 機能一覧表に Azure クラウドサポート行を追加

- テスト: `v146000_tests`
  - `version_is_14_6_0`
  - `changelog_has_v14_5_0_entry`（CHANGELOG ファイルに `v14.5.0` 文字列が存在）
  - `readme_mentions_azure_blob`（README に `AzureBlob` への言及が存在）

---

### v14.7.0 — site/ ドキュメント更新 + rune ファイル精査

**テーマ**: site/content/docs/ の v14.0.0 以前記述を修正し、rune ファイルの ambient スタイルを洗い出す

**背景（2026-06-12 調査結果）:**
- `site/content/docs/introduction.mdx`: 旧 `!Io / !Db / !AWS / !Auth / !Env` 体系で説明。`fav deploy` / `MCP` / `Notebook` という存在しない機能が記載されている
- `site/content/docs/language/effects.mdx`: 旧エフェクトシステムのドキュメント。エラーコード `E0370`（実在しない）。v14.0.0 Capability Context への言及ゼロ
- `site/content/docs/quickstart.mdx`: `!Io`, `!Db`, `!AWS` 旧スタイル全面使用
- `site/content/docs/runes/aws.mdx` 等: `ctx:` なし ambient API を例示
- rune ファイルの `!Effect` 使用状況（v14.4 調査時点）:
  - VM プリミティブラッパー（`ctx: String` + `!Effect`）は v14.x パターンとして **意図的・正しい**
  - ただし `cache.fav`(12箇所)、`fs.fav`(15箇所)、`queue.fav`(11箇所) 等で ambient スタイルが混在している可能性あり → 個別確認が必要

**実装内容:**

- `site/content/docs/introduction.mdx`:
  - 旧エフェクト表を削除し Capability Context 体系に書き直し
  - 存在しない機能（fav deploy / MCP / Notebook）を削除
- `site/content/docs/language/effects.mdx`:
  - v14.0.0 Capability Context を主体に書き直し
  - 旧 `!Effect` は「--legacy モード」として付記
  - エラーコードを E0023 / E0025 に修正
- `site/content/docs/quickstart.mdx`: ctx パラメータスタイルのサンプルに更新
- `site/content/docs/runes/aws.mdx` 等: ctx-aware API 例示に更新
- rune ファイル精査（`cache.fav`, `fs.fav`, `queue.fav`, `log/emitter.fav`, `rune_loader/loader.fav` 等）:
  - `ctx:` パラメータなし ambient スタイル → 修正要否を判定
  - 修正が多い場合は v14.8.0 として分割

**テスト: `v147000_tests`**
  - `version_is_14_7_0`
  - `site_effects_doc_no_e0370`（effects.mdx に `E0370` が含まれない）
  - `site_introduction_no_fav_deploy`（introduction.mdx に `fav deploy` が含まれない）

---

### v14.8.0〜 — 調査結果次第（TBD）

**v14.7.0 の調査・修正作業を経て、追加対応が必要な場合に設ける。**

想定される積み残し候補:

| 候補 | 内容 | 発生条件 |
|---|---|---|
| rune ファイル修正 | ambient スタイル rune を ctx-aware に移行 | v14.7 精査で修正量が多い場合 |
| fav.toml [azure] 実装確認 | `inject_azure_config` の実装状況確認 | v14.2 実装が incomplete だった場合 |
| crosscloud plan 簡略化 | plan.md の複雑な要件（後述）を v15.0 スコープに絞り込む | v15.0 実装開始前に必要 |
| その他積み残し | v14.7 調査で新たに発見されたもの | 随時 |

**バージョン番号は v14.8.0, v14.9.0, ... と必要な数だけ増加させる。**

---

### v15.0.0 — CrossCloud E2E Demo

**テーマ**: AWS → Azure クロスクラウドマイグレーションの E2E デモ完成

> **⚠️ 注記（2026-06-12）: crosscloud/plan.md との乖離について**
>
> `infra/e2e-demo/crosscloud/plan.md` には、当初の想定より大幅に複雑な要件が記載されている:
> - Entra ID → Cognito 連携（ID フェデレーション）
> - HMAC リクエスト整合性チェック
> - API Gateway + Lambda verifier
> - Cognito クレームベースの認可（job_type フィルタリング）
> - 冪等性保証（重複行 INSERT 防止）
> - 成功・失敗両ケースの証跡記録
>
> v15.0.0 では以下の **簡略版シナリオ** に絞る。
> 複雑な要件（Entra ID 連携・Lambda verifier・認可）は v15.1.0 以降とする。
> v14.7〜v14.x 系の調査・修正を経て、実装前に plan.md を「簡略版」と「将来版」に分割すること。

**v15.0.0 スコープ（簡略版 5 ステージ）:**

- `infra/e2e-demo/crosscloud/src/migrate.fav`:
  ```
  fn migrate(aws_ctx: String, azure_ctx: String) -> Result<Unit, String> !AWS !AzureDb !AzureStorage

  seq MigrationPipeline [
    ExtractFromRds
    TransformRows
    LoadToAzurePostgres
    SaveProofToBlob
    VerifyRowCount
  ]

  public fn main(ctx: AppCtx) -> Result<Unit, String>
  ```

- `infra/e2e-demo/crosscloud/terraform/`:
  - `aws/main.tf`: RDS Postgres, Secrets Manager, IAM
  - `azure/main.tf`: Azure DB for PostgreSQL, Storage Account, Blob Container

- `infra/e2e-demo/crosscloud/scripts/`:
  - `run.sh`: AWS credentials + Azure credentials を取得し `fav run` を実行
  - `seed.sh`: AWS RDS にサンプルデータ投入（1000 行の txn テーブル）
  - `verify.sh`: Azure Postgres の行数 + Blob の証跡 JSON を確認

- `infra/e2e-demo/crosscloud/README.md`: セットアップ手順・前提条件

- テスト: `v150000_tests`:
  - `crosscloud_fav_parses`
  - `crosscloud_lineage_aws_and_azure`
  - `crosscloud_effects_declared`
  - `crosscloud_main_has_ctx_param`
  - `crosscloud_e2e_demo_structure`（ファイル存在確認）

- **完了条件 (PASS=5)**:
  1. `ExtractFromRds` — AWS RDS から 1000 行取得
  2. `TransformRows` — スキーマ変換（id/amount/ts → azure 形式）
  3. `LoadToAzurePostgres` — Azure DB for PostgreSQL に INSERT 完了
  4. `SaveProofToBlob` — Azure Blob に証跡 JSON 保存
  5. `VerifyRowCount` — source 行数 == target 行数 検証

---

## 依存関係（更新版）

```
v14.1.0 ✅  v14.2.0 ✅  v14.3.0 ✅  v14.4.0 ✅  v14.5.0 ✅
                                                        |
                                              v14.6.0（README/CHANGELOG）
                                                        |
                                              v14.7.0（site/ + rune 精査）
                                                        |
                                            v14.8.0〜（TBD: 調査次第）
                                                        |
                                              v15.0.0（CrossCloud E2E）
```

v14.6.0 と v14.7.0 は直列。v14.8.0 以降は v14.7.0 の調査結果で決定。
すべて v15.0.0 の前提。

---

## 新規 Cargo 依存（実績）

| Crate | 用途 | 追加バージョン | 状態 |
|---|---|---|---|
| `tokio-postgres-openssl` | Azure Postgres TLS | v14.1.0 | ✅ 追加済み |
| `hmac` / `sha2` / `base64` | Azure Blob Shared Key | 既存 | ✅ 流用 |
| `chrono` | Azure Blob RFC 1123 日付 | 既存 | ✅ 流用 |
| `azure_storage` / `azure_storage_blobs` | Azure Blob SDK | — | ❌ 不使用（ureq + 手動署名で実装） |
| `aws-sdk-secretsmanager` | Secrets Manager SDK | — | ❌ 不使用（SigV4 + ureq で実装） |

---

## 実装ノート

- **Azure Postgres の SSL**: Azure DB for PostgreSQL は SSL 必須。`sslmode=require` を接続文字列に含める。`tokio-postgres` + `tokio-postgres-openssl` + `openssl` crate を使用。
- **Azure Blob の認証**: Shared Key（account + key）。既存 `hmac 0.12` + `sha2 0.10` + `base64 0.22` で実装済み。SAS トークンは v15.x 以降で検討。
- **HMAC 整合性（crosscloud/plan.md）**: plan.md では Lambda verifier として設計されているが、v15.0.0 では Favnir 内でハッシュ計算し Blob に保存する形でシンプル化。Lambda verifier は v15.1.0 以降。
- **CrossCloudCtx の選択**: `fn migrate(aws_ctx: String, azure_ctx: String)` を基本形とする（`runes/ctx/ctx.fav` 未存在のため型付き Ctx は使わない）。
- **Windows dev 環境**: Azure SDK の TLS は OpenSSL が必要。Windows では `OPENSSL_DIR` 環境変数設定が必要になる可能性あり。
- **fav2py（Python トランスパイラ）**: CrossCloud デモは Favnir ネイティブのみ。Python トランスパイルは対象外。
- **rune ファイルの `!Effect` 使用**: VM プリミティブラッパーが `ctx: String` + `!Effect` の組み合わせで書かれているのは v14.x の正しいパターン。一方、`ctx:` パラメータなしで `IO.println` 等を ambient に呼んでいる rune は v14.7.0 で修正対象とする。
- **`let` 構文**: rune ファイル内で `let x = ...` を使うとパースエラー（"expected RBrace, got Ident"）になる。中間値はインライン化するか `bind` を使うこと。
