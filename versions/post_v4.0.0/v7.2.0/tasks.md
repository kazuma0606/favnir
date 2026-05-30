# Favnir v7.2.0 Tasks

Date: 2026-05-27

## Goal

SQL Rune（型安全クエリビルダ）を実装する。
また、それを支える stdlib（List / String）拡張を同時に行う。

---

## Phase A — stdlib 拡張（List / String）

### A-1: List 拡張（vm.rs / checker.rs）

- [x] A-1-1: List.partition
- [x] A-1-2: List.empty

### A-2: String 拡張（vm.rs / checker.rs）

- [x] A-2-1: String.to_upper（String.upper エイリアス）
- [x] A-2-2: String.to_lower（String.lower エイリアス）
- [x] A-2-3: String.join — checker.rs に追加

### A-3: stdlib テスト（vm_stdlib_tests.rs）

- [x] A-3-1: List.partition — matching list length テスト
- [x] A-3-2: List.partition — non-matching list length テスト
- [x] A-3-3: String.join — checker + vm 動作確認
- [x] A-3-4: String.to_upper テスト
- [x] A-3-5: String.to_lower テスト

---

## Phase B — SQL Rune 実装（runes/sql/）

### B-1: rune 基盤

- [x] B-1-1: runes/sql/ ディレクトリ作成
- [x] B-1-2: runes/sql/rune.toml 作成（name="sql", version="0.1.0"）

### B-2: SqlQuery 型定義（runes/sql/query.fav）

- [x] B-2-1: SqlQuery レコード型定義（table, select_clause, where_clauses, params, joins, order_clause, limit_val, offset_val）
- [x] B-2-2: Sql.from(table: String) -> SqlQuery コンストラクタ

### B-3: ビルダー関数（すべて純粋）

- [x] B-3-1: Sql.select
- [x] B-3-2: Sql.add_where（where はキーワードのため add_where）
- [x] B-3-3: Sql.join
- [x] B-3-4: Sql.left_join
- [x] B-3-5: Sql.order_by
- [x] B-3-6: Sql.limit
- [x] B-3-7: Sql.offset

### B-4: SQL 生成 + 実行関数

- [x] B-4-1: Sql.to_sql(q) -> String（純粋）
- [x] B-4-2: Sql.run_raw(handle, q) -> Result<List<Map<String,String>>, DbError> !DbRead
- [x] B-4-3: Sql.count_raw(handle, q) -> Result<Int, DbError> !DbRead

### B-5: 書き込み関数

- [x] B-5-1: Sql.insert_into — !DbWrite
- [x] B-5-2: Sql.update_where — !DbWrite
- [x] B-5-3: Sql.delete_where — !DbWrite

### B-6: fav check 通過確認

- [x] B-6-1: fav check runes/sql/query.fav — no errors found

---

## Phase C — テスト（driver.rs）

- [x] C-1: sql_from_test — Sql.from("users") -> "SELECT * FROM users"
- [x] C-2: sql_add_where_test — WHERE 句付き to_sql 確認
- [x] C-3: sql_limit_offset_test — LIMIT/OFFSET 付き to_sql 確認
- [x] C-4: sql_order_test — ORDER BY 付き to_sql 確認
- [x] C-5: sql_join_test — INNER JOIN 付き to_sql 確認
- [x] C-6: cargo test 全件通過（1061 tests）

---

## Phase D — ドキュメント

- [x] D-1: site/content/docs/runes/sql.mdx 作成
- [x] D-2: site/content/docs/guides/type-safe-queries.mdx 作成

---

## Phase E — 最終確認

- [x] E-1: cargo test — 1061 tests passed; 0 failed
- [x] E-2: fav check runes/sql/query.fav — no errors found
- [x] E-3: このファイルを完了状態に更新

---

## 完了条件

- Sql ビルダー関数群が正しい SQL 文字列を返す ✓
- Sql.run_raw が !DbRead エフェクトで実行できる ✓
- stdlib 追加関数（List/String）が各テストで確認済み ✓
- SQL Rune 統合テスト 5 件通過 ✓
- サイトドキュメント 2 ページ追加（sql.mdx + type-safe-queries.mdx） ✓
- 既存テスト 1061 件が全件通る ✓
