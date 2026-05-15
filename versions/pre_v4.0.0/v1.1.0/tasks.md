# Favnir v1.1.0 タスク一覧 — `interface` システム

作成日: 2026-05-06
更新日: 2026-05-06（実装と突合、完了分を [x] に更新）

> **ゴール**: `interface` / `impl`（手書き + 自動合成）+ `with` 糖衣構文 + Gen/Field の基盤確立
>
> **前提**: v1.0.0 完了（321 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

> **実装メモ**: spec/plan と実装の差異
> - `ImplDecl` → 実装では `InterfaceImplDecl`（同等機能）
> - `Program.interface_decls/impl_decls` → 実装では `Program.items: Vec<Item>` に `Item::InterfaceDecl` / `Item::InterfaceImplDecl` で統合
> - `ImplEntry` → 実装では `InterfaceImplEntry`

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.1.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.1.0` に更新
- [x] `cargo build` が通ること

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

- [x] `InterfaceMethod { name, ty, span }` 構造体を追加
- [x] `InterfaceDecl { name, super_interface, methods, span }` 構造体を追加（visibility フィールドも付加）
- [x] `InterfaceImplDecl { interface_names, type_name, type_params, methods, is_auto, span }` 構造体を追加（仕様名 `ImplDecl` から変更）
- [x] `TypeDef` に `with_interfaces: Vec<String>` フィールドを追加
- [x] `Program` に `interface_decls` / `impl_decls` → 実装では `items: Vec<Item>` に `Item::InterfaceDecl` / `Item::InterfaceImplDecl` として統合（別アプローチで達成）
- [x] 新フィールドの初期化（`items: vec![]` で対応）

### 1-2: lexer.rs の変更

- [x] `TokenKind::Interface` バリアントを追加
- [x] `TokenKind::With` バリアントを追加
- [x] `TokenKind::Impl` — v0.4.0 で追加済みであることを確認
- [x] `TokenKind::For` — v0.4.0 で追加済みであることを確認
- [x] キーワードマップに `"interface" => TokenKind::Interface` を追加
- [x] キーワードマップに `"with" => TokenKind::With` を追加
- [x] 既存テストが全通過すること

### 1-3: parser.rs の変更

- [x] `parse_interface_decl` を実装
  - [x] `interface Name` をパース
  - [x] `: SuperName` オプションをパース
  - [x] `{ method: Type; ... }` のメソッドリストをパース
  - [x] `InterfaceDecl` を返す
- [x] `parse_interface_impl_decl`（仕様名 `parse_impl_decl`）を実装
  - [x] `impl Name ("," Name)*` の interface 名リストをパース
  - [x] `for TypeName` をパース
  - [x] 本体あり: `{ method = |...| ... }` のメソッドリストをパース
  - [x] 本体なし: `{` がなければ `is_auto: true` で返す
  - [x] `InterfaceImplDecl` を返す
- [x] `parse_type_def` に `with` 節を追加
  - [x] `with Name ("," Name)*` をパース
  - [x] `TypeDef.with_interfaces` に格納
- [x] `parse_program` のトップレベル `match` に `Token::Interface` を追加
- [x] `parse_program` のトップレベル `match` に `Token::Impl` を追加
- [x] パーサーテスト: `interface Show { show: Self -> String }` がパースできる（`test_parse_interface_decl`）
- [x] パーサーテスト: `impl Show for Int { show = |x| ... }` がパースできる（`test_parse_interface_impl_decl`）
- [x] パーサーテスト: `impl Show, Eq for UserRow`（本体なし）がパースできる（`test_parse_interface_impl_decl`）
- [x] パーサーテスト: `type T with Show, Eq { ... }` がパースできる（`test_parse_type_with_interfaces`）
- [x] パーサーテスト: `interface Ord : Eq { ... }` のスーパー interface が独立したテストで確認される（`test_parse_interface_decl_with_super` in parser.rs）
- [x] `cargo test` が全通過すること

---

## Phase 2: 型検査統合（InterfaceRegistry）

### 2-1: 基本構造の追加

- [x] `checker.rs` に `InterfaceDef { super_interface, methods }` 構造体を追加
- [x] `checker.rs` に `InterfaceImplEntry { methods, is_auto }` 構造体を追加（仕様名 `ImplEntry` から変更）
- [x] `checker.rs` に `InterfaceRegistry` 構造体を追加
- [x] `InterfaceRegistry::new()` を実装
- [x] `InterfaceRegistry::register_interface(name, super, methods)` を実装
- [x] `InterfaceRegistry::register_impl(interface, type, methods, is_auto)` を実装
- [x] `InterfaceRegistry::is_implemented(interface, type_name) -> bool` を実装
- [x] `InterfaceRegistry::lookup_method(...)` を実装
- [x] `Checker` 構造体に `interface_registry: InterfaceRegistry` フィールドを追加
- [x] `Checker::new()` で `interface_registry` を初期化

### 2-2: Type の拡張

- [x] `Type` enum に `Interface(String, Vec<Type>)` バリアントを追加
- [x] `format_type` で `Type::Interface` を表示する
- [x] `unify` で `Type::Interface` の比較ロジックを追加

### 2-3: `check_interface_decl` の実装

- [x] `check_interface_decl` を `checker.rs` に追加
  - [x] スーパー interface が存在するかチェック
  - [x] `interface_registry.register_interface` を呼ぶ
- [x] `check_program` で全 `InterfaceDecl` を処理

### 2-4: `check_interface_impl_decl` の実装（手書き）

- [x] `check_interface_impl_decl` を `checker.rs` に追加（仕様名 `check_impl_decl`）
  - [x] 各 interface_name が登録済みか確認（未定義なら E041）
  - [x] 手書き実装: 各メソッドの型を検査（不一致なら E042）
  - [x] スーパー interface が満たされているか確認（なければ E043）
  - [x] `interface_registry.register_impl` を呼ぶ
- [x] `check_program` で全 `InterfaceImplDecl` を処理

### 2-5: 明示的な値渡しの型検査

- [x] 関数パラメータに `Type::Interface` が来たとき、呼び出し側の引数型を検査
- [x] `User.ord` のような `TypeName.interface_name` 形式のフィールドアクセスで `Type::Interface(...)` を返す
- [x] 型不一致時に E043 を出す

### 2-6: エラーコード登録

- [x] `E041`: 未定義 interface の `impl` → エラーメッセージ定義
- [x] `E042`: `impl` メソッド型不一致 → エラーメッセージ定義
- [x] `E043`: 要求 interface 未実装 → エラーメッセージ定義

### 2-7: 型検査テスト

- [x] `interface_show_int`: `impl Show for Int { show = |x| ... }` が型検査を通る（`test_interface_show_int_ok`）
- [x] `interface_method_type_mismatch`: method 型不一致で E042 が出る（`test_interface_method_type_mismatch_e042`）
- [x] `interface_super_missing`: `impl Ord for T` で `impl Eq for T` がなければ E043（`test_interface_super_missing_e043`）
- [x] `interface_unknown`: 未定義 interface への `impl` で E041 が出る（`test_interface_unknown_e041`）
- [x] `interface_explicit_passing`: `fn sort<T>(items, ord: Ord<T>)` で未実装時に E043（`test_interface_explicit_passing`）
- [x] `cargo test` が全通過すること

---

## Phase 3: 自動合成 + `with` 糖衣構文

### 3-1: 自動合成の実装

- [x] `synthesize_impl` を `checker.rs` に追加
  - [x] 全フィールドが interface を実装しているか再帰的に確認
  - [x] 未実装フィールドがあれば E044（フィールド名を明示）
  - [x] 合成実装（Show/Eq/Gen）の `InterfaceImplEntry` を構築して登録
- [x] `Show` の合成ロジック
- [x] `Eq` の合成ロジック: 全フィールドの `eq` を AND で結合
- [x] ネストした型（フィールドがカスタム型）への再帰合成の独立したテスト（`test_list_field_show_synthesis_ok`, `test_option_field_show_synthesis_ok` in checker.rs）

### 3-2: `with` 糖衣構文の展開

- [x] `check_program` 内で `TypeDef.with_interfaces` が空でなければ合成処理を呼ぶ
- [x] `type T with Show, Eq { ... }` と `type T { ... }; impl Show, Eq for T` が同一結果になることを確認

### 3-3: 自動合成テスト

- [x] `interface_auto_synthesis_ok`: `impl Show for UserRow`（全フィールド Show あり）が動く（`test_interface_auto_synthesis_ok`）
- [x] `interface_auto_synthesis_fail`: フィールドに Show 未実装があれば E044（`test_interface_auto_synthesis_fail_e044`）
- [x] `type_with_sugar`: `type T with Show, Eq { ... }` が動く（`test_interface_auto_synthesis_ok`・`test_interface_impl_multi_interface` で実質カバー、専用テスト名なし）
- [x] `impl_multi_interface`: `impl Show, Eq, Json for T` が一行で書ける（`test_interface_impl_multi_interface`）
- [x] `cargo test` が全通過すること

---

## Phase 4: 標準 interface 移行（Eq / Ord / Show）

### 4-1: 組み込み interface の登録

- [x] `register_builtin_interfaces` ヘルパー関数を `checker.rs` に追加
- [x] `Show` interface を登録（`show: Self -> String`）
- [x] `Eq` interface を登録（`eq: Self -> Self -> Bool`）
- [x] `Ord` interface を登録（super: `Eq`; `compare: Self -> Self -> Int`）
- [x] `Int` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [x] `Float` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [x] `Bool` の `Show`, `Eq` を `InterfaceRegistry` に登録
- [x] `String` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [x] `Checker::new()` で `register_builtin_interfaces` を呼ぶ
- [x] `List<T>`, `Option<T>`, `Result<T,E>` の `Show`/`Eq` 自動登録（T が Show/Eq のとき）— `is_type_implementing` ヘルパーで再帰チェック実装済み

### 4-2: ブリッジの確認

- [x] 旧 `IMPL_REGISTRY` ベースのコードが引き続き動作することを確認
- [x] `cap` で書かれた v0.4.0 テストが全通過すること（`cap_example_check_emits_w010_but_no_errors`）
  - [x] `examples/cap_sort.fav`（driver.rs test にて確認）
  - [x] `examples/cap_user.fav`（driver.rs test にて確認）

### 4-3: 標準 interface テスト

- [x] `builtin_show_int_registered`: `Int` の `Show` が `InterfaceRegistry` に登録されている（`test_builtin_show_int_registered`）
- [x] `builtin_ord_int_registered`: `Int` の `Ord` が登録されている（`test_builtin_ord_int_registered`）
- [x] `cargo test` が全通過すること

---

## Phase 5: `Gen` + `Field` interface 定義

### 5-1: Gen interface

- [x] `Gen` interface を `InterfaceRegistry` に登録（`gen: Int? -> Self`）
- [x] `Int` の `Gen` を登録
- [x] `Float` の `Gen` を登録
- [x] `Bool` の `Gen` を登録
- [x] `String` の `Gen` を登録
- [x] `impl Gen for T`（本体なし）の自動合成ロジックを追加（全フィールドが Gen のとき）
  - [x] `synthesize_impl` の `Gen` 対応

### 5-2: Field 系列 interface

- [x] `Semigroup` interface を登録（`combine: Self -> Self -> Self`）
- [x] `Monoid` interface を登録（super: `Semigroup`; `empty: Self`）
- [x] `Group` interface を登録（super: `Monoid`; `inverse: Self -> Self`）
- [x] `Ring` interface を登録（super: `Monoid`; `multiply: Self -> Self -> Self`）
- [x] `Field` interface を登録（super: `Ring`; `divide: Self -> Self -> Self!`）
- [x] `Float` の `Semigroup`, `Monoid`, `Group`, `Ring`, `Field` を登録
- [x] `Int` の `Semigroup`, `Monoid`, `Group`, `Ring` を登録（`Field` は除く）

### 5-3: Gen / Field テスト

- [x] `gen_interface_builtin`: `impl Gen for Int` が内部登録されている（`test_builtin_gen_int_registered`）
- [x] `gen_interface_auto_synthesis`: `impl Gen for UserRow`（全フィールド Gen あり）が動く（`test_gen_interface_auto_synthesis_ok`）
- [x] `gen_auto_synthesis_fail`: フィールドに Gen 未実装があれば E044（`test_gen_auto_synthesis_fail_e044`）
- [x] `field_interface_float`: `impl Field for Float` が内部登録されている（`test_field_interface_float_registered`）
- [x] `semigroup_interface_int`: `impl Semigroup for Int` が内部登録されている（`test_semigroup_interface_int_registered`）
- [x] `cargo test` が全通過すること

---

## Phase 6: `cap` 非推奨警告（W010）

### 6-1: TypeWarning の追加

- [x] `checker.rs` に `TypeWarning { code, message, span }` 構造体を追加
- [x] `Checker` に `pub warnings: Vec<TypeWarning>` フィールドを追加
- [x] `Checker::new()` で `warnings: vec![]` を初期化

### 6-2: W010 の発行

- [x] `check_cap_decl`（既存の cap 型チェック）の先頭で `W010` を `warnings` に追加
- [x] 旧 `impl ... for ...`（cap スタイル）でも W010 を発行（`check_impl_def` で対応）
- [x] コンパイルは通す（エラーは追加しない）

### 6-3: driver.rs の変更

- [x] `cmd_check` で `checker.warnings` を `render_warnings` で表示
  - [x] 出力フォーマット: `warning[W010]: ... (file.fav:N)`
- [x] `main.rs` に `--no-warn` フラグを追加
- [x] `--no-warn` フラグで W010 を抑制

### 6-4: 非推奨警告テスト

- [x] `cap_deprecated_warning`: `cap` キーワードに W010 が出る（`test_cap_deprecated_warning_w010`）
- [x] `cap_still_compiles`: W010 が出ても実行結果は変わらない（`cap_example_check_emits_w010_but_no_errors`）
- [x] `cap_no_warn_flag`: `--no-warn` で W010 が抑制される（`cap_example_check_no_warn_suppresses_warning_output`）
- [x] `cargo test` が全通過すること

---

## Phase 7: テスト・ドキュメント

### 7-1: example ファイルの追加

- [x] `examples/interface_basic.fav` を作成
  - [x] `impl Show for Point` + `impl Eq for Point` — Point(x, y) の実際の値を出力
  - [x] `fav run` で動作確認
- [x] `examples/interface_auto.fav` を作成
  - [x] `type UserRow with Eq = { ... }` + 手書き `impl Show for UserRow`
  - [x] `fav run` で動作確認
- [x] `examples/algebraic.fav` を作成
  - [x] `impl Semigroup`, `Monoid`, `Ring` for `Complex` — 実際の複素数演算を出力
  - [x] `fav run` で動作確認

### 7-2: langspec.md の更新

- [x] `versions/v1.0.0/langspec.md` に「5a. Interface System」節を追加
  - [x] `interface` 宣言の構文と例
  - [x] `impl`（手書き）の構文と例
  - [x] `impl`（自動合成）の構文と条件
  - [x] `with` 糖衣構文の説明
  - [x] 明示的な値渡しの原則
  - [x] Gen / Field / Show / Eq / Ord の一覧
  - [x] 「旧 cap キーワード（非推奨）」の説明
  - [x] エラーコード一覧（8. 節）に E041–E044 と W010 を追加

### 7-3: README.md の更新

- [x] v1.1.0 セクションを追加（interface システムの紹介）

### 7-4: 全体確認

- [x] `cargo build` で警告ゼロ（Rust コンパイラ警告）
- [x] `cargo test` 全テスト通過（321 + 新規テスト）
- [x] `interface Show { show: Self -> String }` と `impl Show for Int { show = |x| ... }` が動く
- [x] `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- [x] `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- [x] `impl Gen for UserRow`（本体なし）が動く
- [x] `impl Field for Float` が内部登録されている
- [x] `cap` で書かれた既存コードに W010 警告が出るが、動作は継続する
- [x] `Cargo.toml` バージョンが `"1.1.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `interface Show { show: Self -> String }` と `impl Show for Int { show = |x| Int.to_string(x) }` が動く
- [x] `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- [x] `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- [x] `fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T>` の呼び出しで未実装時に E043 が出る
- [x] `Gen` interface が定義され、`Int/Float/Bool/String` の impl が内部登録される
- [x] `Field` 系列 interface が定義され、`Float` の impl が内部登録される
- [x] `cap` で書かれた既存コードに W010 警告が出るが、動作は継続する
- [x] `Cargo.toml` バージョンが `"1.1.0"`

---

## 未実装（次バージョン以降で対応）

| 項目 | 内容 | 対応バージョン |
|---|---|---|
| `List<T>`, `Option<T>`, `Result<T,E>` の Show/Eq 自動登録 | T が Show/Eq を持つとき自動合成登録 | v1.2.0 以降検討 |
| `interface Ord : Eq { ... }` スーパー interface の専用パーサーテスト | 機能は動作するが独立テスト未作成 | 必要に応じて追加 |
| `type_with_sugar` 専用テスト関数 | 機能は `test_interface_auto_synthesis_ok` 等でカバー済み | 必要に応じて追加 |

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| `abstract type` / `abstract stage` / `abstract seq` | v1.3.0 |
| `invariant` 構文 | v1.2.0 |
| `Stat.one<T>` の実際の動作 | v1.5.0 |
| 演算子オーバーロードの実際の委譲（`+` → `Semigroup::combine`） | v2.0.0 以降 |
| `interface` を rune 境界を越えて使う | v1.3.0 以降 |
| `IMPL_REGISTRY` の削除 | v2.0.0 |
| `fav migrate` コマンド | v2.0.0 |
