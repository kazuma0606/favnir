# Roadmap v95.1.0 〜 v96.0.0 — SAP Real-time 1.0

Date: 2026-08-30
Status: 未着手

マスターロードマップ: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)

---

## 前提

- 直前完了: v95.0.0「SAP Advanced 1.0 宣言」（tests = 4,164）
- 本スプリントは SAP Platform Era の第 1 スプリント
- 目標: v96.0.0「SAP Real-time 1.0 宣言」（tests = 4,186）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v95.0.0 になっていることを確認する
- `runes/sap-odata/batch.fav` が存在することを確認する（v94.1.0 完了済みの証拠） ← [HIGH-5]
- `runes/sap-odata/sap_odata.fav` に `BatchOperation` / `BatchRequest` の re-export があることを確認する（v94.8.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v95000_tests` が存在することを確認する（v95.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `95.0.0` であることを確認する

### スプリントの性格

SAP Platform Era の**リアルタイム基盤スプリント**。

従来のポーリング型から脱却し、OData `$delta`（差分リンク）と SAP Event Mesh（AMQP）によって
「変化を受け取る」Favnir pipeline を実現する。
合わせて Deep Insert / Function Import / `fav sap-mock` でエコシステムを固める。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v95.1.0 | OData `$delta` / `DeltaLink` 型定義（差分取得基盤） | 4164 + 2 = 4166 | 未着手 |
| v95.2.0 | `ctx.sap.delta_fetch<T>()` — 前回以降の差分エンティティ取得 | 4166 + 2 = 4168 | 未着手 |
| v95.3.0 | SAP Event Mesh 接続基盤（`SapEventClient` interface + `effect_catalog.rs`） | 4168 + 2 = 4170 | 未着手 |
| v95.4.0 | イベント駆動 pipeline（`on_event` stage トリガー） | 4170 + 2 = 4172 | 未着手 |
| v95.5.0 | Deep Insert（SalesOrder + Items を 1 リクエストで作成） | 4172 + 2 = 4174 | 未着手 |
| v95.6.0 | Function Import / Action Import（OData RPC スタイル） | 4174 + 2 = 4176 | 未着手 |
| v95.7.0 | バッチ部分失敗ハンドリング（`PartialSuccess<T>` / `BatchItemResult<T>`） | 4177 + 3 = 4180 | 未着手 |
| v95.8.0 | `fav sap-mock`（オフラインテスト用 SAP モックサーバー） | 4180 + 2 = 4182 | 未着手 |
| v95.9.0 | 安定化・コードフリーズ | 4182 + 2 = 4184 | 未着手 |
| v96.0.0 | SAP Real-time 1.0 宣言 ★クリーンアップ | 4184 + 4 = 4188 | 未着手 |

---

## v95.1.0 — OData `$delta` / `DeltaLink` 型定義

OData `$delta` プロトコルに対応する型を定義する。
差分リンク（`@odata.deltaLink`）を保存し、次回呼び出し時に前回以降の変更のみ取得できるようにする。

```favnir
-- 差分取得の結果
type DeltaResult<T> = {
    entities: List<T>,
    delta_link: String,     -- 次回呼び出し用リンク
    has_more: Bool
}

-- 削除されたエンティティを表すトゥームストーン
type DeletedEntity = {
    id: String,
    reason: String          -- "deleted" | "changed"
}
```

**修正ファイル**: `runes/sap-odata/delta.fav`（新規）、`fav/src/driver.rs`（テスト追加）

**補足**: `delta.fav` には v95.2.0 の実装に向けたヘルパー `public fn delta_link_is_valid(link: String) -> Bool` も先行定義する（v95.1.0 のテスト対象外）。

---

## v95.2.0 — `ctx.sap.delta_fetch<T>()`

`DeltaLink` を使って差分エンティティを取得する `SapClient` メソッドを追加する。

```favnir
-- 初回: delta_link なし → 全件 + deltaLink を受け取る
bind result <- ctx.sap.delta_fetch<BusinessPartner>("A_BusinessPartner", Option.none())
bind _      <- ctx.s3.put_object("my-bucket", "delta_link.txt", result.delta_link)

-- 2 回目以降: 保存した delta_link を使って差分のみ取得
bind saved  <- ctx.s3.get_object("my-bucket", "delta_link.txt")
bind result <- ctx.sap.delta_fetch<BusinessPartner>("A_BusinessPartner", Option.some(saved))
```

**修正ファイル**: `runes/sap-odata/types.fav`、`runes/sap-odata/client.fav`、`runes/sap-odata/sap_odata.fav`、`fav/src/driver.rs`
（`runes/ctx/ctx.fav` は変更不要 — `ctx.sap: SapClient` 型のため interface 追加が自動的に反映される）

---

## v95.3.0 — SAP Event Mesh 接続基盤

SAP Event Mesh（AMQP 1.0）への接続基盤を追加する。
ctx パターンに従い `SapEventClient` interface として実装し、`AppCtx` に `sap_event: SapEventClient` フィールドを追加する。
pipeline シグネチャでは `!SapEvent` マーカーを使い、Rust の `Effect` enum に `SapEvent` を追加する。

```favnir
-- ctx interface として定義（effect 宣言ではない）
type SapEventMessage = {
    topic:     String,
    payload:   String,
    timestamp: String
}

-- SapEventClient は SapClient と同様に AppCtx のフィールドとして注入される
-- ctx.sap_event.subscribe(topic) / ctx.sap_event.receive() でアクセス
```

**修正ファイル**: `runes/sap-odata/event_mesh.fav`（新規）、`runes/ctx/ctx.fav`（`sap_event` フィールド追加）、`fav/src/effect_catalog.rs`（新規）、`fav/src/driver.rs`
**Rust 側**: `Effect` enum は v35.4.0 で削除済みのため、`fav/src/effect_catalog.rs` を新規作成し `SAP_EVENT` 定数カタログを追加する。`checker.fav` の effect match は現時点で不要（再評価 v95.4.0）。

---

## v95.4.0 — イベント駆動 pipeline

`!SapEvent` マーカーを持つイベント駆動 pipeline を実装する。
`ctx.sap_event.*` で SAP Event Mesh に接続する。
pipeline シグネチャの `!SapEvent` は ctx interface マーカーであり、`effect` 宣言は行わない。

```favnir
-- !SapEvent は pipeline が SapEventClient を必要とすることを示すマーカー
-- !SapOData は ctx.sap.* アクセスを伴うことを示す（既存の命名規則に合わせる）
pipeline sync_on_event !SapEvent !S3 {
    stage Subscribe {
        bind _ <- ctx.sap_event.subscribe("sap/s4/BusinessPartner/Changed")
    }
    |> stage Process {
        bind msg  <- ctx.sap_event.receive()
        bind bp   <- Json.decode<BusinessPartner>(msg.payload)
        bind json <- Json.encode(bp)
        bind _    <- ctx.s3.put_object("favnir-sap-sync", bp.partner_id, json)
    }
}
```

**修正ファイル**: `infra/e2e-demo/sap-odata/pipeline_realtime.fav`（新規）、`fav/src/driver.rs`

---

## v95.5.0 — Deep Insert

OData の Deep Insert（ネスト構造を 1 リクエストで作成）に対応する。

```favnir
type NewSalesOrderWithItems = {
    customer_id: String,
    currency:    String,
    items:       List<NewSalesOrderItem>   -- ネストされた Items
}

-- 1 リクエストで SalesOrder + Items を作成
bind order <- ctx.sap.create_sales_order_deep(NewSalesOrderWithItems {
    customer_id: "C001",
    currency:    "JPY",
    items: [
        NewSalesOrderItem { material_id: "MAT001", quantity: 10.0, unit: "EA" }
    ]
})
```

**修正ファイル**: `runes/sap-odata/sales_order.fav`、`fav/src/driver.rs`

---

## v95.6.0 — Function Import / Action Import

OData v2/v4 の Function Import / Action Import（RPC スタイル呼び出し）に対応する。

**実装方針**: v95.6.0 は `cfg: SapConfig` スタイルのスタブとして実装する。
ロードマップの `ctx.sap.function_import<T>()` / `ctx.sap.action_import()` 形式（ctx パターン）への移行は後続バージョンで実施する。

```favnir
-- Function Import パラメータ型
type FunctionImportParam = (String, String)

-- Function Import: 冪等な操作（GET）
bind result <- function_import<ReleaseResult>(cfg, "A_SalesOrder_Release", [
    ("SalesOrder", "0000000001")
])

-- Action Import: 副作用あり操作（POST）
bind _ <- action_import(cfg, "A_BusinessPartner_SetBlocked", [
    ("BusinessPartner", "BP001"),
    ("BusinessPartnerIsBlocked", "true")
])
```

**修正ファイル**: `runes/sap-odata/rpc.fav`（新規）、`fav/src/driver.rs`

**テスト数**: 4,174 + 3 = 4,177

---

## v95.7.0 — バッチ部分失敗ハンドリング

`BatchResponse` の詳細解析と部分成功（Partial Success）ハンドリングを実装する。

**実装方針**: v95.7.0 は `cfg: SapConfig` スタイルのスタブとして実装する。
下記コード例の `ctx.sap.batch_with_partial()` 形式（ctx パターン）への移行は後続バージョンで実施する。

```favnir
type BatchItemResult<T> =
    | BatchSuccess(T)
    | BatchFailure(BatchError)

type PartialSuccess<T> = {
    succeeded: List<BatchItemResult<T>>,
    failed:    List<BatchItemResult<T>>,  -- ランタイムでは BatchFailure のみが格納される（型の契約）
    success_rate: Float
}

-- cfg スタイルスタブ（v95.7.0）— ctx パターンへの移行は後続バージョン
bind ps <- batch_with_partial<BusinessPartner>(cfg, req)
bind _  <- ctx.io.println("成功率: " ++ Float.to_string(ps.success_rate))
```

**修正ファイル**: `runes/sap-odata/batch.fav`、`fav/src/driver.rs`

**テスト数**: 4,177 + 3 = 4,180

---

## v95.8.0 — `fav sap-mock`

オフラインテスト用の SAP OData モックサーバーを `fav sap-mock` コマンドで起動できるようにする。

```
$ fav sap-mock --port 8080 --fixtures runes/sap-odata/mock.fav
SAP Mock Server listening on http://localhost:8080
  GET  /sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner
  POST /sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner
  POST /$batch
```

**修正ファイル**: `fav/src/driver.rs`、`runes/sap-odata/mock.fav`（既存）、`fav/src/main.rs`

---

## v95.9.0 — 安定化・コードフリーズ

- スプリント総括テスト 2 件を `driver.rs` に追加する
  - `sprint1_sap_mock_registered`: `main.rs` に `"sap-mock"` が登録されている
  - `sprint1_rpc_fav_complete`: `rpc.fav` に `FunctionImportParam` / `function_import` / `action_import` が含まれる
- 全テスト通過確認（4,184 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過

---

## v96.0.0 — SAP Real-time 1.0 宣言

**宣言文**:

> 「SAP が、Favnir の時間軸で動き始めた。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに変化を知り、
>  Deep Insert で一気に書き込み、`fav sap-mock` でオフラインでもテストできる。
>
>  それが、Favnir SAP Real-time 1.0 である。」

**v96000_tests（4 テスト）**:
- `cargo_toml_version_is_96_0_0`
- `changelog_has_v96_0_0`
- `milestone_has_sap_realtime`
- `readme_mentions_sap_realtime`

---

## スプリント終了時の確認

- [ ] 4,188 tests, 0 failures
- [ ] `cargo clean` を実施する（★クリーンアップ）
- [ ] `cargo test` で 4,188 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v96.0.0 に更新
- [ ] `MILESTONE.md` に v96.0.0 エントリを追加
- [ ] `README.md` に `## v96.0 — SAP Real-time 1.0` セクションを追加
