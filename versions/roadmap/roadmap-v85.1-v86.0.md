# Roadmap v85.1.0 〜 v86.0.0 — SAP Foundation 1.0

Date: 2026-08-22
Status: 未着手（v85.0.0 完了後に開始）

マスターロードマップ: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)

---

## 前提

- 直前完了: v85.0.0「Favnir 4.0 宣言 ★クリーンアップ」（tests = 3,931）
- 本スプリントは SAP Integration Era の第 1 スプリント
- 目標: v86.0.0「SAP Foundation 1.0 宣言」（tests = 3,953）

### 着手前チェックリスト

- `versions/current.md` の現行マスターロードマップが `roadmap-v85.1-v90.0.md` を指していることを確認する
- `versions/v85-v90/v85.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること

### スプリントの性格

SAP OData v4 への接続基盤と `sap-odata` Rune の骨格を作る。
Rust 側の `fav.toml [sap]` 解析・env 注入から、Favnir 型定義、Docker Compose モックサーバー、
SSM Parameter Store Terraform まで、SAP Integration の土台を完成させる。
A（新機能）80% + B（インフラ）20% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v85.1.0 | `SapTomlConfig` + `inject_sap_config()`（Rust 基盤） | 3931 + 2 = 3933 | 未着手 |
| v85.2.0 | `SapConfig` Favnir 型 + `sap_config_from_env()` | 3933 + 2 = 3935 | 未着手 |
| v85.3.0 | Docker Compose — SAP OData モックサーバー構築 | 3935 + 2 = 3937 | 未着手 |
| v85.4.0 | `runes/sap-odata/` 骨格 + `rune.toml` | 3937 + 2 = 3939 | 未着手 |
| v85.5.0 | OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`） | 3939 + 2 = 3941 | 未着手 |
| v85.6.0 | `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク） | 3941 + 2 = 3943 | 未着手 |
| v85.7.0 | `fav new` テンプレート + `fav.toml [sap]` セクション追加 | 3943 + 2 = 3945 | 未着手 |
| v85.8.0 | SSM Parameter Store 設定（`infra/sap/ssm.tf`） | 3945 + 2 = 3947 | 未着手 |
| v85.9.0 | 安定化・コードフリーズ | 3947 + 2 = 3949 | 未着手 |
| v86.0.0 | SAP Foundation 1.0 宣言 ★クリーンアップ | 3949 + 4 = 3953 | 未着手 |

---

## v85.1.0 — `SapTomlConfig` + `inject_sap_config()`

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

## v85.2.0 — `SapConfig` Favnir 型 + `sap_config_from_env()`

SAP 接続設定を Favnir の型として表現する。

**実装内容:**
- `fav/src/driver.rs` に以下の Rust テストを追加（VM primitive として登録済みの `Env.*` を使う形）
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

## v85.3.0 — Docker Compose — SAP OData モックサーバー構築

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

## v85.4.0 — `runes/sap-odata/` 骨格 + `rune.toml`

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

## v85.5.0 — OData v4 HTTP クライアント基盤

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

## v85.6.0 — `SapError` 型 + エラーハンドリング

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

## v85.7.0 — `fav new` テンプレート + `fav.toml [sap]` セクション

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
- `sap_toml_section_parses_correctly`

---

## v85.8.0 — SSM Parameter Store 設定

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

## v85.9.0 — 安定化・コードフリーズ

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

## v86.0.0 — SAP Foundation 1.0 宣言 ★クリーンアップ

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

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v85.0.0（ベース） | 3,931 | — |
| v85.1.0 | 3,933 | +2 |
| v85.2.0 | 3,935 | +2 |
| v85.3.0 | 3,937 | +2 |
| v85.4.0 | 3,939 | +2 |
| v85.5.0 | 3,941 | +2 |
| v85.6.0 | 3,943 | +2 |
| v85.7.0 | 3,945 | +2 |
| v85.8.0 | 3,947 | +2 |
| v85.9.0 | 3,949 | +2 |
| v86.0.0（宣言） | 3,953 | +4 |

**本スプリント合計**: +22 tests
