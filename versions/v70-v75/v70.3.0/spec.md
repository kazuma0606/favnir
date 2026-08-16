# v70.3.0 Spec — `fav bench` サブコマンド完成

Date: 2026-08-09
Status: 計画中

---

## Background

`fav bench` コマンドと `BenchOpts` は driver.rs / main.rs に既に存在し、以下が実装済み:
- `.bench.fav` ファイルのマイクロベンチマーク実行（`cmd_bench`）
- `--json` / `--runs` / `--warmup` / `--filter` / `--compare` / `--fail-on-regression` / `--threshold` フラグ
- `bench.yml` CI に `$FAV bench --all --compare benchmarks/baseline.json --fail-on-regression` が記述済み

**未実装の gap:**

1. `--all` フラグが no-op（main.rs 行 1667: `"--all" => { i += 1; }`）
2. **built-in intrinsic メトリクスが存在しない**
   - ロードマップの JSON 例: `compile_hello_fav_ms`, `run_csv_1k_rows_ms`, `type_check_checker_fav_ms`
   - これらは `.bench.fav` ファイルではなく、**compiler/VM の実際の計測値**を出力すべき
3. `--all` + `--compare` の組み合わせが動作しない

**設計方針:**
- `cmd_bench_all()` を新規追加 — コンパイラ・CSV パース・大規模ソースのフロントエンド速度を計測して JSON 形式で返す
- `BenchOpts` に `all: bool` フィールドを追加し、while ループ後に `if opts.all { ... }` で処理する（既存の `--compare` 処理と競合しない設計）
- `--compare` と `--fail-on-regression` は既存の `cmd_bench_compare` を再利用
- `--all` + `--compare` の場合は `cmd_bench_all()` の結果を `cmd_bench_compare` に渡す

---

## Goals

1. `fav bench --all` が `version`/`timestamp`/`metrics` を含む JSON を stdout に出力する
2. `metrics` に `compile_hello_fav_ms`・`run_csv_1k_rows_ms`・`type_check_checker_fav_ms` が含まれる
3. `fav bench --all --compare benchmarks/baseline.json --fail-on-regression` が regression を検出して非ゼロ終了する
4. 既存テストが全 pass（3563 件）
5. 新規 Rust テスト 2 件追加 → 3565 tests

---

## Syntax / API Examples

```bash
# 全組み込みベンチマークを実行して JSON 出力
$ fav bench --all
{
  "version": "70.3.0",
  "timestamp": "2026-08-09T10:00:00Z",
  "metrics": {
    "compile_hello_fav_ms": 12,
    "run_csv_1k_rows_ms": 45,
    "type_check_checker_fav_ms": 230
  }
}

# ベースラインと比較して regression があれば非ゼロ終了
$ fav bench --all --compare benchmarks/baseline.json --fail-on-regression
OK: all metrics within 5% of baseline.

# --threshold でリグレッション閾値を変更（デフォルト 5%）
$ fav bench --all --compare benchmarks/baseline.json --fail-on-regression --threshold 10
```

**スコープ外（v70.3.0）**:
- `fav bench compile_hello_fav`（単体 built-in ベンチ指定）— ロードマップに記載されているが v70.3.0 では未実装。`--all` 一括実行のみサポート。

**`--compare` の設計**:
- `--all` フラグ使用時は `BenchOpts.all` で検出し、`cmd_bench_all()` の結果 JSON を `cmd_bench_compare` に渡す独立フローで処理する
- 既存の `.bench.fav` ファイル実行系（`opts.compare`）との二重処理は発生しない（`--all` は while ループを抜けた後に処理）

---

## 計測対象メトリクス

| メトリクス名 | 計測内容 |
|---|---|
| `compile_hello_fav_ms` | hello.fav ソースを `Parser::parse_str` + `build_artifact` でコンパイルする時間（ms）— フロントエンド全体 |
| `run_csv_1k_rows_ms` | 1000 行分の CSV テキスト（インメモリ）を Rust `csv::Reader` でパースする時間（ms）|
| `type_check_checker_fav_ms` | `checker.fav`（3000+ 行）を `Parser::parse_str` でパースする時間（ms）— 大規模ソースのフロントエンド速度 |

**注記**: `compile_hello_fav_ms` は `build_artifact`（AST → IR → バイトコード変換）を含む完全なコンパイルパスを計測する。`run_csv_1k_rows_ms` は Rust `csv` クレートを使った実際の CSV パース時間を計測する（v70.3.0 時点で `csv` クレートは `Cargo.toml` に既に依存として登録済み）。

---

## Success Criteria

- [ ] `cmd_bench_all()` が `{"version": "...", "timestamp": "...", "metrics": {...}}` JSON を返す
- [ ] `fav bench --all` 実行時に `cmd_bench_all()` が呼ばれる
- [ ] `fav bench --all --compare ... --fail-on-regression` が regression を検出して `false` を返す
- [ ] `cargo test v703000` で 2 件 pass
- [ ] `cargo test` 全体で 3565 tests pass

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_bench_all()` 新規追加、`BenchOpts` に `all: bool` 追加、`v703000_tests` モジュール追加 |
| `fav/src/main.rs` | `BenchOpts.all` を `true` にセット、ループ後に `--all` 処理ブロックを追加 |
| `fav/Cargo.toml` | `version` を `"70.2.0"` → `"70.3.0"` に更新 |
| `CHANGELOG.md` | v70.3.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.3.0 に更新 |
