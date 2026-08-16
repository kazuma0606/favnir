# v70.3.0 Plan — `fav bench` サブコマンド完成

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: `cmd_bench_all()` を driver.rs に追加

`cmd_bench` 関数の直後（行 6603 付近）に追加。`chrono` は Cargo.toml に既に登録済み。`include_str!("../self/checker.fav")` のパスは driver.rs からの相対パス（`fav/src/` → `fav/self/`）で有効。

```rust
/// Run built-in intrinsic benchmarks and return results as JSON.
/// Measures:
///   compile_hello_fav_ms      — Parser::parse_str + build_artifact on hello.fav
///   run_csv_1k_rows_ms        — csv::Reader parse of 1000-row in-memory CSV
///   type_check_checker_fav_ms — Parser::parse_str on checker.fav (3000+ lines)
pub fn cmd_bench_all() -> String {
    use std::time::Instant;

    // 1. compile_hello_fav_ms — フロントエンド + バイトコード生成
    let hello_ms = {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { add(1, 2) == 3 }\n";
        let t = Instant::now();
        if let Ok(prog) = crate::frontend::parser::Parser::parse_str(src, "hello.fav") {
            let _ = build_artifact(&prog);
        }
        t.elapsed().as_millis() as u64
    };

    // 2. run_csv_1k_rows_ms — 1000 行 CSV のパース
    let csv_ms = {
        let mut data = String::from("id,value\n");
        for i in 0..1000u32 {
            data.push_str(&format!("{},{}\n", i, i * 2));
        }
        let t = Instant::now();
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        for result in rdr.records() {
            let _ = result;
        }
        t.elapsed().as_millis() as u64
    };

    // 3. type_check_checker_fav_ms — 大規模ソース（3000+ 行）のパース
    let checker_src = include_str!("../self/checker.fav");
    let type_check_ms = {
        let t = Instant::now();
        let _ = crate::frontend::parser::Parser::parse_str(checker_src, "checker.fav");
        t.elapsed().as_millis() as u64
    };

    let version = env!("CARGO_PKG_VERSION");
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    serde_json::json!({
        "version": version,
        "timestamp": timestamp,
        "metrics": {
            "compile_hello_fav_ms": hello_ms,
            "run_csv_1k_rows_ms": csv_ms,
            "type_check_checker_fav_ms": type_check_ms,
        }
    }).to_string()
}
```

確認: `cargo test` で既存テストが引き続き pass することを確認。

---

### Step 2: `BenchOpts` に `all: bool` を追加し、main.rs を修正

#### 2a. driver.rs の `BenchOpts` 構造体に `all: bool` を追加

```rust
pub struct BenchOpts {
    pub file: Option<String>,
    pub filter: Option<String>,
    pub runs: u64,
    pub warmup: u64,
    pub json: bool,
    pub stream: bool,
    pub compare: Option<String>,
    pub fail_on_regression: bool,
    pub threshold: f64,
    pub all: bool,  // v70.3.0
}
```

`Default` 実装に `all: false` を追加。

#### 2b. main.rs の `--all` アームを `opts.all = true; i += 1;` に変更

```rust
"--all" => {
    opts.all = true;
    i += 1;
}
```

#### 2c. main.rs のループ後（`cmd_bench(&opts)` の直前）に `--all` 処理を挿入

```rust
// v70.3.0: --all → built-in intrinsic benchmarks
if opts.all {
    let all_json = driver::cmd_bench_all();
    if let Some(cp) = opts.compare.as_deref() {
        let baseline = std::fs::read_to_string(cp).unwrap_or_else(|e| {
            eprintln!("error: cannot read baseline {cp}: {e}");
            process::exit(1);
        });
        let (ok, report) = driver::cmd_bench_compare(&baseline, &all_json, opts.threshold, false);
        println!("{report}");
        if !ok && opts.fail_on_regression {
            process::exit(1);
        }
    } else {
        println!("{all_json}");
    }
    // --all 処理後は .bench.fav 実行をスキップ
}
// （--all でない場合のみ cmd_bench を呼ぶ）
let ok = if !opts.all { cmd_bench(&opts) } else { true };
if !ok && opts.fail_on_regression {
    process::exit(1);
}
```

確認: `cargo test` で既存テストが引き続き pass することを確認。

---

### Step 3: `v703000_tests` モジュール追加

driver.rs の末尾に追加:

```rust
#[cfg(test)]
mod v703000_tests {
    #[test]
    fn bench_subcommand_all_outputs_json() {
        let result = super::cmd_bench_all();
        let v: serde_json::Value = serde_json::from_str(&result)
            .expect("cmd_bench_all should return valid JSON");
        assert!(v["version"].is_string(), "JSON should have 'version' field");
        assert!(v["timestamp"].is_string(), "JSON should have 'timestamp' field");
        let metrics = &v["metrics"];
        assert!(metrics.is_object(), "JSON should have 'metrics' object");
        assert!(metrics["compile_hello_fav_ms"].is_number(), "should have compile_hello_fav_ms");
        assert!(metrics["run_csv_1k_rows_ms"].is_number(), "should have run_csv_1k_rows_ms");
        assert!(metrics["type_check_checker_fav_ms"].is_number(), "should have type_check_checker_fav_ms");
    }

    #[test]
    fn bench_subcommand_regression_fail() {
        // baseline に非常に速い値を設定 → 実測値との比較で regression を検出
        let baseline = r#"{"version":"0.0.0","metrics":{"compile_hello_fav_ms":0,"run_csv_1k_rows_ms":0,"type_check_checker_fav_ms":0}}"#;
        let current = r#"{"version":"70.3.0","metrics":{"compile_hello_fav_ms":1000,"run_csv_1k_rows_ms":1000,"type_check_checker_fav_ms":1000}}"#;
        let (ok, _report) = super::cmd_bench_compare(baseline, current, 5.0, false);
        assert!(!ok, "should detect regression when current is much slower than baseline");
    }
}
```

確認: `cargo test v703000` で 2 件 pass することを確認。

---

### Step 4: Cargo.toml バージョン更新

- `version = "70.2.0"` → `"70.3.0"`
- driver.rs 内の旧バージョン文字列を `replace_all: true` で一括更新

---

### Step 5: CHANGELOG.md 更新

v70.3.0 エントリを v70.2.0 の直前に追加:

```markdown
## [v70.3.0] — 2026-08-09 — fav bench サブコマンド完成

### Added
- `cmd_bench_all()`: 組み込みベンチマーク（compile_hello_fav_ms / run_csv_1k_rows_ms / type_check_checker_fav_ms）を計測して JSON 出力
- `fav bench --all`: built-in intrinsic metrics を JSON で出力（`--compare` / `--fail-on-regression` と組み合わせ可能）
- `v703000_tests`: 2 件追加（3563 → 3565 tests）
  - `bench_subcommand_all_outputs_json`
  - `bench_subcommand_regression_fail`

### Changed
- `fav bench --all` が no-op から built-in benchmarks 実行に変更
```

---

### Step 6: 最終確認

- `cargo test v703000` で 2 件 pass
- `cargo test` 全体で 3565 tests pass（0 failures）
- `versions/current.md` を v70.3.0 進行中に更新
