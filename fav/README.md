# fav — Favnir interpreter

**Favnir** は型付きパイプラインとエフェクトを中心に設計された関数型データ言語です。
`fav` はそのリファレンス実装（ツリーウォーキングインタープリタ）です。

## インストール

```bash
cargo build --release
# バイナリは target/release/fav に生成されます
```

## 使い方

```
fav run <file>    — 型チェック後にプログラムを実行
fav check <file>  — 型チェックのみ（実行しない）
fav help          — ヘルプを表示
```

### 実行例

```bash
fav run examples/hello.fav
fav run examples/pipeline.fav
fav run examples/adt_match.fav
fav check examples/hello.fav
```

## 言語概要

### エントリポイント

```favnir
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
```

### 型定義

```favnir
// レコード型
type User = { name: String  email: String }

// 代数的データ型 (ADT)
type Result = | ok(Int) | err(String)
```

### 関数定義 (`fn`)

```favnir
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

### トランスフォーム定義 (`trf`)

```favnir
trf Double: Int -> Int = |n| { n + n }
trf Inc:    Int -> Int = |n| { n + 1 }
```

### フロー定義 (`flw`)

```favnir
flw DoubleInc = Double |> Inc
```

### パイプライン式 (`|>`)

```favnir
fn process(n: Int) -> Int {
    n |> Double |> Inc
}
```

### バインド束縛 (`bind <-`)

```favnir
fn f() -> Int {
    bind x <- 10;
    bind y <- 20;
    x + y
}
```

### パターンマッチ

```favnir
match value {
    ok(v)  => v
    err(_) => 0
}
```

### エフェクト注釈

v0.1.0 では `Pure` と `Io` の 2 種類をサポートしています。

```favnir
trf Greet: String -> Unit !Io = |name| {
    IO.println(name)
}
```

## 組み込み関数

| 名前空間 | 関数 |
|----------|------|
| `IO`     | `print`, `println` |
| `List`   | `map`, `filter`, `fold`, `length`, `is_empty`, `first`, `last` |
| `String` | `trim`, `lower`, `upper`, `split`, `length`, `is_empty` |
| `Option` | `some`, `none`, `map`, `unwrap_or` |
| `Result` | `ok`, `err`, `map`, `unwrap_or` |

## エラーコード

| コード | 内容 |
|--------|------|
| E001   | 型不一致 |
| E002   | 未定義の識別子 |
| E003   | パイプライン / flw の接続型エラー |
| E004   | エフェクト違反 |
| E005   | 引数の個数不一致 |
| E006   | パターンマッチエラー |
