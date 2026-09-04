# Spec: v95.7.0 — バッチ部分失敗ハンドリング

## Background

v94.1.0〜v94.2.0 で導入した `BatchResponse<T>` は `succeeded: List<T>` と `failed: List<BatchError>` を
分けて保持するが、各操作が成功したか失敗したかを順序付きで追跡する手段がなかった。

本バージョンでは `runes/sap-odata/batch.fav` に以下を追加する：
- `BatchItemResult<T>` — 個別バッチ操作の成否を表す直和型
- `PartialSuccess<T>` — 部分成功レスポンスをまとめた集計型
- `batch_with_partial` — `PartialSuccess<T>` を返すスタブ関数

ロードマップの `ctx.sap.batch_with_partial(req)` 形式（ctx パターン）への移行は後続バージョンで実施する。
v95.7.0 では既存スタイルに合わせた `cfg: SapConfig` 形式のスタブとして実装する。

## Goals

1. `BatchItemResult<T>` 直和型を `batch.fav` に追加する
   - `BatchSuccess(T)` — 成功ケース
   - `BatchFailure(BatchError)` — 失敗ケース
2. `PartialSuccess<T>` レコード型を `batch.fav` に追加する
   - `succeeded: List<BatchItemResult<T>>`
   - `failed:    List<BatchItemResult<T>>`
   - `success_rate: Float`
3. `batch_with_partial` スタブ関数を `batch.fav` に追加する
4. `fav/src/driver.rs` に `mod v95700_tests`（2 件）を追加する

## Syntax / API Examples

```favnir
-- バッチ操作の個別結果型
type BatchItemResult<T> =
    | BatchSuccess(T)
    | BatchFailure(BatchError)

-- 部分成功レスポンス集計型
-- NOTE: failed フィールドの型は List<BatchItemResult<T>> だが、
--       ランタイムでは BatchFailure(BatchError) のみが格納される（ランタイム契約）。
--       BatchSuccess が failed リストに入ることはない。
type PartialSuccess<T> = {
    succeeded:    List<BatchItemResult<T>>,
    failed:       List<BatchItemResult<T>>,
    success_rate: Float
}

-- バッチ部分失敗ハンドリング（cfg スタイルスタブ）
-- NOTE: v95.7.0 は cfg スタイルのスタブ。ctx.sap.batch_with_partial() への移行は後続バージョン。
bind ps <- batch_with_partial<BusinessPartner>(cfg, req)
bind _  <- ctx.io.println("成功率: " ++ Float.to_string(ps.success_rate))
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/batch.fav` | 修正 | `BatchItemResult<T>` / `PartialSuccess<T>` 型追加 + `batch_with_partial` スタブ |
| `fav/src/driver.rs` | 修正 | `mod v95700_tests`（2 件）追加 |

## Success Criteria

- `batch.fav` に `BatchItemResult` が含まれる
- `batch.fav` に `PartialSuccess` が含まれる
- `batch.fav` に `batch_with_partial` が含まれる
- `cargo test` で 4,180 tests, 0 failures

## Out of Scope（次バージョン以降）

- `SapClient` interface への `batch_with_partial` 追加
- `ctx.sap.batch_with_partial()` ctx パターン実装
- 実際の OData $batch 部分失敗レスポンス解析 HTTP 実装
- `sap_odata.fav` への re-export 追加
