# Favnir v9.7.0 Tasks

Date: 2026-06-02
Theme: 名目型ラッパー + `where` バリデーション + `with` 自動合成 + `T?`/`T!`/`??`/`expr?` self-hosted 修正

---

## Phase A: Rust パーサー — 名目型ラッパー構文 AST 追加

- [x] A-1: `src/ast.rs` — `WrapperDef { name, inner_ty, validator, with_impls }` を追加
  - `validator: Option<Expr>` — `where |v| pred` の述語式
  - `with_impls: Vec<String>` — `with Eq, Show, Serialize` のインターフェース名リスト
- [x] A-2: `src/ast.rs` — `Item::Wrapper(WrapperDef)` variant を追加
- [x] A-3: `src/frontend/parser.rs` — `type Name(InnerType)` 構文のパース
  - `parse_type_def` で `(` が続く場合は `WrapperDef` に分岐
- [ ] A-4: `src/frontend/parser.rs` — `where |v| pred` 節のパース（延期）
- [ ] A-5: `src/frontend/parser.rs` — `with` 節パース（延期）
- [ ] A-6: `src/frontend/parser.rs` — `where` / `with` キーワード追加（延期）
- [x] A-7: `src/fmt.rs` — `Item::Wrapper` の fmt 対応
- [x] A-8: `src/middle/ast_lower_checker.rs` — `Item::Wrapper` の lower 対応
- [x] A-9: `cargo build` — exhaustive match エラーなし確認

---

## Phase B: Bug fix — `lexer.fav` / `parser.fav` の `T?` / `??` / `expr?` 対応

### B-1〜B-3: lexer.fav

- [x] B-1: `fav/self/lexer.fav` — `TkQuestion` / `TkQuestionQuestion` トークン variant を追加
- [x] B-2: `fav/self/lexer.fav` — `scan_token` で `?` → `TkQuestion` スキャンルール追加
- [x] B-3: `fav/self/lexer.fav` — テスト追加

### B-4〜B-7: parser.fav — 型パース

- [x] B-4: `fav/self/parser.fav` — `parse_type_expr` に `T?` → `TeOption(T)` の後置処理を追加
- [ ] B-5: `fav/self/parser.fav` — `T!` 対応（延期）
- [x] B-6: `fav/self/parser.fav` — `??` 演算子のパース（`OpNullCoalesce`）
- [x] B-7: compiler.fav に統合テストで確認

### B-8〜B-10: compiler.fav — `expr?` 脱糖

- [x] B-8: `fav/self/compiler.fav` — `EQuestion(expr)` ノード対応
- [x] B-9: `fav/self/compiler.fav` — `EQuestion(expr)` の脱糖（`result_and_then_match_expr`）
  - バグ修正: `result_err_expr` を `ECall("Result","err",...)` に変更（tag正規化対応）
- [x] B-10: `fav/self/compiler.fav` — `??` → `match a { Some(v) -> v  None -> b }` 変換

---

## Phase C: checker.fav — 名目型ラッパー型チェック

- [x] C-1: `fav/self/checker.fav` — `WrapperDef` / `IWrapper` 型定義追加
- [x] C-2: `collect_variant_constructors` に `IWrapper` コンストラクタ登録追加
- [x] C-3: `check_item` に `IWrapper` ハンドラ追加（`Result.ok(wd.name)` 返却）
- [x] C-4: `infer_hm` に `EQuestion` ケース追加（`Unknown` を返す）
- [x] C-5: `infer_op` に `OpNullCoalesce` ケース追加
- [x] C-6: `infer_expr_effects` に `EQuestion` / `EBinOp` ケース追加
- [ ] C-7: E0010/E0011/E0013 は延期
- [x] C-9: self-check 通過確認

---

## Phase D: compiler.fav — 名目型コード生成 + `with` 自動合成

- [x] D-1: `fav/self/compiler.fav` — `WrapperDef` / `IWrapper` 型定義追加
- [x] D-2: `parse_type_or_wrapper_item` 追加（`type Name(Inner)` / `type Name = ...` 分岐）
- [x] D-3: `TkType` dispatch を `parse_type_or_wrapper_item` に変更
- [x] D-4: `compile_items` に `IWrapper(_)` スキップ追加（VM が自動でバリアントCtor処理）
- [x] D-5: `pretty_item` に `IWrapper` → pretty_wrapper_def 追加
- [ ] D-6: `with` 自動合成（延期）
- [x] D-7: self-check 通過確認

---

## Phase E: 統合テスト（`fav/src/driver.rs` または `fav/tests/`）

### T?/??/expr? 修正確認

- [x] E-1: `type_option_postfix_t_question` — `T?` → `Option<T>` 動作確認
- [ ] E-2: `T!` 対応（延期）
- [x] E-3: `null_coalesce_some_returns_inner` / `null_coalesce_none_returns_default` — `??` 動作確認
- [x] E-4: `expr_question_unwraps_ok` / `expr_question_propagates_err` — `expr?` 動作確認
- [ ] E-5: E0013 対応（延期）

### 名目型ラッパー

- [x] E-6: `wrapper_type_constructor_and_match` — `type UserId(Int)` コンストラクタ+パターンマッチ確認
- [x] E-7: `wrapper_type_fn_param_and_return` — ラッパー型を関数パラメータ・戻り値に使用
- [x] E-8: `wrapper_type_in_option` — `Option<WrapperType>` 動作確認
- [x] E-9: `wrapper_type_checker_no_error` — checker.fav が wrapper type をエラーなく通過
- [ ] E-10〜E-12: where/with 機能（延期）

### with（レコード型への適用）

- [ ] E-13: `record_with_serialize` — 延期
- [ ] E-14: `unknown_interface_e0011` — 延期

- [x] E-15: `cargo test v970` — 9件全統合テスト通過（1200 tests OK）

---

## Phase F: self-check + Bootstrap 検証

- [x] F-1: `cargo test checker_fav_wire_self_check` — self-check 通過
- [x] F-2: `cargo test bootstrap` — 23件通過（bytecode 維持確認）
- [x] F-3: `cargo test` — 1200件全通過

---

## Phase G: ドキュメント・バージョン更新

- [x] G-1: `fav/Cargo.toml` の version を `"9.7.0"` に更新
- [x] G-2: `fav/self/cli.fav` のバージョン文字列を `"9.7.0"` に更新
- [x] G-3: `versions/v9.7.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] G-4: `memory/MEMORY.md` に v9.7.0 完了を記録
- [ ] G-5: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `T?` / `T!` / `??` が `fav run`（Favnir pipeline）で正しく動作する | |
| `fav check` と `fav run` の挙動が `T?` に関して一致する | |
| `expr?` が `Result` を返す関数内で使える（E0013 で誤用検出） | |
| `type Name(Inner)` がコンストラクタ・パターンマッチで使える | |
| `where` あり型のコンストラクタが `Result<T, String>` を返す | |
| `with Eq, Show, Serialize, Deserialize` の自動合成が動作する | |
| 型の取り違えを E0010 でコンパイル時に検出できる | |
| 未知インターフェース名を E0011 でコンパイル時に検出できる | |
| `checker.fav` self-check 通過 | |
| Bootstrap 検証（bytecode_A == bytecode_B）維持 | |
| 統合テスト 12 件以上通過 | |

---

## 実装メモ

### 名目型ラッパーの内部表現

名目型ラッパーはバイトコードレベルでは内部型と同一の値として扱う（ボックス化なし）。
型チェッカーレベルでのみ区別される（構造的型と名目型の分離）。

### `where` 述語の評価タイミング

コンストラクタ呼び出し時のみ評価する（1回バリデーション保証）。
ラッパー型の値を受け取った後は述語チェックなし。
これにより「入口で一度だけ検証 → 下流は型が保証」の原則を実現。

### `with` 合成の優先順位

`with` で自動合成された関数はユーザー定義関数よりも低優先度。
同名のユーザー定義関数がある場合はユーザー定義が優先される（将来の `impl` ブロックへの橋渡し）。

### `expr?` の脱糖と `compiler.fav`

`parser.fav` が `expr?` を `EQuestion(expr)` AST ノードとして保持し、
`compiler.fav` の `compile_expr` で `match` に展開する。
`checker.fav` は `EQuestion` の戻り型が `Result` であることを検証し、
そうでなければ E0013 を返す。

### Rust パーサー変更の最小化

`with` キーワードは既存の型定義構文に `opt_with` 節として追加する（既存 AST に影響しない）。
`where` キーワードは `WrapperDef` 専用（`fn` の型制約には使わない）。
`type Name(Inner)` は `type Name = ...` と括弧の有無で区別する（曖昧性なし）。
