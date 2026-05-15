# Favnir v2.7.0 Language Specification

更新日: 2026-05-13

---

## 概要

v2.7.0 では、Favnir 自身で記述した最初の公式 rune として `validate` を追加する。

- production Rust の新規 builtin は追加しない
- `import rune "validate"` を使って pure Favnir 実装を読み込む
- rune 本体、`.fav` テスト、利用デモを同梱する

---

## 1. 配置

```text
runes/
  fav.toml
  validate/
    validate.fav
    validate.test.fav

fav/examples/
  validate_demo/
    fav.toml
    src/main.fav
```

### `runes/fav.toml`

```toml
[rune]
name = "runes"
version = "0.1.0"
src = "."

[runes]
path = "."
```

- `src = "."` により `runes/validate/validate.fav` を project source として扱える
- `runes.path = "."` により `import rune "validate"` が `./validate/validate.fav` を解決する

---

## 2. validate rune API

### 2-1. ValidationError

```favnir
public type ValidationError = {
    path: String
    code: String
    message: String
}
```

各 validator は `Err(...)` 側にこの shape の record を入れる。

### 2-2. 単項 validator

```favnir
public stage Required: String -> String!
public stage Email: String -> String!
```

- `Required("")` は required error を返す
- `Email(s)` は `"@"` と `"."` の両方を含むかで判定する

### 2-3. カリー化 validator

```favnir
public fn MinLen(min: Int) = |s| ...
public fn MaxLen(max: Int) = |s| ...
public fn IntRange(min: Int) = |max| |n| ...
```

呼び出し例:

```favnir
validate.MinLen(3)("abc")
validate.MaxLen(10)("hello")
validate.IntRange(1)(100)(50)
```

### 2-4. all_pass

```favnir
public fn all_pass(value: String) = |results| ...
```

呼び出し例:

```favnir
bind results <- collect {
    yield validate.Required(name);
    yield validate.MinLen(2)(name);
    yield validate.MaxLen(50)(name);
}
bind checked <- validate.all_pass(name)(results)
```

- 全結果に `err(` が含まれなければ `Ok(value)` を返す
- 1 件でも失敗があれば `Err(...)` を返す

---

## 3. 実装方針

`validate.fav` は既存 stdlib と文法だけで書かれる。

主に使う要素:

- `String.is_empty`
- `String.length`
- `String.contains`
- `Result.ok`
- `Result.err`
- `Debug.show`
- `&&` / `||`

---

## 4. テスト

### 4-1. `.fav` テスト

`runes/validate/validate.test.fav` に 15 件のテストを置く。

- Required: 2 件
- MinLen: 3 件
- MaxLen: 2 件
- Email: 3 件
- IntRange: 3 件
- all_pass: 2 件

### 4-2. Rust 統合テスト

`fav/src/driver.rs` に 10 件の統合テストを追加する。

- Required ok / err
- MinLen ok / err
- Email ok / err
- IntRange ok / err
- all_pass ok / err

---

## 5. デモ

`fav/examples/validate_demo/` は `import rune "validate"` の最小デモである。

```toml
[runes]
path = "../../../runes"
```

デモは `Required` / `MinLen` / `Email` の返り値を `Debug.show` で表示する。

---

## 6. 互換性

- v2.6.0 の `import rune "..."` をそのまま使う
- 既存の `fav check` / `fav run` / `fav test` は維持される
- 版番号は `v2.7.0` に更新される
