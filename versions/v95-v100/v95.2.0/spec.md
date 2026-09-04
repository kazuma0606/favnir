# Spec: v95.2.0 — `ctx.sap.delta_fetch<T>()`

## Background

v95.1.0 で `DeltaResult<T>` / `DeletedEntity` の型定義を完了した。
本バージョンでは、その型を使って実際に差分取得を行う `delta_fetch<T>` メソッドを
`SapClient` interface に追加し、`SapODataClient` で実装する。

`ctx.sap.delta_fetch<T>(entity_set, delta_link)` を呼ぶことで:
- `delta_link = Option.none()` → 初回全件取得 + `deltaLink` 付き `DeltaResult<T>` を返す
- `delta_link = Option.some(link)` → 前回以降の差分のみ取得

差分リンクを S3 等に保存 → 次回実行時に読み込む、というパターンで
定期バッチの転送量を大幅に削減できる。

## Goals

1. `SapClient` interface に `delta_fetch<T>` メソッドを追加する（`types.fav`）
2. `SapODataClient` に `delta_fetch<T>` のスタブ実装を追加する（`client.fav`）
3. `client.fav` に `use sap_odata.delta` を追加する
4. `sap_odata.fav` に `delta_fetch<T>` の re-export を追加する
5. `driver.rs` に `mod v95200_tests`（2 テスト）を追加する

## 型定義・API 例

```favnir
-- SapClient interface に追加するシグネチャ
-- entity_set: OData エンティティセット名（例: "A_BusinessPartner"）
-- delta_link: Option.none() → 初回全件 / Option.some(link) → 差分取得
fn delta_fetch<T>(ctx: SapClient, entity_set: String, delta_link: Option<String>) -> Result<DeltaResult<T>, String>
```

```favnir
-- 使用例: 初回全件取得 → deltaLink を S3 に保存
bind result <- ctx.sap.delta_fetch<BusinessPartner>("A_BusinessPartner", Option.none())
bind _      <- ctx.s3.put_object("my-bucket", "delta_link.txt", result.delta_link)

-- 2 回目以降: 差分のみ取得
bind saved  <- ctx.s3.get_object("my-bucket", "delta_link.txt")
bind result <- ctx.sap.delta_fetch<BusinessPartner>("A_BusinessPartner", Option.some(saved))
bind _      <- ctx.io.println("差分件数: " ++ Int.to_string(List.length(result.entities)))
```

## Success Criteria

- `runes/sap-odata/types.fav` の `SapClient` interface に `delta_fetch` が含まれる
- `runes/sap-odata/client.fav` の `SapODataClient` に `delta_fetch` が実装される
- `driver.rs` の `v95200_tests` が 2 テスト全て pass する
- テスト総数: **4,168**（4,166 + 2）

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/types.fav` | 追記 | `SapClient` interface に `delta_fetch<T>` シグネチャを追加 |
| `runes/sap-odata/client.fav` | 追記 | `use sap_odata.delta` + `SapODataClient` に `delta_fetch<T>` スタブ実装 |
| `runes/sap-odata/sap_odata.fav` | 追記 | `delta_fetch<T>` の re-export 追加 |
| `fav/src/driver.rs` | 追記 | `mod v95200_tests`（2 テスト） |

## Notes

- `delta_fetch<T>` の実装はスタブ（`Result.err("not implemented")`）で可。
  実際の HTTP 呼び出し（`$delta` クエリ付き URL 構築）は v95.3.0〜 以降で本実装する。
- `ctx.fav`（AppCtx 定義）は変更不要。`ctx.sap` が `SapClient` interface を実装しており、
  interface に `delta_fetch<T>` を追加すれば `ctx.sap.delta_fetch<T>(...)` の呼び出し形式は自動的に有効になる。
- `MockSapClient` への `delta_fetch<T>` 追加は本バージョンのスコープ外。
  Favnir の型チェッカーは `impl Interface for Type` で全メソッド実装を強制しない（構造的部分実装が許容される）。
  `mock.fav` はすでに `batch` を未実装のまま `impl SapClient` を宣言しており、v95.1.0 時点で 4,166 tests pass している。
  よって `delta_fetch<T>` を interface に追加しても `mock.fav` はコンパイルエラーにならない（確認済み）。
- ロードマップの「修正ファイル」に `runes/ctx/ctx.fav` の記載があるが、設計の再検討により変更不要と確認した。
  `ctx.sap` は `SapClient` interface 型であり、interface へのメソッド追加で `ctx.sap.delta_fetch<T>(...)` は自動的に呼び出し可能になる。
