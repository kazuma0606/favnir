# v70.3.0 タスクリスト — `fav bench` サブコマンド完成

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.2.0` であることを確認
- [x] `cargo test` が全 pass（3563 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）
- [x] main.rs の `--all` ブランチが no-op（`i += 1` のみ）であることを確認

---

## T1: `cmd_bench_all()` を driver.rs に追加

- [x] `fav/self/checker.fav` が存在することを確認（`include_str!("../self/checker.fav")` の参照先）
- [x] `fav/Cargo.toml` に `chrono` / `csv` が依存として登録されていることを確認
- [x] `cmd_bench` 関数の直後（行 6603 付近）に `cmd_bench_all()` を追加する
- [x] 3 メトリクスを正しく計測する:
  - `compile_hello_fav_ms`: `Parser::parse_str` + `build_artifact` の時間（フルパイプライン）
  - `run_csv_1k_rows_ms`: 1000 行の CSV テキストを `csv::Reader` でパースする時間
  - `type_check_checker_fav_ms`: `checker.fav` を `Parser::parse_str` でパースする時間
- [x] 返却する JSON に `version` / `timestamp` / `metrics` フィールドが含まれることを確認
- [x] `pub` であることを確認（テストから `super::` でアクセスするため）
- [x] `cargo test` で既存テスト（3563 件）が全 pass することを確認

---

## T2: `BenchOpts.all` を追加し main.rs を修正

- [x] driver.rs の `BenchOpts` 構造体に `pub all: bool` フィールドを追加する
- [x] `BenchOpts::default()` に `all: false` を追加する
- [x] main.rs の `"--all" => { i += 1; }` を `opts.all = true; i += 1;` に変更する
- [x] main.rs のループ後（`cmd_bench(&opts)` の直前）に `if opts.all { ... }` ブロックを挿入する
  - `cmd_bench_all()` を呼び出す
  - `opts.compare` があれば `cmd_bench_compare()` で比較し、`fail_on_regression` なら非ゼロ終了
  - `--all` 時は `cmd_bench` をスキップする（`.bench.fav` 実行と二重にならないよう）
- [x] `cargo test` で既存テスト（3563 件）が全 pass することを確認

---

## T3: `v703000_tests` モジュール追加

- [x] driver.rs の末尾に `mod v703000_tests` を追加する
- [x] `bench_subcommand_all_outputs_json` テストを実装する
  - `cmd_bench_all()` が valid JSON を返すことを assert
  - JSON に `version`, `timestamp`, `metrics` フィールドがあることを assert
  - `metrics` に 3 メトリクスすべてが数値として含まれることを assert
- [x] `bench_subcommand_regression_fail` テストを実装する
  - baseline に 1ms（非ゼロ）を設定、current に 1000ms を設定（base==0.0 は pct=0.0 になる既存実装のため）
  - `cmd_bench_compare` が `(false, _)` を返すことを assert
- [x] `cargo test v703000` で 2 件 pass することを確認

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.2.0"` → `"70.3.0"` に変更する
- [x] driver.rs 内の旧バージョン文字列アサーション（`version = \"70.2.0\"`）を `replace_all: true` で一括更新

---

## T5: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.2.0 エントリの直前）に v70.3.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `cmd_bench_all()` 新規追加
  - Added: `fav bench --all` が built-in メトリクス出力に変更
  - Added: `v703000_tests` 2 件（3563 → 3565 tests）

---

## T6: versions/current.md 更新

- [x] `versions/current.md` を開く
- [x] 「進行中バージョン」を `v70.3.0`（fav bench 完成）に更新する
- [x] 「次に切る版」を `v70.4.0` に更新する

---

## T7: 最終確認

- [x] `cargo test v703000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3565 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.3.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認
- [x] site/ MDX 更新: `site/content/docs/tools/bench.mdx` が存在しないためスコープ外

---

## コードレビュー指摘対応

- **spec-reviewer [HIGH-3] 対応**: `run_csv_1k_rows_ms` を整数ループから `csv::Reader` による実際の CSV パースに変更
- **spec-reviewer [HIGH-1] 対応**: `--all` ブランチを while ループ内での `return` から `BenchOpts.all` フラグ + ループ後の `if opts.all { ... }` 処理に変更（`--compare` との競合解消）
- **実装時判明**: `cmd_bench_compare` は `base == 0.0` の場合 `pct = 0.0` とする（division-by-zero 回避）。テストの baseline を 0ms → 1ms に修正して regression 検出を確実にした
- **[BUG][MED] code-reviewer 対応**: `type_check_checker_fav_ms` → `parse_checker_fav_ms` にリネーム（計測内容はパースのみ）
- **[BUG][MED] code-reviewer 対応**: `compile_hello_fav_ms` の `if let Ok` を `expect()` に変更 — ハードコード有効ソースのパース失敗は panic で明示検出
- **[BUG][LOW] code-reviewer 対応**: `BenchOpts.emit_md` を追加、main.rs に `--emit-md` アームを追加し `--all --compare --emit-md` で Markdown 出力が機能するよう修正

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `bench_subcommand_all_outputs_json` が pass
- [x] `bench_subcommand_regression_fail` が pass
- [x] テスト総数: 3565（+2）
