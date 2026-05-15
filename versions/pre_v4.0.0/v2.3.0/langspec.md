# Favnir v2.3.0 Langspec

更新日: 2026-05-13

## 1. Record destructuring bind

`bind` は record 値を複数のローカルへ分解できる。

```favnir
bind { name, age } <- user
bind { age: user_age } <- user
bind { name, _ } <- user
```

意味:

- `{ name }` は `user.name` を `name` に束縛する pun。
- `{ age: user_age }` は `user.age` を `user_age` に束縛する alias。
- `{ _ }` は残りのフィールドを無視する wildcard。

lowering:

```favnir
bind { x, y } <- point
```

は概ね次に脱糖される。

```favnir
bind $tmp <- point
bind x <- $tmp.x
bind y <- $tmp.y
```

## 2. Record destructuring errors

- `E072`: destructuring bind の右辺が record 型ではない
- `E073`: 指定したフィールドが record 型に存在しない

例:

```favnir
bind { x } <- 42         // E072
bind { x, y } <- point   // point に y がなければ E073
```

## 3. Function return type inference

関数は `= expr` 形式で戻り型を省略できる。

```favnir
fn double(n: Int) = n * 2
fn greet(name: String) = $"Hello {name}!"
fn is_adult(age: Int) = age >= 18
```

明示指定付きの `= expr` 形式も有効。

```favnir
fn id(x: Int) -> Int = x
fn print_hello() -> Unit !Io = IO.println("hello")
```

ルール:

- `=` 形式では本体式の型を戻り型として使う。
- 既存の `fn name(...) -> Ret { ... }` 形式は継続して使える。
- 戻り型省略は `= expr` のときだけ有効で、`{ ... }` 形式では省略できない。

## 4. Return type inference error

- `E074`: 戻り型を推論できない

典型例は戻り型注釈のない再帰関数。

```favnir
fn loop(n: Int) = loop(n)   // E074
```

この場合は明示的な戻り型を付ける。

## 5. Compatibility

- `v2.2.0` までの `bind name <- expr` はそのまま有効。
- `v2.2.0` までの `fn name(...) -> Ret { ... }` もそのまま有効。
- `Ok` / `Err` / `Some` / `None` の pattern 正規化仕様は `v2.2.0` を維持する。
