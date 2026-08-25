# Plan: v89.2.0 — `OutstandingPayable` 型 + `match_unposted_orders()`

## 実装ステップ

### Step 1: `runes/sap-odata/journal_entry.fav` に追記

`journal_entries()` 関数定義の直後に追加:

```favnir
use sap_odata.purchase_order

-- 未照合発注（支払い未計上の発注伝票）型
-- days_overdue: 発注日からの経過日数（負値は未来の予定）
-- status: "Open" / "PartiallyDelivered" 等、SAP の未決済ステータスをそのまま保持
public type OutstandingPayable = {
    po_number:    String,
    vendor_id:    String,
    total_amount: Float,
    currency:     String,
    days_overdue: Int,
    status:       String
}

-- 発注 × 会計伝票の突き合わせ（v89.2.0 — スタブ）
-- TODO: implement — 一部納品済み発注と会計伝票を突き合わせて未払いを検出する
public fn match_unposted_orders(
    pos:      List<PurchaseOrder>,
    journals: List<JournalEntry>
) -> Result<List<OutstandingPayable>, String> {
    Result.err("not implemented")
}
```

**Note**: `use sap_odata.purchase_order` は `journal_entry.fav` の先頭の use ブロック（既存の `use sap_odata.types` の直後）に追加する（末尾への追記ではない）。
`JournalEntry` は同ファイル内で定義済みのため追加 use 不要。

### Step 2: `runes/sap-odata/sap_odata.fav` に re-export を追加

`journal_entries` ラッパーの直後に追加:

```favnir
public type OutstandingPayable = journal_entry.OutstandingPayable
public fn match_unposted_orders(pos: List<PurchaseOrder>, journals: List<JournalEntry>) -> Result<List<OutstandingPayable>, String> {
    journal_entry.match_unposted_orders(pos, journals)
}
```

### Step 3: `fav/src/driver.rs` に `mod v89200_tests` を追加

`mod v89100_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89200_tests {
    #[test]
    fn outstanding_payable_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/journal_entry.fav")
            .expect("runes/sap-odata/journal_entry.fav should exist");
        assert!(
            content.contains("OutstandingPayable"),
            "journal_entry.fav should define OutstandingPayable type"
        );
    }

    #[test]
    fn match_unposted_orders_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/journal_entry.fav")
            .expect("runes/sap-odata/journal_entry.fav should exist");
        assert!(
            content.contains("public fn match_unposted_orders("),
            "journal_entry.fav should define public fn match_unposted_orders"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,021 + 2 = 4,023 tests, 0 failures を確認する。

### Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
