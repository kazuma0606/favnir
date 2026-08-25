# Spec: v89.1.0 — `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()`

## Background

v89.0.0 で SAP Procurement 1.0 を宣言した。Sprint 5（v89.1〜v90.0）では
SAP Integration 1.0 宣言を目指し、会計伝票（`JournalEntry`）型を追加して
全 4 業務シナリオを完成させる。
本バージョンでは会計伝票の Favnir 型定義と一覧取得関数スタブを実装する。

## Goals

1. `runes/sap-odata/journal_entry.fav` を新規作成する
   - 型: `DebitCredit` / `JournalEntryItem` / `JournalEntry` / `JournalFilter`
   - 関数: `journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String>`（スタブ）
2. `runes/sap-odata/sap_odata.fav` に `use` と re-export を追加する
3. `fav/src/driver.rs` に `mod v89100_tests` を追加する（2 件）

## API / Syntax Examples

```favnir
-- runes/sap-odata/journal_entry.fav（v89.1.0 新規作成）
use sap_odata.types

-- 借方/貸方区分
public type DebitCredit = Debit | Credit

-- 会計伝票明細型
-- item_number: SAP の伝票明細番号（001, 002 ... ）
-- cost_center: 原価センタ（任意）
public type JournalEntryItem = {
    item_number:  Int,
    gl_account:   String,
    amount:       Float,
    currency:     String,
    debit_credit: DebitCredit,
    cost_center:  Option<String>
}

-- 会計伝票ヘッダ型
-- posting_date: ISO 8601 文字列（SAP OData の PostingDate をそのまま保持）
-- items: $expand=Items 指定時のみ展開される（未指定時は None）
public type JournalEntry = {
    document_number: String,
    fiscal_year:     Int,
    posting_date:    String,
    document_type:   String,
    company_code:    String,
    reference:       Option<String>,
    items:           Option<List<JournalEntryItem>>
}

-- 会計伝票フィルタ型
public type JournalFilter = {
    fiscal_year:       Option<Int>,
    posting_date_from: Option<String>,
    company_code:      Option<String>,
    reference:         Option<String>,
    top:               Option<Int>
}

-- 会計伝票一覧取得（v89.1.0 — スタブ）
-- TODO: implement — OData /JournalEntries クエリ + フィルタ適用
public fn journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String> {
    Result.err("not implemented")
}
```

## Success Criteria（Rust テストで担保）

- `journal_entry_type_defined_in_rune`:
  `runes/sap-odata/journal_entry.fav` に `"JournalEntry"` を含む（`JournalEntry` 型名の部分一致で判定。`JournalEntryItem` も含まれる型名のため意図的に弱い検索を使用）
- `journal_entries_function_exists`:
  `runes/sap-odata/journal_entry.fav` に `"public fn journal_entries("` を含む
- `cargo test` で 4,021 tests, 0 failures（4,019 + 2）

## Files to Modify / Create

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/journal_entry.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.journal_entry` + re-export 追加（既に `use sap_odata.types` が含まれており `SapConfig` は参照可能） |
| `fav/src/driver.rs` | `mod v89100_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
