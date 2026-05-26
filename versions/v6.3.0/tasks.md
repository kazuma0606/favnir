# Favnir v6.3.0 Tasks

Date: 2026-05-26

## Goal

`compiler.fav` が `stage` / `seq` / `|>` 構文を処理できるようにする。
Bootstrap 生成物（compiler_artifact）が stage/seq を含むプログラムを
Rust コンパイラと同一バイトコードでコンパイルできることを検証する。

## Phase A — Lexer 拡張

- [x] A-1: `Token` 型に `TkStage` / `TkSeq` / `TkAbstract` を追加
- [x] A-2: `keyword_token` に "stage" / "seq" / "abstract" のエントリを追加
- [x] A-3: `scan_op` で `|>` → `TkPipeGt` を認識（v6.1.0 時点で実装済み）
- [x] A-4: `self_hosted_compiler_type_checks` テストが通ること

## Phase B — AST 型追加

- [x] B-1: `StageDef` 型を追加（is_public / is_abstract / name / param_ty / ret_ty / effects / body）
- [x] B-2: `SeqDef` 型を追加（is_public / is_abstract / name / stages）
- [x] B-3: `Item` 型に `IStage(StageDef)` / `ISeq(SeqDef)` を追加

## Phase C — Parser 追加

- [x] C-1: `parse_effects_acc` ヘルパー実装（`!Ident` の繰り返しを収集）
- [x] C-2: `parse_stage_def` 実装（`stage Name: InType -> OutType !Eff = |p| { body }`）
- [x] C-3: `parse_abstract_stage` 対応（body = None）
- [x] C-4: `parse_seq_pipeline_acc` ヘルパー実装（`Name |> Name |> Name` のリスト収集）
- [x] C-5: `parse_seq_def` 実装（`seq Name = A |> B |> C`）
- [x] C-6: `parse_abstract_seq` 対応（stages = []）
- [x] C-7: `parse_item` に `TkStage` / `TkSeq` / `TkAbstract` の分岐を追加

## Phase D — Codegen 追加

- [x] D-1: `compile_stage_def` 実装（ELambda 抽出 → FnDef に変換して compile_fn_def に委譲）
- [x] D-2: `build_pipe_call` ヘルパー実装（stages リスト → ネストした ECall、$input 使用）
- [x] D-3: `compile_seq_def` 実装（SeqDef → $input パラメータの FnDef に展開）
- [x] D-4: `compile_items` に `IStage` / `ISeq` の分岐を追加

## Phase E — テストと検証

- [x] E-1: `fav/tmp/pipeline_test.fav` を作成（Double |> AddOne を含む最小パイプライン）
- [x] E-2: `fav run fav/tmp/pipeline_test.fav` で DoubleThenAdd(5) == 11 を確認
- [x] E-3: `compiler.fav` に 9 件のインライン tests を追加（lex/parse/compile）
- [x] E-4: `fav test self/compiler.fav` 12 テスト全通過
- [x] E-5: `self_tests.rs` に `bootstrap_stage_seq_self_host_executes_correctly` を追加
- [x] E-6: `cargo test bootstrap_stage_seq_self_host_executes_correctly` 通過
- [x] E-7: `cargo test` 全テスト通過（1033 件、v6.2.0 比 +1）
- [x] E-8: このファイルを完了状態に更新

## Recommended execution order

1. Phase A（Lexer）→ 2. Phase B（AST）→ 3. Phase C（Parser）→ 4. Phase D（Codegen）→ 5. Phase E（テスト）

各フェーズ完了後に `fav check fav/self/compiler.fav` を実行して型エラーがないことを確認する。

## 完了条件まとめ

- `compiler.fav` が `stage` / `seq` / `|>` を含むプログラムをコンパイルできる
- `compiler_artifact` の出力が Rust コンパイラの出力と一致する
- 既存 Bootstrap テストがすべて通る
- `cargo test` 全テスト通過
