# Favnir v1.9.0 タスク一覧 — `for` 式 + `??` 演算子 + `stage`/`seq` + ツール仕上げ

作成日: 2026-05-09

> **ゴール**: `for` 式・`??` 演算子の二構文追加、`stage`/`seq` キーワードによる v2.0.0 移行準備、
> Coverage HTML 出力・`fav bench` 統計強化でツール体験を完成させる。
>
> **前提**: v1.8.0 完了（509 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.9.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.9.0` に更新
- [x] `cargo build` が通ること

---

## Phase 1: `for` 式

### 1-1: 字句解析 (`src/frontend/lexer.rs`)

- [x] `TokenKind` に `For` を追加
- [x] `TokenKind` に `In` を追加
- [x] キーワードマッピングに `"for" => TokenKind::For` を追加
- [x] キーワードマッピングに `"in" => TokenKind::In` を追加
- [x] レキサーのテストで `for x in list` が正しくトークン化されることを確認

### 1-2: AST (`src/ast.rs`)

- [x] `Stmt` に `ForIn { var: String, iter: Expr, body: Block, span: Span }` を追加

### 1-3: パーサー (`src/frontend/parser.rs`)

- [x] `parse_stmt` に `TokenKind::For` のケースを追加
- [x] `parse_for_in` メソッドを追加: `for <ident> in <expr> <block>` を解析
- [x] `for` に visibility modifier が付いたらエラー
- [x] `cargo build` が通ること

### 1-4: 型検査 (`src/middle/checker.rs`)

- [x] `check_stmt` に `Stmt::ForIn` のケースを追加
- [x] `in_collect` フラグが true の場合 E067 を発する
- [x] イテレータの型が `List<T>` でない場合 E065 を発する
- [x] ループ変数 `x: T` をスコープに追加して `check_block` を呼ぶ
- [x] body の最終型が `Unit` でない場合 E066 を発する

### 1-5: コンパイラ (`src/middle/compiler.rs`)

- [x] `compile_stmt` に `Stmt::ForIn` のケースを追加
- [x] `for x in list { body }` を `List.fold(list, Unit, |_, x| { body; Unit })` にデシュガーしてコンパイル

### 1-6: フォーマッタ (`src/fmt.rs`)

- [x] `fmt_stmt` に `Stmt::ForIn` のケースを追加: `for {var} in {iter} {body}` を出力

### 1-7: リント (`src/lint.rs`)

- [x] `lint_block_unused_binds` で `Stmt::ForIn` の `body` を走査するケースを追加
- [x] `collect_block_calls` で `Stmt::ForIn` の `iter` と `body` を対象に追加

---

## Phase 2: `??` 演算子

### 2-1: 字句解析 (`src/frontend/lexer.rs`)

- [x] `TokenKind` に `QuestionQuestion` を追加
- [x] `?` を読んだとき、次も `?` なら `QuestionQuestion` として認識するよう `lex_char` を変更
- [x] レキサーテストで `a ?? b` が `[Ident, QuestionQuestion, Ident]` になることを確認

### 2-2: AST (`src/ast.rs`)

- [x] `BinOp` に `NullCoalesce` を追加

### 2-3: パーサー (`src/frontend/parser.rs`)

- [x] 二項演算子解析の最外層（最低優先順位）に `??` を追加
- [x] `??` は右辺を再帰的に解析（右結合）

### 2-4: 型検査 (`src/middle/checker.rs`)

- [x] `check_binop` に `BinOp::NullCoalesce` のケースを追加
- [x] 左辺が `Option<T>` の場合: 右辺を T と照合して T を返す
- [x] 左辺が `Option<T>` 以外の場合: E068 を発する
- [x] 右辺が T と非互換な場合: E069 を発する

### 2-5: コンパイラ (`src/middle/compiler.rs`)

- [x] `compile_expr` に `BinOp::NullCoalesce` のケースを追加
- [x] `Option.unwrap_or(lhs, rhs)` に相当する IRExpr を生成

### 2-6: フォーマッタ (`src/fmt.rs`)

- [x] `fmt_binop` に `BinOp::NullCoalesce => " ?? "` を追加

---

## Phase 3: `stage`/`seq` エイリアス

### 3-1: 字句解析 (`src/frontend/lexer.rs`)

- [x] `TokenKind` に `Stage` を追加（`Trf` のエイリアス）
- [x] `TokenKind` に `Seq` を追加（`Flw` のエイリアス）
- [x] キーワードマッピングに `"stage" => TokenKind::Stage` を追加
- [x] キーワードマッピングに `"seq" => TokenKind::Seq` を追加

### 3-2: パーサー (`src/frontend/parser.rs`)

- [x] `parse_item` で `TokenKind::Trf | TokenKind::Stage` を同一ブランチで処理
- [x] `parse_item` で `TokenKind::Flw | TokenKind::Seq` を同一ブランチで処理
- [x] `parse_abstract_item` でも同様に `Stage`/`Seq` を対応
- [x] visibility と `abstract` の組み合わせも `stage`/`seq` で動くことを確認

### 3-3: example ファイル (`examples/stage_seq_demo.fav`)

- [x] `stage F: Int -> Int = |x| x * 2` を含む例を作成
- [x] `seq P { F }` を含む例を作成
- [x] `fav check examples/stage_seq_demo.fav` がエラーなし

---

## Phase 4: Coverage HTML 出力

### 4-1: HTML 生成ヘルパー (`src/driver.rs`)

- [x] `sanitize_html_filename(path: &str) -> String` を追加
- [x] `format_coverage_html_index(file_reports: &[...]) -> String` を追加
  - `<!DOCTYPE html>` + テーブル形式で各ファイルのリンク・カバレッジ%を表示
- [x] `format_coverage_html_file(path, source, executed) -> String` を追加
  - 各行を `<div class="line covered">` / `<div class="line uncovered">` で囲む

### 4-2: `cmd_test` の拡張 (`src/driver.rs`)

- [x] coverage_report_dir がある場合に `index.html` を書き出す
- [x] coverage_report_dir がある場合に各ソースファイルの `.html` を書き出す
- [x] 生成したファイル一覧を `println!` で表示

### 4-3: テスト (`src/driver.rs` の tests モジュール)

- [x] `coverage_html_index_created`: 一時ディレクトリに index.html が生成される
- [x] `coverage_html_file_created`: ソース注釈 HTML が生成される
- [x] `coverage_html_contains_percentage`: HTML に `%` が含まれる

---

## Phase 5: `fav bench` 統計強化

### 5-1: 統計構造体と計算 (`src/driver.rs`)

- [x] `pub struct BenchStats { mean_us, min_us, max_us, stddev_us, p50_us, iters }` を追加
- [x] `fn compute_bench_stats(samples: &[f64]) -> BenchStats` を追加
  - mean: 算術平均
  - min/max: 最小・最大
  - stddev: 標準偏差（population）
  - p50: ソート済み中央値

### 5-2: `exec_bench_case` の変更 (`src/driver.rs`)

- [x] 戻り値を `Result<f64, String>` から `Result<Vec<f64>, String>` に変更
- [x] 各イテレーションを個別計測して `Vec<f64>` に記録

### 5-3: フォーマット関数 (`src/driver.rs`)

- [x] `fn format_bench_result_verbose(desc, stats) -> String` を追加（複数行）
- [x] `fn format_bench_result_compact(desc, stats) -> String` を追加（1行）
- [x] `fn format_bench_results_json(results) -> String` を追加（JSON 配列）

### 5-4: `cmd_bench` の変更 (`src/driver.rs`)

- [x] シグネチャに `compact: bool, json_output: bool` を追加
- [x] `exec_bench_case` の戻り値 `Vec<f64>` から `compute_bench_stats` を呼ぶ
- [x] `compact` が true なら `format_bench_result_compact`、false なら `format_bench_result_verbose` を使う
- [x] `json_output` が true なら `format_bench_results_json` を出力

### 5-5: CLI (`src/main.rs`)

- [x] bench コマンドに `--compact` フラグを追加
- [x] bench コマンドに `--json` フラグを追加
- [x] `cmd_bench` 呼び出しに `compact`, `json_output` を渡す

### 5-6: テスト (`src/driver.rs` の tests モジュール)

- [x] `bench_stats_compute_mean`: samples=[1.0,2.0,3.0] → mean=2.0
- [x] `bench_stats_compute_stddev`: 既知データで stddev を確認
- [x] `bench_stats_compute_p50`: samples=[1,2,3,4,5] → p50=3
- [x] `bench_compact_format`: compact 形式が1行であることを確認

---

## Phase 6: テスト・ドキュメント

### 6-1: Phase 1 (`for` 式) のテスト

- [x] `for_in_io_context_iterates_list`: `for x in List.range(1, 4) { IO.println_int(x) }` を含むソースが実行できる
- [x] `for_in_pure_context_unit_result`: Pure コンテキストの `for` が Unit を返す（型検査のみ）
- [x] `for_non_list_iter_errors_e065`: `for x in 42 { ... }` で E065
- [x] `for_in_collect_block_errors_e067`: collect 内の `for` で E067

### 6-2: Phase 2 (`??`) のテスト

- [x] `null_coalesce_returns_value_when_some`: `Option.some(42) ?? 0` が 42 を返す
- [x] `null_coalesce_returns_default_when_none`: `Option.none() ?? 0` が 0 を返す
- [x] `null_coalesce_chained`: `Option.none() ?? Option.none() ?? 1` が 1 を返す
- [x] `null_coalesce_lhs_non_option_errors_e068`: `42 ?? 0` で E068

### 6-3: Phase 3 (`stage`/`seq`) のテスト

- [x] `stage_keyword_parses_like_trf`: `stage F: Int -> Int = |x| x` が parse・型検査を通る
- [x] `seq_keyword_parses_like_flw`: `seq P { F }` が parse・型検査を通る（F は事前定義）
- [x] `trf_and_stage_coexist`: 同一ファイルに `trf` と `stage` が共存できる

### 6-4: example ファイル

- [x] `examples/for_demo.fav` を作成・`fav check` がエラーなし
- [x] `examples/coalesce_demo.fav` を作成・`fav check` がエラーなし
- [x] `examples/stage_seq_demo.fav` を作成・`fav check` がエラーなし

### 6-5: langspec.md

- [x] `versions/v1.9.0/langspec.md` を新規作成
  - `for` 式の構文・型規則・例
  - `??` 演算子の構文・型規則・例・優先順位
  - `stage`/`seq` キーワードの説明
  - Coverage HTML フラグの説明
  - `fav bench` 統計出力の説明

### 6-6: README.md

- [x] `README.md` に v1.9.0 セクションを追加
  - `for` 式・`??` 演算子・`stage`/`seq` のコード例
  - Coverage HTML と bench 統計の説明

### 6-7: 最終確認

- [x] `cargo test` が全テスト通過（目標: 527+ テスト）
- [x] `cargo build` で警告ゼロ（または許容範囲内）
- [x] `fav check examples/for_demo.fav` がエラーなし
- [x] `fav check examples/coalesce_demo.fav` がエラーなし
- [x] `fav check examples/stage_seq_demo.fav` がエラーなし
- [x] `fav bench examples/math.bench.fav` が統計情報付きで出力される
- [x] `fav bench examples/math.bench.fav --json` が JSON を出力する

---

## 補足: エラーコード予約

| コード | Phase | 条件 |
|---|---|---|
| E065 | 1 | `for` のイテレータが `List<T>` 以外 |
| E066 | 1 | `for` ボディの最終式が `Unit` 以外 |
| E067 | 1 | `collect` ブロック内で `for` を使用（v1.9.0 未対応） |
| E068 | 2 | `??` の左辺が `Option<T>` 以外 |
| E069 | 2 | `??` の右辺が左辺のアンラップ型と非互換 |
