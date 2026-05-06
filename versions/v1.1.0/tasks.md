# Favnir v1.1.0 タスク一覧 — `interface` システム

作成日: 2026-05-06

> **ゴール**: `interface` / `impl`（手書き + 自動合成）+ `with` 糖衣構文 + Gen/Field の基盤確立
>
> **前提**: v1.0.0 完了（321 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [ ] `Cargo.toml` の `version` を `"1.1.0"` に更新
- [ ] `main.rs` の HELP テキストを `v1.1.0` に更新
- [ ] `cargo build` が通ること

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

- [ ] `InterfaceMethod { name, ty, span }` 構造体を追加
- [ ] `InterfaceDecl { name, super_interface, methods, span }` 構造体を追加
- [ ] `ImplDecl { interface_names, type_name, type_params, methods, is_auto, span }` 構造体を追加
- [ ] `TypeDecl` に `with_interfaces: Vec<String>` フィールドを追加
- [ ] `Program` に `interface_decls: Vec<InterfaceDecl>` フィールドを追加
- [ ] `Program` に `impl_decls: Vec<ImplDecl>` フィールドを追加
- [ ] `Program::new()` / デフォルト値で新フィールドを初期化

### 1-2: lexer.rs の変更

- [ ] `Token::Interface` バリアントを追加
- [ ] `Token::With` バリアントを追加
- [ ] `Token::Impl` が未定義なら追加（v0.4.0 で追加済みか確認）
- [ ] `Token::For` が未定義なら追加（同上）
- [ ] キーワードマップに `"interface" => Token::Interface` を追加
- [ ] キーワードマップに `"with" => Token::With` を追加
- [ ] 既存テストが全通過すること

### 1-3: parser.rs の変更

- [ ] `parse_interface_decl` を実装
  - [ ] `interface Name` をパース
  - [ ] `: SuperName` オプションをパース
  - [ ] `{ method: Type; ... }` のメソッドリストをパース
  - [ ] `InterfaceDecl` を返す
- [ ] `parse_impl_decl` を実装
  - [ ] `impl Name ("," Name)*` の interface 名リストをパース
  - [ ] `for TypeName` をパース
  - [ ] 本体あり: `{ method = |...| ... }` のメソッドリストをパース
  - [ ] 本体なし: `{` がなければ `is_auto: true` で返す
  - [ ] `ImplDecl` を返す
- [ ] `parse_type_decl` に `with` 節を追加
  - [ ] `with Name ("," Name)*` をパース
  - [ ] `TypeDecl.with_interfaces` に格納
- [ ] `parse_program` のトップレベル `match` に `Token::Interface` を追加
- [ ] `parse_program` のトップレベル `match` に `Token::Impl` を追加
- [ ] パーサーテスト: `interface Show { show: Self -> String }` がパースできる
- [ ] パーサーテスト: `impl Show for Int { show = |x| ... }` がパースできる
- [ ] パーサーテスト: `impl Show, Eq for UserRow`（本体なし）がパースできる
- [ ] パーサーテスト: `type T with Show, Eq { ... }` がパースできる
- [ ] パーサーテスト: `interface Ord : Eq { ... }` のスーパー interface がパースできる
- [ ] `cargo test` が全通過すること

---

## Phase 2: 型検査統合（InterfaceRegistry）

### 2-1: 基本構造の追加

- [ ] `checker.rs` に `InterfaceDef { super_interface, methods }` 構造体を追加
- [ ] `checker.rs` に `ImplEntry { methods, is_auto }` 構造体を追加
- [ ] `checker.rs` に `InterfaceRegistry` 構造体を追加
- [ ] `InterfaceRegistry::new()` を実装
- [ ] `InterfaceRegistry::register_interface(decl)` を実装
- [ ] `InterfaceRegistry::register_impl(decl)` を実装
- [ ] `InterfaceRegistry::is_implemented(interface, type_name) -> bool` を実装
- [ ] `InterfaceRegistry::lookup_method(...)` を実装
- [ ] `Checker` 構造体に `interface_registry: InterfaceRegistry` フィールドを追加
- [ ] `Checker::new()` で `interface_registry` を初期化

### 2-2: Type の拡張

- [ ] `Type` enum に `Interface(String, Vec<Type>)` バリアントを追加
- [ ] `format_type` で `Type::Interface` を表示する
- [ ] `unify` で `Type::Interface` の比較ロジックを追加

### 2-3: `check_interface_decl` の実装

- [ ] `check_interface_decl` を `checker.rs` に追加
  - [ ] 重複 interface 名をチェック
  - [ ] スーパー interface が存在するかチェック
  - [ ] メソッドの型式を検査
  - [ ] `interface_registry.register_interface` を呼ぶ
- [ ] `check_program` で全 `interface_decls` を処理

### 2-4: `check_impl_decl` の実装（手書き）

- [ ] `check_impl_decl` を `checker.rs` に追加
  - [ ] 各 interface_name が登録済みか確認（未定義なら E041）
  - [ ] 手書き実装: 各メソッドの型を検査（不一致なら E042）
  - [ ] スーパー interface が満たされているか確認（なければ E043）
  - [ ] `interface_registry.register_impl` を呼ぶ
- [ ] `check_program` で全 `impl_decls` を処理

### 2-5: 明示的な値渡しの型検査

- [ ] 関数パラメータに `Type::Interface` が来たとき、呼び出し側の引数型を検査
- [ ] `User.ord` のような `TypeName.interface_name` 形式のフィールドアクセスで `Type::Interface(...)` を返す
- [ ] 型不一致時に E043 を出す

### 2-6: エラーコード登録

- [ ] `E041`: 未定義 interface の `impl` → エラーメッセージ定義
- [ ] `E042`: `impl` メソッド型不一致 → エラーメッセージ定義
- [ ] `E043`: 要求 interface 未実装 → エラーメッセージ定義

### 2-7: 型検査テスト

- [ ] `interface_show_int`: `impl Show for Int { show = |x| ... }` が型検査を通る
- [ ] `interface_method_type_mismatch`: method 型不一致で E042 が出る
- [ ] `interface_super_missing`: `impl Ord for T` で `impl Eq for T` がなければ E043
- [ ] `interface_unknown`: 未定義 interface への `impl` で E041 が出る
- [ ] `interface_explicit_passing`: `fn sort<T>(items, ord: Ord<T>)` で未実装時に E043
- [ ] `cargo test` が全通過すること

---

## Phase 3: 自動合成 + `with` 糖衣構文

### 3-1: 自動合成の実装

- [ ] `synthesize_impl` を `checker.rs` に追加
  - [ ] 全フィールドが interface を実装しているか再帰的に確認
  - [ ] 未実装フィールドがあれば E044（フィールド名を明示）
  - [ ] 合成実装（Show/Eq/Gen）の `ImplEntry` を構築して登録
- [ ] `Show` の合成ロジック: `"{ field1: {show(f1)}, ... }"` 形式
- [ ] `Eq` の合成ロジック: 全フィールドの `eq` を AND で結合
- [ ] ネストした型（フィールドがカスタム型）への再帰合成を対応

### 3-2: `with` 糖衣構文の展開

- [ ] `check_program` 内で `TypeDecl.with_interfaces` が空でなければ `ImplDecl` を合成
- [ ] 合成した `ImplDecl` を `check_impl_decl` に渡して処理
- [ ] `type T with Show, Eq { ... }` と `type T { ... }; impl Show, Eq for T` が同一結果になることを確認

### 3-3: 自動合成テスト

- [ ] `interface_auto_synthesis_ok`: `impl Show for UserRow`（全フィールド Show あり）が動く
- [ ] `interface_auto_synthesis_fail`: フィールドに Show 未実装があれば E044（フィールド名付き）
- [ ] `type_with_sugar`: `type T with Show, Eq { ... }` が等価 `impl` と同動作
- [ ] `impl_multi_interface`: `impl Show, Eq, Json for T` が一行で書ける
- [ ] `cargo test` が全通過すること

---

## Phase 4: 標準 interface 移行（Eq / Ord / Show）

### 4-1: 組み込み interface の登録

- [ ] `register_builtin_interfaces` ヘルパー関数を `checker.rs` に追加
- [ ] `Show` interface を登録（`show: Self -> String`）
- [ ] `Eq` interface を登録（`eq: Self -> Self -> Bool`）
- [ ] `Ord` interface を登録（super: `Eq`; `compare: Self -> Self -> Int`）
- [ ] `Int` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [ ] `Float` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [ ] `Bool` の `Show`, `Eq` を `InterfaceRegistry` に登録
- [ ] `String` の `Show`, `Eq`, `Ord` を `InterfaceRegistry` に登録
- [ ] `Checker::new()` で `register_builtin_interfaces` を呼ぶ

### 4-2: ブリッジの確認

- [ ] 旧 `IMPL_REGISTRY` ベースのコードが引き続き動作することを確認
- [ ] `cap Eq / Ord / Show` で書かれた v0.4.0 テストが全通過すること
  - [ ] `examples/cap_sort.fav`
  - [ ] `examples/cap_user.fav`

### 4-3: 標準 interface テスト

- [ ] `builtin_show_int_registered`: `Int` の `Show` が `InterfaceRegistry` に登録されている
- [ ] `builtin_ord_int_registered`: `Int` の `Ord` が登録されている（Eq も充足）
- [ ] `cargo test` が全通過すること

---

## Phase 5: `Gen` + `Field` interface 定義

### 5-1: Gen interface

- [ ] `Gen` interface を `InterfaceRegistry` に登録（`gen: Int? -> Self`）
- [ ] `Int` の `Gen` を登録（`[-1000, 1000]` の乱数）
- [ ] `Float` の `Gen` を登録（`[0.0, 1.0]` の乱数）
- [ ] `Bool` の `Gen` を登録（確率 0.5）
- [ ] `String` の `Gen` を登録（長さ `[0, 16]` の英数字列）
- [ ] `impl Gen for T`（本体なし）の自動合成ロジックを追加（全フィールドが Gen のとき）
  - [ ] `synthesize_impl` の `Gen` 対応（フィールドごとに `gen(seed ^ idx)` を呼ぶ合成）

### 5-2: Field 系列 interface

- [ ] `Semigroup` interface を登録（`combine: Self -> Self -> Self`）
- [ ] `Monoid` interface を登録（super: `Semigroup`; `empty: Self`）
- [ ] `Group` interface を登録（super: `Monoid`; `inverse: Self -> Self`）
- [ ] `Ring` interface を登録（super: `Monoid`; `multiply: Self -> Self -> Self`）
- [ ] `Field` interface を登録（super: `Ring`; `divide: Self -> Self -> Self!`）
- [ ] `Float` の `Semigroup`, `Monoid`, `Group`, `Ring`, `Field` を登録
- [ ] `Int` の `Semigroup`, `Monoid`, `Group`, `Ring` を登録（`Field` は除く）

### 5-3: Gen / Field テスト

- [ ] `gen_interface_builtin`: `impl Gen for Int` が内部登録されている
- [ ] `gen_interface_auto_synthesis`: `impl Gen for UserRow`（全フィールド Gen あり）が動く
- [ ] `gen_auto_synthesis_fail`: フィールドに Gen 未実装があれば E044
- [ ] `field_interface_float`: `impl Field for Float` が内部登録されている
- [ ] `semigroup_interface_int`: `impl Semigroup for Int` が内部登録されている
- [ ] `cargo test` が全通過すること

---

## Phase 6: `cap` 非推奨警告（W010）

### 6-1: TypeWarning の追加

- [ ] `checker.rs` に `TypeWarning { code, message, span }` 構造体を追加
- [ ] `Checker` に `pub warnings: Vec<TypeWarning>` フィールドを追加
- [ ] `Checker::new()` で `warnings: vec![]` を初期化

### 6-2: W010 の発行

- [ ] `check_cap_decl`（既存の cap 型チェック）の先頭で `W010` を `warnings` に追加
  - [ ] メッセージ: `` `cap` is deprecated. Use `interface` instead. ``
- [ ] 旧 `impl ... for ...`（cap スタイル）でも W010 を発行（`is_cap_style` フラグで判定）
- [ ] コンパイルは通す（エラーは追加しない）

### 6-3: driver.rs の変更

- [ ] `cmd_check` で `checker.warnings` を表示するループを追加
  - [ ] 出力フォーマット: `warning[W010]: ... (file.fav:N)`
- [ ] `main.rs` に `--no-warn` フラグを追加
- [ ] `--no-warn` フラグで W010 を抑制

### 6-4: 非推奨警告テスト

- [ ] `cap_deprecated_warning`: `cap` キーワードに W010 が出る
- [ ] `cap_still_compiles`: W010 が出ても実行結果は変わらない（既存 cap テストが通る）
- [ ] `cap_no_warn_flag`: `--no-warn` で W010 が抑制される
- [ ] `cargo test` が全通過すること

---

## Phase 7: テスト・ドキュメント

### 7-1: example ファイルの追加

- [ ] `examples/interface_basic.fav` を作成
  - [ ] `interface Show { show: Self -> String }` の手書き実装例
  - [ ] `impl Show for Point { show = |p| ... }` を含む
  - [ ] `fav run` で動作確認
- [ ] `examples/interface_auto.fav` を作成
  - [ ] `type UserRow with Show, Eq, Json { ... }` の自動合成例
  - [ ] `fav run` で動作確認
- [ ] `examples/algebraic.fav` を作成
  - [ ] `Field` / `Ring` を使った加重平均の例
  - [ ] `impl Ring for Complex { ... }` を含む
  - [ ] `fav run` で動作確認

### 7-2: langspec.md の更新

- [ ] `versions/v1.0.0/langspec.md` に「6. interface システム」節を追加
  - [ ] `interface` 宣言の構文と例
  - [ ] `impl`（手書き）の構文と例
  - [ ] `impl`（自動合成）の構文と条件
  - [ ] `with` 糖衣構文の説明
  - [ ] 明示的な値渡しの原則
  - [ ] Gen / Field / Show / Eq / Ord の一覧
- [ ] 「旧 cap キーワード（非推奨）」節を追記
- [ ] エラーコード一覧（8. 節）に E041–E044 と W010 を追加

### 7-3: README.md の更新

- [ ] v1.1.0 セクションを追加（interface システムの紹介）

### 7-4: 全体確認

- [ ] `cargo build` で警告ゼロ（Rust コンパイラ警告）
- [ ] `cargo test` 全テスト通過（321 + 新規テスト）
- [ ] `interface Show { show: Self -> String }` と `impl Show for Int { show = |x| ... }` が動く
- [ ] `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- [ ] `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- [ ] `impl Gen for UserRow`（本体なし）が動く
- [ ] `impl Field for Float` が内部登録されている
- [ ] `cap` で書かれた既存コードに W010 警告が出るが、動作は継続する
- [ ] `Cargo.toml` バージョンが `"1.1.0"`

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `interface Show { show: Self -> String }` と `impl Show for Int { show = |x| Int.to_string(x) }` が動く
- [ ] `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- [ ] `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- [ ] `fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T>` の呼び出しで未実装時に E043 が出る
- [ ] `Gen` interface が定義され、`Int/Float/Bool/String` の impl が内部登録される
- [ ] `Field` 系列 interface が定義され、`Float` の impl が内部登録される
- [ ] `cap` で書かれた既存コードに W010 警告が出るが、動作は継続する
- [ ] `Cargo.toml` バージョンが `"1.1.0"`

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
