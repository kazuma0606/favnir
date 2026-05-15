# Favnir v2.7.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.7.0"` に変更。
`src/main.rs` の HELP テキストを `v2.7.0` に更新。

---

## Phase 1 — runes/ ディレクトリ構成

### `runes/fav.toml`（新規）

```toml
[rune]
name    = "runes"
version = "0.1.0"
src     = "."

[runes]
path = "."
```

- `runes/` ディレクトリを Favnir プロジェクトとして扱えるようにする
- `src = "."` — `runes/validate/validate.fav` をモジュールとして解決できる
- `runes.path = "."` — `import rune "validate"` が `./validate/validate.fav` を探す

### `runes/validate/` ディレクトリ

```
runes/validate/
  validate.fav        ← 実装本体
  validate.test.fav   ← テストスイート
```

---

## Phase 2 — `runes/validate/validate.fav` の実装

### 2-1. ValidationError 型

```favnir
public type ValidationError = {
    path:    String
    code:    String
    message: String
}
```

### 2-2. Required ステージ

```favnir
public stage Required: String -> Result<String, ValidationError> = |s| {
    if String.is_empty(s) {
        Result.err(ValidationError {
            path: ""  code: "required"  message: "Field is required"
        })
    } else {
        Result.ok(s)
    }
}
```

### 2-3. MinLen ステージ（カリー化）

```favnir
public stage MinLen: Int -> String -> Result<String, ValidationError> = |min| |s| {
    if String.length(s) < min {
        Result.err(ValidationError {
            path: ""  code: "min_len"  message: $"Minimum length is {min}"
        })
    } else {
        Result.ok(s)
    }
}
```

### 2-4. MaxLen ステージ（カリー化）

```favnir
public stage MaxLen: Int -> String -> Result<String, ValidationError> = |max| |s| {
    if String.length(s) > max {
        Result.err(ValidationError {
            path: ""  code: "max_len"  message: $"Maximum length is {max}"
        })
    } else {
        Result.ok(s)
    }
}
```

### 2-5. Email ステージ

```favnir
// 簡易チェック: "@" と "." の両方を含む
public stage Email: String -> Result<String, ValidationError> = |s| {
    if String.contains(s, "@") && String.contains(s, ".") {
        Result.ok(s)
    } else {
        Result.err(ValidationError {
            path: ""  code: "email"  message: "Invalid email format"
        })
    }
}
```

> **注意**: `&&` 演算子は v2.1.0 で追加済み。利用可能。

### 2-6. IntRange ステージ（カリー化）

```favnir
public stage IntRange: Int -> Int -> Int -> Result<Int, ValidationError> = |min| |max| |n| {
    if n < min || n > max {
        Result.err(ValidationError {
            path: ""  code: "range"  message: $"Value must be between {min} and {max}"
        })
    } else {
        Result.ok(n)
    }
}
```

> **注意**: `||` 演算子は v2.1.0 で追加済み。利用可能。

### 2-7. all_pass ヘルパー関数

```favnir
// 複数バリデーション結果を集約する
// すべて Ok  -> Result.ok(value)
// 1つでも Err -> Result.err(errors: List<ValidationError>)
public fn all_pass(
    value:   String,
    results: List<Result<String, ValidationError>>
) -> Result<String, List<ValidationError>> = {
    bind errors <- List.fold(results, collect { }, |acc, r|
        match r {
            Err(e) => List.concat(acc, collect { yield e; })
            Ok(_)  => acc
        }
    )
    if List.length(errors) == 0 {
        Result.ok(value)
    } else {
        Result.err(errors)
    }
}
```

---

## Phase 3 — `runes/validate/validate.test.fav` の作成

```favnir
// validate.test.fav — validate rune のテストスイート

// Phase 2 の実装をテストするため、同一ファイルで実装を再定義するか、
// または import rune "validate" を使用する
// （runes/fav.toml が存在する場合は import rune "validate" が解決できる）

// --- Required ---
test "Required: 空文字列 -> Err" {
    bind result <- Required("")
    assert_eq(Result.is_err(result), true)
}

test "Required: 非空文字列 -> Ok" {
    bind result <- Required("hello")
    assert_eq(Result.is_ok(result), true)
}

// --- MinLen ---
test "MinLen: 短すぎる -> Err" {
    bind result <- MinLen(3)("hi")
    assert_eq(Result.is_err(result), true)
}

test "MinLen: ちょうど最小 -> Ok" {
    bind result <- MinLen(3)("abc")
    assert_eq(Result.is_ok(result), true)
}

test "MinLen: 十分長い -> Ok" {
    bind result <- MinLen(3)("hello")
    assert_eq(Result.is_ok(result), true)
}

// --- MaxLen ---
test "MaxLen: 長すぎる -> Err" {
    bind result <- MaxLen(3)("toolong")
    assert_eq(Result.is_err(result), true)
}

test "MaxLen: ちょうど最大 -> Ok" {
    bind result <- MaxLen(3)("abc")
    assert_eq(Result.is_ok(result), true)
}

// --- Email ---
test "Email: 正しい形式 -> Ok" {
    bind result <- Email("user@example.com")
    assert_eq(Result.is_ok(result), true)
}

test "Email: @ なし -> Err" {
    bind result <- Email("notanemail")
    assert_eq(Result.is_err(result), true)
}

test "Email: . なし -> Err" {
    bind result <- Email("user@nodot")
    assert_eq(Result.is_err(result), true)
}

// --- IntRange ---
test "IntRange: 範囲内 -> Ok" {
    bind result <- IntRange(1)(100)(50)
    assert_eq(Result.is_ok(result), true)
}

test "IntRange: 範囲外（小さい）-> Err" {
    bind result <- IntRange(1)(100)(0)
    assert_eq(Result.is_err(result), true)
}

test "IntRange: 範囲外（大きい）-> Err" {
    bind result <- IntRange(1)(100)(101)
    assert_eq(Result.is_err(result), true)
}

// --- all_pass ---
test "all_pass: 全 Ok -> Ok" {
    bind r1 <- Required("hello")
    bind r2 <- MinLen(2)("hello")
    bind r3 <- MaxLen(10)("hello")
    bind result <- all_pass("hello", collect { yield r1; yield r2; yield r3; })
    assert_eq(Result.is_ok(result), true)
}

test "all_pass: 1つ Err -> Err（全エラー収集）" {
    bind r1 <- Required("")
    bind r2 <- MinLen(2)("")
    bind result <- all_pass("", collect { yield r1; yield r2; })
    assert_eq(Result.is_err(result), true)
}
```

> `runes/fav.toml` を使って `fav test validate/validate.test.fav` で実行する。
> あるいはテストファイル内に型・ステージの定義を直接記述してスタンドアロンで実行する。

---

## Phase 4 — examples/validate_demo の作成

### `fav/examples/validate_demo/fav.toml`

```toml
[rune]
name    = "validate_demo"
version = "0.1.0"
src     = "src"

[runes]
path = "../../../../runes"
```

### `fav/examples/validate_demo/src/main.fav`

```favnir
// validate rune の利用デモ

import rune "validate"

public fn main() -> Unit !Io {
    // 単一ステージ
    bind r1 <- validate.Required("Alice")
    IO.println($"Required('Alice'): {Debug.show(r1)}")

    bind r2 <- validate.Required("")
    IO.println($"Required(''): {Debug.show(r2)}")

    // カリー化ステージ
    bind r3 <- validate.MinLen(3)("hi")
    IO.println($"MinLen(3)('hi'): {Debug.show(r3)}")

    // メールチェック
    bind r4 <- validate.Email("alice@example.com")
    IO.println($"Email('alice@example.com'): {Debug.show(r4)}")

    bind r5 <- validate.Email("notanemail")
    IO.println($"Email('notanemail'): {Debug.show(r5)}")

    // 集約
    bind r6 <- validate.Required("hi")
    bind r7 <- validate.MinLen(5)("hi")
    bind r8 <- validate.MaxLen(10)("hi")
    bind combined <- validate.all_pass("hi", collect { yield r6; yield r7; yield r8; })
    IO.println($"all_pass: {Debug.show(combined)}")
}
```

---

## Phase 5 — Rust 統合テスト（driver.rs）

### `src/driver.rs`

validate rune の読み込み・実行を確認する Rust レベルのテストを追加する。

```rust
#[test]
fn validate_rune_required_ok() {
    // validate.fav の Required を直接実行し、非空文字列で Ok が返ることを確認
    let source = r#"
        // ...validate rune の Required 定義...
        public fn main() -> Unit !Io {
            bind r <- Required("hello")
            IO.println(Debug.show(r))
        }
    "#;
    // run_source_captures_stdout(source) が "ok(hello)" を含む
}

#[test]
fn validate_rune_required_err() {
    // Required("") で Err が返ることを確認
}

#[test]
fn validate_rune_min_len() {
    // MinLen(3)("hi") で Err、MinLen(3)("abc") で Ok
}

#[test]
fn validate_rune_email() {
    // Email("user@example.com") で Ok、Email("bad") で Err
}

#[test]
fn validate_rune_int_range() {
    // IntRange(1)(100)(50) で Ok、IntRange(1)(100)(0) で Err
}

#[test]
fn validate_rune_all_pass_ok() {
    // 全検証 Ok のとき all_pass が Ok を返す
}

#[test]
fn validate_rune_all_pass_collects_errors() {
    // 複数 Err が all_pass でまとめて返る
}
```

---

## Phase 6 — ドキュメント・最終確認

### `versions/v2.7.0/langspec.md` を作成

- `ValidationError` 型の説明
- 各ステージ（Required / MinLen / MaxLen / Email / IntRange）の説明
- `all_pass` の使い方
- `import rune "validate"` の利用方法
- `runes/fav.toml` の設定方法
- 互換性（Rust コード変更なし）

### 最終確認

- `cargo build` 警告ゼロ
- `cargo test` 全テスト通過（目標 607 → 614 程度）
- `fav test validate/validate.test.fav`（`runes/` から実行）で全テスト通過
- `fav run examples/validate_demo/src/main.fav`（デモ動作確認）

---

## テスト数の見込み

v2.6.0 ベースライン: 607

- driver.rs validate 統合テスト: +7
- 目標: **614**（+7 程度）

> `validate.test.fav` の `.fav` テストは `cargo test` に含まれないが、
> CI で `fav test` を実行して確認する。

---

## 注意点

### `&&` / `||` の利用

`Email` と `IntRange` の実装で `&&` / `||` を使う。
これらは v2.1.0 で追加済みのため、現在のコードベースに存在するはず。
存在しない場合は `if String.contains(s, "@") { if String.contains(s, ".") { ... } }` で代替する。

### カリー化 stage の型チェック

`MinLen: Int -> String -> Result<String, ValidationError>` のような多段カリー化 stage は
チェッカーが `Arrow(Int, Arrow(String, Result<String, ValidationError>))` として扱う。
これは既存の v1.x 以来サポートされているため問題ない。

### `all_pass` のフォールドパターン

`List.fold` の初期値として `collect { }` を使い空リストを作る。
`collect { yield e; }` で単要素リストを作り `List.concat` で連結する。
これが動かない場合は VM に `List.empty: Unit -> List<T>` を追加するか、
driver.rs レベルのテストで別アプローチを取る。

### rune ファイルのパス解決

`import rune "validate"` は `resolve_rune_import_file("validate")` を呼び、
`<runes_dir>/validate/validate.fav` を解決する（v2.6.0 実装済み）。
`runes/fav.toml` の `runes.path = "."` によって `runes/validate/validate.fav` が解決できる。
