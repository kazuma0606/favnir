# v25.4.0 タスクリスト — mysql Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.4.0"` に bump + `mysql` crate 追加（`cargo build` で features 確認） | [x] |
| T1 | `fav/src/ast.rs` 更新（`Effect::MySQL` 追加） | [x] |
| T2 | `fav/src/error_catalog.rs` 更新（E0321 追加） | [x] |
| T3 | `fav/src/fmt.rs` / `fav/src/lineage.rs` / `fav/src/emit_python.rs` / `fav/src/lint.rs` / `fav/src/middle/reachability.rs` / `fav/src/middle/ast_lower_checker.rs` 更新（`Effect::MySQL` 対応・6 ファイル） | [x] |
| T4 | `fav/src/middle/checker.rs` 更新（`ns_to_inferred_effect` / `require_mysql_effect` / MySQL builtin fns） | [x] |
| T5 | `fav/src/frontend/parser.rs` 更新（`"MySQL" => Effect::MySQL` アーム追加） | [x] |
| T6 | `fav/src/driver.rs` 更新（`format_effects` / `effect_json_name` に MySQL アーム追加） | [x] |
| T7 | `fav/src/backend/vm.rs` 更新（`MySQL.*_raw` 6 件追加） | [x] |
| T8 | `runes/mysql/mysql.fav` 全面更新（type MySqlConn + connect / query / execute / transaction_begin / commit / rollback） | [x] |
| T9 | `examples/mysql_orders_etl.fav` 新規作成（`import rune "mysql"` 使用） | [x] |
| T10 | `site/content/docs/runes/mysql.mdx` 新規作成（全 API 記載） | [x] |
| T11 | `CHANGELOG.md` 更新（`[v25.4.0]` エントリ追加） | [x] |
| T12 | `benchmarks/v25.4.0.json` 新規作成（test_count: 2000） | [x] |
| T13 | `fav/src/driver.rs` 更新（`v254000_tests` 6 件追加） | [x] |
| T14 | `cargo test v254000` — 6 件 PASS 確認 | [x] |
| T15 | `cargo test` 総テスト数 ≥ 2000 件 確認 | [x] |
| T16 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `MySQL.connect` が `runes/mysql/mysql.fav` に存在する
- [x] `MySQL.query` / `MySQL.execute` が `runes/mysql/mysql.fav` に存在する
- [x] `MySQL.transaction_begin` / `commit` / `rollback` が `runes/mysql/mysql.fav` に存在する
- [x] `MySQL.*_raw` 6 件すべてが `fav/src/backend/vm.rs` に存在する
- [x] `Effect::MySQL` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match エラーなし確認済み）
- [x] E0321 が `fav/src/error_catalog.rs` に存在する（`checker.rs` の `require_mysql_effect` が `"E0321"` を使用していること）
- [x] `examples/mysql_orders_etl.fav` が存在し `import rune "mysql"` / `query` / `execute` を含む
- [x] `CHANGELOG.md` に `v25.4.0` が存在する
- [x] `site/content/docs/runes/mysql.mdx` が存在し全 API を記載している
- [x] `v254000_tests` 6 件すべて PASS
- [x] 総テスト数 ≥ 2000 件

---

## コードレビュー指摘（spec-reviewer）

*実装後に記録*

---

## 実装時に発見した追加修正

*実装後に記録*

---

## メモ

- `mysql` crate の URL 形式: `mysql://user:pass@host:port/db`
- `transaction_begin/commit/rollback` は VM 制約により各 primitive で独立接続を使用（擬似トランザクション）。コメントで明記すること
- `Effect::MySQL` 追加で更新が必要なファイル（見積もり）: ast.rs / checker.rs / reachability.rs / ast_lower_checker.rs / emit_python.rs / lineage.rs / lint.rs / fmt.rs / driver.rs / parser.rs（計 10 ファイル + driver.rs の 2 箇所）
- E0315 が Postgres、E0320 が Redis で使用済み。E0316〜E0319 は未割当の空き番号。連番で E0321 を採用
- 目標テスト数 2000 件（v25.3.0 終了時実測 1994 件 + 6 件 = 2000 件。v25.3.0 コードレビュー修正で +1 件増加した実測値）
- `transaction_begin/commit/rollback` は VM 制約（クロージャ引数渡し未実装）により独立接続での擬似実装。コメント必須
- `mysql` crate v24 を想定。ビルド失敗時は v23 にダウングレード（crates.io で最新安定版を確認）
