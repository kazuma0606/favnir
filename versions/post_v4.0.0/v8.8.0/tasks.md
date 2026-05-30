# Favnir v8.8.0 Tasks

Date: 2026-05-30
Theme: checker.fav — 非ジェネリック関数の引数数チェック（全関数スキーム化）

---

## Phase A: `make_fn_scheme_str` の変更

- [x] A-1: `if String.length(vars_csv) == 0 { ret } else { ... }` の分岐を削除
  — 常に `String.concat("forall|", ...)` を返すようにする
  — 関数本体は else ブランチの内容のみになる

---

## Phase B: `fn_to_scheme_str` の変更

- [x] B-1: `if List.length(all_vars) == 0 { type_expr_to_str(fd.ret) }` の分岐を削除
  — `all_vars = []` でも `String.join([], ",") = ""` → `make_fn_scheme_str("", ...)` を呼ぶ
  — 関数全体をフラットな `bind` チェーンに書き直す

---

## Phase C: `register_variant` の簡略化

- [x] C-1: `Some(te)` ブランチの `String.concat("forall||", ...)` 直接構築を
  `make_fn_scheme_str("", param_str, type_name)` に置き換える
  — Phase A の変更で `make_fn_scheme_str("", p, r)` が `"forall||p|r"` を返すようになるため

---

## Phase D: `infer_call` の戻り型修正

- [x] D-1: `ns == ""` ブランチの `Some(ty) => Result.ok(ty)` を変更
  — `if is_fn_scheme_str(ty) { Result.ok(fn_scheme_ret(ty)) } else { Result.ok(ty) }`
  — スキーム形式なら `fn_scheme_ret` で戻り型を抽出、そうでなければそのまま返す

- [x] D-2: `infer_call_user` の `instantiate_fn_scheme` 呼び出しを修正
  — 非ジェネリック関数（vars_str == ""）は `instantiate_fn_scheme` をスキップ
  — `Result.ok(inf_result_of(fn_scheme_ret(ty), state))` を返す
  — ジェネリック関数のみ `instantiate_fn_scheme` を呼ぶ
  — これにより checker.fav self-check で発生した E0005 誤検知を解消

---

## Phase E: テスト追加・実行

- [x] E-1: `checker_v88_tests` モジュールを `driver.rs` に追加
  — `check_errors` ヘルパーは `checker_v87_tests` と同じパターン
- [x] E-2: `nongeneric_wrong_arity_e0008` テスト追加
  — `fn add(a: Int, b: Int) -> Int` を `add(1)` で呼ぶ → E0008
- [x] E-3: `zero_param_fn_correct_call` テスト追加
  — `fn get_val() -> Int { 42 }` を `get_val()` で呼ぶ → エラーなし
- [x] E-4: `zero_param_fn_wrong_arity_e0008` テスト追加
  — `fn get_val() -> Int { 42 }` を `get_val(1)` で呼ぶ → E0008
- [x] E-5: `cargo test checker_fav` — 既存 17 件 + 新規 3 件通ること ✓
- [x] E-6: `cargo test` — 全件通ること（1128 tests）✓

---

## Phase F: 最終確認・ドキュメント

- [x] F-1: `cargo build` — コンパイルエラーなし
- [x] F-2: `checker_fav_wire_self_check` — 64MB スタックで通ること ✓
- [x] F-3: このファイルを完了状態に更新
- [ ] F-4: commit

---

## 完了条件

- 非ジェネリック関数の引数数不一致が E0008 で検出される ✓
- 0 引数関数の正しい呼び出しがエラーにならない ✓
- ジェネリック関数（v8.7.0 既存）が引き続き動く ✓
- variant コンストラクターが引き続き動く ✓
- 既存テスト全件通る ✓

---

## 実装ノート

### 変更の順序
A → B → C → D の順で変更し、D まで完了してから cargo test を実行すること。
D（`infer_call` 修正）なしだと `infer_expr` の非 HM パスが scheme 文字列を
そのまま型として返し、一部のテストが失敗する可能性がある。

### `String.join([], ",")` = `""`
型変数なしの関数で `all_vars = []` のとき `vars_csv = ""` になる。
`make_fn_scheme_str("", params, ret)` = `"forall||params|ret"` ✓

### `instantiate_fn_scheme` への影響（実装時の発見）
非ジェネリック関数のスキーム `"forall||Int|Int"` を `instantiate_fn_scheme` に渡すと、
checker.fav が自身をセルフチェックする際に E0005 誤検知が発生した。
原因: `infer_arg_tys` がフィールドアクセス等の複雑な式を `"Unknown"` と推論し、
`unify("List<KVPair>", "Unknown", subst)` が E0005 を出す。

対処: `infer_call_user` で `vars_str == ""` の場合（非ジェネリック）は
`instantiate_fn_scheme` をスキップし、`fn_scheme_ret(ty)` を直接返す。
これにより:
- アリティチェック（E0008）は実施される ✓
- 型変数のない関数で不要な unification が起きない ✓
- generic 関数は引き続き `instantiate_fn_scheme` を呼ぶ ✓
