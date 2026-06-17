# v17.6.0 — `fav bench` 統計強化 Spec

Date: 2026-06-15

## 概要

v1.8.0 で実装した `fav bench` は `bench "..." { }` 構文・`--filter` オプションを持つが、
統計は単純な平均（µs/iter）のみで、p50/p95/min/max がない。
v17.6.0 では各イテレーション時間を個別計測し、avg / p50 / p95 / min / max を出力する。
また `--warmup N`・`--json` オプションを追加する。

---

## 現在の状態（v17.5.0）

```
BenchDef 構造体: ast.rs に定義済み
Item::BenchDef: コンパイラ/チェッカー対応済み
cmd_bench(file, filter, iters): iters 回実行後、総時間/iters を平均として出力
--filter: ベンチ名部分一致フィルタ（実装済み）
--iters: イテレーション数（デフォルト 100）
出力: bench  {desc:<40}  {:.2} µs/iter  ({iters}  {path})
```

---

## 追加仕様

### 出力形式

```
fav bench src/pipeline.fav

running 3 benchmarks (100 runs)

  transform 10k rows:  avg  12.3ms  p50  11.9ms  p95  14.8ms  min  11.2ms  max  21.4ms
  json parse large:    avg   1.2ms  p50   1.1ms  p95   1.5ms  min   1.0ms  max   2.1ms
  bigquery query:      avg 245.0ms  p50 241.0ms  p95 280.0ms  min 230.0ms  max 310.0ms

bench result: 3 benchmarks completed.
```

時間単位は自動選択: `< 1000µs` → `µs`、`< 1000ms` → `ms`、それ以上 → `s`

### `--json` 出力

```json
{
  "benchmarks": [
    {
      "name": "transform 10k rows",
      "runs": 100,
      "avg_us": 12340.5,
      "p50_us": 11900.0,
      "p95_us": 14800.0,
      "min_us": 11200.0,
      "max_us": 21400.0
    }
  ]
}
```

### 新規 CLI オプション

```bash
fav bench [file]
  --runs N       イテレーション回数（デフォルト 100）。--iters の別名
  --iters N      --runs と同義（後方互換のため維持）
  --warmup N     ウォームアップ回数（デフォルト 5）
  --filter PAT   ベンチ名でフィルタ（既存）
  --json         JSON 形式で出力（新規）
```

---

## 実装スコープ

変更対象は `fav/src/driver.rs` と `fav/src/main.rs` のみ。
AST / IR / VM に変更不要（`BenchDef` は既存）。

---

## 新規関数（driver.rs）

### `BenchStats` 構造体

```rust
pub struct BenchStats {
    pub name: String,
    pub runs: u64,
    pub avg_us: f64,
    pub p50_us: f64,
    pub p95_us: f64,
    pub min_us: f64,
    pub max_us: f64,
}
```

### `compute_bench_stats(name, timings_us) -> BenchStats`

- `timings_us`: `Vec<f64>` — 各イテレーションの µs 時間
- 平均 = 総和 / n
- p50/p95 = ソート後インデックスで取得（`timings_us[n * 50 / 100]` 等）
- min/max = `Vec::iter().fold()`

```rust
pub fn compute_bench_stats(name: &str, timings_us: Vec<f64>) -> BenchStats
```

### `exec_bench_case_v2(prog, description, runs, warmup) -> Result<Vec<f64>, String>`

- ウォームアップ: `warmup` 回実行（時間計測なし）
- 本計測: `runs` 回、各イテレーションを `Instant` で個別計測
- 返り値: 各イテレーションの µs 時間 `Vec<f64>`

### `format_duration_us(us: f64) -> String`

- `us < 1000.0` → `"1.2µs"`
- `us < 1_000_000.0` → `"1.2ms"`
- それ以上 → `"1.2s"`

### `bench_stats_to_json(results: &[BenchStats]) -> String`

- `serde_json::json!` を使って上記 JSON 形式の文字列を返す
- 新規 Cargo 依存不要（`serde_json` は既存依存）

---

## `BenchOpts` 構造体

```rust
pub struct BenchOpts {
    pub file: Option<String>,
    pub filter: Option<String>,
    pub runs: u64,      // default: 100
    pub warmup: u64,    // default: 5
    pub json: bool,     // default: false
}
```

`cmd_bench` のシグネチャを変更：

```rust
// Before
pub fn cmd_bench(file: Option<&str>, filter: Option<&str>, iters: u64)

// After
pub fn cmd_bench(opts: &BenchOpts)
```

---

## `main.rs` 更新

```bash
bench [--runs <n>] [--iters <n>] [--warmup <n>] [--filter <pat>] [--json] [file]
```

- `--runs N` / `--iters N`: どちらも `opts.runs` に設定
- `--warmup N`: `opts.warmup` に設定
- `--json`: `opts.json = true`

---

## テスト（v176000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_6_0` | Cargo.toml に "17.6.0" が含まれる |
| `bench_def_parses` | `bench "name" { }` が AST として正しく解析される |
| `bench_stats_computed` | `compute_bench_stats` が avg/p50/p95/min/max を正しく計算する |
| `bench_filter_option` | `collect_bench_cases` に filter を渡すと絞り込まれる |
| `bench_json_output` | `bench_stats_to_json` が JSON 形式文字列を返す |

---

## 完了条件（PASS=5）

1. `bench "..." { }` が AST として解析される（既存動作の確認）
2. 各イテレーション時間を個別計測し、p50/p95/min/max が正しく計算される
3. `--warmup N` でウォームアップ回数を変更できる
4. `--filter "keyword"` でベンチを絞り込める
5. `--json` フラグで JSON 形式の出力が得られる

---

## 非対応（スコープ外）

- stddev（標準偏差）— p50/p95 があれば十分、後続バージョンで追加
- `--output <file>` によるファイル出力 — stdout のみ
- ベンチ結果の比較（`fav bench --compare before.json after.json`）— v18.x 以降
- HTML レポート — v18.x 以降
