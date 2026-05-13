# Favnir v2.7.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.7.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.7.0` に更新

---

## Phase 1 — runes/ ディレクトリ構成

- [x] `runes/fav.toml` を作成
  - [x] `[rune] name = "runes"`, `version = "0.1.0"`, `src = "."`
  - [x] `[runes] path = "."`
- [x] `runes/validate/` ディレクトリを作成

---

## Phase 2 — `runes/validate/validate.fav` の実装

- [x] `public type ValidationError = { path: String  code: String  message: String }` を定義
- [x] `public stage Required: String -> String!` を実装
  - [x] `String.is_empty(s)` が true なら `Result.err(ValidationError { ... })` を返す
  - [x] 非空なら `Result.ok(s)` を返す
- [x] `public fn MinLen(min: Int)` を実装（カリー化、fn で実装）
  - [x] `String.length(s) < min` なら Err
  - [x] エラーメッセージに `$"Minimum length is {min}"` を使用
- [x] `public fn MaxLen(max: Int)` を実装（カリー化、fn で実装）
  - [x] `String.length(s) > max` なら Err
  - [x] エラーメッセージに `$"Maximum length is {max}"` を使用
- [x] `public stage Email: String -> String!` を実装
  - [x] `String.contains(s, "@") && String.contains(s, ".")` が true なら Ok
  - [x] いずれかが false なら Err
- [x] `public fn IntRange(min: Int)` を実装（カリー化、fn で実装）
  - [x] `n < min || n > max` なら Err
  - [x] エラーメッセージに `$"Value must be between {min} and {max}"` を使用
- [x] `public fn all_pass(value: String)` を実装
  - [x] `List.flat_map` + `|> match { Err(e) => ... Ok(_) => ... }` でエラーを収集
  - [x] エラーなし → `Result.ok(value)`
  - [x] エラーあり → `Result.err(errors)`（`errors: List<ValidationError>`）

---

## Phase 3 — `runes/validate/validate.test.fav` の作成

テストファイルは型・ステージを直接定義するスタンドアロン形式で実装。

### Required のテスト

- [x] `test "Required: empty -> Err"`: `Required("")` が Err を返す
- [x] `test "Required: non-empty -> Ok"`: `Required("hello")` が Ok を返す

### MinLen のテスト

- [x] `test "MinLen: shorter -> Err"`: `MinLen(3)("hi")` が Err を返す
- [x] `test "MinLen: equal -> Ok"`: `MinLen(3)("abc")` が Ok を返す
- [x] `test "MinLen: longer -> Ok"`: `MinLen(3)("hello")` が Ok を返す

### MaxLen のテスト

- [x] `test "MaxLen: longer -> Err"`: `MaxLen(3)("toolong")` が Err を返す
- [x] `test "MaxLen: equal -> Ok"`: `MaxLen(3)("abc")` が Ok を返す

### Email のテスト

- [x] `test "Email: valid -> Ok"`: `Email("user@example.com")` が Ok を返す
- [x] `test "Email: missing at -> Err"`: `Email("notanemail")` が Err を返す
- [x] `test "Email: missing dot -> Err"`: `Email("user@nodot")` が Err を返す

### IntRange のテスト

- [x] `test "IntRange: in range -> Ok"`: `IntRange(1)(100)(50)` が Ok を返す
- [x] `test "IntRange: below min -> Err"`: `IntRange(1)(100)(0)` が Err を返す
- [x] `test "IntRange: above max -> Err"`: `IntRange(1)(100)(101)` が Err を返す

### all_pass のテスト

- [x] `test "all_pass: all Ok -> Ok"`: Required + MinLen + MaxLen 全 Ok のとき Ok
- [x] `test "all_pass: errors -> Err"`: 複数 Err のとき Err(errors) を返す

---

## Phase 4 — validate_demo サンプルの作成

- [x] `fav/examples/validate_demo/fav.toml` を作成
  - [x] `[rune] name = "validate_demo"`, `src = "src"`
  - [x] `[runes] path = "../../../runes"`（正しい相対パス）
- [x] `fav/examples/validate_demo/src/main.fav` を作成
  - [x] `import rune "validate"` でインポート
  - [x] `validate.Required`, `validate.MinLen`, `validate.Email` のデモ
  - [x] `validate.all_pass` の集約デモを追加（修正済み）

---

## Phase 5 — Rust 統合テスト（src/driver.rs）

- [x] `validate_rune_required_ok`: `Required("hello")` が Ok("hello") を返す
- [x] `validate_rune_required_err`: `Required("")` が Err(ValidationError{code:"required"...}) を返す
- [x] `validate_rune_min_len_err`: `MinLen(3)("hi")` が Err を返す
- [x] `validate_rune_min_len_ok`: `MinLen(3)("abc")` が Ok を返す
- [x] `validate_rune_email_ok`: `Email("user@example.com")` が Ok を返す
- [x] `validate_rune_email_err`: `Email("notanemail")` が Err を返す
- [x] `validate_rune_int_range_ok`: `IntRange(1)(100)(50)` が Ok を返す
- [x] `validate_rune_int_range_err`: `IntRange(1)(100)(0)` が Err を返す
- [x] `validate_rune_all_pass_ok`: 全 Ok のとき all_pass が Ok("hello") を返す
- [x] `validate_rune_all_pass_collects_errors`: Err を含むとき all_pass が Err を返す

---

## Phase 6 — ドキュメント・最終確認

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.6.0 の 607 → 617）

### ドキュメント作成

- [x] `versions/v2.7.0/langspec.md` を作成

---

## 完了条件チェック

- [x] `validate.fav` に Rust コードが一行もない
- [x] `Required("")` が `Err(ValidationError { code: "required" ... })` を返す
- [x] `Required("hello")` が `Ok("hello")` を返す
- [x] `MinLen(3)("hi")` が Err を返す
- [x] `MinLen(3)("abc")` が Ok を返す
- [x] `MaxLen(3)("toolong")` が Err を返す
- [x] `Email("user@example.com")` が Ok を返す
- [x] `Email("notanemail")` が Err を返す
- [x] `IntRange(1)(100)(50)` が Ok を返す
- [x] `IntRange(1)(100)(0)` が Err を返す
- [x] `all_pass` が全 Ok のとき Ok を返す
- [x] `all_pass` が Err を含むとき全エラーを収集して Err を返す（List.flat_map で実装）
- [x] `import rune "validate"` でユーザーコードから使える
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.7.0"`
- [x] `versions/v2.7.0/langspec.md` 作成済み
