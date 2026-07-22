# Tasks: v51.4.0 — `fav bench` 差分回帰検出

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3119 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` の `BenchOpts` struct に `compare` / `fail_on_regression` / `threshold` が**存在しない**ことを確認
- [x] `driver.rs` に `bench_stats_to_compare_json` が**存在しない**ことを確認（新規追加対象）
- [x] `cmd_bench` のシグネチャが `pub fn cmd_bench(opts: &BenchOpts)` で戻り値が `()`（変更前）であることを確認
- [x] `main.rs` の `bench` アームに `--compare` / `--fail-on-regression` が**未実装**であることを確認
- [x] `main.rs` の `bench` アームに v24.3.0 の `--baseline` / `--current` 分岐（行 1247〜1281）が**存在する**ことを確認（変更しない）
- [x] `benchmarks/v51.3.0.json` が**存在しない**ことを確認（作成対象）
- [x] `cmd_bench_compare` が `pub fn cmd_bench_compare(baseline_json: &str, current_json: &str, threshold: f64, emit_md: bool) -> (bool, String)` として存在することを確認（使用するため）

## T1 — `BenchOpts` 拡張（`driver.rs`）

- [x] `BenchOpts` struct に `compare: Option<String>` / `fail_on_regression: bool` / `threshold: f64` を追加
- [x] `Default` 実装を更新: `compare: None, fail_on_regression: false, threshold: 10.0` を追加
- [x] `cargo build` が通ることを確認（フルリテラル構築エラーが出た場合は `..Default::default()` で対処）

## T2 — `bench_stats_to_compare_json` 追加（`driver.rs`）

- [x] `bench_stats_to_json` の直後（行 5626 付近）に `bench_stats_to_compare_json` を追加
  - [x] シグネチャ: `pub fn bench_stats_to_compare_json(version: &str, stats: &[BenchStats]) -> String`
  - [x] `avg_us / 1000.0` で μs → ms 変換（f64 のまま）
  - [x] `serde_json::Map` に `s.name` → `avg_ms` のエントリを追加
  - [x] `serde_json::json!({ "version": version, "metrics": metrics }).to_string()` を返す
- [x] `cargo build` が通ることを確認

## T3 — `cmd_bench` 更新（`driver.rs`）

- [x] `cmd_bench` シグネチャを `pub fn cmd_bench(opts: &BenchOpts) -> bool` に変更
- [x] `all_stats` 収集完了後（`if opts.json { ... }` ブロックの後）に compare ロジックを追加:
  - [x] `opts.compare` が `Some(path)` の場合: `std::fs::read_to_string(path)` でベースライン JSON を読む
  - [x] エラー時は `eprintln!("error: cannot read baseline {path}: {e}")` + `return true`
  - [x] `bench_stats_to_compare_json("51.4.0", &all_stats)` で current JSON を生成
  - [x] `cmd_bench_compare(&baseline_json, &current_json, opts.threshold, false)` を呼ぶ
  - [x] `println!("{report}")` で結果出力
  - [x] `return ok`
- [x] compare なしのパスで関数末尾が `true` を返すことを確認
- [x] `cargo build` が通ることを確認

## T4 — `main.rs` CLI 拡張

- [x] `bench` アームの `--stream` ハンドラの直後（`other =>` より前）に以下を追加:
  - [x] `"--compare"`: `opts.compare = Some(...)`, `i += 2`
  - [x] `"--fail-on-regression"`: `opts.fail_on_regression = true`, `i += 1`
  - [x] `"--threshold"`: `opts.threshold = raw.parse::<f64>()`, `i += 2`
- [x] `cmd_bench(&opts);` を `let ok = cmd_bench(&opts);` + `if !ok && opts.fail_on_regression { process::exit(1); }` に変更
- [x] `bench_stats_to_compare_json` は `cmd_bench` 内部から呼ばれるため `main.rs` の `use driver::` 変更は不要であることを確認
- [x] `--baseline` 既存分岐（行 1247〜1281）が変更されていないことを確認
- [x] `cargo build` が通ることを確認

## T5 — `benchmarks/v51.3.0.json` 作成

- [x] `benchmarks/v51.3.0.json` を作成
  - [x] `{ "version": "51.3.0", "date": "2026-07-19", "milestone": "Performance & Scale Sprint", "tests_passed": 3119, "tests_failed": 0, "metrics": { "checker_ms": 12, "compiler_ms": 8, "total_pipeline_ms": 25 }, "regression": false, "notes": "プレースホルダー値。fav bench --json の実測値に更新することを推奨。" }`

## T6 — `v51400_tests` 追加 + バージョン更新

- [x] `v51400_tests` モジュールを `v51300_tests` の直前に追加（2 件）:
  - [x] `use super::cmd_bench_compare;` のみ（`bench_stats_to_compare_json` / `BenchStats` は不使用）
  - [x] `bench_regression_detected`:
    - [x] baseline: `checker_ms: 12`, current: `checker_ms: 18`（+50%）
    - [x] `cmd_bench_compare(baseline, current, 10.0, false)` で `ok=false` を assert
    - [x] report に `"REGRESSION"` と `"checker_ms"` が含まれることを assert
  - [x] `bench_no_regression_passes`:
    - [x] baseline: `checker_ms: 12`, current: `checker_ms: 13`（+8.3%）
    - [x] `cmd_bench_compare(baseline, current, 10.0, false)` で `ok=true` を assert
    - [x] report に `"OK"` が含まれることを assert
- [x] `fav/Cargo.toml` version → `"51.4.0"`
- [x] `cargo test` 3121 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.4.0 エントリ追加
- [x] `versions/current.md` を v51.4.0（3121 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.4.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
