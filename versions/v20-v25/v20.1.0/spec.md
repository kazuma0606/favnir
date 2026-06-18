# v20.1.0 Spec — ベンチマーク基盤整備（Benchmark Infrastructure）

## 概要

v20.1.0 は「何も最適化しない」バージョンである。
v20.0.0 で達成した Production Performance の現在地を数値で確定し、
以降のすべての最適化施策の比較基準（ベースライン）を整備する。

**テーマ**: Runtime Excellence シリーズ第1弾 — 計測なしに最適化しない

---

## 動機

v20.0.0 時点では「速い」という主観はあるが、数字がない。

- v20.2（スーパー命令）の「+20〜30%」はどの基準に対する 20〜30% か？
- v20.3（NaN-boxing）の「2〜3x」はどこから？
- リグレッションが入ったとき、どの PR が壊したか CI でわかるか？

この問いに「今の数字がない」まま進むことが最大のリスクである。
v20.1.0 は技術的には地味だが、**以降の v20.x 全体の土台**となる。

---

## 成果物一覧

| 成果物 | 役割 |
|---|---|
| `.github/workflows/bench.yml` | master push ごとにベンチマークを実行・比較 |
| `benchmarks/suite/run_all.sh` | 全スイートを JSON 形式で出力するラッパー |
| `benchmarks/suite/01_cold_start.sh` | Lambda コールドスタート計測（--precompiled あり/なし） |
| `benchmarks/suite/02_csv_10gb.fav` | 10GB CSV ストリーミングのスループット / ピークメモリ |
| `benchmarks/suite/03_tight_loop.fav` | 整数演算タイトループ（純粋 VM 速度） |
| `benchmarks/suite/04_record_transform.fav` | レコード変換 100万行（アロケーション速度） |
| `benchmarks/suite/05_compile_time.sh` | コンパイル時間（cold / incremental） |
| `benchmarks/suite/06_duckdb_query.fav` | DuckDB クエリ実行時間（比較用） |
| `benchmarks/suite/07_arrow_parquet.fav` | Arrow → Parquet 書き込みスループット |
| `benchmarks/suite/08_concurrent_stages.fav` | `par [A, B]` 並列 stage のスレッド効率 |
| `benchmarks/compare.fav` | ベースライン比較スクリプト（閾値超えで非ゼロ終了） |
| `benchmarks/v20.0.0.json` | v20.0.0 の実測ベースライン（CI が生成・コミット） |

---

## benchmarks/v20.0.0.json フォーマット

CI が生成する機械可読な正本。`benchmarks/results.md` は `compare.fav --emit-md` が生成する（手書き禁止）。

```json
{
  "version": "20.0.0",
  "timestamp": "2026-06-18T00:00:00Z",
  "metrics": {
    "cold_start_full_ms":          320,
    "cold_start_precompiled_ms":    18,
    "csv_10gb_throughput_mbs":     340,
    "tight_loop_10m_iter_ms":       85,
    "record_transform_1m_ms":      210,
    "compile_cold_ms":            2400,
    "compile_incremental_ms":      180,
    "arrow_parquet_write_1gb_ms": 3200
  }
}
```

> 初回コミット時は参考値を記載する。`bench.yml` が master 上で初回実行されたとき、実測値で上書きコミットする。

---

## benchmarks/compare.fav 仕様

```favnir
import rune "json"
import rune "io"

// CLI: fav run benchmarks/compare.fav
//   -- --baseline <path> --current <path> --threshold <N> [--emit-md]

fn main() -> Result<Unit, String> {
  bind args     <- IO.args()
  bind baseline <- parse_arg(args, "--baseline")
  bind current  <- parse_arg(args, "--current")
  bind threshold_str <- parse_arg(args, "--threshold")
  bind threshold <- Int.parse(threshold_str)
  bind emit_md  <- has_flag(args, "--emit-md")

  bind b_json   <- IO.read_file(baseline)
  bind c_json   <- IO.read_file(current)
  bind b_data   <- Json.parse(b_json)
  bind c_data   <- Json.parse(c_json)

  bind b_metrics <- Json.get_object(b_data, "metrics")
  bind c_metrics <- Json.get_object(c_data, "metrics")

  bind regressions <- check_regressions(b_metrics, c_metrics, threshold)

  if emit_md {
    bind _ <- write_results_md(c_data)
    ()
  } else { () }

  if List.length(regressions) > 0 {
    bind _ <- IO.println(f"REGRESSION DETECTED ({List.length(regressions)} metrics exceeded {threshold}% threshold):")
    bind _ <- List.for_each(regressions, |r| IO.println(f"  {r}"))
    Result.err("benchmark regression")
  } else {
    bind _ <- IO.println("All benchmarks within threshold.")
    Result.ok(())
  }
}
```

---

## .github/workflows/bench.yml 仕様

```yaml
name: Benchmark
on:
  push:
    branches: [master]

jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build release
        run: cargo build --release
        working-directory: fav

      - name: Run benchmarks
        run: bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json

      - name: Compare with baseline
        run: |
          ./fav/target/release/fav run benchmarks/compare.fav \
            -- --baseline benchmarks/v20.0.0.json \
               --current  benchmarks/latest.json \
               --threshold 10
```

---

## テスト（v201000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_1_0` | Cargo.toml に `"20.1.0"` が含まれる |
| `bench_suite_files_exist` | `benchmarks/suite/run_all.sh` が存在する |
| `bench_compare_fav_exists` | `benchmarks/compare.fav` が存在する |
| `bench_workflow_exists` | `.github/workflows/bench.yml` が存在する |
| `bench_baseline_exists` | `benchmarks/v20.0.0.json` が存在し valid JSON である |

---

## 完了条件

- [ ] `.github/workflows/bench.yml` が存在する
- [ ] `benchmarks/suite/run_all.sh` が存在する
- [ ] `benchmarks/suite/01_cold_start.sh` 〜 `08_concurrent_stages.fav` の8ファイルが存在する
- [ ] `benchmarks/compare.fav` が存在し、`fav check benchmarks/compare.fav` で parse エラーなし（手動確認 — driver.rs テストはファイル存在のみチェック）
- [ ] `benchmarks/v20.0.0.json` が存在し、`metrics` フィールドを持つ valid JSON である
- [ ] `fav/Cargo.toml` version が `20.1.0`
- [ ] `cargo test v201000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし

---

## 技術ノート

### v20.0.0 既存 benchmarks/ との関係

v20.0.0 で `benchmarks/` ディレクトリ配下に3ファイルが作成済み。
v20.1.0 では `benchmarks/suite/` サブディレクトリを新設し、計測スクリプトを格納する。
既存ファイルへの変更は `v20.0.0.json` の追加のみ。

### run_all.sh の出力形式

```bash
#!/usr/bin/env bash
# 各スクリプトを実行して結果を JSON に集約
# 出力: {"version": "20.x.x", "timestamp": "...", "metrics": {...}}
```

各スクリプトは `KEY=VALUE` 形式で結果を stdout に出力し、`run_all.sh` が JSON に変換する。

### CSV 10GB / Arrow 1GB スクリプト

本番計測には大きなファイルが必要だが、CI では小さなサンプルで代用する。
`CI=true` 環境変数が設定されている場合は 1MB のサンプルを使用し実行時間を記録する。

### compile_time 計測（05_compile_time.sh）

```bash
# cold compile: キャッシュ削除後に計測
rm -rf ~/.fav/cache/
time fav build src/pipeline.fav

# incremental: 1 ファイル変更後に計測
touch src/pipeline.fav
time fav build src/pipeline.fav
```
