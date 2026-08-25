# Plan: v89.1.0 — `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()`

## 実装ステップ

### Step 1: `runes/sap-odata/journal_entry.fav` を新規作成

```favnir
-- 会計伝票型定義 + 一覧取得関数（v89.1.0）
-- journal_entries() はスタブ（API 完全化済み）。アルゴリズム本実装は将来バージョンで行う。
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

### Step 2: `runes/sap-odata/sap_odata.fav` に `use` と re-export を追加

ファイル末尾の `use sap_odata.stock` の後に追加:

```favnir
use sap_odata.journal_entry
```

re-export ブロックの末尾（`detect_stock_shortage` wrapper の後）に追加:

```favnir
public type DebitCredit    = journal_entry.DebitCredit
public type JournalEntryItem = journal_entry.JournalEntryItem
public type JournalEntry   = journal_entry.JournalEntry
public type JournalFilter  = journal_entry.JournalFilter
public fn journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String> {
    journal_entry.journal_entries(cfg, filter)
}
```

### Step 3: `fav/src/driver.rs` に `mod v89100_tests` を追加

`mod v88900_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89100_tests {
    #[test]
    fn journal_entry_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/journal_entry.fav")
            .expect("runes/sap-odata/journal_entry.fav should exist");
        assert!(
            content.contains("JournalEntry"),
            "journal_entry.fav should define JournalEntry type"
        );
    }

    #[test]
    fn journal_entries_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/journal_entry.fav")
            .expect("runes/sap-odata/journal_entry.fav should exist");
        assert!(
            content.contains("public fn journal_entries("),
            "journal_entry.fav should define public fn journal_entries"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,019 + 2 = 4,021 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
