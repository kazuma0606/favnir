# Favnir v7.2.0 Plan — SQL Rune

作成日: 2026-05-27

## 全体方針

5 フェーズ構成。Phase A（stdlib 拡張）→ Phase B（SQL Rune コア）→ Phase C（テスト）→ Phase D（ドキュメント）→ Phase E（最終確認）。

Phase A の stdlib 拡張は SQL Rune の実装に使われるだけでなく、
Favnir を使うデータエンジニア全般の生産性を上げる。

---

## Phase A — stdlib 拡張（List / String / Map）

**目的**: SQL Rune 実装に必要な関数を先に整備し、それ自体も価値を持つ改善とする。

### A-1: List 拡張
実装場所: `fav/src/backend/vm.rs`（VM 組み込み）、`fav/src/middle/checker.rs`（型）

追加する関数:
- `List.zip : List<A> -> List<B> -> List<(A, B)>`
  - 短い方の長さで打ち切り（Haskell の zip と同じ）
- `List.flatten : List<List<A>> -> List<A>`
- `List.flat_map : List<A> -> (A -> List<B>) -> List<B>`
- `List.partition : List<A> -> (A -> Bool) -> (List<A>, List<A>)`
  - `(true_items, false_items)` のタプルを返す
- `List.sum_int : List<Int> -> Int`
- `List.min_int : List<Int> -> Option<Int>`
- `List.max_int : List<Int> -> Option<Int>`
- `List.count : List<A> -> (A -> Bool) -> Int`

### A-2: String 拡張
追加する関数:
- `String.split : String -> String -> List<String>`（区切り文字で分割）
- `String.trim : String -> String`（前後の空白を除去）
- `String.to_upper : String -> String`
- `String.to_lower : String -> String`
- `String.starts_with : String -> String -> Bool`
- `String.ends_with : String -> String -> Bool`
- `String.replace : String -> String -> String -> String`（`replace(s, from, to)`）
- `String.pad_left : String -> Int -> String -> String`（`pad_left(s, width, pad_char)`）
- `String.pad_right : String -> Int -> String -> String`

### A-3: Map 拡張
追加する関数:
- `Map.merge : Map<K, V> -> Map<K, V> -> Map<K, V>`（右辺が優先）
- `Map.from_list : List<(String, V)> -> Map<String, V>`
- `Map.map_values : Map<K, V> -> (V -> W) -> Map<K, W>`

### A-4: テスト
`fav/src/backend/vm.rs` の既存テストモジュールに各関数のテストを追加。

---

## Phase B — SQL Rune コア実装

**目的**: `runes/sql/query.fav` を Favnir で実装する。

### B-1: SqlQuery 型定義

```favnir
type SqlQuery = {
  table:         String
  select_clause: String
  where_clauses: List<String>
  params:        List<String>
  joins:         List<String>
  order_clause:  String
  limit_val:     Option<Int>
  offset_val:    Option<Int>
}
```

### B-2: ビルダー関数実装

- `Sql.from(table: String) -> SqlQuery` — 初期クエリ生成
- `Sql.select(q: SqlQuery, cols: String) -> SqlQuery`
- `Sql.where(q: SqlQuery, cond: String, params: List<String>) -> SqlQuery`
- `Sql.join(q: SqlQuery, table: String, on: String) -> SqlQuery`
- `Sql.left_join(q: SqlQuery, table: String, on: String) -> SqlQuery`
- `Sql.order_by(q: SqlQuery, col: String, dir: String) -> SqlQuery`
- `Sql.limit(q: SqlQuery, n: Int) -> SqlQuery`
- `Sql.offset(q: SqlQuery, n: Int) -> SqlQuery`

### B-3: SQL 生成関数

- `Sql.to_sql(q: SqlQuery) -> String`
  - `SELECT {select} FROM {table} {joins} WHERE {wheres} ORDER BY {order} LIMIT {limit} OFFSET {offset}`
  - 各節が空の場合はスキップ
  - `String.join`（既存）で WHERE 句を ` AND ` で結合

### B-4: 実行関数

- `Sql.run_raw(q: SqlQuery) -> !DbRead` → `List<Map<String, String>>`
  - `Db.query_raw(Sql.to_sql(q), q.params)` を呼び出す
- `Sql.count_raw(q: SqlQuery) -> !DbRead` → `Int`
  - COUNT(*) クエリを生成して実行

### B-5: 書き込み関数（独立した関数、SqlQuery を使わない）

- `Sql.insert_into(table: String, cols: List<String>, values: List<List<String>>) -> !DbWrite` → `Int`
  - `INSERT INTO {table} ({cols}) VALUES ...` を生成して実行
- `Sql.update(table: String, assignments: List<(String, String)>, where_cond: String) -> !DbWrite` → `Int`
  - `UPDATE {table} SET {assignments} WHERE {where_cond}` を実行
- `Sql.delete_from(table: String, where_cond: String, params: List<String>) -> !DbWrite` → `Int`

### B-6: fav.toml + runes/ 配置

`runes/sql/fav.toml`:
```toml
[rune]
name    = "sql"
version = "0.1.0"
```

`runes/sql/query.fav` — メイン実装ファイル

---

## Phase C — テスト

### C-1: stdlib テスト（A-1 〜 A-3）

各追加関数に対して vm_stdlib_tests.rs に 1 件以上のテストを追加。
- `List.zip` — 等長 / 短い方で打ち切り
- `List.flatten` — ネストしたリストを平坦化
- `List.flat_map` — 変換 + 平坦化
- `String.split` — カンマ区切り分割
- `String.trim` / `to_upper` / `to_lower`
- `String.replace`
- `Map.merge` / `Map.from_list`

### C-2: SQL Rune テスト（driver.rs の統合テスト）

`fav check runes/sql/query.fav` が通ること（型チェック通過）。

Favnir ソース内で SQL Rune をインポートした統合テスト 5 件:
1. `Sql.from("users") |> Sql.to_sql` → `"SELECT * FROM users"`
2. `Sql.from("users") |> Sql.where("active = ?", ["true"]) |> Sql.to_sql` → WHERE 句あり
3. `Sql.from("users") |> Sql.limit(10) |> Sql.to_sql` → LIMIT 付き
4. `Sql.from("users") |> Sql.order_by("name", "asc") |> Sql.to_sql` → ORDER BY 付き
5. JOIN を含むクエリの to_sql 出力

---

## Phase D — ドキュメント

### D-1: `runes/sql.mdx`（新規）

- SQL Rune の概要・インストール方法
- `SqlQuery` 型の説明
- 全関数リファレンス（シグネチャ + 使用例）
- `T.validate` との組み合わせパターン

### D-2: `guides/type-safe-queries.mdx`（新規）

- ユースケースストーリー（DBからデータを型安全に取得する）
- Step 1: `fav infer --db` でスキーマ取得
- Step 2: SQL Rune でクエリ構築
- Step 3: `T.validate` で型変換
- Step 4: `fav explain --lineage` でデータリネージ確認

---

## Phase E — 最終確認

- `cargo test` 全件通過（1051 + 新規テスト分）
- `fav check runes/sql/query.fav` エラーなし
- `roadmap-v7.md` のコメントを更新
- `versions/v7.2.0/tasks.md` を完了状態に更新

---

## 実装順序と依存関係

```
Phase A (stdlib拡張)
  └→ Phase B (SQL Rune) ← A の String/List 関数を使う
       └→ Phase C (テスト) ← B の実装を検証
            └→ Phase D (ドキュメント) ← C で動作確認済みの API をドキュメント化
                 └→ Phase E (最終確認)
```

## リスクと対策

| リスク | 対策 |
|--------|------|
| Favnir の Rune で `Option<Int>` が正しく扱えない | 事前に `checker.rs` の Option 型シグネチャを確認 |
| `List<(String, String)>` タプル型が Favnir で書けない | タプルの代わりに `type Pair = { key: String, val: String }` を定義 |
| `Db.query_raw` の実際のシグネチャ確認 | `runes/db/query.fav` を事前確認して合わせる |
| stdlib に既存の同名関数がある | checker.rs を事前 grep して衝突を避ける |
