# Spec: v95.1.0 — OData `$delta` / `DeltaLink` 型定義

## Background

v95.0.0 までの SAP 統合はすべて**ポーリング型**であった。
`ctx.sap.business_partners(filter)` を定期実行しても「前回以降に変わったものだけ」を得る手段がなく、
全件取得 → 差分検出 をアプリケーション側で行う必要があった。

OData プロトコルには `$delta` という差分同期機能が標準で存在する。
初回リクエストで全件 + `@odata.deltaLink` を受け取り、
以降は `deltaLink` を使って「前回以降の変更・削除のみ」を取得できる。

v95.1.0 ではこの仕組みを Favnir の型として表現する**基盤型**を定義する。
実際の `ctx.sap.delta_fetch<T>()` API は v95.2.0 で実装する。

## Goals

1. `DeltaResult<T>` 型を `runes/sap-odata/delta.fav` に定義する
2. `DeletedEntity` 型（トゥームストーン）を定義する
3. `delta_link_is_valid` ヘルパー関数を定義する（deltaLink 文字列の基本バリデーション）
4. `driver.rs` に `mod v95100_tests` を追加し、2 テストが通ることを確認する

## 型定義・API 例

```favnir
-- OData $delta レスポンスの結果型
-- T はエンティティ型（BusinessPartner / SalesOrder 等）
type DeltaResult<T> = {
    entities:   List<T>,
    delta_link: String,     -- 次回呼び出し用 @odata.deltaLink
    has_more:   Bool        -- @odata.nextLink が存在するかどうか
}

-- 削除されたエンティティを表すトゥームストーン
-- OData $delta レスポンスの @removed アノテーション付きオブジェクトに対応
type DeletedEntity = {
    id:     String,
    reason: String          -- "deleted" | "changed"
}

-- deltaLink の簡易バリデーション（空文字列でないかチェック）
public fn delta_link_is_valid(link: String) -> Bool {
    String.length(link) > 0
}
```

## Success Criteria

- `runes/sap-odata/delta.fav` が存在する
- `delta.fav` に `DeltaResult` という文字列が含まれる
- `driver.rs` の `v95100_tests` が 2 テスト全て pass する
- テスト総数: **4,166**（4,164 + 2）

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/delta.fav` | 新規作成 | `DeltaResult<T>` / `DeletedEntity` / `delta_link_is_valid` |
| `fav/src/driver.rs` | 追記 | `mod v95100_tests`（2 テスト） |

## Notes

- `DeltaResult<T>` はジェネリック型。`delta.fav` 内でのみ定義し、`sap_odata.fav` への re-export は v95.2.0 以降で行う
- `delta_link_is_valid` は本バージョンのテスト対象ではなく、v95.2.0 のユーティリティとして先行定義する。`DeletedEntity` 型の存在確認テストは v95.2.0 で追加する
- driver.rs の 2 テストは `delta_fav_exists`（ファイル存在確認）と `delta_result_type_defined`（`DeltaResult` 文字列確認）のみ。`DeletedEntity` / `delta_link_is_valid` のテストは v95.2.0 で追加する
- `has_more: Bool` フィールドは `@odata.nextLink` の有無を表す（`true` の場合は `odata_list_paged` との組み合わせが必要）
- `DeletedEntity.reason` は OData 仕様の `reason: "deleted"` / `reason: "changed"` に対応する文字列
