# Favnir v11.2.0 Tasks

Date: 2026-06-06
Theme: stage / seq → Python パイプライン変換

---

## Phase A — `to_snake` ユーティリティ

- [x] A-1: `emit_python.rs` の Util セクションに `to_snake(name: &str) -> String` 追加
  - 大文字の前に `_` を挿入（連続大文字 `IOHelper` → `io_helper` 対応）
  - 全小文字化

---

## Phase B — `TrfDef` 変換（stage）

- [x] B-1: `emit_trf_def(&mut self, td: &ast::TrfDef)` メソッド追加
  - エフェクトコメント出力
  - `TrfDef.params.first()` から引数名を取得（なければ `x`）
  - `input_ty` / `output_ty` を `map_type` で変換
  - ステージ名を `to_snake` で変換
  - `def name(param: A) -> B:` → `emit_block_body`
- [x] B-2: `emit_program` の TrfDef アームを `emit_trf_def(td)` に切り替え（TODO コメント削除）

---

## Phase C — `FlwDef` 変換（seq）

- [x] C-1: `build_chain_expr(&self, input: &str, steps: &[FlwStep]) -> String` 追加
  - シンプルステージは `stage(prev)` を順にネスト
- [x] C-2: `emit_flw_with_par(&mut self, fn_name: String, steps: &[FlwStep])` 追加
  - `ThreadPoolExecutor` を使った並列実行コード生成
- [x] C-3: `emit_flw_def(&mut self, fd: &ast::FlwDef)` メソッド追加
  - par ステップがあれば `emit_flw_with_par`、なければ `build_chain_expr` でシンプルチェーン
  - seq 名を `to_snake` で変換
- [x] C-4: `emit_program` の FlwDef アームを `emit_flw_def(fd)` に切り替え（TODO コメント削除）

---

## Phase D — `IO.argv()` 正式変換 + `fn main()` ガード

- [x] D-1: `emit_apply` の IO セクションに `"argv"` → `sys.argv[1:]` を追加
- [x] D-2: `emit_program` に `has_main` フラグを追加
  - `fn main` を検出したら `has_main = true`
  - イテレーション完了後、`has_main` なら末尾に `if __name__ == "__main__": main()` を出力

---

## Phase E — テスト

- [x] E-1: `v11200_tests` モジュール追加（8 件）
  - [x] `transpile_stage_basic`
  - [x] `transpile_stage_effects_comment`
  - [x] `transpile_stage_multiline_body`
  - [x] `transpile_seq_two_stages`
  - [x] `transpile_seq_three_stages`
  - [x] `transpile_seq_snake_case`
  - [x] `transpile_main_guard`
  - [x] `transpile_io_argv`
- [x] E-2: `cargo test v11200 --lib` — 8 件通過
- [x] E-3: `cargo test --lib` — 691 件全件通過

---

## Phase F — バージョン更新 + コミット

- [x] F-1: `fav/Cargo.toml` version → `"11.2.0"`（その後 v11.2.5 で `11.2.5` に更新）
- [x] F-2: `fav/self/cli.fav` version 文字列 → `"11.2.0"`
- [x] F-3: コミット & プッシュ

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `stage Foo: A -> B = \|x\| { ... }` → `def foo(x: A) -> B:` | ✓ |
| `seq P = A \|> B \|> C` → `def p(x): return c(b(a(x)))` | ✓ |
| `fn main()` → `if __name__ == "__main__": main()` 付き | ✓ |
| `IO.argv()` → `sys.argv[1:]` | ✓ |
| `cargo test v11200 --lib` 8 件通過 | ✓ |
| `cargo test --lib` 全件通過（691 件） | ✓ |

> 完了: 2026-06-06
