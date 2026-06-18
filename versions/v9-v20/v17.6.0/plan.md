# v17.6.0 — `fav bench` 統計強化 実装計画

## 方針

AST / IR / VM の変更は不要。`driver.rs` と `main.rs` のみ変更する。
`BenchDef` 構造体・パーサー・コンパイラは v1.8.0 実装済み。

変更の主眼:
1. `exec_bench_case` を各イテレーション個別計測版（`Vec<f64>` 返却）に刷新
2. `compute_bench_stats` で avg/p50/p95/min/max を計算
3. `cmd_bench` の引数を `BenchOpts` 構造体にまとめ、`--warmup`・`--json` を追加
4. 出力フォーマットを avg/p50/p95/min/max 表示に更新
5. テスト 5 件追加、バージョンを 17.6.0 に更新

---

## 実装ステップ

### Step 1: `BenchStats` 構造体 + `compute_bench_stats`

`cmd_bench` の近くに追加：

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

pub fn compute_bench_stats(name: &str, mut timings_us: Vec<f64>) -> BenchStats {
    assert!(!timings_us.is_empty());
    timings_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = timings_us.len();
    let avg_us = timings_us.iter().sum::<f64>() / n as f64;
    let p50_us = timings_us[(n * 50).saturating_sub(1) / 100];
    let p95_us = timings_us[(n * 95).saturating_sub(1) / 100];
    let min_us = timings_us[0];
    let max_us = timings_us[n - 1];
    BenchStats {
        name: name.to_string(),
        runs: n as u64,
        avg_us,
        p50_us,
        p95_us,
        min_us,
        max_us,
    }
}
```

### Step 2: `format_duration_us`

```rust
pub fn format_duration_us(us: f64) -> String {
    if us < 1_000.0 {
        format!("{:.1}µs", us)
    } else if us < 1_000_000.0 {
        format!("{:.1}ms", us / 1_000.0)
    } else {
        format!("{:.2}s", us / 1_000_000.0)
    }
}
```

### Step 3: `bench_stats_to_json`

```rust
pub fn bench_stats_to_json(results: &[BenchStats]) -> String {
    let entries: Vec<serde_json::Value> = results
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "runs": s.runs,
                "avg_us": s.avg_us,
                "p50_us": s.p50_us,
                "p95_us": s.p95_us,
                "min_us": s.min_us,
                "max_us": s.max_us,
            })
        })
        .collect();
    serde_json::json!({ "benchmarks": entries }).to_string()
}
```

### Step 4: `BenchOpts` 構造体

```rust
pub struct BenchOpts {
    pub file: Option<String>,
    pub filter: Option<String>,
    pub runs: u64,
    pub warmup: u64,
    pub json: bool,
}

impl Default for BenchOpts {
    fn default() -> Self {
        BenchOpts { file: None, filter: None, runs: 100, warmup: 5, json: false }
    }
}
```

### Step 5: `exec_bench_case` 刷新

既存の `exec_bench_case` を置き換える（または別名で追加し cmd_bench から新版を呼ぶ）：

```rust
fn exec_bench_case_timed(
    prog: &ast::Program,
    description: &str,
    runs: u64,
    warmup: u64,
) -> Result<Vec<f64>, String> {
    let fn_name = format!("$bench:{}", description);
    let artifact = build_artifact(prog);
    let fn_idx = artifact
        .fn_idx_by_name(&fn_name)
        .ok_or_else(|| format!("bench function not found: {fn_name}"))?;
    // warmup
    for _ in 0..warmup {
        VM::run(&artifact, fn_idx, vec![]).map_err(|e| e.message.clone())?;
    }
    // timed runs
    let mut timings = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let t = std::time::Instant::now();
        VM::run(&artifact, fn_idx, vec![]).map_err(|e| e.message.clone())?;
        timings.push(t.elapsed().as_micros() as f64);
    }
    Ok(timings)
}
```

### Step 6: `cmd_bench` 更新

```rust
pub fn cmd_bench(opts: &BenchOpts) {
    // ... programs loading (既存ロジックを流用)

    let (cases, total_discovered) = collect_bench_cases(programs, opts.filter.as_deref());
    let filtered = total_discovered.saturating_sub(cases.len());
    if cases.is_empty() {
        println!("no benchmarks found");
        return;
    }

    if !opts.json {
        println!(
            "running {} benchmark{} ({} runs, {} warmup)",
            cases.len(),
            if cases.len() == 1 { "" } else { "s" },
            opts.runs,
            opts.warmup,
        );
        println!();
    }

    let _suppress = crate::backend::vm::SuppressIoGuard::new(true);
    let mut results: Vec<BenchStats> = Vec::new();

    for (_path, desc, prog) in &cases {
        match exec_bench_case_timed(prog, desc, opts.runs, opts.warmup) {
            Ok(timings) => {
                let stats = compute_bench_stats(desc, timings);
                results.push(stats);
            }
            Err(e) => {
                if !opts.json {
                    println!("ERROR  {:<40}  {}", desc, e);
                }
            }
        }
    }

    if opts.json {
        println!("{}", bench_stats_to_json(&results));
    } else {
        let name_width = results.iter().map(|r| r.name.len()).max().unwrap_or(20) + 2;
        for s in &results {
            println!(
                "  {:<width$}  avg {:>8}  p50 {:>8}  p95 {:>8}  min {:>8}  max {:>8}",
                format!("{}:", s.name),
                format_duration_us(s.avg_us),
                format_duration_us(s.p50_us),
                format_duration_us(s.p95_us),
                format_duration_us(s.min_us),
                format_duration_us(s.max_us),
                width = name_width,
            );
        }
        println!();
        println!(
            "bench result: {} benchmark{} completed. {} filtered",
            results.len(),
            if results.len() == 1 { "" } else { "s" },
            filtered,
        );
    }
}
```

### Step 7: `main.rs` 更新

`Some("bench")` ブランチを更新：

```rust
Some("bench") => {
    let mut opts = BenchOpts::default();
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--filter" => { opts.filter = Some(args[i+1].clone()); i += 2; }
            "--runs" | "--iters" => {
                opts.runs = args[i+1].parse().unwrap_or_else(|_| {
                    eprintln!("error: --runs requires a number"); process::exit(1);
                });
                i += 2;
            }
            "--warmup" => {
                opts.warmup = args[i+1].parse().unwrap_or_else(|_| {
                    eprintln!("error: --warmup requires a number"); process::exit(1);
                });
                i += 2;
            }
            "--json" => { opts.json = true; i += 1; }
            other => { opts.file = Some(other.to_string()); i += 1; }
        }
    }
    cmd_bench(&opts);
}
```

### Step 8: テスト追加

```rust
#[cfg(test)]
mod v176000_tests {
    use super::{compute_bench_stats, bench_stats_to_json, collect_bench_cases, BenchStats};
    use crate::frontend::parser::Parser;

    #[test]
    fn version_is_17_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.6.0\""), "Cargo.toml should have version 17.6.0");
    }

    #[test]
    fn bench_def_parses() {
        let src = r#"bench "transform rows" { 1 + 1 }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let has_bench = prog.items.iter().any(|i| matches!(i, crate::ast::Item::BenchDef(_)));
        assert!(has_bench, "bench def should be parsed");
    }

    #[test]
    fn bench_stats_computed() {
        let timings = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let stats = compute_bench_stats("test", timings);
        assert_eq!(stats.runs, 10);
        assert!((stats.avg_us - 55.0).abs() < 1.0, "avg should be ~55");
        assert!((stats.min_us - 10.0).abs() < 1.0, "min should be 10");
        assert!((stats.max_us - 100.0).abs() < 1.0, "max should be 100");
        // p50 = index 4 (50th percentile of 10 items = index 4)
        assert!(stats.p50_us >= 40.0 && stats.p50_us <= 60.0, "p50 should be around 50");
    }

    #[test]
    fn bench_filter_option() {
        let src = r#"
bench "transform rows" { 1 + 1 }
bench "json parse" { 2 + 2 }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let programs = vec![("test.fav".to_string(), prog)];
        let (filtered, total) = collect_bench_cases(programs, Some("transform"));
        assert_eq!(total, 2, "should find 2 benches total");
        assert_eq!(filtered.len(), 1, "filter should match 1 bench");
        assert_eq!(filtered[0].1, "transform rows");
    }

    #[test]
    fn bench_json_output() {
        let stats = vec![BenchStats {
            name: "my bench".to_string(),
            runs: 100,
            avg_us: 1234.5,
            p50_us: 1200.0,
            p95_us: 1500.0,
            min_us: 1000.0,
            max_us: 2000.0,
        }];
        let json = bench_stats_to_json(&stats);
        assert!(json.contains("\"benchmarks\""), "json should have benchmarks key");
        assert!(json.contains("\"my bench\""), "json should have bench name");
        assert!(json.contains("\"avg_us\""), "json should have avg_us");
        assert!(json.contains("\"p50_us\""), "json should have p50_us");
        assert!(json.contains("\"p95_us\""), "json should have p95_us");
    }
}
```

### Step 9: バージョン更新

- `fav/Cargo.toml`: `17.5.0` → `17.6.0`
- `cargo build` で `Cargo.lock` 更新

### Step 10: ドキュメント

- `site/content/docs/tools/bench.mdx` 新規作成

---

## 実装順序まとめ

1. `BenchStats` + `compute_bench_stats` + `format_duration_us`
2. `bench_stats_to_json`
3. `BenchOpts` 構造体
4. `exec_bench_case_timed`（既存 `exec_bench_case` を置き換え）
5. `cmd_bench` 更新
6. `main.rs` 更新（`--warmup`・`--json`・`--runs` オプション）
7. テスト追加（v176000_tests）
8. バージョン更新（17.6.0）
9. ドキュメント

---

## リスク・注意点

- **`cmd_bench` のシグネチャ変更**: `main.rs` の呼び出し元を同時に更新しないとコンパイルエラー。
- **既存の `exec_bench_case` の扱い**: `bench_collect_bench_cases_finds_bench_defs` テスト（line 8678）が既存の `exec_bench_case` を使っていないことを確認。直接 `cmd_bench` をテストしているわけではないので安全。
- **パーセンタイル計算**: `(n * 95).saturating_sub(1) / 100` — `n=1` のときは `index=0` になる（境界値チェック済み）。
- **`serde_json` は既存依存**: `Cargo.toml` の `[dependencies]` に `serde_json` があるか確認。なければ追加。
