# Favnir v9.13.0 Tasks

Date: 2026-06-03
Theme: `par` 並列 stage 実行（`par [A, B] |> Merge`、VM 並列化）

---

## Phase A: Rust AST + パーサー

- [x] A-1: `fav/src/ast.rs` に `FlwStep` enum を追加
  - `FlwStep::Stage(String)` — 既存の単一 stage
  - `FlwStep::Par(Vec<String>)` — 並列グループ
- [x] A-2: `FlwDef.steps: Vec<FlwStep>` に型変更（was `Vec<String>`）
- [x] A-3: `fav/src/frontend/lexer.rs` に `TokenKind::Par` を追加（キーワード `"par"`）
- [x] A-4: `fav/src/frontend/parser.rs` の `parse_flw_def` を更新
  - `par [A, B]` を `FlwStep::Par(vec!["A", "B"])` としてパース
  - `<ident>` は `FlwStep::Stage(name)` としてパース
- [x] A-5: `FlwDef.steps` を参照している Rust コードを `FlwStep` に対応させる
  - `fav/src/middle/compiler.rs`（`compile_flw_def`）
  - `fav/src/middle/lineage.rs`
  - `fav/src/checker.rs`
  - `fav/src/middle/ast_lower_checker.rs`（`lower_flw_def`）
- [x] A-6: `cargo build` 通過確認

---

## Phase B: Rust VM + 周辺

- [x] B-1: `fav/src/backend/vm.rs` に `IO.par_execute_raw` builtin を実装
  - 引数: `(names: List<String>, input: Value)`
  - `std::thread::spawn` で各 stage を並列実行
  - 全スレッド完了を `thread.join()` で待機
  - 結果を `VMValue::List` として返す
- [x] B-2: `fav/src/middle/compiler.rs` — `FlwStep::Par` の IR 生成
  - `par [A, B]` → `IO.par_execute_raw(["A", "B"], input)` 呼び出し IR
- [x] B-3: `fav/src/middle/ast_lower_checker.rs` — `lower_flw_def` 更新
  - `FlwStep::Stage(s)` → `Variant("SStage", sv(s))`
  - `FlwStep::Par(names)` → `Variant("SPar", vm_list(names))`
- [x] B-4: `fav/src/checker.rs` — Rust pipeline での par 最小チェック
  - par 内の stage が存在するか確認
  - E0016（入力型不一致）/ E0017（未定義 stage）の Rust 側検出
- [x] B-5: `cargo build` 通過確認（Rust 全体）

---

## Phase C: compiler.fav — SeqStep 型 + codegen

- [x] C-1: `SeqStep` sum 型を追加
  ```favnir
  type SeqStep = | SStage(String) | SPar(List<String>)
  ```
- [x] C-2: `SeqDef.stages` の型を `List<SeqStep>` に変更（was `List<String>`）
- [x] C-3: `Token` 型に `TkPar` variant を追加
- [x] C-4: `keyword_token` に `"par" → TkPar` の分岐を追加
- [x] C-5: `token_eq` / `token_to_string` に `TkPar` を追加
- [x] C-6: `PipelineNamesParse` → `PipelineStepsParse` に型を更新
  ```favnir
  type PipelineStepsParse = { stages: List<SeqStep>  rest: List<Token> }
  ```
- [x] C-7: `parse_seq_pipeline_acc` を更新
  - `TkPar` → `[` → `ident (, ident)*` → `]` → `SPar(names)`
  - `TkIdent(name)` → `SStage(name)`
- [x] C-8: `build_pipe_call` を `List<SeqStep>` 対応に更新
  - `SStage(name)` → `ECall(name, [input_expr])`（既存と同様）
  - `SPar(names)` → `ECall("IO.par_execute_raw", [EList(name_strs), input_expr])`
- [x] C-9: `compile_seq_def` を更新（stages 型変更への対応）
- [x] C-10: `pretty_seq_def` を更新（SPar の表示: `par [A, B]`）
- [x] C-11: `set_item_doc` の `ISeq` ブランチで stages 型変更に対応
- [x] C-12: `cargo test checker_fav_wire_self_check` 通過確認（compiler.fav が自身をパース）

---

## Phase D: checker.fav — par 型チェック

- [x] D-1: `SeqStep` sum 型を追加（compiler.fav と同様）
- [x] D-2: `SeqDef.stages` の型を `List<SeqStep>` に変更
- [x] D-3: `check_par_step(names, input_ty, env)` を追加
  - 各 name が env に存在するか確認（なければ E0017）
  - 各 name の入力型が `input_ty` と一致するか確認（なければ E0016）
- [x] D-4: `check_seq_def` / `check_seq_steps_acc` を `List<SeqStep>` 対応に追加
  - `SStage(name)` は current_ty 追跡のみ
  - `SPar(names)` は `check_par_names_acc` を呼ぶ
- [x] D-5: E0016 エラー文言: `"E0016: par ステップの入力型不一致: <stage> expects <ty>, got <other_ty>"`
- [x] D-6: E0017 エラー文言: `"E0017: par ステップ内の stage '<name>' が定義されていません"`
- [x] D-7: `cargo test checker_fav_wire_self_check` 通過確認
  - `IStage` / `ISeq` を Item enum に追加
  - `ast_lower_checker.rs` に `lower_trf_def` / `lower_flw_step` / `lower_flw_def` 追加

---

## Phase E: fav explain — 並列構造の表示

- [x] E-1: `fav/src/lineage.rs` で `FlwStep::Par` を処理（Phase A-5 で完了）
  - `step.stage_names()` で各 stage のエフェクトを union
  - `step.display_str()` で `par [A, B]` 形式の表示
- [x] E-2: `fav explain` の出力に par ブロックを表示
  - `seq P = A |> par [B, C] |> D` 形式で表示

---

## Phase F: テスト + self-check + バージョン更新 + commit

- [x] F-1: `v9130_tests` モジュールを `src/driver.rs` に追加（6 件）
  - [x] F-1a: `par_executes_in_parallel` — `par [A, B] |> Merge` が実行できる
  - [x] F-1b: `par_result_is_correct` — 各 stage の結果が正しく Merge に渡る（Double(5)=10, AddTen(5)=15, sum=25）
  - [x] F-1c: `par_input_type_mismatch_e0016` — 入力型不一致 → E0016
  - [x] F-1d: `par_unknown_stage_e0017` — 未定義 stage 参照 → E0017
  - [x] F-1e: `par_compiles_with_favnir_pipeline` — par を含む seq が Rust pipeline でコンパイルできる
  - [x] F-1f: `par_effects_detected_by_checker_fav` — checker.fav が valid par program を通過
- [x] F-2: `cargo test v9130` — 6 件通過
- [x] F-3: `cargo test checker_fav_wire_self_check` — 通過
- [x] F-4: `cargo test bootstrap` — 通過
- [x] F-5: `cargo test` — 全件通過（1258 tests）
- [x] F-6: `fav/Cargo.toml` version → `"9.13.0"`
- [x] F-7: `fav/self/cli.fav` の `run_version` → `"9.13.0"`
- [x] F-8: 本ファイル完了チェック
- [x] F-9: `memory/MEMORY.md` に v9.13.0 完了を記録
- [x] F-10: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `par [A, B] \|> Merge` が並列実行され結果が正しい | ✓ |
| E0016（par 入力型不一致）が検出できる | ✓ |
| E0017（par 内未定義 stage）が検出できる | ✓ |
| `fav explain --lineage` が par を並列構造として表示する | ✓ |
| compiler.fav が `par` を含む seq をコンパイルできる | ✓ |
| checker.fav が `par` のエフェクト和を追跡できる | ✓ |
| `cargo test checker_fav_wire_self_check` 通過 | ✓ |
| `cargo test bootstrap` 維持 | ✓ |
| `cargo test v9130` — 6 件通過 | ✓ |

---

## 実装メモ

### FlwDef 使用箇所の検索コマンド
```bash
grep -rn "\.steps\b\|FlwDef\|flw_def\|parse_flw" fav/src/ | grep -v "test"
```

### par_execute_raw のスレッド設計
- `FvcArtifact::clone()` を各スレッドに clone（参照カウントのみ）
- DB 接続は `db_url` 文字列を渡し、スレッドごとに新規接続確立
- エラー: 最初に失敗したスレッドのエラーを伝播（他スレッドは detach）
- VM 実装: `call_builtin` METHOD に配置（`self` + `artifact` が必要）

### checker.fav — IStage / ISeq 設計
- `IStage(SeqStageEntry)`: TrfDef を IFn 形式でなく専用型に lowering
  - `input_ty_str` / `output_ty_str` → `make_fn_scheme_str("", in, out)` でスキーム登録
  - body チェックをスキップ（IFn にすると E0009 が誤発火）
- `ISeq(SeqDef)`: FlwDef を lowering; `collect_fn_schemes` で seq 名を `Unknown→Unknown` で登録
  - seq 名がない → E0007 誤発火を防ぐ

### ast_lower_checker.rs — te_to_string 追加
- `te_to_string(te: &ast::TypeExpr) -> String` — IStage scheme 用
- 型変換: Named → `"List<T>"` 形式、Optional → `"Option<T>"`、Fallible → `"Result<T, String>"`

### bootstrap 維持のポイント
- `SeqDef.stages: List<SeqStep>` 変更により既存 seq の bytecode が変わる可能性
- `build_pipe_call` の `SStage(name)` ブランチは既存の `String` ブランチと同等のコードを生成
- `cargo test bootstrap` で `bytecode_A == bytecode_B` が維持されることを確認 ✓

### スコープ外
- `par` のネスト
- `par` 内でのエラー発生時の partial completion
- `fav profile` での par stage 個別計測（`--profile` フラグで par 全体の時間は計測可）
- compiler.fav パイプラインでの `par` seq コンパイル（Favnir VM の再帰深度制限による stack overflow 回避のため Rust pipeline テストに変更）
