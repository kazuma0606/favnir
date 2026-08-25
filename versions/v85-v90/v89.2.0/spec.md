# Spec: v89.2.0 — `OutstandingPayable` 型 + `match_unposted_orders()`

## Background

v89.1.0 で `JournalEntry` / `JournalFilter` / `journal_entries()` を追加した。
本バージョンでは未照合発注を検出するための `OutstandingPayable` 型と
`match_unposted_orders()` 関数スタブを実装する。
これにより v89.3.0 のシナリオ 4（購買→支払サイクル照合）の型基盤が揃う。

本バージョンでの新規追加は `OutstandingPayable` 型と `match_unposted_orders()` 関数のみ。
（ロードマップタイトルにあった「JournalEntryFilter」という記述は v89.1.0 で追加した `JournalFilter` と同一のものを指しており、ロードマップ側を修正済み）

## Goals

1. `runes/sap-odata/journal_entry.fav` に追記する
   - 型: `OutstandingPayable`
   - 関数: `match_unposted_orders(pos: List<PurchaseOrder>, journals: List<JournalEntry>) -> Result<List<OutstandingPayable>, String>`（スタブ）
2. `runes/sap-odata/sap_odata.fav` に `OutstandingPayable` re-export と `match_unposted_orders` ラッパーを追加する
3. `fav/src/driver.rs` に `mod v89200_tests` を追加する（2 件）

## API / Syntax Examples

```favnir
-- runes/sap-odata/journal_entry.fav に追記（v89.2.0）
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

## Success Criteria（Rust テストで担保）

- `outstanding_payable_type_exists`:
  `runes/sap-odata/journal_entry.fav` に `"OutstandingPayable"` を含む
- `match_unposted_orders_function_exists`:
  `runes/sap-odata/journal_entry.fav` に `"public fn match_unposted_orders("` を含む
- `cargo test` で 4,023 tests, 0 failures（4,021 + 2）

## Files to Modify / Create

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/journal_entry.fav` | 追記（`use sap_odata.purchase_order` をファイル先頭の use ブロックに追加、`OutstandingPayable` 型 + `match_unposted_orders()` 関数を末尾に追加） |
| `runes/sap-odata/sap_odata.fav` | `OutstandingPayable` re-export + `match_unposted_orders` ラッパー追加（`PurchaseOrder` は既に re-export 済み） |
| `fav/src/driver.rs` | `mod v89200_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
**Note**: `match_unposted_orders` は v89.3.0 のシナリオ 4（`sap_odata.match_unposted_orders(pos, journals)`）で呼び出されるため `public fn` とし、`sap_odata.fav` 経由で公開する。
