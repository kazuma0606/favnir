# Favnir v2.7.0 Language Specification

作成日: 2026-05-13

---

## テーマ

Favnir 自身で書いた最初の公式 rune として `validate` を提供する。

v2.7.0 では `runes/validate/validate.fav` を純粋な Favnir で実装し、
v2.6.0 のモジュールシステム（`import rune "validate"`）上で動作させる。
Rust コードは一行も追加しない。

---

## 1. validate rune の API

### 1-1. 型定義

```favnir
// バリデーションエラーを表す型
public type ValidationError = {
    path:    String   // フィールドパス（例: "email"）
    code:    String   // エラーコード（例: "required"）
    message: String   // 人間向けメッセージ
}
```

### 1-2. フィールドレベル検証ステージ

```favnir
// 空文字列を拒否する
public stage Required: String -> Result<String, ValidationError>

// 最小文字数チェック（カリー化: MinLen(3) で stage を返す）
public stage MinLen: Int -> String -> Result<String, ValidationError>

// 最大文字数チェック（カリー化）
public stage MaxLen: Int -> String -> Result<String, ValidationError>

// 簡易メール形式チェック（"@" と "." の存在確認）
public stage Email: String -> Result<String, ValidationError>

// 整数の範囲チェック（カリー化: IntRange(1)(100) で stage を返す）
public stage IntRange: Int -> Int -> Int -> Result<Int, ValidationError>
```

### 1-3. 複数結果の集約

```favnir
// 複数バリデーション結果をまとめて評価する
// すべて Ok なら Ok(value) を返す
// 1つ以上 Err なら Err(errors) を返す（全エラーを収集）
public fn all_pass(
    value:   String,
    results: List<Result<String, ValidationError>>
) -> Result<String, List<ValidationError>>
```

---

## 2. 使い方

### 2-1. 基本使用

```favnir
import rune "validate"

public fn main() -> Unit !Io {
    bind name_result <- validate.Required("Alice")
    bind email_result <- validate.Email("alice@example.com")
    IO.println(Debug.show(name_result))
    IO.println(Debug.show(email_result))
}
```

### 2-2. カリー化ステージの利用

```favnir
import rune "validate"

// MinLen(3) は String -> Result<String, ValidationError> を返す
bind check <- validate.MinLen(3)("hi")
// -> Err(ValidationError { path: ""  code: "min_len"  message: "Minimum length is 3" })
```

### 2-3. 複数チェックの集約

```favnir
import rune "validate"

public fn validate_name(s: String) -> Result<String, List<ValidationError>> = {
    validate.all_pass(s, collect {
        yield validate.Required(s);
        yield validate.MinLen(2)(s);
        yield validate.MaxLen(50)(s);
    })
}
```

### 2-4. パイプラインでの利用

```favnir
import rune "validate"

// Result は chain で短絡評価できる
public stage ValidateName: String -> Result<String, ValidationError> =
    Required |> chain |> MinLen(2) |> chain |> MaxLen(50)
```

---

## 3. validate rune のディレクトリ構成

```
runes/
  fav.toml                    ← rune 開発用プロジェクト設定
  validate/
    validate.fav              ← public API 実装（Rust コードなし）
    validate.test.fav         ← テストスイート
```

### `runes/fav.toml`

```toml
[rune]
name    = "runes"
version = "0.1.0"
src     = "."

[runes]
path = "."
```

- `src = "."` にすることで `runes/validate/validate.fav` が `import "validate/validate"` で参照できる
- `runes.path = "."` にすることで `import rune "validate"` が `./validate/validate.fav` を解決する

---

## 4. テスト実行

```bash
# runes/ ディレクトリ内でテスト実行
cd runes
fav test validate/validate.test.fav

# またはリポジトリルートから
fav test runes/validate/validate.test.fav --project runes/fav.toml
```

`validate.test.fav` は `test "description" { ... }` 構文を使い、
`assert_eq` で期待値を検証する。

---

## 5. ステージの実装詳細

### Required

```favnir
public stage Required: String -> Result<String, ValidationError> = |s| {
    if String.is_empty(s) {
        Result.err(ValidationError {
            path:    ""
            code:    "required"
            message: "Field is required"
        })
    } else {
        Result.ok(s)
    }
}
```

### MinLen

```favnir
public stage MinLen: Int -> String -> Result<String, ValidationError> = |min| |s| {
    if String.length(s) < min {
        Result.err(ValidationError {
            path:    ""
            code:    "min_len"
            message: $"Minimum length is {min}"
        })
    } else {
        Result.ok(s)
    }
}
```

### MaxLen

```favnir
public stage MaxLen: Int -> String -> Result<String, ValidationError> = |max| |s| {
    if String.length(s) > max {
        Result.err(ValidationError {
            path:    ""
            code:    "max_len"
            message: $"Maximum length is {max}"
        })
    } else {
        Result.ok(s)
    }
}
```

### Email

```favnir
// "x@y.z" 形式の簡易チェック（@ と . の有無）
public stage Email: String -> Result<String, ValidationError> = |s| {
    if String.contains(s, "@") && String.contains(s, ".") {
        Result.ok(s)
    } else {
        Result.err(ValidationError {
            path:    ""
            code:    "email"
            message: "Invalid email format"
        })
    }
}
```

### IntRange

```favnir
public stage IntRange: Int -> Int -> Int -> Result<Int, ValidationError> = |min| |max| |n| {
    if n < min || n > max {
        Result.err(ValidationError {
            path:    ""
            code:    "range"
            message: $"Value must be between {min} and {max}"
        })
    } else {
        Result.ok(n)
    }
}
```

### all_pass

```favnir
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

## 6. examples への追加

```
fav/examples/validate_demo/
  fav.toml         ← [runes] path = "../../../../runes"
  src/
    main.fav       ← import rune "validate" のデモ
```

---

## 7. 互換性

- Rust コードは一切追加・変更しない
- 既存のテストに影響しない（rune は独立ファイル）
- v2.6.0 の `import rune "..."` 機能を前提とする
- `fav check` / `fav run` / `fav test` は既存動作のまま
