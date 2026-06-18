# v20.1.0 実装計画 — ベンチマーク基盤整備

## 実装順序

```
T1（benchmarks/suite/ — 8 計測スクリプト）      ← 最初（他と独立）
T2（benchmarks/suite/run_all.sh）               ← T1 と並列可
T3（benchmarks/compare.fav）                    ← T1 と並列可
T4（.github/workflows/bench.yml）               ← T2, T3 完了後
T5（benchmarks/v20.0.0.json）                   ← T1 と並列可
T6（fav/Cargo.toml バージョン更新）              ← 任意のタイミング
T7（fav/src/driver.rs — v201000_tests）         ← T1〜T5 完了後
```

**Rust コードへの変更は T6 と T7 のみ。**
T1〜T5 はすべてプロジェクトルート配下のファイル作成。

---

## T1: `benchmarks/suite/` — 8 計測スクリプト

### 01_cold_start.sh

```bash
#!/usr/bin/env bash
# コールドスタート計測（--precompiled あり/なし）
set -e

FAV="${FAV:-./fav/target/release/fav}"
PROG="${BENCH_PROG:-fav/fav/tmp/hello.fav}"

# フルコンパイル（コールドスタート）
t0=$(date +%s%3N)
$FAV run "$PROG" > /dev/null 2>&1
t1=$(date +%s%3N)
echo "cold_start_full_ms=$((t1 - t0))"

# --precompiled（事前コンパイル済み）
$FAV compile "$PROG" -o /tmp/bench_cold.favc > /dev/null 2>&1
t2=$(date +%s%3N)
$FAV run --precompiled /tmp/bench_cold.favc > /dev/null 2>&1
t3=$(date +%s%3N)
echo "cold_start_precompiled_ms=$((t3 - t2))"
```

### 02_csv_10gb.fav

```favnir
import rune "io"
import rune "csv"

// CI=true のとき 1MB サンプルで代用
fn main() -> Result<Unit, String> {
  bind is_ci    <- IO.env_or("CI", "false")
  bind path     <- if is_ci == "true" {
    Result.ok("benchmarks/fixtures/sample_1mb.csv")
  } else {
    Result.ok("benchmarks/fixtures/sample_10gb.csv")
  }
  bind t0 <- IO.time_ms()
  bind _  <- Csv.stream_file(path, |_row| Result.ok(()))
  bind t1 <- IO.time_ms()
  IO.println(f"csv_10gb_throughput_mbs={t1 - t0}")
}
```

### 03_tight_loop.fav

```favnir
fn tight_loop(n: Int, acc: Int) -> Int {
  if n <= 0 { acc }
  else { tight_loop(n - 1, acc + n) }
}

fn main() -> Result<Unit, String> {
  bind t0  <- IO.time_ms()
  bind _   <- tight_loop(10_000_000, 0)
  bind t1  <- IO.time_ms()
  IO.println(f"tight_loop_10m_iter_ms={t1 - t0}")
}
```

### 04_record_transform.fav

```favnir
import rune "io"

type Row = { id: Int, name: String, amount: Float }
type Out = { id: Int, name: String, amount: Float, label: String }

fn transform_row(r: Row) -> Out {
  { id: r.id, name: r.name, amount: r.amount * 1.1, label: "processed" }
}

fn main() -> Result<Unit, String> {
  bind rows <- generate_rows(1_000_000)
  bind t0   <- IO.time_ms()
  bind _    <- List.map(rows, transform_row)
  bind t1   <- IO.time_ms()
  IO.println(f"record_transform_1m_ms={t1 - t0}")
}
```

### 05_compile_time.sh

```bash
#!/usr/bin/env bash
set -e

FAV="${FAV:-./fav/target/release/fav}"
SRC="${BENCH_SRC:-fav/self/compiler.fav}"

# cold compile（キャッシュ削除後）
rm -rf ~/.fav/cache/
t0=$(date +%s%3N)
$FAV build "$SRC" > /dev/null 2>&1 || true
t1=$(date +%s%3N)
echo "compile_cold_ms=$((t1 - t0))"

# incremental（1ファイル変更後）
touch "$SRC"
t2=$(date +%s%3N)
$FAV build "$SRC" > /dev/null 2>&1 || true
t3=$(date +%s%3N)
echo "compile_incremental_ms=$((t3 - t2))"
```

### 06_duckdb_query.fav

```favnir
import rune "duckdb"

fn main() -> Result<Unit, String> {
  bind db  <- DuckDb.open(":memory:")
  bind _   <- DuckDb.execute(db, "CREATE TABLE t AS SELECT * FROM range(1000000)")
  bind t0  <- IO.time_ms()
  bind _   <- DuckDb.query(db, "SELECT sum(range) FROM t")
  bind t1  <- IO.time_ms()
  IO.println(f"duckdb_query_sum_1m_ms={t1 - t0}")
}
```

### 07_arrow_parquet.fav

```favnir
fn main() -> Result<Unit, String> {
  bind rows <- generate_rows(1_000_000)
  bind batch <- ArrowBatch.from_list(rows)
  match batch {
    Err(e) => Result.err(e)
    Ok(b) => {
      bind t0 <- IO.time_ms()
      bind _  <- ArrowBatch.write_parquet(b, "benchmarks/fixtures/out_bench.parquet")
      bind t1 <- IO.time_ms()
      IO.println(f"arrow_parquet_write_1gb_ms={t1 - t0}")
    }
  }
}
```

### 08_concurrent_stages.fav

```favnir
fn main() -> Result<Unit, String> {
  bind t0 <- IO.time_ms()
  bind _  <- seq ParBench = par [StageA, StageB, StageC] |> Merge
  bind t1 <- IO.time_ms()
  IO.println(f"concurrent_stages_3way_ms={t1 - t0}")
}
```

---

## T2: `benchmarks/suite/run_all.sh`

```bash
#!/usr/bin/env bash
# 全ベンチマークスイートを実行し JSON を出力する
# 使用例: bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json

set -e
SUITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAV="${FAV:-./fav/target/release/fav}"
# --format json (default)
FORMAT="json"
for arg in "$@"; do
  case "$arg" in
    --format) FORMAT_NEXT=1 ;;
    *) [[ "$FORMAT_NEXT" == "1" ]] && FORMAT="$arg" && FORMAT_NEXT=0 ;;
  esac
done
VERSION=$(grep '^version' fav/Cargo.toml | head -1 | sed 's/.*= "//;s/"//')
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

declare -A METRICS

run_sh() {
  local script="$1"
  while IFS='=' read -r key val; do
    [[ "$key" =~ ^[a-z_]+$ ]] && METRICS["$key"]="$val"
  done < <(bash "$script" 2>/dev/null)
}

run_fav() {
  local script="$1"
  while IFS='=' read -r key val; do
    [[ "$key" =~ ^[a-z_]+$ ]] && METRICS["$key"]="$val"
  done < <($FAV run "$script" 2>/dev/null)
}

run_sh  "$SUITE_DIR/01_cold_start.sh"
run_fav "$SUITE_DIR/02_csv_10gb.fav"
run_fav "$SUITE_DIR/03_tight_loop.fav"
run_fav "$SUITE_DIR/04_record_transform.fav"
run_sh  "$SUITE_DIR/05_compile_time.sh"
run_fav "$SUITE_DIR/06_duckdb_query.fav"
run_fav "$SUITE_DIR/07_arrow_parquet.fav"
run_fav "$SUITE_DIR/08_concurrent_stages.fav"

# JSON 出力
echo "{"
echo "  \"version\": \"$VERSION\","
echo "  \"timestamp\": \"$TIMESTAMP\","
echo "  \"metrics\": {"
first=true
for key in "${!METRICS[@]}"; do
  [[ "$first" == "true" ]] || echo ","
  printf "    \"%s\": %s" "$key" "${METRICS[$key]}"
  first=false
done
echo ""
echo "  }"
echo "}"
```

---

## T3: `benchmarks/compare.fav`

```favnir
import rune "json"
import rune "io"

fn parse_flag(args: List<String>, flag: String) -> Result<String, String> {
  bind idx <- List.find_index(args, |a| a == flag)
  match List.get(args, idx + 1) {
    ok(v)  => Result.ok(v)
    err(_) => Result.err(f"missing value for {flag}")
  }
}

fn has_flag(args: List<String>, flag: String) -> Bool {
  List.any(args, |a| a == flag)
}

fn pct_change(baseline: Float, current: Float) -> Float {
  if baseline == 0.0 { 0.0 }
  else { (current - baseline) / baseline * 100.0 }
}

fn check_regressions(
  b_metrics: Map<String, Float>,
  c_metrics: Map<String, Float>,
  threshold: Int
) -> List<String> {
  bind pairs <- Map.entries(c_metrics)
  // filter_map: ok(v) で keep、err(()) で除外
  List.filter_map(pairs, |kv| {
    bind key <- kv.key
    bind cur <- kv.value
    match Map.get(b_metrics, key) {
      err(_)   => err(())  // ベースラインにないキーはスキップ
      ok(base) =>
        bind pct <- pct_change(base, cur)
        if pct > Float.from_int(threshold) {
          ok(f"{key}: +{pct}% (baseline={base}, current={cur})")
        } else { err(()) }  // 閾値内 → スキップ
    }
  })
}

fn write_results_md(data: JsonValue) -> Result<Unit, String> {
  bind version <- Json.get_string(data, "version")
  bind ts      <- Json.get_string(data, "timestamp")
  bind metrics <- Json.get_object(data, "metrics")
  bind lines   <- Map.entries(metrics)
    |> List.map(|kv| f"| {kv.key} | {kv.value} |")
  bind body    <- String.join(lines, "\n")
  bind content <- f"# Benchmark Results\n\nVersion: {version} / {ts}\n\n| Metric | Value |\n|---|---|\n{body}\n"
  IO.write_file("benchmarks/results.md", content)
}

fn main() -> Result<Unit, String> {
  bind args      <- IO.args()
  bind baseline  <- parse_flag(args, "--baseline")
  bind current   <- parse_flag(args, "--current")
  bind thr_str   <- parse_flag(args, "--threshold")
  bind threshold <- Int.parse(thr_str)
  bind emit_md   <- Result.ok(has_flag(args, "--emit-md"))

  bind b_json  <- IO.read_file(baseline)
  bind c_json  <- IO.read_file(current)
  bind b_data  <- Json.parse(b_json)
  bind c_data  <- Json.parse(c_json)

  if emit_md {
    bind _ <- write_results_md(c_data)
    ()
  } else { () }

  bind b_metrics <- Json.get_object(b_data, "metrics")
  bind c_metrics <- Json.get_object(c_data, "metrics")
  bind regressions <- Result.ok(check_regressions(b_metrics, c_metrics, threshold))

  if List.length(regressions) > 0 {
    bind _ <- IO.println(f"REGRESSION: {List.length(regressions)} metric(s) exceeded {threshold}% threshold:")
    bind _ <- List.for_each(regressions, |r| IO.println(f"  {r}"))
    Result.err("benchmark regression detected")
  } else {
    IO.println(f"OK: all metrics within {threshold}% of baseline.")
  }
}
```

---

## T4: `.github/workflows/bench.yml`

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

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            fav/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('fav/Cargo.lock') }}

      - name: Build release
        run: cargo build --release -j 8
        working-directory: fav

      - name: Run benchmarks
        env:
          CI: "true"
          FAV: ./fav/target/release/fav
        run: |
          bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json
          cat benchmarks/latest.json

      - name: Compare with baseline
        env:
          FAV: ./fav/target/release/fav
        run: |
          $FAV run benchmarks/compare.fav \
            -- --baseline benchmarks/v20.0.0.json \
               --current  benchmarks/latest.json \
               --threshold 10 \
               --emit-md
```

---

## T5: `benchmarks/v20.0.0.json`

```json
{
  "version": "20.0.0",
  "timestamp": "2026-06-18T00:00:00Z",
  "_note": "Initial reference values. Updated by CI on first run.",
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

---

## T6: `fav/Cargo.toml` バージョン更新

`version = "20.0.0"` → `"20.1.0"`

---

## T7: `fav/src/driver.rs` — `v201000_tests` 追加

`v200000_tests::version_is_20_0_0` に `#[ignore]` を追加。

```rust
// ── v201000_tests (v20.1.0) — ベンチマーク基盤整備 ──────────────────────────
#[cfg(test)]
mod v201000_tests {
    #[test]
    fn version_is_20_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.1.0"), "Cargo.toml should have version 20.1.0");
    }

    #[test]
    fn bench_suite_files_exist() {
        assert!(std::path::Path::new("../benchmarks/suite/run_all.sh").exists(),
            "benchmarks/suite/run_all.sh should exist");
        assert!(std::path::Path::new("../benchmarks/suite/01_cold_start.sh").exists(),
            "benchmarks/suite/01_cold_start.sh should exist");
    }

    #[test]
    fn bench_compare_fav_exists() {
        assert!(std::path::Path::new("../benchmarks/compare.fav").exists(),
            "benchmarks/compare.fav should exist");
    }

    #[test]
    fn bench_workflow_exists() {
        assert!(std::path::Path::new("../.github/workflows/bench.yml").exists(),
            ".github/workflows/bench.yml should exist");
    }

    #[test]
    fn bench_baseline_valid_json() {
        let path = std::path::Path::new("../benchmarks/v20.0.0.json");
        assert!(path.exists(), "benchmarks/v20.0.0.json should exist");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"metrics\""),
            "v20.0.0.json should contain metrics field");
    }
}
```

---

## 注意点

### `compare.fav` の parse チェック

T7 のテストには `compare.fav` の Favnir パースチェックは含まない
（`fav run` コマンドへの依存が生まれるため）。
パースチェックは手動で `fav check benchmarks/compare.fav` で確認する。

### CI での実測値への更新フロー

1. v20.1.0 をコミット（`benchmarks/v20.0.0.json` は参考値）
2. `bench.yml` が master で初回実行される
3. `latest.json` が生成されるが、baseline 比較は参考値のため警告のみ
4. v20.2.0 以降の実装前に CI を手動実行し、`benchmarks/v20.0.0.json` を実測値で更新してコミット

### `benchmarks/` ディレクトリの既存ファイル

v20.0.0 で `benchmarks/` 配下に作成済みのファイルは変更しない。
`benchmarks/suite/` はサブディレクトリとして新設する。
