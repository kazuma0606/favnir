# Favnir v8.9.0 Tasks

Date: 2026-05-30
Theme: checker.fav — 未定義変数検出（E0001）

---

## Phase A: `infer_hm` EVar None ケース変更

- [x] A-1: `infer_hm` の `EVar None` ブランチを変更
  — `fresh_var` による fresh type var 生成を削除
  — `Result.err(fmt_err("E0001", String.concat("undefined variable: ", name)))` に変更

---

## Phase B: テスト追加

- [x] B-1: `checker_v89_tests` モジュールを `driver.rs` に追加
  — `check_errors` ヘルパーは `checker_v87_tests` と同じパターン
- [x] B-2: `undefined_var_e0001` テスト追加
  — `public fn main() -> Int { x }` → E0001
- [x] B-3: `fn_param_not_e0001` テスト追加
  — `public fn main(n: Int) -> Int { n }` → エラーなし
- [x] B-4: `let_bound_not_e0001` テスト追加
  — `bind r <- add(1, 2)` 後に `r` を参照 → エラーなし

---

## Phase C: テスト実行・確認

- [x] C-1: `cargo test checker_v89` — 新規 3 件通ること ✓
- [x] C-2: `cargo test checker_fav` — 既存 17 件 + 新規 3 件 = 20 件通ること ✓
- [x] C-3: `cargo test` — 全件通ること（1131 tests）✓

---

## Phase D: 最終確認・ドキュメント

- [x] D-1: `cargo build` — コンパイルエラーなし
- [x] D-2: `checker_fav_wire_self_check` — 64MB スタックで通ること ✓
- [x] D-3: このファイルを完了状態に更新
- [ ] D-4: commit

---

## 完了条件

- 未定義変数の参照が E0001 で検出される ✓
- 関数パラメータの参照がエラーにならない ✓
- bind 変数の参照がエラーにならない ✓
- checker.fav 自身の self-check が通る ✓
- 既存テスト全件通る ✓

---

## 実装ノート

### E0001 が適用されるスコープ

`infer_hm` パスで評価される EVar のみ。具体的には:
- 関数ボディの bind チェーン（`infer_hm_let` → `infer_hm`）
- EIf の then/else ブランチ（`infer_hm` で再帰）
- ECall の直接処理（`infer_call_hm` → 中ではなく引数は `infer_arg_tys`）

E0001 が適用されない（現バージョン）:
- match アームボディ（`infer_expr` パス → `env_from_pat` で変数追加済み）
- lambda 本体（`infer_expr` パス）
- 関数呼び出しの引数（`infer_arg_tys` → `infer_expr` パス）
- EIf の condition（`infer_hm` で処理されない）

### `fresh_var` の扱い

`fresh_var` 関数は削除しない。HM テスト (`checker_fav_fresh_var`) で参照されている。

### self-check 安全性の根拠

checker.fav 内のすべての EVar は、`infer_hm` パスで評価される時点で
必ず env に登録されている。具体的には:
- 関数パラメータ → `build_param_env` で登録
- bind 変数 → `infer_hm_let` で `env_insert`
- 関数名（再帰呼び出し含む）→ `collect_fn_schemes` で登録
- variant コンストラクター → `collect_variant_constructors` で登録
