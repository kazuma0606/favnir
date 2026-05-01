# Favnir Main and Unit

更新日: 2026-04-26

## 1. `main`

### 基本方針

Favnir では `main` は必要。

ただし、特別すぎる構文にはせず、**普通の `public fn`** として扱うのが自然。

### 推奨形

デフォルト:

```fav
public fn main() -> Unit !Io {
    ...
}
```

必要なら引数も取れる。

```fav
public fn main(args: List<String>) -> Unit !Io {
    ...
}
```

終了コードや結果を返したい場合は、`Unit` 以外も許す。

```fav
public fn main(args: List<String>) -> Int !Io {
    ...
}
```

### 結論

- 基本は `main() -> Unit !Io`
- 必要なら `main() -> T !Io` を許可
- `main` は普通の public 関数として扱う

## 2. `Void` ではなく `Unit`

Favnir では `Void` より `Unit` を使う方が自然。

理由:

- 既存の関数型言語と整合しやすい
- 「値がない」ではなく「値は一つだけある」型として扱える
- 式指向との相性が良い

したがって、返り値なし相当は `Unit` にする。
