# Favnir v2.3.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.3.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.3.0` に更新

---

## Phase 1 — 分割 bind のコンパイラ対応

### `src/ast.rs` 確認

- [x] `Pattern::Record` の内側の型（`PatternField` 等）を確認し、`Pun` / `Alias` / `Wildcard` の構造を把握

### `src/middle/compiler.rs`

- [x] `compile_stmt` の `Stmt::Bind` アームで `Pattern::Record` の場合を特別処理
  - [x] 右辺式をコンパイルして中間スロット（`$tmp`）に格納
  - [x] `PatternField::Pun(name)` — フィールド名と同名のローカルを定義し `$tmp.field` を代入
  - [x] `PatternField::Alias(field_name, bind_name)` — `bind_name` のローカルを定義し `$tmp.field_name` を代入
  - [x] `PatternField::Wildcard` — スキップ
- [x] `bind { x, y } <- Point { x: 3  y: 4 }` で `x` / `y` が参照できることを手動確認

---

## Phase 2 — E072 / E073 エラーコードの追加

### `src/middle/checker.rs`

- [x] `check_stmt` の `Stmt::Bind` + `Pattern::Record` アームで型チェックを強化
  - [x] 右辺型がレコード型でない場合に E072 を報告
  - [x] 指定フィールドがレコード型に存在しない場合に E073 を報告
  - [x] エイリアス構文（`{ age: user_age }`）の束縛先の型を正しく `user_age` に割り当て

### エラーコード登録

- [x] `E072` を `FavError` / エラーコード一覧に追加（`bind { } <- 非レコード型`）
- [x] `E073` を `FavError` / エラーコード一覧に追加（`bind { 存在しないフィールド } <- record`）

---

## Phase 3 — 戻り型推論

### `src/ast.rs`

- [x] `FnDef.return_ty: TypeExpr` → `return_ty: Option<TypeExpr>` に変更
  - [x] `None` = 戻り型省略（推論対象）として定義

### `src/frontend/parser.rs`

- [x] `parse_fn_def` を修正
  - [x] `->` がある場合 → 従来通り型を解析して `Some(ty)` を設定
  - [x] `=` が来た場合 → `return_ty = None`、`=` を消費して単一式を解析
  - [x] `{` が来た場合（ブロック）→ `return_ty` が `None` ならエラー（`=` 構文のみ省略可）

### `src/middle/checker.rs`

- [x] `check_fn_def` を修正
  - [x] `return_ty` が `Some(ty)` → 従来通り宣言型を採用
  - [x] `return_ty` が `None` → 本体式の型を推論して戻り型として採用
  - [x] 推論結果が `Type::Unknown` の場合に E074 を報告
  - [x] 再帰関数で `None` の場合のヒントメッセージを追加（"add explicit return type for recursive functions"）

### `src/middle/compiler.rs`

- [x] `compile_fn_def` で `fn_def.return_ty` が `Option` になったことに対応
  - [x] `None` の場合はチェッカーが解決した型（IRFnDef の型情報）を使用

### `src/backend/wasm_codegen.rs`

- [x] `IRFnDef` の戻り型参照が `Option` になった場合の対応

### `src/fmt.rs`

- [x] フォーマッタで `return_ty` が `None` の場合 `-> ty` を出力しない

### エラーコード登録

- [x] `E074` を `FavError` / エラーコード一覧に追加（戻り型推論不可）

---

## Phase 4 — テスト追加

### `src/backend/vm_stdlib_tests.rs`

**分割 bind テスト**:

- [x] `test_destructure_bind_basic`: `bind { x, y } <- Point { x: 3  y: 4 }` で `x=3, y=4`
- [x] `test_destructure_bind_alias`: `bind { age: user_age } <- user` で `user_age` が束縛される
- [x] `test_destructure_bind_wildcard`: `bind { name, _ } <- user` で `name` だけ使える

**戻り型推論テスト**:

- [x] `test_return_type_inference_int`: `fn double(n: Int) = n * 2` で `double(5)` → `Int(10)`
- [x] `test_return_type_inference_string`: `fn greet(name: String) = $"Hello {name}!"` が型チェックを通る
- [x] `test_return_type_inference_bool`: `fn is_adult(age: Int) = age >= 18` で `is_adult(20)` → `Bool(true)`

### `src/middle/checker.rs`

- [x] `test_e072_destructure_bind_non_record`: `bind { x } <- 42` → E072
- [x] `test_e073_destructure_bind_missing_field`: `bind { x, y } <- Point { x: 1 }` → E073（y が存在しない）

---

## Phase 5 — 最終確認・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.2.0 の 567 → 579）
- [x] `fav run` で `bind { x, y } <- pt` が動くことを確認
- [x] `fav run` で `fn double(n: Int) = n * 2` が動くことを確認
- [x] `fav check` で E072 / E073 / E074 が適切に出ることを確認

### ドキュメント作成

- [x] `versions/v2.3.0/langspec.md` を作成
  - [x] 分割 bind 構文（pun / alias / wildcard）の説明
  - [x] 分割 bind の脱糖ルール（中間変数 `$tmp` の説明）
  - [x] 戻り型推論の構文と制約（再帰不可、エフェクト別途明示）
  - [x] E072 / E073 / E074 エラーコードの説明と使用例
  - [x] v2.2.0 との互換性（完全上位互換）

---

## 完了条件チェック

- [x] `bind { x, y } <- point` が `bind x <- point.x; bind y <- point.y` と等価に動く
- [x] `bind { age: user_age } <- user` でエイリアス束縛が動く
- [x] `bind { name, _ } <- user` でワイルドカードが動く
- [x] `bind { x } <- 42` で E072 が出る
- [x] 存在しないフィールド指定で E073 が出る
- [x] `fn double(n: Int) = n * 2` が型チェックを通り正しく実行される
- [x] `fn id(x: Int) -> Int = x` との混在が可能
- [x] 再帰関数で戻り型省略時に E074 + ヒントが出る
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.3.0"`
- [x] `versions/v2.3.0/langspec.md` 作成済み
