# Roadmap v89.1.0 〜 v90.0.0 — SAP Integration 1.0 宣言

Date: 2026-08-22
Status: 未着手（v89.0.0 完了後に開始）

マスターロードマップ: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)

---

## 前提

- 直前完了: v89.0.0「SAP Procurement 1.0 宣言」（tests = 4,019）
- 本スプリントは SAP Integration Era の第 5 スプリント（最終スプリント）
- 目標: v90.0.0「SAP Integration 1.0 宣言」（tests = 4,041）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v89.0.0 になっていることを確認する
- `versions/v85-v90/v89.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/` に BusinessPartner / SalesOrder / Material / PurchaseOrder の型・関数が実装済みであることを確認する
- `infra/e2e-demo/sap-odata/terraform/` が存在することを確認する

### スプリントの性格

会計伝票（`JournalEntry`）を追加し、全 4 業務シナリオを完成させる。
`fav infer --from sap` コマンド、サイトドキュメント、OSS 整備、パフォーマンス確認を経て
SAP Integration 1.0 を宣言する。
B（統合・完成）50% + C（宣言・ドキュメント）50% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v89.1.0 | `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()` | 4019 + 2 = 4021 | 未着手 |
| v89.2.0 | `OutstandingPayable` 型 + `match_unposted_orders()` | 4021 + 2 = 4023 | 未着手 |
| v89.3.0 | シナリオ 4: 購買→支払サイクル照合（PO × JE） | 4023 + 2 = 4025 | 未着手 |
| v89.4.0 | `fav infer --from sap --entity <name>` コマンド | 4025 + 2 = 4027 | 未着手 |
| v89.5.0 | E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ） | 4027 + 2 = 4029 | 未着手 |
| v89.6.0 | `site/content/docs/runes/sap-odata.mdx` ドキュメント | 4029 + 2 = 4031 | 未着手 |
| v89.7.0 | OSS 整備（CONTRIBUTING SAP セクション + ISSUE_TEMPLATE） | 4031 + 2 = 4033 | 未着手 |
| v89.8.0 | パフォーマンス確認（ページネーション / バッチ / Lambda cold start） | 4033 + 2 = 4035 | 未着手 |
| v89.9.0 | 安定化・コードフリーズ | 4035 + 2 = 4037 | 未着手 |
| v90.0.0 | SAP Integration 1.0 宣言 ★クリーンアップ | 4037 + 4 = 4041 | 未着手 |

---

## v89.1.0 — `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()`

会計伝票の Favnir 型と一覧取得関数を実装する。

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
    document_number: String,
    fiscal_year:     Int,
    posting_date:    String,
    document_type:   String,
    company_code:    String,
    reference:       Option<String>,
    items:           Option<List<JournalEntryItem>>
}

type JournalFilter = {
    fiscal_year:       Option<Int>,
    posting_date_from: Option<String>,
    company_code:      Option<String>,
    reference:         Option<String>,
    top:               Option<Int>
}

public fn journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String>
```

**実装ファイル:** `runes/sap-odata/journal_entry.fav`（新規作成）

**完了条件**: Rust テスト 2 件（4019 + 2 = 4021）
- `journal_entry_type_defined_in_rune`
- `journal_entries_function_exists`

---

## v89.2.0 — `OutstandingPayable` 型 + `match_unposted_orders()`

未照合発注を検出するための型と関数を実装する。

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

**実装ファイル:**
- `runes/sap-odata/journal_entry.fav`（`journal_entries` の後に追加）
- `runes/sap-odata/sap_odata.fav`（`OutstandingPayable` re-export + `match_unposted_orders` ラッパー追加）

**完了条件**: Rust テスト 2 件（4021 + 2 = 4023）
- `outstanding_payable_type_exists`
- `match_unposted_orders_function_exists`

---

## v89.3.0 — シナリオ 4: 購買→支払サイクル照合

業務シナリオ 4 の E2E 実装。一部納品済み発注と会計伝票を突き合わせ、未払いを検出する。

```favnir
fn outstanding_payables(ctx: AppCtx) -> Result<List<OutstandingPayable>, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind pos      <- sap_odata.purchase_orders(cfg, PurchaseOrderFilter {
        status:        Option.some(PurchaseOrderStatus.PartiallyDelivered),
        vendor_id:     Option.none(),
        created_after: Option.none(),
        plant:         Option.none(),
        top:           Option.none()
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

## v89.4.0 — `fav infer --from sap --entity <name>` コマンド

SAP OData メタデータ（`$metadata`）から Favnir の型定義を自動生成するコマンド。

**実装内容:**
- `cmd_infer` に `--from sap` オプションを追加
- `--entity <EntitySetName>` でエンティティを指定
- エンティティ名からテンプレート型定義を生成（`A_` プレフィックス除去）
  （実際の OData $metadata XML パースはスコープ外 — テンプレート生成のみ）
- 例: `fav infer --from sap --entity A_SalesOrder` → `SalesOrder` 型テンプレートを出力

**完了条件**: Rust テスト 2 件（4025 + 2 = 4027）
- `cmd_infer_sap_entity_exists`
- `fav_infer_from_sap_generates_favnir_type`

---

## v89.5.0 — E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ）

全 4 業務シナリオを `infra/e2e-demo/sap-odata/pipeline.fav` に統合する。

**実装内容:**
- `infra/e2e-demo/sap-odata/pipeline.fav` に 4 シナリオ全実装
  （sync_business_partners / daily_sales_report / check_stock_vs_orders / outstanding_payables）
- Lambda デプロイ確認（`infra/e2e-demo/sap-odata/terraform/`）
- `scripts/run-sap-demo.sh`（モックサーバー起動 → パイプライン実行 → S3 確認の一括スクリプト）

**完了条件**: Rust テスト 2 件（4027 + 2 = 4029）
- `sap_e2e_demo_pipeline_has_journal_entry_scenario`
- `sap_e2e_run_script_exists`

---

## v89.6.0 — `site/content/docs/runes/sap-odata.mdx` ドキュメント

sap-odata Rune の公式ドキュメントをサイトに追加する。

**実装内容:**
- `site/content/docs/runes/sap-odata.mdx` を作成:
  - 概要・セットアップ（`fav.toml [sap]`）
  - 各エンティティ別のサンプルコード（BusinessPartner / SalesOrder / Material / JournalEntry）
  - 4 業務シナリオの解説
  - Docker Compose モックサーバーでの開発手順
  - Rune Registry からのインポート方法（`import rune "sap-odata"`）

**完了条件**: Rust テスト 2 件（4029 + 2 = 4031）
- `docs_sap_odata_mdx_exists`
- `docs_sap_odata_contains_business_partner_section`

---

## v89.7.0 — OSS 整備

SAP Rune のコントリビューション手順と Issue テンプレートを整備する。

**実装内容:**
- `CONTRIBUTING.md` に SAP Rune エンティティ追加手順を追記:
  - 新エンティティの追加手順（型定義 → 関数実装 → テスト → driver.rs テスト → Registry デプロイ）
- `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` を作成

**完了条件**: Rust テスト 2 件（4,031 + 2 = 4,033）
- `contributing_has_sap_section`
- `issue_template_sap_feedback_exists`

---

## v89.8.0 — パフォーマンス確認

SAP パイプラインのパフォーマンスを計測・記録する。

**実装内容:**
- `cargo test --release` で全テスト通過確認
- `fav bench --all` でベースラインとの乖離確認
- ページネーション（1000 件超）の実行時間計測
- Lambda cold start 時間の計測・記録（`benchmarks/sap-odata-v89.8.0.json`）

**完了条件**: Rust テスト 2 件（4033 + 2 = 4035）
- `sap_perf_benchmark_json_exists`
- `sap_perf_benchmark_has_duration_ms`

---

## v89.9.0 — 安定化・コードフリーズ

v89.1〜v89.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認（4035 tests）
- 全 4 シナリオの E2E 動作確認
- Rune Registry デプロイ済みの `import rune "sap-odata"` 最終確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4035 + 2 = 4037）
- `sap_all_four_scenarios_in_pipeline`
- `sap_integration_rune_registry_deployed`

---

## v90.0.0 — SAP Integration 1.0 宣言 ★クリーンアップ

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
- driver.rs 内の旧 `cargo_toml_version` テスト（33 件）を `90.0.0` に一括更新

**完了条件**: `v90000_tests` 4 件（4037 + 4 = 4041）
- `cargo_toml_version_is_90_0_0`
- `changelog_has_v90_0_0`
- `milestone_has_sap_integration`
- `readme_mentions_sap_integration`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v89.0.0（ベース） | 4,019 | — |
| v89.1.0 | 4,021 | +2 |
| v89.2.0 | 4,023 | +2 |
| v89.3.0 | 4,025 | +2 |
| v89.4.0 | 4,027 | +2 |
| v89.5.0 | 4,029 | +2 |
| v89.6.0 | 4,031 | +2 |
| v89.7.0 | 4,033 | +2 |
| v89.8.0 | 4,035 | +2 |
| v89.9.0 | 4,037 | +2 |
| v90.0.0（宣言） | 4,041 | +4 |

**本スプリント合計**: +22 tests（SAP Integration Era 全体: +110 tests）
