# Favnir v1.2.0 タスク一覧 — `invariant` + `std.states` ルーン

作成日: 2026-05-06

> **ゴール**: 型にビジネスルールを埋め込み、`TypeName.new()` で invariant を自動チェックする
>
> **前提**: v1.1.0 完了
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.2.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.2.0` に更新
- [x] `cargo build` が通ること

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

- [x] `TypeDef` に `invariants: Vec<Expr>` フィールドを追加
- [x] `TypeDef` の全コンストラクタ・デフォルト値で `invariants: vec![]` を初期化
- [x] `TypeDef` を参照している全箇所（compiler.rs, checker.rs 等）でコンパイルエラーがないこと

### 1-2: lexer.rs の変更

- [x] `Token::Invariant` バリアントを追加
- [x] キーワードマップに `"invariant" => Token::Invariant` を追加
- [x] 既存テストが全通過すること

### 1-3: parser.rs の変更

- [x] `parse_type_def` でフィールドパース後に `Token::Invariant` を検出する
  - [x] `invariant <expr>` のループ処理を追加（複数 invariant に対応）
  - [x] `<expr>` は既存の `parse_expr` で処理
  - [x] 各 invariant 式を `TypeDef.invariants` に格納
- [x] パーサーテスト: `type PosInt = { value: Int  invariant value > 0 }` がパースできる
- [x] パーサーテスト: 複数 invariant が `TypeDef.invariants` に複数格納される
- [x] パーサーテスト: `invariant String.contains(value, "@")` がパースできる
- [x] `cargo test` が全通過すること

---

## Phase 2: 型検査統合

### 2-1: E045 エラーコードの定義

- [x] E045「invariant 式が Bool 型でない」のエラーメッセージを定義
- [x] 既存のエラーコード一覧（README または langspec）への追記は Phase 7 で行う

### 2-2: `check_type_def` の拡張

- [x] `check_type_def` 内で `TypeBody::Record(fields)` からフィールドを取り出してローカルスコープに追加する処理を実装
- [x] `TypeBody::Sum` に `invariant` が付いている場合はエラーとする（v1.2.0 は Record 型のみ対応）
  - 注: `invariant` は `{}` ブロック内でのみ解析されるため、Sum 型（`| Variant` 構文）はパーサーレベルで `invariants: vec![]` が固定され、構文的に書けない。明示的なエラーコード（E047）の発行は v2.0.0 セルフホスト移植時に対応予定（roadmap-v2.md 参照）。
- [x] 各 `invariant` 式を `check_expr` で型検査するループを追加
- [x] `Bool` 以外の型が返ったとき E045 を出す
- [x] フィールド名が `invariant` 式内で正しく解決されること（E002 は既存処理で対応）

### 2-3: `bind x: T <- expr` の展開

- [x] `check_stmt` の `Stmt::Bind` 処理で型注釈を検出する
- [x] 型注釈が invariant 付き型（`has_invariants` で判定）のとき `chain x <- T.new(expr)` に展開
- [x] `has_invariants(type_name: &str) -> bool` ヘルパーを `Checker` に追加
- [x] chain コンテキスト外で使ったとき E024 が出ることを確認

### 2-4: 型検査テスト

- [x] `invariant_type_check_bool`: Bool の invariant が型検査を通る
- [x] `invariant_type_check_e045`: `invariant value + 1`（Int 型）で E045 が出る
- [x] `invariant_field_scope`: invariant 内でフィールド名が正しく解決される
- [x] `invariant_unknown_field`: 存在しない名前を使うと E002 が出る（既存処理）
- [x] `bind_state_annotation_chain`: `bind age: PosInt <- 25` が chain に展開されること
- [x] `cargo test` が全通過すること

---

## Phase 3: コンストラクタ自動生成 + VM

### 3-1: compiler.rs の変更

- [x] `compile_type_def` で `TypeDef.invariants` が空でなければコンストラクタ IR を生成
- [x] `compile_type_def_constructor` を実装
  - [x] `TypeBody::Record(fields)` からフィールドリストを取得してパラメータに変換
  - [x] 関数名: `{TypeName}.new`
  - [x] パラメータ: 全フィールドを順番に
  - [x] 戻り値型: `Result<TypeName, String>`（= `TypeName!`）
  - [x] 本体: レコード構築 → invariant チェック → Ok/Err 返却
- [x] `build_constructor_body` を実装
  - [x] レコード構築 IR の生成
  - [x] 複数 invariant を `&&` で結合した条件式の生成
  - [x] `if !(cond) { return Result.err("InvariantViolation: {TypeName}") }` の IR 生成
  - [x] `return Result.ok(t)` の IR 生成
- [x] invariant 式内のフィールド名をコンストラクタのパラメータ変数として解決
- [x] 生成された IR 関数を `IRProgram.fn_defs` に追加

### 3-2: 静的 invariant 検査

- [x] `try_static_invariant_check(type_name, lit) -> Option<bool>` を実装
  - [x] RHS がリテラルのみで invariant が評価可能な場合のみ `Some` を返す
  - [x] `Some(false)` → コンパイルエラー（E001）
  - [x] `Some(true)` → `.new()` 省略でレコード直接構築
  - [x] `None` → 通常の `.new()` 呼び出し
- [x] リテラルパターン: `Int` / `Float` / `String` / `Bool` リテラルのみ対応

### 3-3: コンストラクタ実行テスト

- [x] `constructor_ok`: `PosInt.new(5)` が `Ok(PosInt { value: 5 })` を返す
- [x] `constructor_err`: `PosInt.new(-1)` が `Err(...)` を返す
- [x] `constructor_multi_invariant`: `UserAge.new(-1)` / `UserAge.new(200)` がそれぞれ Err
- [x] `constructor_multi_field`: 複数フィールド型の `.new(a, b)` が全 invariant をチェック
- [x] `constructor_ok_chain_context`: chain コンテキストで `PosInt.new(5)` が `PosInt` を取り出す
- [x] `static_check_literal_ok`: `bind n: PosInt <- 42` がコンパイル時に検査される
- [x] `static_check_literal_fail`: `bind n: PosInt <- -5` がコンパイルエラー（E001）になる
- [x] `cargo test` が全通過すること

---

## Phase 4: `std.states` ルーン

### 4-1: `register_stdlib_states` ヘルパーの実装

- [x] `register_state_type(name, field_ty, invariants: &[&str])` を `Checker` に追加
  - [x] 内部で `TypeDef` を合成して checker に登録（`TypeBody::Record(vec![Field { name: "value", ... }])` で構築）
  - [x] コンストラクタ IR を `compiler.rs` 側に通知（フラグまたは事前登録）
- [x] `register_stdlib_states(checker)` ヘルパーを実装
  - [x] `PosInt`: `value: Int`, `invariant value > 0`
  - [x] `NonNegInt`: `value: Int`, `invariant value >= 0`
  - [x] `Probability`: `value: Float`, `invariant value >= 0.0 && value <= 1.0`
  - [x] `PortNumber`: `value: Int`, `invariant value >= 1 && value <= 65535`
  - [x] `NonEmptyString`: `value: String`, `invariant String.length(value) > 0`
  - [x] `Email`: `value: String`, `invariant String.contains(value, "@") && String.length(value) > 3`
  - [x] `Url`: `value: String`, `invariant starts_with "http://" || "https://"`
  - [x] `Slug`: `value: String`, `invariant String.is_slug(value)`
- [x] `Checker::new()` で `register_stdlib_states` を呼ぶ

### 4-2: `String.is_slug` ビルトインの追加

- [x] `vm.rs` の `vm_call_builtin` に `"String.is_slug"` を追加
  - [x] 英数字・ハイフン・アンダースコアのみ AND 非空文字で `Bool` を返す
- [x] `compiler.rs` に `String.is_slug` のビルトイン登録を追加（checker が解決できるよう）

### 4-3: `resolver.rs` の `std.states` モジュール解決

- [x] `resolver.rs` の `load_module` で `"std.states"` を特別処理
  - [x] `std.states.PosInt` / `std.states.Email` 等を個別インポートに対応
  - [x] `use std.states.*` で全 8 型をスコープに追加

### 4-4: `std.states` 動作テスト

- [x] `std_states_pos_int_ok`: `use std.states.PosInt` + `PosInt.new(1)` → Ok
- [x] `std_states_pos_int_err`: `PosInt.new(0)` → Err
- [x] `std_states_email_ok`: `Email.new("a@b.com")` → Ok
- [x] `std_states_email_err`: `Email.new("bad")` → Err
- [x] `std_states_probability_ok`: `Probability.new(0.5)` → Ok
- [x] `std_states_probability_err_above`: `Probability.new(1.5)` → Err
- [x] `std_states_probability_err_below`: `Probability.new(-0.1)` → Err
- [x] `std_states_port_ok`: `PortNumber.new(8080)` → Ok
- [x] `std_states_port_err`: `PortNumber.new(0)` → Err
- [x] `std_states_nonempty_ok`: `NonEmptyString.new("hello")` → Ok
- [x] `std_states_nonempty_err`: `NonEmptyString.new("")` → Err
- [x] `std_states_slug_ok`: `Slug.new("hello-world")` → Ok
- [x] `std_states_slug_err`: `Slug.new("hello world")` → Err（スペース含む）
- [x] `std_states_url_ok`: `Url.new("https://example.com")` → Ok
- [x] `std_states_url_err`: `Url.new("ftp://example.com")` → Err
- [x] `cargo test` が全通過すること

---

## Phase 5: `fav explain` invariant 表示

### 5-1: `cmd_explain` の変更

- [x] `format_invariants(invs: &[Expr]) -> String` ヘルパーを `driver.rs` に追加
  - [x] 空なら `"—"` を返す
  - [x] 各式を `format_expr_compact` で文字列化し `;` 区切りで結合
- [x] `cmd_explain` の型一覧出力に `INVARIANTS` 列を追加
- [x] `std.states` 型は `(stdlib)` ラベルを付けて表示

### 5-2: `format_expr_compact` の実装

- [x] IR/AST の式を簡潔な文字列に変換する `format_expr_compact` を実装
  - [x] BinOp: `a > 0`, `a >= 0 && a <= 150` 等
  - [x] Call: `String.contains(value, "@")` 等
  - [x] 長さ 60 文字を超えたら `...` で切る

### 5-3: 表示テスト

- [x] `explain_shows_invariants`: `fav explain` の出力に invariant 列が含まれる
- [x] `explain_stdlib_label`: `std.states` 型に `(stdlib)` が付く
- [x] `cargo test` が全通過すること

---

## Phase 6: DB スキーマ CHECK 出力

### 6-1: `--schema` フラグの追加

- [x] `main.rs` の `explain` コマンドに `--schema` フラグを追加
- [x] `driver.rs` の `cmd_explain` が `schema: bool` を受け取るよう変更

### 6-2: `cmd_explain_schema` の実装

- [x] `invariant_to_sql(expr: &Expr) -> String` を実装
  - [x] `>`, `>=`, `<`, `<=` → SQL 比較演算子
  - [x] `&&` → `AND`, `||` → `OR`
  - [x] `String.contains(v, s)` → `v LIKE '%s%'`
  - [x] `String.starts_with(v, s)` → `v LIKE 's%'`
  - [x] `String.length(v)` → `length(v)`
  - [x] 変換不可 → `-- [unsupported invariant: ...]`
- [x] `favnir_type_to_sql(ty: &Type) -> &str` を実装
  - [x] `Int` → `INTEGER`, `Float` → `REAL`, `String` → `TEXT`, `Bool` → `INTEGER`
- [x] `to_snake_case(s: &str) -> String` を実装（型名のスネークケース変換）
- [x] `cmd_explain_schema` で型ごとに CREATE TABLE + CHECK を出力

### 6-3: スキーマ出力テスト

- [x] `schema_pos_int`: `PosInt` が `CHECK (value > 0)` を含む SQL を出力する
- [x] `schema_email`: `Email` が `CHECK (value LIKE '%@%' AND ...)` を含む
- [x] `schema_unsupported`: 変換不可の invariant がコメントとして出力される
- [x] `cargo test` が全通過すること

---

## Phase 7: テスト・ドキュメント

### 7-1: example ファイルの追加

- [x] `examples/invariant_basic.fav` を作成
  - [x] `type Age = { value: Int  invariant ... }` の手書き定義
  - [x] `Age.new(30)` → Ok のパターン
  - [x] `Age.new(-1)` → Err のパターン
  - [x] `fav run` で動作確認
- [x] `examples/std_states.fav` を作成
  - [x] `use std.states.Email`, `use std.states.PosInt`
  - [x] 正常・異常両パターンを match で処理
  - [x] `fav run` で動作確認

### 7-2: langspec.md の更新

- [x] `versions/v1.2.0/langspec.md` を新規作成（v1.1.0/langspec.md を起点に invariant 節を追加）
  - [x] `invariant` 構文と例
  - [x] `.new()` コンストラクタの挙動（`T!` を返す）
  - [x] `std.states` 型一覧（表形式）
  - [x] `bind x: T <- expr` の展開ルール
- [x] エラーコード一覧（8. 節）に E045 を追加

### 7-3: README.md の更新

- [x] v1.2.0 セクションを追加（invariant + std.states の紹介）

### 7-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（v1.1.0 継承 + 新規テスト）
- [x] `type Email { value: String  invariant String.contains(value, "@") }` が定義できる
- [x] `Email.new("bad")` が Err、`Email.new("a@b.com")` が Ok を返す
- [x] `use std.states.PosInt` + `bind age: PosInt <- 25` が動く
- [x] `invariant value + 1` で E045 が出る
- [x] `fav explain` に invariant 列が表示される
- [x] `Cargo.toml` バージョンが `"1.2.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `type Email { value: String  invariant String.contains(value, "@") }` が定義できる
- [x] `Email.new("bad")` が `Err` を返す
- [x] `Email.new("a@b.com")` が `Ok(Email { value: "a@b.com" })` を返す
- [x] `use std.states.PosInt` で `bind age: PosInt <- 25` が chain コンテキストで動く
- [x] `invariant value + 1`（非 Bool）で E045 が出る
- [x] `fav explain` で `Email` の invariant 一覧が表示される
- [x] `Cargo.toml` バージョンが `"1.2.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| `fav check --sample N`（実データの invariant 適合率） | v1.5.0 |
| LSP による State 型の提案（型ホール `_`） | v1.5.0 以降 |
| `fav state sync`（DB スキーマ → 型自動生成） | v1.5.0 以降 |
| Invariant の静的 SMT 証明 | v2.0.0 以降 |
| `Invariant.min` / `Invariant.max` 合成 API | v1.3.0 以降 |
| `abstract trf` / `abstract flw` | v1.3.0 |
