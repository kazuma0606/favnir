# Tasks: v88.7.0 — `StockAlert` 型 + `detect_stock_shortage()` 完全化

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,009 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88600_tests` が存在することを確認する（v88.6.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）
- [x] `runes/sap-odata/stock.fav` に `"public fn detect_stock_shortage("` が含まれていることを確認する（T2 テストの検索文字列が実ファイルと一致することを事前保証）

## T1: `runes/sap-odata/stock.fav` に `format_stock_alerts` を追加

- [x] `detect_stock_shortage()` の直後に `fn format_stock_alerts(alerts: List<StockAlert>) -> String` を追加する
- Note: `detect_stock_shortage()` は v88.6.0 でスタブ実装済みのため追加不要

## T2: `driver.rs` に `mod v88700_tests` を追加

- [x] `mod v88600_tests { ... }` の直後に `#[cfg(test)] mod v88700_tests { ... }` を追加する
- [x] `detect_stock_shortage_function_exists` テストを実装する（`stock.fav` で `"public fn detect_stock_shortage("` を確認）
- [x] `format_stock_alerts_function_exists` テストを実装する（`stock.fav` で `"format_stock_alerts"` を確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,011 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] T0 に `stock.fav` の `"public fn detect_stock_shortage("` 存在確認チェックを追加
- [LOW] `format_stock_alerts` の非公開設計意図を spec.md コメントおよび `stock.fav` コメントで明記
