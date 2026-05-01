# Favnir Async Design Notes

更新日: 2026-04-26

`async/await` は v1.0.0 スコープ外。post-v1 で検討する。

## 1. `async`

### 基本方針

Favnir では、`await` だけでなく **`async` も必要**。

理由:

- 非同期境界をシグネチャで明示したい
- effect と一貫して扱いたい
- `await` だけだと、どこが非同期か見えにくい

### 対象

- `async fn`
- `async trf`

例:

```fav
public async fn main() -> Unit !Io {
    ...
}
```

```fav
async trf FetchUsers: Url -> List<User> !Network = |url| {
    ...
}
```

## 2. `await`

### 基本方針

`await` は型付き操作として扱うのがよい。

例:

```fav
bind text <- await IO.read_file("users.csv")
```

### 内部意味

Favnir では内部的に `Future<T>` を持つのが自然。

イメージ:

```text
await : Future<T> -> T
```

つまり:

- `async` は定義側の宣言
- `await` は使用側の解除操作

## 3. 内部モデル

表面上は:

```fav
async fn load_users(path: String) -> List<User> !Io
```

でも、内部的には:

```text
load_users : String -> Future<List<User>> !Io
```

のように扱える。

同様に:

```fav
async trf FetchUsers: Url -> List<User> !Network
```

は内部的には:

```text
FetchUsers : Url -> Future<List<User>> !Network
```

のような意味を持つ。

## 4. `await?` は後回し

Forge では `await?` のような形があったが、Favnir の初期仕様では急がなくてよい。

まずは:

- `await expr`
- `T!` + `match`

で十分。

例:

```fav
bind res <- await Http.get(url)
match res {
    ok(value) => ...
    err(message) => ...
}
```

`await?` は後で sugar として検討すればよい。

## 5. `async` と effect

非同期は effect と矛盾しない。
むしろ effect と一緒に扱う方が Favnir らしい。

例:

- `async fn ... -> T !Io`
- `async trf ... -> T !Network`

つまり:

- `async` は execution model
- `!Io`, `!Db`, `!Network` は effect

として分けて考える。

## 6. 設計の利点

- 非同期境界がシグネチャで見える
- `await` の使用可能位置を checker が制限できる
- effect と非同期を同時に整理できる
- 将来 `Future<T?>` や `Future<T!>` のような組み合わせにも伸ばしやすい
