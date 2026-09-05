# Roadmap v103.1.0 〜 v104.0.0 — SAP Data Products 1.0

Date: 2026-09-05
Status: 未着手

マスターロードマップ: [roadmap-v100.1-v105.0.md](roadmap-v100.1-v105.0.md)

---

## 前提

- 直前完了: v103.0.0「SAP API Exposure 1.0 宣言」（tests = 4,345）
- 本スプリントは SAP Real-World Platform Era の第 4 スプリント
- 目標: v104.0.0「SAP Data Products 1.0 宣言」（tests = 4,367）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v103.0.0 になっていることを確認する
- `fav/src/driver.rs` に `mod v103000_tests` が存在することを確認する
- `fav/Cargo.toml` の version が `103.0.0` であることを確認する
- `infra/e2e-demo/sap-odata/api.fav` が存在することを確認する（v102.7.0 完了済みの証拠）

### スプリントの性格

SAP Real-World Platform Era の**データ製品管理スプリント**。

Sprint 3 で「SAP データを外に出す」手段を作った。
Sprint 4 では「誰が所有し、どの SLA で提供し、どのスキーマで公開するか」を
コードで宣言・管理する Data Product 基盤を構築する。
`data_product` キーワードで SAP データを製品として定義し、
`fav catalog` でチームが管理できる状態を作る。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v103.1.0 | `data_product` キーワード（オーナー・SLA・スキーマの宣言構文） | 4345+2=4347 | 未着手 |
| v103.2.0 | `fav catalog list` — データ製品一覧表示 | 4347+2=4349 | 未着手 |
| v103.3.0 | データ製品間スキーマ契約チェック（`fav mesh validate`） | 4349+2=4351 | 未着手 |
| v103.4.0 | SLA 違反検出（`SlaViolation` 型 + `fav mesh check-sla`） | 4351+2=4353 | 未着手 |
| v103.5.0 | データ製品バージョン管理（後方互換性チェック） | 4353+2=4355 | 未着手 |
| v103.6.0 | カタログ JSON 出力（OpenMetadata / DataHub 互換フォーマット） | 4355+2=4357 | 未着手 |
| v103.7.0 | E2E デモ（SAP BP / SalesOrder を data_product 化 → catalog 登録 → SLA チェック） | 4357+2=4359 | 未着手 |
| v103.8.0 | サイトドキュメント（Data Products ガイド） | 4359+2=4361 | 未着手 |
| v103.9.0 | 安定化・コードフリーズ | 4361+2=4363 | 未着手 |
| v104.0.0 | SAP Data Products 1.0 宣言 ★クリーンアップ | 4363+4=4367 | 未着手 |

---

## v103.1.0 — `data_product` キーワード定義

Favnir の型と pipeline を「データ製品」として宣言する `data_product` キーワードを追加する。
オーナー・SLA・スキーマ・バージョンをコードで宣言する。

```favnir
-- データ製品宣言
data_product BusinessPartnerProduct {
    owner:       "data-platform-team",
    version:     "1.0.0",
    description: "SAP BusinessPartner マスターデータ",
    sla: {
        availability:   0.999,      -- 99.9%
        max_latency_ms: 500,
        update_cadence: "daily"
    },
    schema: BusinessPartner,        -- Favnir 型をスキーマとして宣言
    source: fn(ctx: AppCtx) -> Result<List<BusinessPartner>, String> {
        ctx.sap.business_partners(BusinessPartnerFilter {
            country: Option.none(), category: Option.none(),
            changed_after: Option.none(), top: Option.some(5000)
        })
    }
}
```

**修正ファイル**: `fav/src/frontend/lexer.rs`（`data_product` トークン追加）、`fav/src/frontend/parser.rs`（data_product 宣言パース）、`fav/src/ast.rs`（`DataProductDecl` ノード追加）、`fav/src/driver.rs`

---

## v103.2.0 — `fav catalog list`

プロジェクト内の `data_product` 宣言を収集して一覧表示する `fav catalog list` コマンドを追加する。

```bash
$ fav catalog list ./products/
Data Products in ./products/:

  BusinessPartnerProduct   v1.0.0   owner: data-platform-team   SLA: 99.9% / 500ms
  SalesOrderProduct        v1.0.0   owner: sales-team            SLA: 99.5% / 1000ms
  MaterialProduct          v0.9.0   owner: procurement-team      SLA: 99.0% / 2000ms

Total: 3 products
```

**修正ファイル**: `fav/src/main.rs`（`catalog` サブコマンド追加）、`fav/src/driver.rs`

---

## v103.3.0 — データ製品間スキーマ契約チェック

複数の `data_product` 間でスキーマの依存関係・互換性を検証する `fav mesh validate` を追加する。

```bash
$ fav mesh validate ./products/
Validating Data Mesh contracts...

  BusinessPartnerProduct -> SalesOrderProduct
    Field: SalesOrder.sold_to_party references BusinessPartner.partner_id
    Status: COMPATIBLE ✓

  SalesOrderProduct -> MaterialProduct
    Field: SalesOrderItem.material references Material.material_id
    Status: COMPATIBLE ✓

Validation result: 2/2 contracts valid
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v103.4.0 — SLA 違反検出（`fav mesh check-sla`）

データ製品の SLA（可用性・レイテンシ）の実績を確認し、違反を検出する
`fav mesh check-sla` コマンドを追加する。
SLA 定義は `SlaDefinition` 型（v99.6.0 で定義済み）を流用する。

```bash
$ fav mesh check-sla --product BusinessPartnerProduct --period 7d
SLA check: BusinessPartnerProduct (last 7 days)

  Availability target:  99.9%   Actual: 99.95%  ✓
  Latency target:       500ms   P95:    312ms   ✓
  Update cadence:       daily   Last:   2h ago  ✓

SLA status: OK
```

```bash
$ fav mesh check-sla --product SalesOrderProduct --period 7d
SLA check: SalesOrderProduct (last 7 days)

  Availability target:  99.5%   Actual: 98.2%   ✗ VIOLATION
  Latency target:       1000ms  P95:    1450ms  ✗ VIOLATION

SLA status: VIOLATED (2 violations)
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v103.5.0 — データ製品バージョン管理

`data_product` の `version` フィールドを使って後方互換性を自動チェックする機能を追加する。
スキーマのフィールド追加（互換）/ フィールド削除・型変更（非互換）を検出する。

```bash
$ fav catalog diff --product BusinessPartnerProduct --from v1.0.0 --to v1.1.0
Schema diff: BusinessPartnerProduct v1.0.0 -> v1.1.0

  + email: String          ADDED (compatible)
  + phone: Option<String>  ADDED (compatible)
  ~ region: String -> Option<String>  CHANGED (compatible — optional化)

Compatibility: BACKWARD_COMPATIBLE ✓
```

```bash
$ fav catalog diff --product SalesOrderProduct --from v1.0.0 --to v2.0.0
  - amount: Float          REMOVED (BREAKING)

Compatibility: BREAKING_CHANGE ✗
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v103.6.0 — カタログ JSON 出力

`data_product` 宣言から OpenMetadata / DataHub 互換の JSON カタログを出力する
`fav catalog export` コマンドを追加する。

```bash
$ fav catalog export ./products/ --format openmetadata --out catalog.json
Exporting catalog...
  Format: OpenMetadata
  Output: catalog.json

  Exported: 3 data products
```

```json
{
  "entities": [
    {
      "name": "BusinessPartnerProduct",
      "owner": "data-platform-team",
      "version": "1.0.0",
      "sla": { "availability": 0.999, "max_latency_ms": 500 },
      "schema": { "type": "object", "properties": { ... } }
    }
  ]
}
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v103.7.0 — E2E デモ

SAP データを `data_product` として定義し、カタログ登録・SLA チェックまでの縦断デモを作成する。

**デモ手順**:
1. `infra/e2e-demo/sap-odata/products/` に `data_product` 宣言ファイルを作成
2. `fav catalog list ./products/` でデータ製品一覧確認
3. `fav mesh validate ./products/` でスキーマ契約チェック
4. `fav mesh check-sla --product BusinessPartnerProduct` で SLA 確認
5. `fav catalog export ./products/ --format openmetadata` でカタログ出力

**新規作成**:
- `infra/e2e-demo/sap-odata/products/business_partner_product.fav`
- `infra/e2e-demo/sap-odata/products/sales_order_product.fav`

**修正ファイル**: `fav/src/driver.rs`

---

## v103.8.0 — サイトドキュメント

**新規作成**:
- `site/content/docs/guides/sap-data-products.mdx` — SAP データを製品として管理するガイド

**修正ファイル**: `fav/src/driver.rs`

---

## v103.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,363 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- `versions/current.md` の次バージョン欄を v104.0.0 に更新

---

## v104.0.0 — SAP Data Products 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「SAP データが、製品になった。
>
>  `data_product` でオーナーと SLA を宣言し、
>  `fav catalog list` でチームが管理し、
>  `fav mesh validate` で契約の整合性を保ち、
>  `fav mesh check-sla` で品質を測る。
>
>  SAP データは今、型安全な製品として流通する——
>  これが、SAP Data Products 1.0 である。」

**v104000_tests（4 テスト）**:
- `cargo_toml_version_is_104_0_0`
- `changelog_has_v104_0_0`
- `sap_data_products_guide_exists`
- `sap_products_dir_exists`

**クリーンアップ**:
- `cargo clean` 実施
- `cargo test` で 4,367 tests, 0 failures を再確認
- `cargo build` で `./target/debug/fav` を再生成

---

## スプリント終了時の確認

- [ ] 4,367 tests, 0 failures
- [ ] `data_product` キーワードがパーサーで解析できる
- [ ] `fav catalog list` が動作する
- [ ] `fav mesh validate` が動作する
- [ ] `fav mesh check-sla` が動作する
- [ ] `fav catalog export --format openmetadata` が JSON を出力する
- [ ] `infra/e2e-demo/sap-odata/products/` に製品ファイルが存在する
- [ ] `cargo clean` を実施する
- [ ] `cargo test` で 4,367 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v104.0.0 に更新
- [ ] `MILESTONE.md` に v104.0.0 エントリを追加
- [ ] `roadmap-v100.1-v105.0.md` の Sprint 4 状態を「完了」に更新
