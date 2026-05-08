# Favnir v1.8.0 タスク一覧 — `Task<T>` 非同期完成 + Coverage 強化 + `fav bench`

作成日: 2026-05-09

> **ゴール**: v1.7.0 で同期透明として確立した `Task<T>` を拡張し、
> 並列 API・`async fn main()`・`chain` 統合を追加。
> Coverage を関数単位レポートに強化し、`fav bench` を新設する。
>
> **前提**: v1.7.0 完了（498 テスト通過）
>
> **スコープ管理が最優先。tokio 統合はスコープ外。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [ ] `Cargo.toml` の `version` を `"1.8.0"` に更新
- [ ] `main.rs` の HELP テキストを `v1.8.0` に更新
- [ ] `cargo build` が通ること

---

## Phase 1: `Task` 並列 API

### 1-1: `checker.rs` の変更

- [ ] `Task.all` のビルトイン型登録: `List<Task<T>> -> Task<List<T>>`
- [ ] `Task.race` のビルトイン型登録: `List<Task<T>> -> Task<T>`
- [ ] `Task.timeout` のビルトイン型登録: `Task<T> -> Int -> Task<Option<T>>`
- [ ] 型変数 `T` を unify で単相化する処理を追加

### 1-2: `vm.rs` の変更

- [ ] `vm_call_builtin` に `"Task.all"` ハンドラを追加
  - [ ] 引数リストが空なら E061 ランタイムエラー
  - [ ] 各要素を順次実行（v1.8.0: 同期）して `VMValue::List` で返す
- [ ] `vm_call_builtin` に `"Task.race"` ハンドラを追加
  - [ ] 引数リストが空なら E061 ランタイムエラー
  - [ ] 先頭要素を実行して返す（v1.8.0 簡易実装）
- [ ] `vm_call_builtin` に `"Task.timeout"` ハンドラを追加
  - [ ] v1.8.0 では常に `VMValue::Variant("some", value)` を返す

### 1-3: テスト

- [ ] テスト: `task_all_collects_results` — `Task.all` が全結果をリストで返す
- [ ] テスト: `task_all_empty_list_error` — 空リストで E061 相当のエラー
- [ ] テスト: `task_race_returns_first` — `Task.race` が先頭結果を返す
- [ ] テスト: `task_timeout_returns_some` — `Task.timeout` が `Some(value)` を返す
- [ ] `cargo test` が全通過すること

---

## Phase 2: `async fn main()`

### 2-1: `checker.rs` の変更

- [ ] main の型として `() -> Task<Unit>` を受理するよう `ensure_valid_main` 相当処理を拡張
  - [ ] `() -> Unit !Io` は従来どおり通過
  - [ ] `() -> Task<Unit> !Io` も通過
  - [ ] それ以外は E003 / E004 を継続

### 2-2: `driver.rs` の変更

- [ ] `exec_artifact_main` で main の戻り値が Task<Unit> になる場合の確認
  - [ ] v1.8.0 では Task が透明値のため追加処理不要（確認のみ）
  - [ ] 将来の `VMValue::Task` 対応の分岐コメントを追加

### 2-3: テスト

- [ ] テスト: `async_main_executes_correctly` — `async fn main() -> Unit !Io` がエラーなく通過する
- [ ] テスト: `async_main_task_type_accepted` — main の型 `Task<Unit>` が型検査を通る
- [ ] `cargo test` が全通過すること

---

## Phase 3: `chain` + `Task<T>` 統合

### 3-1: `checker.rs` の変更

- [ ] `check_chain_stmt`（または `check_stmt` の chain アーム）に Task ラッパー剥がし処理を追加
  - [ ] rhs の型が `Task<Result<T,E>>` → Task を剥がして既存 chain ロジックへ
  - [ ] rhs の型が `Task<Option<T>>` → Task を剥がして既存 chain ロジックへ
  - [ ] rhs の型が `Task<T>` で T が Result でも Option でもない → E063
- [ ] E063 エラーを `errors.rs` または checker 内に定義

### 3-2: テスト

- [ ] テスト: `chain_task_result_unwraps_both` — `Task<T!>` の chain で Task と Result が解除される
- [ ] テスト: `chain_task_option_unwraps_both` — `Task<T?>` の chain で Task と Option が解除される
- [ ] `cargo test` が全通過すること

---

## Phase 4: Coverage 強化

### 4-1: `driver.rs` の変更 — 関数単位レポート

- [ ] `collect_fn_line_ranges(program: &IRProgram) -> HashMap<String, (u32, u32)>` を実装
  - [ ] TrackLine の行番号から関数ごとの min/max 行を収集
- [ ] `format_coverage_report_by_fn` を実装
  - [ ] ファイル全体サマリ（既存 `format_coverage_report` の出力）
  - [ ] 関数ごとのカバレッジ行を追記（`  fn foo  2 / 3 (66.7%)`）
- [ ] `cmd_test` の coverage 出力を `format_coverage_report_by_fn` に切り替え

### 4-2: `driver.rs` の変更 — `--coverage-report <dir>`

- [ ] `cmd_test` シグネチャに `coverage_report_dir: Option<&str>` を追加
- [ ] `coverage_report_dir` が Some の場合:
  - [ ] `std::fs::create_dir_all(dir)` でディレクトリを作成
  - [ ] `dir/coverage.txt` にテキストレポートを書き出す
  - [ ] 書き出し完了メッセージを stdout に表示

### 4-3: `main.rs` の変更

- [ ] `test` コマンドに `--coverage-report <dir>` フラグを追加
- [ ] 収集した `coverage_report_dir` を `cmd_test` に渡す

### 4-4: テスト

- [ ] テスト: `coverage_report_by_fn` — レポートに関数名が含まれる
- [ ] テスト: `coverage_report_dir_creates_file` — `--coverage-report` でファイルが生成される
- [ ] `cargo test` が全通過すること

---

## Phase 5: `fav bench`

### 5-1: `ast.rs` の変更

- [ ] `BenchDef { description: String, body: Expr, span: Span }` 構造体を追加
- [ ] `Item::BenchDef(BenchDef)` バリアントを追加
- [ ] 既存の `Item` マッチ箇所に `BenchDef` ケースを追加（`_ => {}` でも可）

### 5-2: `lexer.rs` の変更

- [ ] `TokenKind::Bench` を追加
- [ ] `"bench" => TokenKind::Bench` のキーワードマッピングを追加

### 5-3: `parser.rs` の変更

- [ ] `parse_bench_def` を実装
  - [ ] `bench` トークンを消費
  - [ ] 文字列リテラルをパース（description）
  - [ ] ブロック式をパース（body）
- [ ] `parse_item` の先頭分岐に `TokenKind::Bench => parse_bench_def()` を追加
- [ ] パーサーテスト: `bench "desc" { 1 + 1 }` がパースできる

### 5-4: `checker.rs` の変更

- [ ] `check_bench_def` を実装
  - [ ] body を型チェック（戻り型は不問）
  - [ ] `!File` / `!Db` / `!Network` エフェクトがあれば E064
- [ ] `register_item_signatures` と `check_item` に `Item::BenchDef` ケースを追加

### 5-5: `fmt.rs` の変更

- [ ] `Item::BenchDef` のフォーマットを追加

### 5-6: `compiler.rs` の変更

- [ ] `Item::BenchDef` を `compile_item` でスキップ（ベンチはランタイムで直接実行）

### 5-7: `driver.rs` の変更

- [ ] `collect_bench_cases(program: &Program) -> Vec<&BenchDef>` を実装
- [ ] `exec_bench_case(artifact, bench_def) -> VMValue` を実装
  - [ ] bench 本体を関数としてコンパイル・実行
- [ ] `format_bench_results(results: &[(String, BenchTiming)]) -> String` を実装
  - [ ] `  bench  {desc:<40} {avg:.2} µs/iter  ({iters} iters)` 形式
- [ ] `cmd_bench(file: Option<&str>, filter: Option<&str>, iters: u64)` を実装
  - [ ] bench ケース収集 → フィルタ → 計測ループ → レポート表示

### 5-8: `main.rs` の変更

- [ ] `bench` コマンドを HELP テキストに追加
- [ ] `Some("bench")` ブランチを追加
  - [ ] `--filter <keyword>` フラグ
  - [ ] `--iters <N>` フラグ（デフォルト 100）
  - [ ] ファイル引数

### 5-9: テスト

- [ ] テスト: `bench_collect_bench_cases` — `bench` ブロックが正しく収集される
- [ ] テスト: `bench_runs_and_reports_timing` — ベンチ結果が `µs/iter` 形式で出力される
- [ ] テスト: `bench_filter_skips_non_matching` — フィルタで非マッチがスキップされる
- [ ] `cargo test` が全通過すること

---

## Phase 6: テスト・ドキュメント

### 6-1: example ファイルの追加

- [ ] `examples/task_parallel_demo.fav` を作成
  - [ ] `Task.all` + `Task.race` + `Task.timeout` の基本パターン
  - [ ] `fav check` でエラーなしを確認
- [ ] `examples/async_main_demo.fav` を作成
  - [ ] `async fn main()` の最小例
  - [ ] `fav run` でエラーなしを確認
- [ ] `examples/math.bench.fav` を作成
  - [ ] `bench` ブロックの基本例（加算・fold）
  - [ ] `fav bench` で実行できることを確認

### 6-2: `langspec.md` の作成

- [ ] `versions/v1.8.0/langspec.md` を新規作成
  - [ ] `Task.all` / `Task.race` / `Task.timeout` の型と挙動
  - [ ] v1.8.0 の同期制限と v1.9.0 での並列化予定
  - [ ] `async fn main()` の宣言と実行フロー
  - [ ] `chain` + `Task<T>` の統合規則
  - [ ] E061 / E062 / E063 / E064 エラーコード
  - [ ] `fav test --coverage` 関数単位レポートの出力フォーマット
  - [ ] `fav test --coverage-report <dir>` フラグ
  - [ ] `fav bench` の構文・CLI オプション・出力フォーマット

### 6-3: `README.md` の更新

- [ ] v1.8.0 セクションを追加（Task 並列 API / async main / bench）

### 6-4: 全体確認

- [ ] `cargo build` で Rust コンパイラ警告ゼロ
- [ ] `cargo test` 全テスト通過（v1.7.0 継承 498 + 新規テスト）
- [ ] `Task.all([t1, t2])` が型チェックを通り実行できる
- [ ] `async fn main() -> Unit !Io` が `fav run` で動く
- [ ] `chain x <- Task<T!>` で Task と Result が両方解除される
- [ ] `fav test --coverage` で関数名付きカバレッジが出力される
- [ ] `fav bench` でベンチが計測・表示される
- [ ] `Cargo.toml` バージョンが `"1.8.0"`

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `Task.all` / `Task.race` / `Task.timeout` が動作する
- [ ] `async fn main()` が動作する
- [ ] `chain` + `Task<T!>` が動作する
- [ ] `fav test --coverage` に関数単位レポートが含まれる
- [ ] `fav test --coverage-report <dir>` でレポートが書き出される
- [ ] `fav bench` が動作する
- [ ] v1.7.0 の全テスト（498）が引き続き通る
- [ ] `Cargo.toml` バージョンが `"1.8.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| tokio / async-std による真の並列実行 | v1.9.0 |
| `Task.timeout` の実際のタイムアウト動作 | v1.9.0 |
| `Task.race` の真の並列競合 | v1.9.0 |
| `fav bench --coverage` 連携 | v1.9.0 以降 |
| Coverage の HTML 出力 | v1.9.0 以降 |
| `bench` 内での `async fn` 計測 | v1.9.0 以降 |
| `fav migrate` (v1.x → v2.0.0 変換) | v2.0.0 |
| `trf` → `stage` / `flw` → `seq` リネーム | v2.0.0 |
| セルフホスト（パーサー Favnir 移植） | v2.0.0 |
