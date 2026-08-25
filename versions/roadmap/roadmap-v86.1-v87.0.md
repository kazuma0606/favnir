# Roadmap v86.1.0 〜 v87.0.0 — SAP Master Data 1.0

Date: 2026-08-22
Status: 未着手（v86.0.0 完了後に開始）

マスターロードマップ: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)

---

## 前提

- 直前完了: v86.0.0「SAP Foundation 1.0 宣言」（tests = 3,953）
- 本スプリントは SAP Integration Era の第 2 スプリント
- 目標: v87.0.0「SAP Master Data 1.0 宣言」（tests = 3,975）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v86.0.0 になっていることを確認する
- `versions/v85-v90/v86.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/rune.toml` が存在し `name = "sap-odata"` であることを確認する

### スプリントの性格

`BusinessPartner`（得意先・仕入先の統合マスタ）を型安全に扱えるようにする。
CRUD 全操作（一覧・単件取得・作成・更新）と、業務シナリオ 1（マスタデータ → S3 同期）を実装する。
Sprint 末に `sap-odata` Rune を Rune Registry（AWS Lambda）にデプロイする。
A（新機能）60% + B（統合・デプロイ）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v86.1.0 | `BusinessPartner` / `BusinessPartnerAddress` 型定義 | 3953 + 2 = 3955 | 未着手 |
| v86.2.0 | `BusinessPartnerFilter` + `business_partners()` クエリ | 3955 + 2 = 3957 | 未着手 |
| v86.3.0 | `business_partner_by_id()` + `$expand=to_BusinessPartnerAddress` | 3957 + 2 = 3959 | 未着手 |
| v86.4.0 | `create_business_partner()` POST 実装 | 3959 + 2 = 3961 | 未着手 |
| v86.5.0 | `update_business_partner()` PATCH 実装 | 3961 + 2 = 3963 | 未着手 |
| v86.6.0 | シナリオ 1: マスタデータ同期（BusinessPartner → S3 JSON） | 3963 + 2 = 3965 | 未着手 |
| v86.7.0 | モックサーバーテスト — CRUD 全操作をモックで検証 | 3965 + 2 = 3967 | 未着手 |
| v86.8.0 | Rune Registry デプロイ（`sap-odata` を Lambda Registry に登録） | 3967 + 2 = 3969 | 未着手 |
| v86.9.0 | 安定化・コードフリーズ | 3969 + 2 = 3971 | 未着手 |
| v87.0.0 | SAP Master Data 1.0 宣言 ★クリーンアップ | 3971 + 4 = 3975 | 未着手 |

---

## v86.1.0 — `BusinessPartner` / `BusinessPartnerAddress` 型定義

SAP マスタデータの基本型を定義する。

```favnir
type BusinessPartnerCategory = Person | Organization | Group

type BusinessPartner = {
    partner_id:  String,
    name:        String,
    category:    BusinessPartnerCategory,
    country:     String,
    language:    String,
    currency:    String,
    created_at:  String,
    addresses:   Option<List<BusinessPartnerAddress>>
}

type BusinessPartnerAddress = {
    address_id:  String,
    street:      String,
    city:        String,
    postal_code: String,
    country:     String,
    region:      Option<String>
}
```

**実装ファイル:** `runes/sap-odata/types.fav`（v85.2.0 の `SapConfig` の後に追加）

**完了条件**: Rust テスト 2 件（3953 + 2 = 3955）
- `business_partner_type_defined_in_rune`
- `business_partner_address_type_defined_in_rune`

---

## v86.2.0 — `BusinessPartnerFilter` + `business_partners()` クエリ

BusinessPartner の一覧取得関数を実装する。

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

**実装ファイル:** `runes/sap-odata/business_partner.fav`（v86.1.0 で作成済み、追記）

**完了条件**: Rust テスト 2 件（3955 + 2 = 3957）
- `business_partners_function_exists`
- `business_partner_filter_type_exists`

---

## v86.3.0 — `business_partner_by_id()` + `$expand`

単一 BusinessPartner の取得関数と住所 expand を実装する。

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

## v86.4.0 — `create_business_partner()` POST

BusinessPartner の新規作成を実装する。

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

## v86.5.0 — `update_business_partner()` PATCH

BusinessPartner の部分更新を実装する。

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

## v86.6.0 — シナリオ 1: マスタデータ同期（BusinessPartner → S3）

業務シナリオ 1 の E2E 実装。SAP の得意先マスタを S3 に JSON 同期する。

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
        category:      Option.none()
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

## v86.7.0 — モックサーバーテスト — CRUD テスト追加（スタブ）

`runes/sap-odata/sap_odata.test.fav` に CRUD 全操作のテスト関数を追加する（スタブ実装）。
実際の Docker Compose モックサーバーへの HTTP リクエスト検証は v87.0.0 以降に実施する。

**実装内容:**
- `sap_odata.test.fav` に BusinessPartner CRUD テスト関数を追加（Create / Read / Update / List）
- モックサーバー起動確認スクリプト（`scripts/test-with-mock.sh`）のスタブ作成

**完了条件**: Rust テスト 2 件（3965 + 2 = 3967）
- `sap_odata_test_fav_exists`
- `sap_odata_test_contains_business_partner_tests`

---

## v86.8.0 — Rune Registry デプロイ

`sap-odata` Rune を既存の Lambda Rune Registry に登録する。

**実装内容:**
- `rune.toml` のバージョンを `86.8.0` に更新
- `deploy-registry` スキルを実行して `sap-odata` Rune をデプロイ
- DynamoDB (`favnir-rune-registry`) にメタデータ登録・確認
- S3 (`favnir-rune-packages`) に `.fav` ファイルをアップロード・確認
- `rune.toml` の整合性（version/entry）を Rust テストで確認
- 注: `import rune "sap-odata"` の実行動作確認は v86.9.0 安定化スプリントで実施する

**完了条件**: Rust テスト 2 件（3967 + 2 = 3969）
- `sap_odata_rune_version_matches_cargo`
- `sap_odata_rune_entry_file_is_sap_odata_fav`

---

## v86.9.0 — 安定化・コードフリーズ

v86.1〜v86.8 の全機能を通しで確認する安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認
- BusinessPartner CRUD 全操作の動作確認
- Rune Registry デプロイ後の `import rune "sap-odata"` 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3969 + 2 = 3971）
- `sap_master_data_business_partner_crud_covered`
- `sap_master_data_scenario1_pipeline_exists`

---

## v87.0.0 — SAP Master Data 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「SAP の BusinessPartner が、Favnir の型になった。
>  得意先も仕入先も、`business_partners()` で型安全に取得できる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `87.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v87.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テスト（33 件）を `87.0.0` に一括更新

**完了条件**: `v87000_tests` 4 件（3971 + 4 = 3975）
- `cargo_toml_version_is_87_0_0`
- `changelog_has_v87_0_0`
- `milestone_has_sap_master_data`
- `sap_odata_rune_toml_has_name_sap_odata`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v86.0.0（ベース） | 3,953 | — |
| v86.1.0 | 3,955 | +2 |
| v86.2.0 | 3,957 | +2 |
| v86.3.0 | 3,959 | +2 |
| v86.4.0 | 3,961 | +2 |
| v86.5.0 | 3,963 | +2 |
| v86.6.0 | 3,965 | +2 |
| v86.7.0 | 3,967 | +2 |
| v86.8.0 | 3,969 | +2 |
| v86.9.0 | 3,971 | +2 |
| v87.0.0（宣言） | 3,975 | +4 |

**本スプリント合計**: +22 tests
