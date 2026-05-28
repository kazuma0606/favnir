# Favnir v7.4.0 Plan

Date: 2026-05-27
Theme: stdlib 高レベル層（Favnir 化）+ email Rune

---

## 実装順序

```
Phase A-1: VM primitive 追加（Map.empty, String.compare）
Phase A-2: runes/stdlib/list.fav — List 高レベル操作
Phase A-3: runes/stdlib/map.fav — Map 高レベル操作
Phase B:   runes/email/ — email Rune（Email.send_raw VM + Favnir 層）
Phase C:   テスト（driver.rs）
Phase D:   ドキュメント
Phase E:   最終確認
```

---

## Phase A-1: VM primitive 追加（vm.rs / checker.rs / compiler.rs）

### Map.empty

```rust
"Map.empty" => Ok(VMValue::Record(std::collections::HashMap::new()))
```

checker.rs: `("Map", "empty") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)))`

### String.compare

`List.sort_by` の実装に必要。辞書順比較で `Int` を返す（負/0/正）。

```rust
"String.compare" => {
    // returns -1, 0, or 1
    match (a.cmp(b)) { Less => -1, Equal => 0, Greater => 1 }
}
```

checker.rs: `("String", "compare") => Some(Type::Int)`

---

## Phase A-2: runes/stdlib/list.fav

### List.group_by

```favnir
public fn group_by(xs: List<A>, key_fn: A -> String) -> Map<String, List<A>>
```

実装:
```favnir
public fn group_by(xs: List<A>, key_fn: A -> String) -> Map<String, List<A>> {
    List.fold_left(xs, Map.empty(), |acc, x|
        bind k <- key_fn(x)
        match Map.get(acc, k) {
            None     => Map.set(acc, k, List.singleton(x))
            Some(vs) => Map.set(acc, k, List.concat(vs, List.singleton(x)))
        })
}
```

### List.zip_with

```favnir
public fn zip_with(xs: List<A>, ys: List<B>, f: (A, B) -> C) -> List<C>
```

実装（`List.zip` が `{ first: A, second: B }` を返すことを利用）:
```favnir
public fn zip_with(xs: List<A>, ys: List<B>, f: A -> B -> C) -> List<C> {
    bind pairs <- List.zip(xs, ys)
    List.map(pairs, |pair| f(pair.first, pair.second))
}
```

> Favnir は多引数クロージャを持たないため、f は `A -> B -> C`（カリー化）形式を想定。
> あるいは pair レコードを受け取る `{ first: A, second: B } -> C` とする。
> 実装時に型チェッカーの挙動を確認しながら調整する。

### List.sort_by

```favnir
public fn sort_by(xs: List<A>, key_fn: A -> String) -> List<A>
```

実装（`List.sort` の comparator に key_fn を使う）:
```favnir
public fn sort_by(xs: List<A>, key_fn: A -> String) -> List<A> {
    List.sort(xs, |a, b| String.compare(key_fn(a), key_fn(b)))
}
```

### List.intersperse

```favnir
public fn intersperse(xs: List<A>, sep: A) -> List<A>
```

実装（fold で acc が空なら push、それ以外は sep → x の順で concat）:
```favnir
fn intersperse_step(acc: List<A>, x: A, sep: A) -> List<A> {
    if List.is_empty(acc) {
        List.singleton(x)
    } else {
        List.concat(List.concat(acc, List.singleton(sep)), List.singleton(x))
    }
}
public fn intersperse(xs: List<A>, sep: A) -> List<A> {
    List.fold_left(xs, List.empty(), |acc, x| intersperse_step(acc, x, sep))
}
```

### List.tail / List.head

```favnir
public fn tail(xs: List<A>) -> List<A>   { List.drop(xs, 1) }
public fn head(xs: List<A>) -> Option<A> { List.first(xs) }
```

---

## Phase A-3: runes/stdlib/map.fav

Map は vm.rs にほぼ揃っているため、追加は薄い:

### Map.empty のラッパー

```favnir
public fn empty() -> Map<String, A> { Map.empty() }
```

### Map.of_list（`Map.from_list` の別名 + 便利ラッパー）

`Map.from_list` はすでに vm.rs にあるが、checker に型シグネチャが登録されているか確認。
されていなければ追加する。

### Map.update

既存値を関数で変換するか、なければデフォルト値を設定:
```favnir
public fn update(m: Map<String, A>, key: String, f: Option<A> -> A) -> Map<String, A> {
    Map.set(m, key, f(Map.get(m, key)))
}
```

---

## Phase B: email Rune

### B-1: VM primitive（vm.rs）

SES の `SendEmail` API を `aws_post` ヘルパー経由で呼ぶ。

```rust
"Email.send_raw" => {
    // args: from, to, subject, body (plain text)
    // POST https://email.<region>.amazonaws.com/ with form-encoded params
    // Action=SendEmail&Source=<from>&Destination.ToAddresses.member.1=<to>
    //   &Message.Subject.Data=<subject>&Message.Body.Text.Data=<body>
}
```

`!Email` を BUILTIN_EFFECTS と compiler.rs 両リストに追加。

### B-2: runes/email/email.fav

```favnir
// シンプル送信
public fn send(from, to, subject, body) -> Result<Unit, String> !Email

// 複数宛先（List<String>）
public fn send_multi(from, to_list, subject, body) -> Result<Int, String> !Email

// HTML ボディビルダー（純粋）
public fn build_html_body(title: String, content: String) -> String
```

`send_multi` は `List.fold_left` で各宛先に `send` を呼ぶ。

---

## Phase C: テスト（driver.rs）

### stdlib_list_tests

- `list_group_by_test` — `["a","b","a"]` を group_by identity で `{"a":[...], "b":[...]}` に変換
- `list_zip_with_test` — `[1,2,3]` と `[10,20,30]` を加算 → `[11,22,33]`
- `list_sort_by_test` — 文字列キーでソート
- `list_intersperse_test` — `[1,2,3]` に 0 を挿入 → `[1,0,2,0,3]`
- `list_tail_test` — `[1,2,3]` の tail → length 2

### email_rune_tests

HTTP 呼び出しは不要な純粋関数のみテスト:
- `email_build_html_body_test` — `<title>` タグを含む HTML 文字列を返す
- `email_fav_check_test` — `fav check runes/email/email.fav` passes

---

## Phase D: ドキュメント

- `site/content/docs/stdlib/list.mdx` — group_by / zip_with / sort_by / intersperse / tail / head
- `site/content/docs/runes/email.mdx` — email Rune リファレンス

---

## 注意点

- **`bind inside closure` 制約**: `group_by` の実装で `key_fn(x)` の結果を bind するのは OK（closure の外）
- **`List.zip` の戻り値型**: `List<{ first: A, second: B }>` なので `pair.first` / `pair.second` でアクセス
- **`Map.empty()` は vm.rs に追加必要**: `List.fold_left` 初期値として使う
- **SES リージョン**: `AWS_REGION` 環境変数から取得（既存の `get_aws_config()` を使う）
- **`String.compare` は vm.rs に追加必要**: `sort_by` の実装に必須
