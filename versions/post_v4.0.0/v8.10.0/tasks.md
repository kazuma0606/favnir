# Favnir v8.10.0 Tasks

Date: 2026-05-30
Theme: `else if` 構文サポート + checker.fav — 関数戻り型チェック（E0009）

---

## Phase A: パーサーに `else if` 対応追加

- [x] A-1: `parse_if_expr` の else ブランチを変更
  — `else` 後に `If` トークンがある場合: `parse_if_expr()` を再帰呼び出しして Block でラップ
  — `else { if ... }` への desugar として実装
  — 変更ファイル: `fav/src/frontend/parser.rs`

---

## Phase B: checker.fav ヘルパー関数追加

- [x] B-1: `outer_type(s: String) -> String` を追加
  — `String.split(s, "<")` の先頭要素を返す
  — 配置: `is_type_var_extended` の直後、`list_dedup_inner` の前
- [x] B-2: `types_compatible(inferred: String, declared: String) -> Bool` を追加
  — `else if` 構文を使って4段階の互換チェックを実装
  — exact match → "Unknown" → `is_type_var_extended` → `outer_type` 比較

---

## Phase C: checker.fav `check_fn_def` 更新

- [x] C-1: `check_body_ty(fname, ret, r)` ヘルパー関数を追加
  — `check_fn_def` 直前に配置
  — `types_compatible(apply_subst(r.subst, r.ty), type_expr_to_str(ret))` で互換チェック
  — 非互換なら `Result.err(fmt_err("E0009", ...))`
  — ラムダボディ制約（`|r| if ...` が parse できない）を回避するため別関数化
- [x] C-2: `check_fn_def` の `Result.ok(fd.name)` を `check_body_ty(fd.name, fd.ret, r)` に変更

---

## Phase D: テスト追加

- [x] D-1: `checker_v810_tests` モジュールを `driver.rs` に追加
- [x] D-2: `return_type_mismatch_e0009` テスト追加
  — `fn bad() -> Int { "hello" }` → E0009
- [x] D-3: `return_type_correct_literal` テスト追加
  — `public fn main() -> Int { 42 }` → エラーなし
- [x] D-4: `return_type_correct_call` テスト追加
  — `fn double(x: Int) -> Int { x + x }` 呼び出し → エラーなし

---

## Phase E: テスト実行・確認

- [x] E-1: `cargo test checker_v810` — 新規 3 件通ること ✓
- [x] E-2: `cargo test checker_fav` — 全 17 件通ること（self-check 含む）✓
- [x] E-3: `cargo test` — 全件通ること（1134 tests）✓

---

## Phase F: 最終確認・ドキュメント

- [x] F-1: `cargo build` — コンパイルエラーなし
- [x] F-2: `checker_fav_wire_self_check` — 64MB スタックで通ること ✓
- [x] F-3: このファイルを完了状態に更新
- [ ] F-4: commit

---

## 完了条件

- `else if` 構文が Favnir パーサーで使えるようになった ✓
- 宣言戻り型と推論型の明らかな不一致が E0009 で検出される ✓
- 正しい戻り型の関数がエラーにならない ✓
- checker.fav 自身の self-check が通る ✓
- 既存テスト全件通る ✓

---

## 実装ノート

### `else if` の実装方針

`parse_if_expr` で `else` 消費後に次のトークンが `If` なら:
```rust
let if_expr = self.parse_if_expr()?;  // 再帰
Some(Box::new(Block { stmts: vec![], expr: Box::new(if_expr), span }))
```
`else { if ... }` と同等の AST を生成する。

### ラムダボディ制約

Favnir パーサーはラムダボディ（`|x| EXPR`）として `if` 式を直接受け付けない。
`check_body_ty` ヘルパー関数を使い、ラムダボディを単純な関数呼び出し
`|r| check_body_ty(fd.name, fd.ret, r)` にすることで回避。

### `types_compatible` の判定ロジック

| ケース | 判定 |
|---|---|
| `inferred == declared` | 互換（exact match） |
| `inferred == "Unknown"` | 互換（推論不可→スキップ） |
| `is_type_var_extended(inferred)` | 互換（型変数→不確定） |
| `outer_type(inferred) == outer_type(declared)` | 互換（"List" vs "List\<X\>" 等） |
| それ以外 | 非互換 → E0009 |

### `type_expr_to_str` のベア型（self-check 安全性）

`TeResult(a, b) = "Result"`, `TeMap(k, v) = "Map"` のため、
Result や Map を返す関数は宣言・推論ともにベア型で一致 → false positive なし。
