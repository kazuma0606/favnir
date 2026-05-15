# Favnir v0.5.0 タスク一覧

更新日: 2026-04-29

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Lexer / AST

### 新キーワード (Lexer)

- [x] 1-1: `TokenKind::Chain` を追加し `"chain"` にマップする
- [x] 1-2: `TokenKind::Yield` を追加し `"yield"` にマップする
- [x] 1-3: `TokenKind::Collect` を追加し `"collect"` にマップする
- [x] 1-4: `TokenKind::Where` を追加し `"where"` にマップする
- [x] 1-5: `test_keywords` に `chain yield collect where` を追加する

### `Effect::Trace` (AST)

- [x] 1-6: `ast.rs` の `Effect` 列挙体に `Trace` バリアントを追加する
- [x] 1-7: `Effect::display()` に `Trace → "Trace"` のアームを追加する
- [x] 1-8: `parse_effect` で `"Trace"` → `Effect::Trace` を認識する

### `Stmt` の拡張 (AST)

- [x] 1-9: `Stmt::Chain { name: String, expr: Expr, span: Span }` を追加する
- [x] 1-10: `Stmt::Yield { expr: Expr, span: Span }` を追加する
- [x] 1-11: `Stmt::span()` に `Chain` / `Yield` のアームを追加する
- [x] 1-12: `Stmt` を参照している `match` が網羅的であることを確認する（checker / eval に `_` アームを追加）

### `Expr::Collect` (AST)

- [x] 1-13: `Expr::Collect(Box<Block>, Span)` を追加する
- [x] 1-14: `Expr::span()` に `Collect` のアームを追加する
- [x] 1-15: `Expr` を参照している `match` が網羅的であることを確認する

### `MatchArm.guard` (AST)

- [x] 1-16: `MatchArm` に `guard: Option<Box<Expr>>` フィールドを追加する
- [x] 1-17: 既存の `MatchArm` 構築箇所を全て `guard: None` に更新する

---

## Phase 2: Parser

### `chain` / `yield` のパース

- [x] 2-1: `parse_stmt` で `TokenKind::Chain` → `parse_chain_stmt()` を呼ぶ
- [x] 2-2: `parse_chain_stmt` を実装する（セミコロン消費なし）
- [x] 2-3: `parse_stmt` で `TokenKind::Yield` → `parse_yield_stmt()` を呼ぶ
- [x] 2-4: `parse_yield_stmt` を実装する（セミコロン必須）

### `collect` のパース

- [x] 2-5: `parse_primary` で `TokenKind::Collect` → `parse_collect_expr()` を呼ぶ
- [x] 2-6: `parse_collect_expr` を実装する（`collect block`）

### `pipe match` のデシュガー

- [x] 2-7: `parse_pipe_expr` で `|>` の後に `TokenKind::Match` が来た場合のデシュガーを実装する
  - 左辺を subject として、`match subject { arms }` の `Expr::Match` を生成する

### パターンガードのパース

- [x] 2-8: `parse_match_arm` でパターン後に `TokenKind::Where` があれば guard 式をパースする
- [x] 2-9: guard 式をパースして `MatchArm.guard` にセットする

### パーサの単体テスト

- [x] 2-10: `test_parse_chain_stmt` — `chain n <- some_fn(x)` がパースできる
- [x] 2-11: `test_parse_yield_stmt` — `yield expr;` がパースできる
- [x] 2-12: `test_parse_collect_expr` — `collect { yield 1; yield 2; }` がパースできる
- [x] 2-13: `test_parse_match_guard` — `x where x > 0 => x` がパースできる
- [x] 2-14: `test_parse_pipe_match` — `n |> match { ok(v) => v  err(_) => 0 }` がパースできる

---

## Phase 3: Checker

### 新フィールド

- [x] 3-1: `Checker` に `chain_context: Option<Type>` を追加する
- [x] 3-2: `Checker` に `in_collect: bool` を追加する
- [x] 3-3: `new()` / `new_with_resolver()` に初期値を追加する

### `check_fn_def` の更新

- [x] 3-4: `check_fn_def` で戻り型から `chain_context` をセットする
  - `Option<T>` / `Named("Option", ...)` → Some(return_ty)
  - `Result<T,E>` / `Named("Result", ...)` → Some(return_ty)
  - その他 → None
- [x] 3-5: `check_fn_def` 終了時に `chain_context = None` にリセットする

### `Stmt::Chain` の検査

- [x] 3-6: `check_stmt` で `Stmt::Chain` を処理する
  - `chain_context == None` → E024
  - `expr_ty` が chain コンテキストと不一致 → E025
  - `x` の型 = inner type として env に登録
- [x] 3-7: `check_chain_expr_type` ヘルパーを実装する
  - `Result<T, E>` から `T` を取り出す
  - `Option<T>` から `T` を取り出す
  - エラー型の不一致（E → E'）は unify して確認

### `Stmt::Yield` の検査

- [x] 3-8: `check_stmt` で `Stmt::Yield` を処理する
  - `in_collect == false` → E026
  - `expr` の型を返す（collect 側で収集）

### `Expr::Collect` の検査

- [x] 3-9: `check_expr` で `Expr::Collect` を処理する
  - `in_collect = true` にセットしてブロックをチェック
  - `yield` の型を全て unify して要素型を決める
  - `Type::List(elem_ty)` を返す
  - 終了後 `in_collect` をリセット
- [x] 3-10: クロージャ評価時に `in_collect = false` にリセットする（クロージャ内の yield は E026）

### パターンガードの検査

- [x] 3-11: `check_match_arm` でガードが存在する場合 `check_expr(guard)` を呼ぶ
- [x] 3-12: ガード型が `Bool` でない場合 E027 を報告する

### `!Trace` effect の検査

- [x] 3-13: `require_trace_effect` メソッドを追加する（E010）
- [x] 3-14: `Trace.*` アクセス時に `require_trace_effect` を呼ぶ
- [x] 3-15: `register_builtins` で `Trace` 型名前空間を env に追加する

### Checker の単体テスト

- [x] 3-16: `test_chain_result_ok` — Result コンテキストで `chain` が通る
- [x] 3-17: `test_chain_option_ok` — Option コンテキストで `chain` が通る
- [x] 3-18: `test_chain_outside_context` — E024 が出る
- [x] 3-19: `test_chain_type_mismatch` — E025 が出る（chain 式の型不一致）
- [x] 3-20: `test_yield_outside_collect` — E026 が出る
- [x] 3-21: `test_collect_type` — `collect { yield 1; yield 2; }` の型が `List<Int>`
- [x] 3-22: `test_guard_non_bool` — E027 が出る

---

## Phase 4: 評価器

### `EvalResult` の導入

- [x] 4-1: `eval.rs` に `EvalResult { Value(Value), Escape(Value) }` を追加する
  - 実装: `RuntimeError.escape: Option<Value>` フィールドで代替
- [x] 4-2: `eval_block` の返り値を `EvalResult` に変更する
  - 実装: `?` 演算子で escape が自動伝播
- [x] 4-3: 既存の `eval_block` 呼び出し箇所を全て `EvalResult::Value` に対応させる
  - 実装: 既存コードは変更不要（Result の ? チェーンで伝播）

### `Stmt::Chain` の評価

- [x] 4-4: `eval_block` で `Stmt::Chain` を処理する
  - 評価結果が `ok(v)` / `some(v)` → `x = v` で継続
  - 評価結果が `err(e)` / `none` → `EvalResult::Escape(v)` を返す

### 関数境界での `Escape` キャッチ

- [x] 4-5: `eval_call` で `EvalResult::Escape(v)` を受け取ったら `v` を関数の返り値にする
- [x] 4-6: クロージャ呼び出しでも同様に対応する

### `COLLECT_STACK` と `Stmt::Yield` の評価

- [x] 4-7: `COLLECT_STACK` スレッドローカルを追加する
- [x] 4-8: `collect_push_frame` / `collect_yield` / `collect_pop_frame` を実装する
- [x] 4-9: `Stmt::Yield { expr }` の評価を実装する（`collect_yield` を呼ぶ）

### `Expr::Collect` の評価

- [x] 4-10: `eval_expr` で `Expr::Collect(block)` を処理する
  - `collect_push_frame()`
  - `eval_block(block, env)` → `Escape` が来たらフレームをポップして伝播
  - `collect_pop_frame()` → `Value::List(items)` を返す

### `Trace.print` / `Trace.log` の実装

- [x] 4-11: `register_builtins` で `Trace` を `Value::Namespace("Trace")` として登録する
- [x] 4-12: `eval_field_access` で `"Trace"` を処理する（`print`, `log` → `Value::Builtin`）
  - 実装: 既存の Namespace → Builtin 変換で自動処理
- [x] 4-13: `eval_builtin("trace_print", args)` を実装する（`eprintln!` + 値を返す）
- [x] 4-14: `eval_builtin("trace_log", args)` を実装する（`eprintln!("{}: {}", ...)` + 値を返す）

### パターンガードの評価

- [x] 4-15: `eval_match` で各アームのガード式を評価する
  - ガードが `false` → 次のアームへ
  - ガードが `true` → body を評価

### 評価器の単体テスト

- [x] 4-16: `test_eval_chain_ok` — `chain n <- ok(42)` → `n = 42` で継続
- [x] 4-17: `test_eval_chain_escape_err` — `chain n <- err("x")` → 関数から `err("x")` が返る
- [x] 4-18: `test_eval_chain_escape_none` — `chain n <- none` → 関数から `none` が返る
- [x] 4-19: `test_eval_collect_yield` — `collect { yield 1; yield 2; }` → `[1, 2]`
- [x] 4-20: `test_eval_collect_empty` — `collect { () }` → `[]`
- [x] 4-21: `test_eval_match_guard_true` — ガードが true のアームが選ばれる
- [x] 4-22: `test_eval_match_guard_false` — ガードが false なら次のアームへ
- [x] 4-23: `test_eval_pipe_match` — `42 |> match { n where n > 0 => "pos" _ => "neg" }` → `"pos"`

---

## Phase 5: サンプルと動作確認

- [x] 5-1: `examples/chain.fav` を作成する
  - Result の伝播例（`parse_int` + `validate` + 表示）
  - Option の伝播例（`Map.get` + `some`）
  - `fav run examples/chain.fav` が正しく動く
- [x] 5-2: `examples/collect.fav` を作成する
  - 連番の collect + yield
  - 条件付き yield（if ガード）
  - `fav run examples/collect.fav` が正しく動く
- [x] 5-3: `examples/pipe_match.fav` を作成する
  - `|> match` + `where` ガードの組み合わせ
  - `fav run examples/pipe_match.fav` が正しく動く
- [x] 5-4: `fav check examples/chain.fav` が型エラーなく通る
- [x] 5-5: `fav check examples/collect.fav` が型エラーなく通る
- [x] 5-6: `fav check examples/pipe_match.fav` が型エラーなく通る

---

## ドキュメント

- [x] 6-1: `README.md` に v0.5.0 の使い方（chain / collect / pipe match / where / Trace）を追記する
- [x] 6-2: `versions/roadmap.md` の v0.5.0 完了日を記録する
