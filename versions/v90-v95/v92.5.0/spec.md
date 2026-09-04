# Spec: v92.5.0 — W060 N+1 lint ルール追加

Status: COMPLETE

---

## Background

v92.4.0 で `fetch_all_pages<T>` を定義し、ページネーション一括取得が可能になった。
v92.5.0 は `List.map` / `List.flat_map` 等のコールバック内で `ctx.sap.*` を呼び出す N+1 クエリパターンを
コンパイル時に検出する lint ルール **W060** を追加する。

N+1 クエリは SAP OData においてパフォーマンス問題の主因であり、ループごとに HTTP リクエストが発生するため
大規模データで致命的な遅延を引き起こす。W060 により `fetch_all_pages` や一括取得への移行を促す。

---

## Goals

1. `fav/src/linter.rs` に W020 ルールを追加する
2. `driver.rs` に `mod v92500_tests`（2 件）を追加する

---

## W060 警告メッセージ

```
W060: N+1 クエリを検出しました。
  ループ内で `ctx.sap.sales_orders(...)` を呼び出しています。
  `fetch_all_pages` または一括取得を使用することを検討してください。
```

---

## Syntax / Detection Examples

```favnir
-- 問題あり（W020 検出対象）: List.map コールバック内で ctx.sap.* を呼び出す
bind results <- List.map(customer_ids, fn(id) {
    ctx.sap.sales_orders(SalesOrderFilter { sold_to: id })
})

-- 推奨（一括取得）
bind q   <- query<SalesOrder>()
bind q2  <- with_filter(q, Or(Eq("SoldToParty", "C1"), Eq("SoldToParty", "C2")))
bind all <- ctx.sap.sales_orders_query(q2)
```

### 検出対象パターン

- `List.map(_, fn(_) { ctx.sap.* })`
- `List.flat_map(_, fn(_) { ctx.sap.* })`
- `List.filter_map(_, fn(_) { ctx.sap.* })`（将来拡張）

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `fav/src/lint.rs` | W060 ルールを追加（`List.map` / `List.flat_map` コールバック内の `ctx.sap.*` 検出） |
| `fav/src/driver.rs` | `mod v92500_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,107 tests, 0 failures**（4,105 + 2）
- `lint.rs` に `W060` が含まれる
- `lint.rs` の W060 メッセージに `N+1` が含まれる
- `mod v92500_tests` 内の 2 テストが pass する:
  - `w060_lint_rule_defined`: `lint.rs` に `W060` が含まれる
  - `w060_lint_message_mentions_n_plus_1`: `lint.rs` に `N+1` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4095（4093+2）だが、v92.4.0 の実測が 4,105 のため、本バージョンは 4,105 + 2 = **4,107** が目標。

> **`bind` 再束縛（E0018）**: ロードマップの推奨例は `bind q <- with_filter(q, ...)` と同名再束縛しているが E0018 違反。spec の例では別名を使用する。

> **W060 の実装範囲**: v92.5.0 は `List.map` / `List.flat_map` を対象とする。`List.filter_map` や `for`/`while` 等への拡張は将来バージョンで行う。

> **既存 lint ルールとの整合**: `fav/src/lint.rs` の既存ルール（W001〜W059）の実装パターンに従って W060 を追加する。W020（`check_w020_deprecated_call`、v24.4.0）はすでに使用済みのため W060 を使用する。実装前に lint.rs の構造を確認すること（plan.md Step 0 参照）。
