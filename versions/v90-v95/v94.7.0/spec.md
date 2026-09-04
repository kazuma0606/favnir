# Spec: v94.7.0 — E2E デモ更新（$batch + SnapStart 完全デモ）

## Background

v94.1.0〜v94.6.0 で SAP Advanced Era の各機能（BatchRequest / ChangeSet / Lambda SnapStart /
ベンチマーク / OSS 整備）を実装した。v94.7.0 ではこれらを統合した完全 E2E デモとして
`infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成する。

既存の `pipeline.fav`（4 シナリオ: マスタ同期 / 日次レポート / 在庫チェック / 会計伝票）に加え、
`$batch` + `QueryBuilder` を組み合わせたシナリオ 5 を追加する。

> **実装方針について**: ロードマップのコード例では `fetch_all_pages` + `with_filter` を使ったページング付き
> パターンを参考として示しているが、本バージョンでは `business_partner_filter` ヘルパーを用いた
> シンプルなパターンで実装する。`ctx.sap.batch` の呼び出しが含まれていれば完了条件を満たす。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成する
   - `import rune "sap-odata"` / `import rune "s3"` を使用
   - `ctx.sap.batch(req)` の呼び出しを含む（テスト要件）
   - QueryBuilder による取引先フィルタリング → バッチ更新の完全フロー
2. `driver.rs` に `mod v94700_tests` を追加する（2 件）

## Syntax/API Examples

```favnir
-- infra/e2e-demo/sap-odata/pipeline_advanced.fav
-- シナリオ 5: $batch + QueryBuilder による取引先一括同期（v94.7.0）

import rune "sap-odata"
import rune "s3"

fn advanced_sap_pipeline(ctx: AppCtx) -> Result<String, String> {
    -- 1. QueryBuilder で日本の取引先を取得
    bind filter <- business_partner_filter(Option.some("JP"), Option.some("1"), Option.some(100))
    bind bps    <- ctx.sap.business_partners(filter)

    -- 2. S3 にバックアップ保存
    bind json   <- Json.encode(bps)
    bind _      <- ctx.s3.put_object("sap-sync", "bps_jp.json", json)

    -- 3. $batch で一括更新（BatchUpdate × n件）
    bind ops    <- List.map(bps, fn(bp) { BatchUpdate(bp.BusinessPartner, bp) })
    bind req    <- batch_request_builder("A_BusinessPartner", ops)
    bind resp   <- ctx.sap.batch(req)

    Result.ok(String.concat("synced ", Int.to_string(List.length(resp.succeeded))))
}
```

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline_advanced.fav` が存在する
- `pipeline_advanced.fav` に `ctx.sap.batch` が含まれる
- `driver.rs` の `mod v94700_tests` が pass する
  - `pipeline_advanced_fav_exists`: `"../infra/e2e-demo/sap-odata/pipeline_advanced.fav"` が存在する
  - `pipeline_advanced_uses_batch`: ファイルに `"ctx.sap.batch"` が含まれる
- `cargo test 2>&1 | grep "test result"` が 4,156 tests, 0 failures を示す（着手前: 4,154）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `infra/e2e-demo/sap-odata/pipeline_advanced.fav` | **新規作成** | $batch + QueryBuilder による完全デモ（シナリオ 5） |
| `fav/src/driver.rs` | **追加** | `mod v94700_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.7.0 エントリ |
