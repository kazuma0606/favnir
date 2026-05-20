# Favnir v5.5.0 仕様書 — 標準ライブラリ型シグネチャ補完

作成日: 2026-05-20

---

## 概要

vm.rs に実装済みでありながら checker.rs に型シグネチャが登録されていない関数を補完する。
対象は `List` / `String` / `Option` / `Result` / `Map` の各名前空間。

加えて、vm.rs にも未実装の 3 関数を新規追加する:
- `Map.remove`
- `Map.contains_key`
- `String.from_chars`

これらの補完により、v6.0.0 セルフホスト（レキサー・パーサー・型チェッカーを Favnir で実装）
に必要な全関数が型安全に使えるようになる。

---

## A. List — 型シグネチャ補完

checker.rs に以下の型シグネチャを追加する（vm.rs 実装済み）。

```favnir
List.flat_map(xs: List<T>, f: T -> List<U>) -> List<U>
List.sort(xs: List<T>, cmp: (T, T) -> Int)  -> List<T>
List.find(xs: List<T>, pred: T -> Bool)     -> Option<T>
List.any(xs: List<T>, pred: T -> Bool)      -> Bool
List.all(xs: List<T>, pred: T -> Bool)      -> Bool
List.index_of(xs: List<T>, pred: T -> Bool) -> Option<Int>
List.zip(a: List<A>, b: List<B>)            -> List<{first: A, second: B}>
List.range(start: Int, end: Int)            -> List<Int>
List.reverse(xs: List<T>)                   -> List<T>
List.concat(a: List<T>, b: List<T>)         -> List<T>
List.take(xs: List<T>, n: Int)              -> List<T>
List.drop(xs: List<T>, n: Int)              -> List<T>
```

### 補足

- `List.sort` の比較関数は `cmp(a, b) -> Int`: 負=a<b、0=等、正=a>b
- `List.zip` の戻り値レコードフィールドは `first` と `second`（vm.rs の実装に合わせる）
- `List.find` は条件に合う最初の要素を `Option<T>` で返す
- `List.index_of` は条件に合う最初のインデックスを `Option<Int>` で返す

---

## B. String — 型シグネチャ補完 + 新規追加

### checker.rs への追加（vm.rs 実装済み）

```favnir
String.concat(a: String, b: String)        -> String
String.replace(s: String, from: String, to: String) -> String
String.slice(s: String, start: Int, end: Int)       -> String
String.repeat(s: String, n: Int)           -> String
String.char_at(s: String, i: Int)          -> Option<String>
String.to_int(s: String)                   -> Option<Int>
String.to_float(s: String)                 -> Option<Float>
```

### 新規追加（vm.rs + checker.rs）

```favnir
String.from_chars(chars: List<String>) -> String
```

`List<String>` の各要素を連結して 1 つの String を生成する。`String.chars` の逆操作。

```favnir
// 使用例
let chars = String.chars("hello");          // ["h", "e", "l", "l", "o"]
let s = String.from_chars(chars);           // "hello"

// レキサーでのトークン組み立て
fn take_while_digit(chars: List<String>) -> String {
  let digits = List.take_while(chars, |c| String.contains("0123456789", c));
  String.from_chars(digits)
}
```

### `String.slice` の動作仕様

- `String.slice(s, start, end)`: Unicode スカラー単位での部分文字列
- `start <= end`、`end <= String.length(s)` であること
- 範囲外の場合はランタイムエラー

```favnir
String.slice("hello", 1, 3)  // "el"
String.slice("abcde", 0, 5)  // "abcde"
```

### `String.char_at` の動作仕様

- 0 ベースのインデックスで Unicode スカラーを取得
- 範囲外の場合は `None` を返す

```favnir
String.char_at("hello", 0)  // Some("h")
String.char_at("hello", 10) // None
```

---

## C. Option — コンビネータ型シグネチャ補完

checker.rs に以下を追加する（vm.rs 実装済み）。

```favnir
Option.and_then(opt: Option<T>, f: T -> Option<U>) -> Option<U>
Option.or_else(opt: Option<T>, f: () -> Option<T>) -> Option<T>
Option.is_some(opt: Option<T>)                     -> Bool
Option.is_none(opt: Option<T>)                     -> Bool
Option.to_result(opt: Option<T>, err: E)           -> Result<T, E>
```

### 用途（v6.0.0 パーサーでの典型的な使い方）

```favnir
// and_then でパーサーをチェーン
fn parse_pair(s: String) -> Option<{first: Int, second: Int}> {
  Option.and_then(parse_int(s), |n| {
    Option.map(parse_int(rest), |m| { first: n second: m })
  })
}
```

---

## D. Result — コンビネータ型シグネチャ補完

checker.rs に以下を追加する（vm.rs 実装済み）。

```favnir
Result.and_then(res: Result<T, E>, f: T -> Result<U, E>) -> Result<U, E>
Result.map_err(res: Result<T, E>, f: E -> F)             -> Result<T, F>
Result.is_ok(res: Result<T, E>)                          -> Bool
Result.is_err(res: Result<T, E>)                         -> Bool
Result.to_option(res: Result<T, E>)                      -> Option<T>
```

### 用途（v6.0.0 型チェッカーでの典型的な使い方）

```favnir
// エラーを変換しながらチェーン
fn check_and_compile(src: String) -> Result<List<Int>, String> {
  Result.and_then(parse(src), |ast|
    Result.and_then(type_check(ast), |typed|
      Result.map_err(compile(typed), |e| $"compile error: {e}")
    )
  )
}
```

---

## E. Map — 型シグネチャ強化 + 新規追加

### checker.rs の既存 Map 節を強化（型 Unknown → 具体型）

```favnir
// 既存（Unknown → 具体型に変更）
Map.keys(m: Map<K, V>)   -> List<K>     // 既存だが K=Unknown → K に改善
Map.values(m: Map<K, V>) -> List<V>     // 同上

// checker.rs の catch-all で Unknown を返していた関数を明示
Map.size(m: Map<K, V>)   -> Int
Map.is_empty(m: Map<K, V>) -> Bool
Map.map_values(m: Map<K, V>, f: V -> U) -> Map<K, U>
Map.filter_values(m: Map<K, V>, pred: V -> Bool) -> Map<K, V>
Map.merge(base: Map<K, V>, overrides: Map<K, V>) -> Map<K, V>
Map.to_list(m: Map<K, V>) -> List<{first: K, second: V}>
Map.from_list(pairs: List<{first: String, second: V}>) -> Map<String, V>
```

### 新規追加（vm.rs + checker.rs）

```favnir
Map.remove(m: Map<K, V>, key: K)     -> Map<K, V>
Map.contains_key(m: Map<K, V>, key: K) -> Bool
```

```favnir
// 使用例（型チェッカーのスコープ管理）
fn exit_scope(env: Map<String, String>, names: List<String>) -> Map<String, String> {
  List.fold(names, env, |acc, name| Map.remove(acc, name))
}

fn is_defined(env: Map<String, String>, name: String) -> Bool {
  Map.contains_key(env, name)
}
```

---

## F. 型シグネチャの方針

### 型変数の扱い

checker.rs は `Type::Unknown` で型変数を近似する（完全な HM 型推論は v6 以降）。
例えば `List.flat_map` の戻り型は、クロージャの返り型から推論する。

```rust
// checker.rs での実装イメージ
("List", "flat_map") => {
    let _elem = self.expect_list_arg(&arg_tys, 0, span);
    let out = if let Some(f_ty) = arg_tys.get(1) {
        f_ty.as_callable()
            .map(|(_, o)| match o {
                Type::List(inner) => *inner,
                _ => Type::Unknown,
            })
            .unwrap_or(Type::Unknown)
    } else {
        Type::Unknown
    };
    Some(Type::List(Box::new(out)))
}
```

### エフェクトなし

追加する全関数はエフェクトなし（純粋関数）。
`!Io` / `!AWS` 等のエフェクトチェックは不要。

---

## 完了条件

- `cargo test` 全件通過（971 件 + 新規テスト）
- `List.flat_map` / `List.sort` / `List.zip` 等が型エラーなしで使える
- `Option.and_then` / `Result.and_then` が正しい型を推論する
- `Map.remove` / `Map.contains_key` / `String.from_chars` が実装済みで型付き
- 新規関数の単体テストが存在する
