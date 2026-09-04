# Roadmap v90.1.0 〜 v91.0.0 — SAP Ctx 統合 1.0

Date: 2026-08-25
Status: 完了（v91.0.0 宣言済み・2026-08-30）

マスターロードマップ: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)

---

## 前提

- 直前完了: v90.0.0「SAP Integration 1.0 宣言」（tests = 4,041）
- 本スプリントは SAP Advanced Era の第 1 スプリント
- 目標: v91.0.0「SAP Ctx 統合 1.0 宣言」（tests = 4,063）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v90.0.0 になっていることを確認する
- `versions/v90-v95/v90.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/` の全 Rune ファイルが存在することを確認する
- `infra/e2e-demo/sap-odata/pipeline.fav` が存在することを確認する（書き換え対象）
- `fav/src/driver.rs` に `mod v90000_tests` が存在することを確認する（v90.0.0 完了済みの証拠）

### スプリントの性格

SAP Integration Era で積み残した**アーキテクチャ上の負債**（Ctx 非統合）を解消する。
新機能はなく、既存の SAP アクセスを `ctx.sap.*` パターンへ**リファクタリング**するスプリント。

テスト・ドキュメント・MockSapClient の整備により、次スプリント（OData クエリ深化）の土台を作る。

A（基盤・リファクタ）70% + C（ドキュメント）30% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v90.1.0 | `SapClient` interface 定義（5 関数） | 4041 + 2 = 4043 | 未着手 |
| v90.2.0 | `AppCtx` に `sap: SapClient` フィールドを追加 | 4043 + 2 = 4045 | 未着手 |
| v90.3.0 | `MockSapClient` 実装（テスト用スタブ） | 4045 + 2 = 4047 | 未着手 |
| v90.4.0 | `Ctx.build` に SAP 設定注入を統合（`fav.toml [sap]`） | 4047 + 2 = 4049 | 未着手 |
| v90.5.0 | `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応 | 4049 + 2 = 4051 | 未着手 |
| v90.6.0 | `infra/e2e-demo/sap-odata/pipeline.fav` を `ctx.sap.*` で書き換え | 4051 + 2 = 4053 | 未着手 |
| v90.7.0 | `Ctx.mock` に `sap: MockSapClient` を追加 | 4053 + 2 = 4055 | 未着手 |
| v90.8.0 | サイトドキュメント更新（ctx.sap パターンガイド） | 4055 + 2 = 4057 | 未着手 |
| v90.9.0 | 安定化・コードフリーズ | 4057 + 2 = 4059 | 未着手 |
| v91.0.0 | SAP Ctx 統合 1.0 宣言 ★クリーンアップ | 4059 + 4 = 4063 | 未着手 |

---

## v90.1.0 — `SapClient` interface 定義

`ctx.sap` フィールドの型となる `SapClient` interface を定義する。
v13.x.x で実装した `DbRead` / `HttpClient` 等と同じパターンで実装する。

```favnir
interface SapClient {
    business_partners: (BusinessPartnerFilter) -> Result<List<BusinessPartner>, String>,
    business_partner_by_id: (String) -> Result<BusinessPartner, String>,
    sales_orders: (SalesOrderFilter) -> Result<List<SalesOrder>, String>,
    materials: (MaterialFilter) -> Result<List<Material>, String>,
    journal_entries: (JournalFilter) -> Result<List<JournalEntry>, String>
}
```

**実装内容:**
- `runes/sap-odata/types.fav` に `SapClient` interface を追加
- `driver.rs` に `mod v90100_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4041 + 2 = 4043）
- `sap_client_interface_defined`: `runes/sap-odata/types.fav` に `SapClient` が含まれる
- `sap_client_has_business_partners_method`: `SapClient` に `business_partners` が含まれる

---

## v90.2.0 — `AppCtx` に `sap: SapClient` フィールドを追加

既存の `AppCtx` type 定義（`runes/ctx/ctx.fav` または相当ファイル）に `sap` フィールドを追加する。

```favnir
-- 変更後の AppCtx
type AppCtx = {
    s3:  StorageClient,
    db:  DbClient,
    io:  IoClient,
    sap: SapClient    -- ← 追加
}
```

**実装内容:**
- `AppCtx` 型定義に `sap: SapClient` を追加
- `driver.rs` に `mod v90200_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4043 + 2 = 4045）
- `app_ctx_has_sap_field`: AppCtx 型定義ファイルに `sap` フィールドが含まれる
- `sap_field_type_is_sap_client`: `sap: SapClient` という記述が存在する

---

## v90.3.0 — `MockSapClient` 実装

テスト用スタブとして `MockSapClient` を実装する。
`MockDb` / `MockStorage`（v13.5.0）と同じパターンで実装する。

```favnir
type MockSapClient = {
    business_partners_result: Result<List<BusinessPartner>, String>,
    sales_orders_result:      Result<List<SalesOrder>, String>,
    materials_result:         Result<List<Material>, String>,
    journal_entries_result:   Result<List<JournalEntry>, String>
}

impl SapClient for MockSapClient {
    fn business_partners(self: MockSapClient, filter: BusinessPartnerFilter)
        -> Result<List<BusinessPartner>, String> { self.business_partners_result }
    -- ... 以下同様
}
```

**実装内容:**
- `runes/sap-odata/mock.fav`（新規作成）に `MockSapClient` を定義
- `SapClient` interface を実装
- `driver.rs` に `mod v90300_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4045 + 2 = 4047）
- `mock_sap_client_file_exists`: `runes/sap-odata/mock.fav` が存在する
- `mock_sap_client_implements_sap_client`: `mock.fav` に `impl SapClient for MockSapClient` が含まれる

---

## v90.4.0 — `Ctx.build` に SAP 設定注入を統合

`fav.toml [sap]` の設定を `AppCtx` 構築時に自動注入する。
`inject_sap_config()` の呼び出しが `Ctx.build` 内部に移動し、ユーザーコードから cfg を意識しなくて済む。

**実装内容:**
- `Ctx.build` 関数に `sap_config_from_env()` 呼び出しを統合
- `SapODataClient`（実 HTTP クライアント）が `SapClient` interface を実装するよう追加
- `driver.rs` に `mod v90400_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4048 + 2 = 4050）
- `ctx_build_integrates_sap`: `Ctx.build` 関数に `sap` フィールドが含まれる
- `sap_odata_client_impl_exists`: `SapODataClient` の `impl SapClient` が存在する

---

## v90.5.0 — `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応

Rune の公開関数シグネチャを `cfg: SapConfig` 受け取りから `ctx: AppCtx` 受け取りに変更する。

```favnir
-- 変更前
public fn business_partners(cfg: SapConfig, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String>

-- 変更後
public fn business_partners(ctx: AppCtx, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String> {
    ctx.sap.business_partners(filter)
}
```

**実装内容:**
- `runes/sap-odata/sap_odata.fav` の全公開関数シグネチャを更新
- 後方互換性のため旧シグネチャを deprecated コメントで残す（削除は v91.0.0）
- `driver.rs` に `mod v90500_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4050 + 2 = 4052）
- `sap_odata_fav_uses_app_ctx`: `sap_odata.fav` に `ctx: AppCtx` が含まれる
- `sap_odata_fav_delegates_to_ctx_sap`: `ctx.sap.business_partners` への委譲が含まれる

---

## v90.6.0 — `pipeline.fav` を `ctx.sap.*` で書き換え

`infra/e2e-demo/sap-odata/pipeline.fav` の全 4 シナリオを `ctx.sap.*` スタイルに書き換える。

```favnir
-- 変更前
fn sync_business_partners(ctx: AppCtx) -> Result<..., String> {
    bind cfg <- sap_odata.sap_config_from_env()
    bind bps <- sap_odata.business_partners(cfg, filter)
    bind _   <- ctx.s3.put_object(...)

-- 変更後
fn sync_business_partners(ctx: AppCtx) -> Result<..., String> {
    bind bps <- ctx.sap.business_partners(filter)
    bind _   <- ctx.s3.put_object(...)
```

**実装内容:**
- `pipeline.fav` の全 4 シナリオを書き換え
- `sap_config_from_env()` の明示呼び出しを削除
- `driver.rs` に `mod v90600_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4052 + 2 = 4054）
- `pipeline_fav_uses_ctx_sap`: `pipeline.fav` に `ctx.sap.` が含まれる
- `pipeline_fav_no_explicit_cfg`: `pipeline.fav` に `sap_config_from_env` が含まれない

---

## v90.7.0 — `Ctx.mock` に `sap: MockSapClient` を追加

`Ctx.mock` ユーティリティに SAP モックを追加し、パイプラインのユニットテストで SAP をスタブ化できるようにする。

```favnir
bind ctx <- Ctx.mock(MockSapClient {
    business_partners_result: Result.ok([sample_bp]),
    sales_orders_result:      Result.err("not implemented"),
    materials_result:         Result.err("not implemented"),
    journal_entries_result:   Result.err("not implemented")
})
```

**完了条件**: Rust テスト 2 件（4054 + 2 = 4056）
- `ctx_mock_has_sap_field`: `Ctx.mock` 定義に `sap` フィールドが含まれる
- `mock_sap_client_default_exists`: `MockSapClient` のデフォルト値が定義されている

---

## v90.8.0 — サイトドキュメント更新

`site/content/docs/runes/sap-odata.mdx` を `ctx.sap.*` パターンに対応するよう更新する。

**追加セクション:**
- `ctx.sap` パターンの使い方（v90.1〜v90.7 で実装した内容）
- `MockSapClient` を使ったユニットテストの書き方
- `Ctx.build` への自動設定注入の説明

**完了条件**: Rust テスト 3 件（4056 + 3 = 4059）
- `docs_sap_odata_mentions_ctx_sap`: `sap-odata.mdx` に `ctx.sap` が含まれる
- `docs_sap_odata_mentions_mock_sap_client`: `sap-odata.mdx` に `MockSapClient` が含まれる
- `docs_sap_odata_no_sap_config_from_env`: `sap-odata.mdx` に `sap_config_from_env` が含まれない（code-reviewer 対応追加）

---

## v90.9.0 — 安定化・コードフリーズ

v90.1〜v90.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認（4,059 tests）
- pipeline.fav の `ctx.sap.*` 書き換え動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4059 + 2 = 4061）
- `sap_ctx_integration_smoke_all_scenarios`: `pipeline.fav` に 4 シナリオ関数（`sync_business_partners` 等）と `ctx.sap.` の両方が含まれる
- `sap_ctx_mock_client_in_rune_dir`: `runes/sap-odata/` ディレクトリ内に `mock.fav` が存在する（v90.3.0 の再確認を新規テスト名で担保）

---

## v91.0.0 — SAP Ctx 統合 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`ctx.sap.business_partners(filter)` と書けば、SAP にアクセスできる。
>  設定は `AppCtx` に隠れ、テストは `MockSapClient` で差し替わる。
>  それが、Favnir SAP Ctx 統合 1.0 である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `91.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v91.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テストを `91.0.0` に一括更新
- 旧シグネチャ（`cfg: SapConfig` 受け取り関数）を削除
  - 対象: `runes/sap-odata/sap_odata.fav`（v90.5.0 で deprecated コメントを付けた全関数）
  - 対象: `runes/sap-odata/business_partner.fav` / `sales_order.fav` / `material.fav` / `journal_entry.fav`（cfg 受け取り variants）

**完了条件**: `v91000_tests` 4 件（4061 + 4 = 4065）
- `cargo_toml_version_is_91_0_0`
- `changelog_has_v91_0_0`
- `milestone_has_sap_ctx_integration`
- `readme_mentions_ctx_sap`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v90.0.0（ベース） | 4,041 | — |
| v90.1.0 | 4,043 | +2 |
| v90.2.0 | 4,045 | +2 |
| v90.3.0 | 4,048 | +3 |
| v90.4.0 | 4,050 | +2 |
| v90.5.0 | 4,052 | +2 |
| v90.6.0 | 4,054 | +2 |
| v90.7.0 | 4,056 | +2 |
| v90.8.0 | 4,059 | +3 |
| v90.9.0 | 4,061 | +2 |
| v91.0.0（宣言） | 4,065 | +4 |

**本スプリント合計**: +22 tests（SAP Advanced Era 全体: +110 tests）
