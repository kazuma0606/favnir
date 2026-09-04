# Spec: v91.7.0 — `JournalEntryQuery` 実装

Status: 未着手

---

## Background

v91.6.0 で `MaterialQuery` / `PurchaseOrderQuery` を実装した。v91.7.0 では SAP 会計エンティティである **会計伝票（JournalEntry）** のクエリ型を追加する。

`JournalEntryQuery` は他クエリ型にはない **`fiscal_year: Option<String>`** フィールドを持つ。SAP 会計伝票の検索では `FiscalYear` が必須に近い絞り込みキーであるため、専用フィールドとして型に組み込む。

### 循環 dep 制約（v91.4.0〜引き継ぎ）

`types.fav` は `query.fav` を import できないため、`SapClient` interface への `journal_entries_query` 追加は **v91.8.0 へ引き続き延期**する。

---

## Goals

1. `runes/sap-odata/query.fav` に `JournalEntryQuery` 型・`journal_entry_query()` ビルダーを追加する
2. Rust テスト 2 件を `driver.rs` に追加する

---

## Syntax / API Examples

```favnir
-- 会計伝票クエリオプション（$filter / $select / $top / $skip + 会計年度専用フィールド）（v91.7.0）
-- fiscal_year: SAP 会計年度（例: "2024"）。Option.none() の場合は全年度対象。
-- expand: なし（journal_entry.fav にナビゲーションプロパティ未定義のため。将来追加予定）
-- SapClient.journal_entries_query への統合は v91.8.0 で実装（循環 dep のため延期）
public type JournalEntryQuery = {
    filter:      Option<FilterExpr<JournalEntry>>,
    select:      Option<SelectClause<JournalEntry>>,
    fiscal_year: Option<String>,
    top:         Option<Int>,
    skip:        Option<Int>
}

-- 全フィールドを none() で初期化するビルダー
public fn journal_entry_query() -> JournalEntryQuery {
    JournalEntryQuery {
        filter:      Option.none(),
        select:      Option.none(),
        fiscal_year: Option.none(),
        top:         Option.none(),
        skip:        Option.none()
    }
}

-- 使用例: 特定会計年度の高額伝票を取得
bind q <- journal_entry_query()
bind q <- {
    q |
    fiscal_year: Option.some("2024"),
    filter: Option.some(Gt("AmountInTransactionCurrency", "1000"))
}
-- ctx.sap.journal_entries_query(q) は v91.8.0 以降で対応予定
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query.fav` | `use sap_odata.journal_entry` 追加、`JournalEntryQuery` 型・`journal_entry_query()` 追加 |
| `fav/src/driver.rs` | `mod v91700_tests` 追加（2 件） |

**変更しないファイル（循環 dep 制約）:**
- `runes/sap-odata/types.fav` — SapClient 拡張は v91.8.0 へ延期
- `runes/sap-odata/client.fav` / `mock.fav` — 同上

---

## Success Criteria

- `cargo test` 全 pass: **4,083 tests, 0 failures**（4,081 + 2）
- `runes/sap-odata/query.fav` に `public type JournalEntryQuery` が存在する
- `runes/sap-odata/query.fav` に `public fn journal_entry_query` が存在する
- `mod v91700_tests` 内の 2 テストが pass する:
  - `journal_entry_query_type_defined`
  - `journal_entry_query_builder_defined`

---

## Note

> **CHANGELOG**: v91.7.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ一覧表の計画値は 4079 + 2 = 4081 だが、v91.3.0 実測時に +2 超過（計画 +2 → 実測 +4）したため以降のベース値が +2 ずれており、実測は 4,081 ベース（→ 4,083）。ロードマップ一覧表・推移表の修正は v92.0.0 宣言時に実施する。

> **`fiscal_year` フィールドの設計**: 他クエリ型にはない専用フィールド。SAP 会計伝票では FiscalYear が事実上の必須絞り込み条件であるため、`FilterExpr` ではなく型レベルのフィールドとして明示する。`Option.none()` の場合は全年度対象（パフォーマンス注意）。

> **`expand` なし**: 現バージョンの `JournalEntry` 型（journal_entry.fav）にナビゲーションプロパティが未定義のため `expand` は省略。将来バージョンで追加予定。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は v92.0.0 宣言時にまとめて実施する。
