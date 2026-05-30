# Favnir v7.2.0 Spec — SQL Rune（型安全クエリビルダ）

作成日: 2026-05-27

## テーマ

**dbt 代替の第一歩**。型安全なクエリを Favnir の流暢な API で記述する SQL Rune を実装する。

## 背景

v7.0.0 で `!DbRead` / `!DbWrite` エフェクトが完成した。
v7.1.0 でデータリネージの静的解析（`fav explain --lineage`）が完成した。
v7.2.0 では「データを取得する」部分そのものを型安全に構築する手段を提供する。

現在のデータアクセスは `Db.query_raw(sql, params)` の形で SQL 文字列を直書きする必要がある。
これは：
- SQLインジェクションリスクがある
- 型との整合性がコンパイル時に検証されない
- クエリの構造がコードで追いにくい

SQL Rune はこの問題を解決する。

## 目標

```favnir
import rune "sql"

// クエリビルダの使用例
bind result <- Sql.from("users")
  |> Sql.where("active = ?", [true])
  |> Sql.select("id, name, email")
  |> Sql.order_by("name", "asc")
  |> Sql.limit(100)
  |> Sql.run_raw  // !DbRead → List<Map<String, String>>

// 型付きクエリ（schemas/*.yaml と連携）
bind users <- Sql.from("users")
  |> Sql.where("active = ?", [true])
  |> Sql.run<User>  // !DbRead → Result<List<User>, String>
```

## v7.2.0 のスコープ（v1）

### 実装するもの

**SQL Rune（`runes/sql/query.fav`）**

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `Sql.from` | `String -> SqlQuery` | テーブル名からクエリを開始 |
| `Sql.where` | `SqlQuery -> String -> List<String> -> SqlQuery` | WHERE 句を追加 |
| `Sql.select` | `SqlQuery -> String -> SqlQuery` | SELECT 列を指定 |
| `Sql.order_by` | `SqlQuery -> String -> String -> SqlQuery` | ORDER BY 句を追加 |
| `Sql.limit` | `SqlQuery -> Int -> SqlQuery` | LIMIT を設定 |
| `Sql.offset` | `SqlQuery -> Int -> SqlQuery` | OFFSET を設定 |
| `Sql.join` | `SqlQuery -> String -> String -> SqlQuery` | JOIN 句を追加（INNER）|
| `Sql.left_join` | `SqlQuery -> String -> String -> SqlQuery` | LEFT JOIN |
| `Sql.run_raw` | `SqlQuery -> !DbRead -> List<Map<String, String>>` | 実行（未型付き）|
| `Sql.insert_into` | `String -> List<String> -> List<List<String>> -> !DbWrite -> Int` | INSERT |
| `Sql.update` | `String -> List<(String, String)> -> String -> !DbWrite -> Int` | UPDATE |
| `Sql.to_sql` | `SqlQuery -> String` | SQL 文字列へ変換（デバッグ用）|

**SqlQuery 型**

```favnir
type SqlQuery = {
  table:    String
  select_clause: String
  where_clauses: List<String>
  params:   List<String>
  joins:    List<String>
  order_clause: String
  limit_val:   Option<Int>
  offset_val:  Option<Int>
}
```

**stdlib 拡張（Phase A）**

SQL Rune の実装に必要な関数を stdlib に追加する。

| 関数 | 説明 |
|------|------|
| `List.zip` | `List<A> -> List<B> -> List<(A, B)>` |
| `List.flatten` | `List<List<A>> -> List<A>` |
| `List.flat_map` | `List<A> -> (A -> List<B>) -> List<B>` |
| `List.partition` | `List<A> -> (A -> Bool) -> (List<A>, List<A>)` |
| `List.sum_int` | `List<Int> -> Int` |
| `List.min_int` | `List<Int> -> Option<Int>` |
| `List.max_int` | `List<Int> -> Option<Int>` |
| `List.count` | `List<A> -> (A -> Bool) -> Int` |
| `String.split` | `String -> String -> List<String>` |
| `String.trim` | `String -> String` |
| `String.to_upper` | `String -> String` |
| `String.to_lower` | `String -> String` |
| `String.starts_with` | `String -> String -> Bool` |
| `String.ends_with` | `String -> String -> Bool` |
| `String.replace` | `String -> String -> String -> String` |
| `String.pad_left` | `String -> Int -> String -> String` |
| `String.pad_right` | `String -> Int -> String -> String` |
| `Map.merge` | `Map<K, V> -> Map<K, V> -> Map<K, V>` |
| `Map.from_list` | `List<(K, V)> -> Map<K, V>` |
| `Map.map_values` | `Map<K, V> -> (V -> W) -> Map<K, W>` |

### スコープ外（v7.3.0 以降）

- 型パラメータを使った `Sql.run<User>` の完全実装（型システム拡張が必要）
- `Sql.from<User>()` — 型からテーブル名を自動導出
- JOINの型安全な結合（複数型パラメータ）
- サブクエリ
- `GROUP BY` / `HAVING`
- DuckDB SQL Rune 対応（`DuckSql.*`）

## 設計決定

### なぜ v1 は文字列ベースか

Favnir の型パラメータは現在ランタイム型情報を保持しない。
`Sql.run<User>` のような型安全実行には、コンパイラの型-テーブル名マッピングが必要。
これは v7.3.0 以降で `schemas/*.yaml` との統合として実現する。

v7.2.0 は**流暢な API の習慣を作る**ことを優先し、型安全実行は `T.validate` との組み合わせで実現する：

```favnir
bind raw <- Sql.from("users") |> Sql.where("active = ?", ["true"]) |> Sql.run_raw
bind users <- List.map(raw, |row| User.validate(row))
```

### SqlQuery はイミュータブルなビルダ

各 `Sql.*` 関数は新しい `SqlQuery` レコードを返す（純粋関数）。
副作用は最後の `Sql.run_raw` / `Sql.insert_into` / `Sql.update` のみ。

### パラメータバインディング

`Sql.where("active = ?", ["true"])` — `?` プレースホルダと `List<String>` のパラメータリストで SQL インジェクション対策。
内部では `Db.query_raw(sql, params)` に委譲。

## ドキュメント

- `site/content/docs/runes/sql.mdx` — SQL Rune リファレンス（新規）
- `site/content/docs/guides/type-safe-queries.mdx` — 型安全クエリガイド（新規）

## 完了条件

- `runes/sql/query.fav` が `fav check` エラーなし
- `Sql.from("table") |> Sql.where("id = ?", ["1"]) |> Sql.to_sql` が `"SELECT * FROM table WHERE id = ?"` を返す
- stdlib 追加関数のテスト（各 1 件以上）
- SQL Rune の統合テスト 5 件以上
- サイトドキュメント 2 ページ追加
- 既存テスト 1051 件が全件通る
