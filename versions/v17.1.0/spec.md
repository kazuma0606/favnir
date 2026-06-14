# v17.1.0 Spec — 境界付きジェネリクス（Bounded Generics）

Date: 2026-06-14

---

## 概要

`fn f<T with Interface>(...)` 構文で型パラメータに制約（bound）を付けられるようにする。
現在のジェネリクス（`fn f<T>(x: T) -> T`）は制約がなく `T` に対して何もできない。
`T with Ord` 制約を付けることで `a < b` のような演算が可能になる。

---

## 構文

### 単一制約

```fav
fn max<T with Ord>(a: T, b: T) -> T {
  if a > b { a } else { b }
}

fn serialize<T with Serialize>(val: T) -> String {
  Json.stringify_raw(val)
}
```

### 複数制約（`with` を連ねる）

```fav
fn sort_and_serialize<T with Ord with Serialize>(items: List<T>) -> List<String> {
  List.map(List.sort_by(items, |x| x), |x| serialize(x))
}
```

### stage にも適用

```fav
stage Rank<T with Ord>(rows: List<T>) -> List<T> {
  Result.ok(List.sort_by(rows, |x| x))
}
```

### カスタム interface との組み合わせ

```fav
interface Scored {
  fn score(self) -> Float
}

fn top_n<T with Scored with Ord>(items: List<T>, n: Int) -> List<T> {
  let ranked = List.sort_by_desc(items, |x| x.score())
  List.take(ranked, n)
}
```

---

## 組み込み Interface（自動実装）

| Interface | 意味 | 自動実装される型 |
|---|---|---|
| `Ord` | 順序比較（`<` `>` `<=` `>=`） | Int / Float / String |
| `Eq` | 等値比較（`==` `!=`） | 全プリミティブ型 + レコード |
| `Serialize` | JSON シリアライズ | 全レコード型（フィールドが全て Serialize を満たす場合） |
| `Display` | 文字列表現（f-string 補間） | String / Int / Float / Bool |
| `Hash` | ハッシュ値計算 | Int / String |
| `Clone` | 値の複製 | 全値型（デフォルト） |

---

## エラーコード

- **E0325**: `型名 does not implement Interface名`
  - 例: `max(Row { x: 1 }, Row { x: 2 })` → `E0325: Row does not implement Ord`

---

## 実装ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `GenericParam { name: String, bounds: Vec<String> }` 追加、`Vec<String>` → `Vec<GenericParam>` |
| `fav/src/frontend/parser.rs` | `parse_generic_params` で `<T with Ord with Serialize>` を解析、`with` キーワード追加 |
| `fav/src/middle/checker.rs` | `check_bounded_call`、組み込み bound テーブル、E0325 追加 |
| `self/checker.fav` | `check_bounded_generics` 関数を Favnir 実装に追加 |
| `fav/src/driver.rs` | `v171000_tests` モジュール（5 件） |
| `site/content/docs/language/generics.mdx` | 境界付きジェネリクスガイド（新規作成） |

---

## テスト（v171000_tests）

| # | テスト名 | 内容 |
|---|---|---|
| 1 | `version_is_17_1_0` | `Cargo.toml` に `"17.1.0"` が含まれる |
| 2 | `bounded_generic_ord` | `max<T with Ord>` が Int / Float / String で動作 |
| 3 | `bounded_generic_serialize` | `serialize<T with Serialize>` が全レコード型で動作 |
| 4 | `bounded_generic_violation` | `Ord` を満たさない型で E0325 が出る |
| 5 | `bounded_generic_multi` | 複数 bound（`T with Ord with Serialize`）が動作 |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `fn max<T with Ord>(a: T, b: T) -> T` が Int / Float / String で動作する | [ ] |
| `fn f<T with Serialize>(v: T)` が全レコード型で動作する | [ ] |
| `T with Ord with Serialize` の複数制約が動作する | [ ] |
| bound を満たさない型を渡すと E0325 でエラーになる | [ ] |
| カスタム `interface` との組み合わせが動作する | [ ] |
| `cargo test v171000` → 5/5 PASS | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
