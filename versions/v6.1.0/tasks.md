# Favnir v6.1.0 タスクリスト — セルフホスト完全実装

作成日: 2026-05-21

## 概要

v6.0.0 でセルフホストのインフラ（骨格・型チェック・ブートストラップパイプライン）が完成した。
v6.1.0 では `compiler.fav` のスタブを本実装に差し替え、Favnir で書いたコンパイラが
実際にバイトコードを生成できることを検証する。

完了条件：
- `fav run fav/self/compiler.fav -- fav/tmp/hello.fav` が非ゼロのバイト列を出力する
- Stage 1 (Rust VM でコンパイル) と Stage 2 (Favnir コンパイラでコンパイル) の出力が一致する

---

## Phase A: lexer.fav の統合

compiler.fav のスタブ `lex_src` を lexer.fav の本実装に差し替える。
Favnir はファイル間インポート不可のため、lexer.fav の全関数を compiler.fav にインライン化する。

- [x] A-1: `compiler.fav` の既存 `Token` 型定義が `lexer.fav` と一致することを確認
- [x] A-2: `lexer.fav` から全ヘルパー関数（`is_digit`/`is_alpha`/`is_whitespace`/`scan_*`/`keyword_token`/`scan_op`/`scan`/`lex`）を `compiler.fav` にコピー
- [x] A-3: `lex_src` スタブを削除し、`lex` 関数を直接呼び出すように変更
- [x] A-4: `fav check fav/self/compiler.fav` エラーなし確認
- [x] A-5: `fav run fav/self/compiler.fav -- fav/tmp/hello.fav` でレキサーが動くことを確認（"Lex error" が出なければ OK）

---

## Phase B: parser.fav の統合

compiler.fav のスタブ `parse_src` を parser.fav の本実装に差し替える。

- [x] B-1: `compiler.fav` の既存 AST 型定義が `parser.fav` と一致することを確認・差分修正
- [x] B-2: `parser.fav` から全パーサー関数（`ParseState`/`peek`/`advance`/`expect`/`parse_expr`/`parse_if`/`parse_match`/`parse_block`/`parse_fn_def`/`parse_type_def`/`parse`）を `compiler.fav` にコピー
- [x] B-3: `parse_src` スタブを削除し、`parse` 関数を直接呼び出すように変更
- [x] B-4: `fav check fav/self/compiler.fav` エラーなし確認
- [x] B-5: `fav run fav/self/compiler.fav -- fav/tmp/hello.fav` でパーサーが動くことを確認（"Parse error" が出なければ OK）

---

## Phase C: codegen.fav の統合

compiler.fav のスタブ `compile_prog` を codegen.fav の本実装に差し替える。

- [x] C-1: `compiler.fav` の既存 Bytecode 型定義が `codegen.fav` と一致することを確認・差分修正
- [x] C-2: `codegen.fav` から全コードジェン関数（`bc_byte`/`u16_lo`/`u16_hi`/`emit_*`/`add_const`/`compile_lit`/`compile_binop`/`compile_if_*`/`compile_expr`/`compile_fn_def`/`compile`）を `compiler.fav` にコピー
- [x] C-3: `compile_prog` スタブを削除し、`compile` 関数を直接呼び出すように変更
- [x] C-4: `fav check fav/self/compiler.fav` エラーなし確認
- [x] C-5: `fav run fav/self/compiler.fav -- fav/tmp/hello.fav` で非ゼロのバイト列が出力されることを確認

---

## Phase D: ブートストラップ検証（真のセルフホスト確認）

- [x] D-1: `fav run fav/self/compiler.fav -- fav/tmp/hello.fav` の出力バイト列を保存（Stage 2 出力）
- [x] D-2: Rust VM で `fav/tmp/hello.fav` をコンパイルしたバイト列と比較（diff ゼロ確認）
- [x] D-3: `bootstrap_stage1_output_is_deterministic` テストが非空出力で一致することを確認
- [x] D-4: driver.rs に `bootstrap_stage1_compiles_hello_fav_correctly` テストを追加
  - Stage 1 で hello.fav をコンパイルした出力バイト列を確認
  - Rust VM のバイト列出力と一致することをアサート
- [x] D-5: `cargo test` 全件通過確認

---

## Phase E: まとめ

- [x] E-1: `cargo test` 全件通過 (995 tests)
- [x] E-2: `versions/v6.1.0/tasks.md` にチェックを入れる
- [x] E-3: `MEMORY.md` を更新
- [x] E-4: `feat: self-hosting complete — Favnir compiler generates real bytecode (v6.1.0)` でコミット
