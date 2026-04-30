# Favnir Pattern Binding Draft

更新日: 2026-04-26

## 結論

Favnir では、「分割代入」ではなく **pattern binding** を中核に置く方がよい。

つまり:

```fav
bind { x, _ } <- response
```

のように、`bind <-` の中で分解束縛を行う。

`{ x, _ } = response` のような形は採らない。

理由:

- `=` を代入に見せたくない
- `bind <-` の思想を崩したくない
- `match` と同じ pattern を共有できる

## 基本方針

pattern は次の場所で共通に使えるようにするのが望ましい。

1. `bind pattern <- expr`
2. `match value { pattern => ... }`
3. 関数引数

これにより、分解規則を 1 つにまとめられる。

## 例

### record pattern

```fav
bind { name, email } <- user
```

### wildcard を含む pattern

```fav
bind { x, _ } <- response
```

### variant pattern

```fav
bind ok(value) <- parse_user(row)
bind err(message) <- result
```

### tuple / list pattern の候補

```fav
bind (x, y) <- point
bind [head, tail] <- items
```

## `match` との共有

`bind` と `match` は、できるだけ同じ pattern 体系を共有した方がよい。

```fav
match parse_user(row) {
    ok(user) => ...
    err(message) => ...
}
```

これと

```fav
bind ok(user) <- parse_user(row)
```

が同じ pattern ルールで動くのが理想。

## 関数引数 destructuring

将来的には、関数引数でも pattern を使えると強い。

```fav
fn format_user({ name, email }: User) -> String {
    ...
}
```

ただし、これは `bind` / `match` が固まった後でもよい。

## `_` の二重の役割

Favnir では `_` が 2 つの役割を持つ可能性がある。

### 1. pattern 内の `_`

これは wildcard。

```fav
bind { x, _ } <- response
```

意味:

- その位置の値は無視する

### 2. 式文脈の `_`

これは placeholder。

```fav
rows |> map(_.email)
numbers |> filter(_ > 0)
```

意味:

- 単一引数関数の暗黙引数

## 両立方針

この 2 つは文脈で区別する。

- pattern 文脈の `_` は wildcard
- 関数を期待する式文脈の `_` は placeholder

この分離は一般的であり、Favnir でも十分成立する。

## 最初に入れる pattern

初期仕様では、次を優先するのがよい。

### 1. record pattern

```fav
bind { name, email } <- user
```

### 2. wildcard

```fav
bind { x, _ } <- response
```

### 3. variant pattern

```fav
bind ok(value) <- parse_user(row)
```

## 後で検討する pattern

次は後回しでもよい。

- nested pattern
- tuple/list pattern の拡張
- pattern guard
- rest pattern
- alias pattern

理由:

- 最初は record / variant / wildcard だけでもかなり強い
- parser / checker を軽く保てる

## TypeScript っぽさとの関係

`{ x, y }` のような record destructuring は、TS/JS 的な読みやすさを持っている。  
Favnir でもこの感覚は活かせる。

ただし、Favnir では「代入」ではなく「束縛」に統一する。

なので:

- TS/JS 風の見た目は活かす
- でも構文は `bind pattern <- expr` に寄せる

これが一番筋がよい。

## 仮の結論

Favnir では、分割代入を別物として導入するより、

- `bind pattern <- expr`
- `match` と同じ pattern 体系
- wildcard と placeholder の文脈分離

で整理する方がきれい。

特に最初は:

- record pattern
- wildcard
- variant pattern

を先に入れるのがよい。
