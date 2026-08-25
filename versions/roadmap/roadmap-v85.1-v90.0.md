# Favnir ロードマップ v85.1〜v90.0 — SAP Integration Era

Date: 2026-08-22
Status: 完了（v90.0.0 宣言済み）

---

## 背景と方針

v85.0.0「Favnir 4.0 宣言」をもって、Quality-First Era が完成した。
テスト・品質・契約・可観測性の 4 層が揃い、Favnir は「データパイプラインの品質をコードと同じ言語で語れる」言語になった。

次のフェーズでは **SAP との型安全な統合** に取り組む。

SAP は世界最大の ERP システムであり、多くの企業においてデータソース No.1 の地位を占める。
しかし SAP データへのアクセスは従来 ABAP / RFC といった専用プロトコルに閉じており、
データエンジニアが型安全なパイプラインを書ける環境は存在しなかった。

v85.1〜v90.0 では **SAP OData v4 REST API** をベースに、
Favnir の ctx パターン（`ctx.sap.*`）で SAP データを型安全に扱える `sap-odata` Rune を実装する。

```
v86.0 — SAP Foundation 1.0  : 「SAP に型安全に接続できる」
v87.0 — SAP Master Data 1.0 : 「BusinessPartner が型になる」
v88.0 — SAP Sales 1.0       : 「SalesOrder が型になる」
v89.0 — SAP Procurement 1.0 : 「Material と PurchaseOrder が型になる」
v90.0 — SAP Integration 1.0 : 「SAP 全体が Favnir で語れる」
```

### 設計方針

- **OData v4 のみ**（RFC / BAPI は C ライブラリ依存のため対象外）
- **`!Sap` エフェクトは使わない**（E0025 — bang 記法は廃止済み）
- **案 B: エンティティ別関数**（`sap_odata.sales_orders(cfg, filter)` 等、型安全な関数単位）
  - `cfg: SapConfig` を第一引数に取る形式で統一（Snowflake Rune と同じパターン）
  - ユーザーコードは `bind cfg <- sap_odata.sap_config_from_env()` で設定を取得してから Rune 関数を呼ぶ
  - `ctx.sap.*` フィールドは実装しない（Rune 関数に直接 `cfg` を渡す設計）
- **`fav.toml [sap]`** → `inject_sap_config()` → env vars → `sap_config_from_env()` で読み取り（Snowflake と同パターン）
- **開発環境**: Docker Compose で SAP OData モックサーバー（SAP 公式 OSS）をローカル起動
- **AWS**: Rune Registry（既存 Lambda）にデプロイ + E2E デモ Lambda + SSM Parameter Store

### 業務シナリオ（テスト基軸）

| シナリオ | エンティティ | Sprint |
|---|---|---|
| 1. マスタデータ同期（SAP → S3） | BusinessPartner | Sprint 2 |
| 2. 日次売上レポート（受注集計） | SalesOrder + SalesOrderItem | Sprint 3 |
| 3. 在庫 × 受注クロスチェック | Material × SalesOrder | Sprint 4 |
| 4. 購買 → 支払サイクル照合 | PurchaseOrder × JournalEntry | Sprint 5 |

---

## テスト数推移（本スプリント全体）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v85.0.0（ベース） | 3,931 | — |
| v85.1.0〜v85.9.0 | +2 × 9 = +18 | 3,949 |
| v86.0.0（宣言） | +4 | 3,953 |
| v86.1.0〜v86.9.0 | +2 × 9 = +18 | 3,971 |
| v87.0.0（宣言） | +4 | 3,975 |
| v87.1.0〜v87.9.0 | +2 × 9 = +18 | 3,993 |
| v88.0.0（宣言） | +4 | 3,997 |
| v88.1.0〜v88.9.0 | +2 × 9 = +18 | 4,015 |
| v89.0.0（宣言） | +4 | 4,019 |
| v89.1.0〜v89.9.0 | +2 × 9 = +18 | 4,037 |
| v90.0.0（宣言） | +4 | 4,041 |

**本スプリント合計**: +110 tests（3,931 → 4,041）

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 1: SAP Foundation 1.0（v85.1〜v86.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SAP OData v4 への接続基盤と `sap-odata` Rune の骨格を作る。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v85.1.0 | `SapTomlConfig` + `inject_sap_config()`（Rust 基盤） | 3931 + 2 = 3933 | 完了 |
| v85.2.0 | `SapConfig` Favnir 型 + `sap_config_from_env()` | 3933 + 2 = 3935 | 完了 |
| v85.3.0 | Docker Compose — SAP OData モックサーバー構築 | 3935 + 2 = 3937 | 完了 |
| v85.4.0 | `runes/sap-odata/` 骨格 + `rune.toml` | 3937 + 2 = 3939 | 完了 |
| v85.5.0 | OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`） | 3939 + 2 = 3941 | 完了 |
| v85.6.0 | `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク） | 3941 + 2 = 3943 | 完了 |
| v85.7.0 | `fav new` テンプレート + `fav.toml [sap]` セクション追加 | 3943 + 2 = 3945 | 完了 |
| v85.8.0 | SSM Parameter Store 設定（`infra/sap/ssm.tf`） | 3945 + 2 = 3947 | 完了 |
| v85.9.0 | 安定化・コードフリーズ | 3947 + 2 = 3949 | 完了 |
| v86.0.0 | SAP Foundation 1.0 宣言 ★クリーンアップ | 3949 + 4 = 3953 | 完了 |

---

### v85.1.0 — `SapTomlConfig` + `inject_sap_config()`

`fav.toml [sap]` セクションを解析し、env vars に注入する Rust 基盤。
Snowflake（v10.7.0）・Postgres（v11.5.0）と同じパターンで実装する。

**実装内容:**
- `fav/src/toml.rs` に `SapTomlConfig` 構造体を追加:
  - `base_url: Option<String>`（例: `"https://my-s4hana.example.com"`）
  - `client: Option<String>`（SAP クライアント番号、例: `"100"`）
  - `username: Option<String>`（`"${SAP_USER}"` 形式の env var 展開対応）
  - `password: Option<String>`（`"${SAP_PASS}"` 形式）
  - `auth: Option<String>`（`"basic"` | `"oauth2"`、デフォルト `"basic"`）
- `fav/src/driver.rs` に `inject_sap_config(cfg: &SapTomlConfig)` を追加:
  - `SAP_BASE_URL` / `SAP_CLIENT` / `SAP_USER` / `SAP_PASS` / `SAP_AUTH` を env に設定
  - `expand_env_vars()` で `${VAR}` 形式を展開
- `FavTomlProject` に `sap: Option<SapTomlConfig>` フィールドを追加
- `cmd_run` / `cmd_check` の config 読み込み箇所で `inject_sap_config` を呼ぶ

**完了条件**: Rust テスト 2 件（3931 + 2 = 3933）
- `sap_toml_config_parses_base_url`
- `inject_sap_config_sets_env_vars`

---

### v85.2.0 — `SapConfig` Favnir 型 + `sap_config_from_env()`

SAP 接続設定を Favnir の型として表現する。

**実装内容:**
- `runes/sap-odata/types.fav` を新規作成し、型定義を追加:
- `fav/src/driver.rs` に `mod v85200_tests`（Rust テスト 2 件）を追加
- Favnir 側（`runes/sap-odata/types.fav`）に型定義を追加:
  ```favnir
  type SapConfig = {
      base_url: String,
      client:   String,
      username: String,
      password: String,
      auth:     String
  }

  public fn sap_config_from_env() -> Result<SapConfig, String> {
      bind base_url <- Env.require("SAP_BASE_URL")
      bind username <- Env.require("SAP_USER")
      bind password <- Env.require("SAP_PASS")
      Result.ok(SapConfig {
          base_url,
          username,
          password,
          client: Env.get_or("SAP_CLIENT", "100"),
          auth:   Env.get_or("SAP_AUTH", "basic")
      })
  }
  -- 注: Env.require は Result<String, String> を返すため bind を使う。
  -- Env.get_or はデフォルト値ありで String を直接返すため bind を使わない。
  ```

**完了条件**: Rust テスト 2 件（3933 + 2 = 3935）
- `sap_config_from_env_returns_ok_when_vars_set`
- `sap_config_from_env_returns_err_when_base_url_missing`

---

### v85.3.0 — Docker Compose — SAP OData モックサーバー構築

SAP 公式 OSS モックサーバーを Docker Compose で起動できるようにする。
本番 SAP ライセンスなしでローカル開発・テストが可能になる。

**実装内容:**
- `infra/e2e-demo/sap-odata/docker-compose.yml` を作成:
  - `sap-mock` サービス: SAP UI5 FE Mock Server（Node.js）
  - `favnir-runner` サービス: パイプライン実行コンテナ
- `infra/e2e-demo/sap-odata/mock/` にモックデータ（JSON）を配置:
  - `BusinessPartnerCollection.json`（10 件のサンプル）
  - `SalesOrderCollection.json`（10 件のサンプル）
- `infra/e2e-demo/sap-odata/README.md` に起動手順を記述
- `scripts/start-sap-mock.sh` 起動スクリプト

**完了条件**: Rust テスト 2 件（3935 + 2 = 3937）
- `sap_mock_docker_compose_exists`（`infra/e2e-demo/sap-odata/docker-compose.yml` が存在する）
- `sap_mock_data_business_partner_exists`（`infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json` が存在する）

---

### v85.4.0 — `runes/sap-odata/` 骨格 + `rune.toml`

Rune の骨格ファイルを作成し、Rune Registry に登録できる状態にする。

**実装内容:**
- `runes/sap-odata/rune.toml` を作成:
  ```toml
  [rune]
  name        = "sap-odata"
  version     = "85.4.0"
  entry       = "sap_odata.fav"
  description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"
  ```
  （`effects` フィールドは省略 — ctx パターンのため `!Sap` エフェクト不要）
- `runes/sap-odata/sap_odata.fav`（エントリポイント、他ファイルを use）
- `runes/sap-odata/types.fav`（v85.2.0 作成済み — 本バージョンで変更なし）
- `runes/sap-odata/sap_odata.test.fav`（テストファイル骨格）

**完了条件**: Rust テスト 2 件（3937 + 2 = 3939）
- `sap_odata_rune_toml_exists`
- `sap_odata_rune_entry_exists`

---

### v85.5.0 — OData v4 HTTP クライアント基盤

OData v4 の GET / LIST クエリを HTTP Rune で実装する基盤。

**実装内容:**
- `runes/sap-odata/client.fav` に内部クライアント関数を実装:
  ```favnir
  -- 単一エンティティ取得（GET /Entity('key')）
  -- sap_odata.fav 経由で re-export するため client.fav では public fn を使う
  public fn odata_get(cfg: SapConfig, entity_set: String, key: String) -> Result<String, String>

  -- コレクション取得（GET /EntitySet?$filter=...&$top=...）
  public fn odata_list(cfg: SapConfig, entity_set: String, params: ODataParams) -> Result<String, String>

  -- OData クエリパラメータ型（types.fav に定義）
  type ODataParams = {
      filter:  Option<String>,   -- $filter
      select:  Option<String>,   -- $select
      expand:  Option<String>,   -- $expand
      top:     Option<Int>,      -- $top
      skip:    Option<Int>,      -- $skip
      orderby: Option<String>    -- $orderby
  }
  ```
- Basic 認証ヘッダー生成（`Authorization: Basic base64(user:pass)`）
- `x-csrf-token` フェッチは v86.x 系に延期（POST/PATCH 操作用のため GET/LIST 基盤の本バージョンでは対象外）

**完了条件**: Rust テスト 2 件（3939 + 2 = 3941）
- `odata_list_function_exists_in_rune`（`sap_odata.fav` に `odata_list` が含まれることを確認）
- `odata_params_type_exists`（`types.fav` に `ODataParams` が含まれることを確認）

---

### v85.6.0 — `SapError` 型 + エラーハンドリング

SAP 固有のエラー応答を型で表現し、わかりやすいエラーメッセージを返す。

**実装内容:**
- `runes/sap-odata/types.fav` に追加:
  ```favnir
  type SapErrorCode = NotFound | Unauthorized | Forbidden | BadRequest | ServerError | NetworkError

  type SapError = {
      code:    SapErrorCode,
      message: String,
      detail:  Option<String>   -- SAP OData error.innererror の詳細
  }
  ```
- HTTP 400 → `BadRequest` / 401 → `Unauthorized` / 403 → `Forbidden` / 404 → `NotFound` / 5xx → `ServerError`
- OData v4 エラーレスポンス（`{"error": {"code": "...", "message": "..."}}`）に対応する型定義（パース処理の実装は v85.9.0 安定化バージョンで実施）

**完了条件**: Rust テスト 2 件（3941 + 2 = 3943）
- `sap_error_type_exists`
- `sap_error_code_variants_exist`

---

### v85.7.0 — `fav new` テンプレート + `fav.toml [sap]` セクション

`fav new` で生成される `fav.toml` に SAP 設定のコメントを追加する。

**実装内容:**
- `fav/src/driver.rs` の `default_fav_toml()` に `[sap]` テンプレートコメントを追加:
  ```toml
  # [sap]
  # base_url = "${SAP_BASE_URL}"   # SAP S/4HANA エンドポイント
  # client   = "100"               # SAP クライアント番号
  # username = "${SAP_USER}"
  # password = "${SAP_PASS}"
  # auth     = "basic"             # "basic" | "oauth2"
  ```

**完了条件**: Rust テスト 2 件（3943 + 2 = 3945）
- `fav_new_template_contains_sap_comment`
- `sap_toml_section_parses_correctly`（`[sap]` セクションの解析テスト）

---

### v85.8.0 — SSM Parameter Store 設定

SAP 接続情報を AWS SSM Parameter Store で安全に管理する Terraform。

**実装内容:**
- `infra/sap/ssm.tf` を作成:
  ```hcl
  resource "aws_ssm_parameter" "sap_base_url" {
    name  = "/favnir/sap/base_url"
    type  = "String"
    value = var.sap_base_url
  }
  resource "aws_ssm_parameter" "sap_username" {
    name  = "/favnir/sap/username"
    type  = "SecureString"
    value = var.sap_username
  }
  resource "aws_ssm_parameter" "sap_password" {
    name  = "/favnir/sap/password"
    type  = "SecureString"
    value = var.sap_password
  }
  ```
- `infra/sap/variables.tf` / `infra/sap/providers.tf` / `infra/sap/outputs.tf`
- `infra/sap/README.md`（セットアップ手順）

**完了条件**: Rust テスト 2 件（3945 + 2 = 3947）
- `sap_infra_ssm_tf_exists`
- `sap_infra_readme_exists`

---

### v85.9.0 — 安定化・コードフリーズ

v85.1〜v85.8 の全機能を通しで確認する安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認
- Docker Compose でモックサーバー起動確認
- `fav.toml [sap]` の解析・env 注入の動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3947 + 2 = 3949）
- `sap_foundation_rune_toml_has_correct_name`
- `sap_foundation_docker_compose_has_sap_mock_service`

---

### v86.0.0 — SAP Foundation 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「SAP に、型安全に接続できるようになった。
>  `fav.toml [sap]` を書けば、Favnir が SAP OData v4 と話せる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `86.0.0` に更新
- driver.rs 内の旧 `cargo_toml_version` テスト（35 件）を `86.0.0` に一括更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v86.0.0 に更新

**完了条件**: `v86000_tests` 4 件（3949 + 4 = 3953）
- `cargo_toml_version_is_86_0_0`
- `changelog_has_v86_0_0`
- `milestone_has_sap_foundation`
- `readme_mentions_sap_integration`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 2: SAP Master Data 1.0（v86.1〜v87.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: `BusinessPartner`（得意先・仕入先の統合マスタ）を型安全に扱えるようにする。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v86.1.0 | `BusinessPartner` / `BusinessPartnerAddress` 型定義 | 3953 + 2 = 3955 | 完了 |
| v86.2.0 | `BusinessPartnerFilter` + `business_partners()` クエリ | 3955 + 2 = 3957 | 完了 |
| v86.3.0 | `business_partner_by_id()` 単一取得 + `$expand=to_BusinessPartnerAddress` | 3957 + 2 = 3959 | 完了 |
| v86.4.0 | `create_business_partner()` POST 実装 | 3959 + 2 = 3961 | 完了 |
| v86.5.0 | `update_business_partner()` PATCH 実装 | 3961 + 2 = 3963 | 完了 |
| v86.6.0 | シナリオ 1: マスタデータ同期（BusinessPartner → S3 JSON） | 3963 + 2 = 3965 | 完了 |
| v86.7.0 | モックサーバーテスト — CRUD 全操作をモックで検証 | 3965 + 2 = 3967 | 完了 |
| v86.8.0 | Rune Registry デプロイ（`sap-odata` を Lambda Registry に登録） | 3967 + 2 = 3969 | 完了 |
| v86.9.0 | 安定化・コードフリーズ | 3969 + 2 = 3971 | 完了 |
| v87.0.0 | SAP Master Data 1.0 宣言 ★クリーンアップ | 3971 + 4 = 3975 | 完了 |

---

### v86.1.0 — `BusinessPartner` / `BusinessPartnerAddress` 型定義

```favnir
type BusinessPartnerCategory = Person | Organization | Group

type BusinessPartner = {
    partner_id:       String,
    name:             String,
    category:         BusinessPartnerCategory,
    country:          String,
    language:         String,
    currency:         String,
    created_at:       String,
    addresses:        Option<List<BusinessPartnerAddress>>
}

type BusinessPartnerAddress = {
    address_id:   String,
    street:       String,
    city:         String,
    postal_code:  String,
    country:      String,
    region:       Option<String>
}
```

**完了条件**: Rust テスト 2 件（3953 + 2 = 3955）
- `business_partner_type_defined_in_rune`
- `business_partner_address_type_defined_in_rune`

---

### v86.2.0 — `BusinessPartnerFilter` + `business_partners()` クエリ

```favnir
type BusinessPartnerFilter = {
    country:       Option<String>,
    category:      Option<BusinessPartnerCategory>,
    changed_after: Option<String>,   -- ISO8601 日付文字列
    top:           Option<Int>
}

public fn business_partners(
    cfg:    SapConfig,
    filter: BusinessPartnerFilter
) -> Result<List<BusinessPartner>, String>
```

**完了条件**: Rust テスト 2 件（3955 + 2 = 3957）
- `business_partners_function_exists`
- `business_partner_filter_type_exists`

---

### v86.3.0 — `business_partner_by_id()` + `$expand`

```favnir
public fn business_partner_by_id(
    cfg:            SapConfig,
    partner_id:     String,
    expand_address: Bool
) -> Result<BusinessPartner, String>
```

`expand_address = true` の場合 `$expand=to_BusinessPartnerAddress` を付与し、
住所情報を 1 リクエストで取得する。

**完了条件**: Rust テスト 2 件（3957 + 2 = 3959）
- `business_partner_by_id_function_exists`
- `business_partner_expand_address_in_rune`

---

### v86.4.0 — `create_business_partner()` POST

```favnir
type NewBusinessPartner = {
    name:     String,
    category: BusinessPartnerCategory,
    country:  String,
    currency: String
}

public fn create_business_partner(
    cfg:  SapConfig,
    body: NewBusinessPartner
) -> Result<BusinessPartner, String>
```

POST 前に `x-csrf-token` を取得し、リクエストヘッダーに付与する。

**完了条件**: Rust テスト 2 件（3959 + 2 = 3961）
- `create_business_partner_function_exists`
- `new_business_partner_type_exists`

---

### v86.5.0 — `update_business_partner()` PATCH

```favnir
type BusinessPartnerPatch = {
    name:     Option<String>,
    currency: Option<String>,
    language: Option<String>
}

public fn update_business_partner(
    cfg:        SapConfig,
    partner_id: String,
    patch:      BusinessPartnerPatch
) -> Result<Unit, String>
```

**完了条件**: Rust テスト 2 件（3961 + 2 = 3963）
- `update_business_partner_function_exists`
- `business_partner_patch_type_exists`

---

### v86.6.0 — シナリオ 1: マスタデータ同期（BusinessPartner → S3）

業務シナリオ 1 の E2E テスト実装。

```favnir
-- infra/e2e-demo/sap-odata/pipeline.fav
-- 注: v86.6.0 時点では Registry デプロイ（v86.8.0）前のため、
--     ローカル Rune ファイルを直接参照する。
--     v86.8.0 以降は import rune "sap-odata" に切り替える。

import rune "s3"

fn sync_business_partners(ctx: AppCtx) -> Result<Int, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind partners <- sap_odata.business_partners(cfg, BusinessPartnerFilter {
        country:       Option.some("JP"),
        changed_after: Option.some("2026-08-01"),
        top:           Option.some(500),
        category:      Option.none(),
    })
    bind json     <- Json.encode(partners)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "partners/latest.json", json)
    Result.ok(List.length(partners))
}
```

**完了条件**: Rust テスト 2 件（3963 + 2 = 3965）
- `sap_e2e_pipeline_fav_exists`
- `sap_e2e_pipeline_contains_sync_business_partners`

---

### v86.7.0 — モックサーバーテスト — CRUD 全操作をモックで検証

`sap_odata.test.fav` に CRUD 全操作のテストを追加。
Docker Compose モックサーバーへの実際の HTTP リクエストで検証。

**完了条件**: Rust テスト 2 件（3965 + 2 = 3967）
- `sap_odata_test_fav_exists`
- `sap_odata_test_contains_business_partner_tests`

---

### v86.8.0 — Rune Registry デプロイ

`sap-odata` Rune を既存の Lambda Rune Registry に登録する。

**実装内容:**
- `deploy-registry` スキルを実行して `sap-odata` Rune をデプロイ
- DynamoDB (`favnir-rune-registry`) にメタデータ登録
- S3 (`favnir-rune-packages`) に `.fav` ファイルをアップロード
- `import rune "sap-odata"` で使えることを確認

**完了条件**: Rust テスト 2 件（3967 + 2 = 3969）
- `sap_odata_rune_version_matches_cargo`
- `sap_odata_rune_entry_file_is_sap_odata_fav`

---

### v86.9.0 — 安定化・コードフリーズ

**完了条件**: Rust テスト 2 件（3969 + 2 = 3971）
- `sap_master_data_business_partner_crud_covered`
- `sap_master_data_scenario1_pipeline_exists`

---

### v87.0.0 — SAP Master Data 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「SAP の BusinessPartner が、Favnir の型になった。
>  得意先も仕入先も、`business_partners()` で型安全に取得できる。」

**完了条件**: `v87000_tests` 4 件（3971 + 4 = 3975）
- `cargo_toml_version_is_87_0_0`
- `changelog_has_v87_0_0`
- `milestone_has_sap_master_data`
- `sap_odata_rune_toml_has_name_sap_odata`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 3: SAP Sales 1.0（v87.1〜v88.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 受注伝票（`SalesOrder` / `SalesOrderItem`）を型安全に扱えるようにする。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v87.1.0 | `SalesOrder` / `SalesOrderItem` 型定義 | 3975 + 2 = 3977 | 完了 |
| v87.2.0 | `SalesOrderFilter` + `sales_orders()` クエリ | 3977 + 2 = 3979 | 完了 |
| v87.3.0 | `sales_order_by_id()` + `$expand=to_Item` | 3979 + 2 = 3981 | 完了 |
| v87.4.0 | `create_sales_order()` + `NewSalesOrder` | 3981 + 2 = 3983 | 完了 |
| v87.5.0 | シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3） | 3983 + 2 = 3985 | 完了 |
| v87.6.0 | ページネーション基盤（`$top` / `$skip` / `@odata.nextLink`） | 3985 + 2 = 3987 | 完了 |
| v87.7.0 | `SalesReport` 集計型 + `group_by_currency()` | 3987 + 2 = 3989 | 完了 |
| v87.8.0 | モックサーバーテスト — 受注シナリオ全操作検証 | 3989 + 2 = 3991 | 完了 |
| v87.9.0 | 安定化・コードフリーズ | 3991 + 2 = 3993 | 完了 |
| v88.0.0 | SAP Sales 1.0 宣言 ★クリーンアップ | 3993 + 4 = 3997 | 完了 |

---

### v87.1.0 — `SalesOrder` / `SalesOrderItem` 型定義

```favnir
type SalesOrderStatus = Open | InProcess | Completed | Cancelled

type SalesOrderItem = {
    item_number:  Int,
    material_id:  String,
    description:  String,
    quantity:     Float,
    unit:         String,
    net_amount:   Float,
    currency:     String
}

type SalesOrder = {
    order_id:      String,
    customer_id:   String,
    status:        SalesOrderStatus,
    total_amount:  Float,
    currency:      String,
    sales_org:     String,
    created_at:    String,
    items:         Option<List<SalesOrderItem>>
}
```

**完了条件**: Rust テスト 2 件（3975 + 2 = 3977）
- `sales_order_type_defined_in_rune`
- `sales_order_item_type_defined_in_rune`

---

### v87.2.0 — `SalesOrderFilter` + `sales_orders()` クエリ

```favnir
type SalesOrderFilter = {
    customer_id:    Option<String>,
    status:         Option<SalesOrderStatus>,
    created_after:  Option<String>,
    created_before: Option<String>,
    sales_org:      Option<String>,
    top:            Option<Int>
}

public fn sales_orders(
    cfg:    SapConfig,
    filter: SalesOrderFilter
) -> Result<List<SalesOrder>, String>
```

**完了条件**: Rust テスト 2 件（3977 + 2 = 3979）
- `sales_orders_function_exists`
- `sales_order_filter_type_exists`

---

### v87.3.0 — `sales_order_by_id()` + `$expand=to_Item`

```favnir
public fn sales_order_by_id(
    cfg:         SapConfig,
    order_id:    String,
    expand_items: Bool
) -> Result<SalesOrder, String>
```

`expand_items = true` で `$expand=to_Item` を付与し、明細を含む完全な受注を取得する。

**完了条件**: Rust テスト 2 件（3979 + 2 = 3981）
- `sales_order_by_id_function_exists`
- `sales_order_expand_items_in_rune`

---

### v87.4.0 — `create_sales_order()` + `NewSalesOrder`

```favnir
type NewSalesOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String
}

type NewSalesOrder = {
    customer_id: String,
    sales_org:   String,
    currency:    String,
    items:       List<NewSalesOrderItem>
}

public fn create_sales_order(
    cfg:   SapConfig,
    order: NewSalesOrder
) -> Result<SalesOrder, String>
```

**完了条件**: Rust テスト 2 件（3981 + 2 = 3983）
- `create_sales_order_function_exists`
- `new_sales_order_type_exists`

---

### v87.5.0 — シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）

業務シナリオ 2 の E2E 実装。

```favnir
type SalesReport = {
    report_date:   String,
    total_orders:  Int,
    total_amount:  Float,
    by_currency:   List<CurrencyTotal>
}

type CurrencyTotal = {
    currency: String,
    amount:   Float,
    count:    Int
}

fn daily_sales_report(ctx: AppCtx) -> Result<SalesReport, String> {
    bind cfg    <- sap_odata.sap_config_from_env()
    bind orders <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status:        Option.some(SalesOrderStatus.Completed),
        created_after: Option.some("2026-08-22"),
        customer_id:   Option.none(),
        created_before: Option.none(),
        sales_org:     Option.none(),
        top:           Option.none()
    })
    bind report <- build_sales_report("2026-08-22", orders)
    bind json   <- Json.encode(report)
    bind _      <- ctx.s3.put_object("favnir-sap-demo", "reports/daily/2026-08-22.json", json)
    Result.ok(report)
}
```

**完了条件**: Rust テスト 2 件（3983 + 2 = 3985）
- `sales_report_type_exists`
- `sap_e2e_pipeline_contains_daily_sales_report`

---

### v87.6.0 — ページネーション基盤

大量データ（SAP は 100 万件超の受注を持つケースがある）を安全に処理するためのページネーション。

```favnir
type PagedResult = {
    items:      List<String>,   -- JSON 行のリスト
    next_token: Option<String>  -- @odata.nextLink から抽出した URL
}

fn odata_list_paged(
    cfg:       SapConfig,
    entity_set: String,
    params:    ODataParams,
    page_size: Int
) -> Result<PagedResult, String>

fn odata_collect_all(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    max_pages:  Int
) -> Result<List<String>, String>
```

**完了条件**: Rust テスト 2 件（3985 + 2 = 3987）
- `paged_result_type_exists`
- `odata_list_paged_function_exists`

---

### v87.7.0 — `SalesReport` 集計型 + `group_by_currency()`

```favnir
fn group_by_currency(orders: List<SalesOrder>) -> List<CurrencyTotal>
fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String>
fn format_sales_report(report: SalesReport) -> String
```

**完了条件**: Rust テスト 2 件（3987 + 2 = 3989）
- `group_by_currency_function_exists`
- `format_sales_report_function_exists`

---

### v87.8.0 — モックサーバーテスト — 受注シナリオ全操作検証

**完了条件**: Rust テスト 2 件（3989 + 2 = 3991）
- `sap_odata_test_contains_sales_order_tests`
- `sap_odata_test_contains_pagination_test`

---

### v87.9.0 — 安定化・コードフリーズ

**完了条件**: Rust テスト 2 件（3991 + 2 = 3993）
- `sap_sales_order_crud_covered`
- `sap_sales_scenario2_report_pipeline_exists`

---

### v88.0.0 — SAP Sales 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「受注が型になった。
>  `sales_orders()` で絞り込み、`sales_order_by_id()` で明細まで取得できる。
>  日次売上レポートが、Favnir の 10 行で書ける。」

**完了条件**: `v88000_tests` 4 件（3993 + 4 = 3997）
- `cargo_toml_version_is_88_0_0`
- `changelog_has_v88_0_0`
- `milestone_has_sap_sales`
- `sap_odata_rune_has_sales_order_type`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 4: SAP Procurement 1.0（v88.1〜v89.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 品目マスタ（`Material`）・発注伝票（`PurchaseOrder`）を型安全に扱えるようにする。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v88.1.0 | `Material` 型定義 + `MaterialFilter` + `materials()` | 3997 + 2 = 3999 | 完了 |
| v88.2.0 | `material_by_id()` + `MaterialType` enum | 3999 + 2 = 4001 | 完了 |
| v88.3.0 | `PurchaseOrder` / `PurchaseOrderItem` 型定義 | 4001 + 2 = 4003 | 完了 |
| v88.4.0 | `purchase_orders()` / `purchase_order_by_id()` クエリ | 4003 + 2 = 4005 | 完了 |
| v88.5.0 | `create_purchase_order()` POST 実装 | 4005 + 2 = 4007 | 完了 |
| v88.6.0 | シナリオ 3: 在庫 × 受注クロスチェック（Material × SalesOrder） | 4007 + 2 = 4009 | 完了 |
| v88.7.0 | `StockAlert` 型 + `detect_stock_shortage()` | 4009 + 2 = 4011 | 完了 |
| v88.8.0 | E2E デモ Lambda 基盤（`infra/e2e-demo/sap-odata/terraform/`） | 4011 + 2 = 4013 | 完了 |
| v88.9.0 | 安定化・コードフリーズ | 4013 + 2 = 4015 | 完了 |
| v89.0.0 | SAP Procurement 1.0 宣言 ★クリーンアップ | 4015 + 4 = 4019 | 完了 |

---

### v88.1.0 — `Material` 型定義 + `MaterialFilter` + `materials()`

```favnir
type MaterialType = FinishedProduct | RawMaterial | SemiFinished | Trading | Service

type Material = {
    material_id:   String,
    description:   String,
    material_type: MaterialType,
    base_unit:     String,
    weight:        Option<Float>,
    weight_unit:   Option<String>,
    plant:         Option<String>
}

type MaterialFilter = {
    material_type: Option<MaterialType>,
    plant:         Option<String>,
    top:           Option<Int>
}

public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String>
```

**完了条件**: Rust テスト 2 件（3997 + 2 = 3999）
- `material_type_defined_in_rune`
- `materials_function_exists`

---

### v88.2.0 — `material_by_id()` + `MaterialType` enum 完全化

```favnir
public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String>
```

**完了条件**: Rust テスト 2 件（3999 + 2 = 4001）
- `material_by_id_function_exists`
- `material_type_enum_has_finished_product`

---

### v88.3.0 — `PurchaseOrder` / `PurchaseOrderItem` 型定義

```favnir
type PurchaseOrderStatus = Open | PartiallyDelivered | Completed | Cancelled

type PurchaseOrderItem = {
    item_number:  Int,
    material_id:  String,
    quantity:     Float,
    unit:         String,
    net_price:    Float,
    currency:     String,
    plant:        String
}

type PurchaseOrder = {
    po_number:    String,
    vendor_id:    String,
    status:       PurchaseOrderStatus,
    total_amount: Float,
    currency:     String,
    created_at:   String,
    items:        Option<List<PurchaseOrderItem>>
}
```

**完了条件**: Rust テスト 2 件（4001 + 2 = 4003）
- `purchase_order_type_defined_in_rune`
- `purchase_order_item_type_defined_in_rune`

---

### v88.4.0 — `purchase_orders()` / `purchase_order_by_id()` クエリ

```favnir
type PurchaseOrderFilter = {
    vendor_id:     Option<String>,
    status:        Option<PurchaseOrderStatus>,
    created_after: Option<String>,
    plant:         Option<String>,
    top:           Option<Int>
}

public fn purchase_orders(cfg: SapConfig, filter: PurchaseOrderFilter) -> Result<List<PurchaseOrder>, String>
public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String>
```

**完了条件**: Rust テスト 2 件（4003 + 2 = 4005）
- `purchase_orders_function_exists`
- `purchase_order_filter_type_exists`

---

### v88.5.0 — `create_purchase_order()` POST

```favnir
type NewPurchaseOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String,
    plant:       String
}

type NewPurchaseOrder = {
    vendor_id: String,
    currency:  String,
    items:     List<NewPurchaseOrderItem>
}

public fn create_purchase_order(cfg: SapConfig, order: NewPurchaseOrder) -> Result<PurchaseOrder, String>
```

**完了条件**: Rust テスト 2 件（4005 + 2 = 4007）
- `create_purchase_order_function_exists`
- `new_purchase_order_type_exists`

---

### v88.6.0 — シナリオ 3: 在庫 × 受注クロスチェック

業務シナリオ 3 の E2E 実装。

```favnir
fn check_stock_vs_orders(ctx: AppCtx) -> Result<List<StockAlert>, String> {
    bind cfg       <- sap_odata.sap_config_from_env()
    bind orders    <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status: Option.some(SalesOrderStatus.Open),
        customer_id: Option.none(), created_after: Option.none(),
        created_before: Option.none(), sales_org: Option.none(), top: Option.none()
    })
    bind materials <- sap_odata.materials(cfg, MaterialFilter {
        material_type: Option.some(MaterialType.FinishedProduct),
        plant: Option.none(), top: Option.none()
    })
    bind alerts    <- sap_odata.detect_stock_shortage(orders, materials)
    Result.ok(alerts)
}
```

**完了条件**: Rust テスト 2 件（4007 + 2 = 4009）
- `sap_e2e_pipeline_contains_check_stock_vs_orders`
- `stock_alert_type_exists`

---

### v88.7.0 — `StockAlert` 型 + `detect_stock_shortage()`

```favnir
type StockSeverity = Critical | Warning | Info

type StockAlert = {
    material_id:   String,
    description:   String,
    severity:      StockSeverity,
    open_quantity: Float,
    message:       String
}

-- 注: detect_stock_shortage は SAP への HTTP アクセスをしない純粋な計算関数のため
--     cfg: SapConfig を取らない（他の Rune 関数との例外的な違い）。
public fn detect_stock_shortage(
    orders:    List<SalesOrder>,
    materials: List<Material>
) -> Result<List<StockAlert>, String>

fn format_stock_alerts(alerts: List<StockAlert>) -> String
```

**完了条件**: Rust テスト 2 件（4009 + 2 = 4011）
- `detect_stock_shortage_function_exists`
- `format_stock_alerts_function_exists`

---

### v88.8.0 — E2E デモ Lambda 基盤

SAP パイプラインを AWS Lambda で実行するデモ基盤。

**実装内容:**
- `infra/e2e-demo/sap-odata/terraform/main.tf`（Lambda + IAM + S3 出力バケット）
- `infra/e2e-demo/sap-odata/terraform/ssm.tf`（SSM パラメータ参照）
- `infra/e2e-demo/sap-odata/terraform/variables.tf`
- `infra/e2e-demo/sap-odata/scripts/run.sh`（デモ実行スクリプト）

**完了条件**: Rust テスト 2 件（4011 + 2 = 4013）
- `sap_e2e_demo_terraform_exists`
- `sap_e2e_demo_run_script_exists`

---

### v88.9.0 — 安定化・コードフリーズ

**完了条件**: Rust テスト 2 件（4013 + 2 = 4015）
- `sap_procurement_material_and_po_covered`
- `sap_procurement_scenario3_pipeline_exists`

---

### v89.0.0 — SAP Procurement 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

**完了条件**: `v89000_tests` 4 件（4015 + 4 = 4019）
- `cargo_toml_version_is_89_0_0`
- `changelog_has_v89_0_0`
- `milestone_has_sap_procurement`
- `sap_odata_rune_has_material_type`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 5: SAP Integration 1.0 宣言（v89.1〜v90.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 会計伝票（`JournalEntry`）追加 + 全 4 シナリオ E2E 完成 + SAP Integration 1.0 宣言。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v89.1.0 | `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()` | 4019 + 2 = 4021 | 完了 |
| v89.2.0 | `JournalEntryFilter` + `OutstandingPayable` 型 | 4021 + 2 = 4023 | 完了 |
| v89.3.0 | シナリオ 4: 購買→支払サイクル照合（PO × JE） | 4023 + 2 = 4025 | 完了 |
| v89.4.0 | `fav infer --from sap --entity <name>` コマンド | 4025 + 2 = 4027 | 完了 |
| v89.5.0 | E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ） | 4027 + 2 = 4029 | 完了 |
| v89.6.0 | `site/content/docs/runes/sap-odata.mdx` ドキュメント | 4029 + 2 = 4031 | 完了 |
| v89.7.0 | OSS 整備（CONTRIBUTING SAP セクション + ISSUE_TEMPLATE） | 4031 + 2 = 4033 | 完了 |
| v89.8.0 | パフォーマンス確認（ページネーション / バッチ / Lambda cold start） | 4033 + 2 = 4035 | 完了 |
| v89.9.0 | 安定化・コードフリーズ | 4035 + 2 = 4037 | 完了 |
| v90.0.0 | SAP Integration 1.0 宣言 ★クリーンアップ | 4037 + 4 = 4041 | 完了 |

---

### v89.1.0 — `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()`

```favnir
type DebitCredit = Debit | Credit

type JournalEntryItem = {
    item_number:  Int,
    gl_account:   String,
    amount:       Float,
    currency:     String,
    debit_credit: DebitCredit,
    cost_center:  Option<String>
}

type JournalEntry = {
    document_number:  String,
    fiscal_year:      Int,
    posting_date:     String,
    document_type:    String,
    company_code:     String,
    reference:        Option<String>,
    items:            Option<List<JournalEntryItem>>
}

type JournalFilter = {
    fiscal_year:      Option<Int>,
    posting_date_from: Option<String>,
    company_code:     Option<String>,
    reference:        Option<String>,
    top:              Option<Int>
}

public fn journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String>
```

**完了条件**: Rust テスト 2 件（4019 + 2 = 4021）
- `journal_entry_type_defined_in_rune`
- `journal_entries_function_exists`

---

### v89.2.0 — `JournalEntryFilter` + `OutstandingPayable` 型

```favnir
type OutstandingPayable = {
    po_number:    String,
    vendor_id:    String,
    total_amount: Float,
    currency:     String,
    days_overdue: Int,
    status:       String
}

fn match_unposted_orders(
    pos:      List<PurchaseOrder>,
    journals: List<JournalEntry>
) -> Result<List<OutstandingPayable>, String>
```

**完了条件**: Rust テスト 2 件（4021 + 2 = 4023）
- `outstanding_payable_type_exists`
- `match_unposted_orders_function_exists`

---

### v89.3.0 — シナリオ 4: 購買→支払サイクル照合

業務シナリオ 4 の E2E 実装。

```favnir
fn outstanding_payables(ctx: AppCtx) -> Result<List<OutstandingPayable>, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind pos      <- sap_odata.purchase_orders(cfg, PurchaseOrderFilter {
        status: Option.some(PurchaseOrderStatus.PartiallyDelivered),
        vendor_id: Option.none(), created_after: Option.none(),
        plant: Option.none(), top: Option.none()
    })
    bind journals <- sap_odata.journal_entries(cfg, JournalFilter {
        fiscal_year:       Option.some(2026),
        posting_date_from: Option.none(),
        company_code:      Option.none(),
        reference:         Option.none(),
        top:               Option.none()
    })
    bind unpaid   <- sap_odata.match_unposted_orders(pos, journals)
    bind json     <- Json.encode(unpaid)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "payables/outstanding.json", json)
    Result.ok(unpaid)
}
```

**完了条件**: Rust テスト 2 件（4023 + 2 = 4025）
- `sap_e2e_pipeline_contains_outstanding_payables`
- `sap_e2e_pipeline_has_all_four_scenarios`

---

### v89.4.0 — `fav infer --from sap --entity <name>` コマンド

SAP OData メタデータ（`$metadata`）から Favnir の型定義を自動生成するコマンド。

**実装内容:**
- `cmd_infer` に `--from sap` オプションを追加
- `--entity <EntitySetName>` でエンティティを指定
- OData $metadata XML をパースして Favnir 型を生成
- 例: `fav infer --from sap --entity A_SalesOrder` → `SalesOrder` 型を出力

**完了条件**: Rust テスト 2 件（4025 + 2 = 4027）
- `cmd_infer_sap_entity_exists`
- `fav_infer_from_sap_generates_favnir_type`

---

### v89.5.0 — E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ）

**実装内容:**
- `infra/e2e-demo/sap-odata/pipeline.fav` に 4 シナリオ全実装
- Lambda デプロイ確認（`infra/e2e-demo/sap-odata/terraform/`）
- `scripts/run-sap-demo.sh`（モックサーバー起動 → パイプライン実行 → S3 確認の一括スクリプト）

**完了条件**: Rust テスト 2 件（4027 + 2 = 4029）
- `sap_e2e_demo_pipeline_has_journal_entry_scenario`
- `sap_e2e_run_script_exists`

---

### v89.6.0 — `site/content/docs/runes/sap-odata.mdx` ドキュメント

**実装内容:**
- `site/content/docs/runes/sap-odata.mdx` を作成:
  - 概要・セットアップ（`fav.toml [sap]`）
  - 各エンティティ別のサンプルコード（BusinessPartner / SalesOrder / Material / JournalEntry）
  - 4 業務シナリオの解説
  - Docker Compose モックサーバーでの開発手順

**完了条件**: Rust テスト 2 件（4029 + 2 = 4031）
- `docs_sap_odata_mdx_exists`
- `docs_sap_odata_contains_business_partner_section`

---

### v89.7.0 — OSS 整備

**実装内容:**
- `CONTRIBUTING.md` に SAP Rune エンティティ追加手順を追記:
  - 新エンティティの追加手順（型定義 → 関数実装 → テスト → driver.rs テスト → Registry デプロイ）
- `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` を作成

**完了条件**: Rust テスト 2 件（4031 + 2 = 4033）
- `contributing_has_sap_section`
- `issue_template_sap_feedback_exists`

---

### v89.8.0 — パフォーマンス確認

**実装内容:**
- `cargo test --release` で全テスト通過確認
- `fav bench --all` でベースラインとの乖離確認
- ページネーション（1000 件超）の実行時間計測
- Lambda cold start 時間の計測・記録（`benchmarks/sap-odata-v89.8.0.json`）

**完了条件**: Rust テスト 2 件（4033 + 2 = 4035）
- `sap_perf_benchmark_json_exists`
- `sap_perf_benchmark_has_duration_ms`

---

### v89.9.0 — 安定化・コードフリーズ

**完了条件**: Rust テスト 2 件（4035 + 2 = 4037）
- `sap_all_four_scenarios_in_pipeline`
- `sap_integration_rune_registry_deployed`

---

### v90.0.0 — SAP Integration 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「SAP が、Favnir の型になった。
>
>  `business_partners()` で得意先を取得し、
>  `sales_orders()` で受注を集計し、
>  `materials()` で在庫を確認し、
>  `journal_entries()` で支払を照合する。
>
>  世界最大の ERP データが、型安全なパイプラインとして流れる。
>  それが、Favnir SAP Integration 1.0 である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `90.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v90.0.0 に更新
- `roadmap-v85.1-v90.0.md` の全行を「完了」に更新

**完了条件**: `v90000_tests` 4 件（4037 + 4 = 4041）
- `cargo_toml_version_is_90_0_0`
- `changelog_has_v90_0_0`
- `milestone_has_sap_integration`
- `readme_mentions_sap_integration`

---

## 全スプリント総括

| スプリント | マイルストーン | バージョン範囲 | テスト増加 |
|---|---|---|---|
| Sprint 1 | SAP Foundation 1.0 | v85.1〜v86.0 | +22 |
| Sprint 2 | SAP Master Data 1.0 | v86.1〜v87.0 | +22 |
| Sprint 3 | SAP Sales 1.0 | v87.1〜v88.0 | +22 |
| Sprint 4 | SAP Procurement 1.0 | v88.1〜v89.0 | +22 |
| Sprint 5 | SAP Integration 1.0 宣言 | v89.1〜v90.0 | +22 |
| **合計** | | | **+110** |

**到達テスト数: 3,931 → 4,041**

---

## 参考リンク

- 前フェーズ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 次フェーズ: （未計画 — v90.0.0 宣言後に策定）
- SAP API Business Hub: `api.sap.com`
- SAP OData Mock Server: `github.com/SAP-samples/s4hana-cloud-extension-process-automation`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
