# Plan: v53.2.0 — bench × par 統合（par stage 個別計測）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3165 passed, 0 failed を確認

rg -n "par_stages" fav/src/driver.rs          # → 0 件（未実装確認）
rg -n "collect_par_stage_names" fav/src/driver.rs  # → 0 件
rg -n "v53200_tests" fav/src/driver.rs         # → 0 件
rg -n "v53100_tests" fav/src/driver.rs         # → 行番号を特定（挿入位置）
```

`BenchStats` の現状フィールドを確認:
- `rg -n "pub struct BenchStats" fav/src/driver.rs` → 行番号特定
- `par_stages` フィールドが存在しないこと

`bench_stats_to_json` の現状を確認:
- `rg -n "bench_stats_to_json" fav/src/driver.rs` → 行番号特定

---

## ステップ 2: `collect_par_stage_names` 追加

`driver.rs` の bench セクション（`// ── bench ──` コメント直下、`collect_bench_cases` の前後）に追加:

```rust
/// v53.2.0: AST から par/par_distributed ブロック内の stage 名を収集する。
/// FlwDef.steps の FlwStep::Par / ParDistributed から重複なしで抽出。
pub fn collect_par_stage_names(program: &ast::Program) -> Vec<String> {
    let mut names = Vec::new();
    for item in &program.items {
        if let ast::Item::FlwDef(flw) = item {
            for step in &flw.steps {
                match step {
                    ast::FlwStep::Par(ns) | ast::FlwStep::ParDistributed(ns) => {
                        for n in ns {
                            if !names.contains(n) {
                                names.push(n.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    names
}
```

---

## ステップ 3: `BenchStats` に `par_stages` フィールド追加

```rust
pub struct BenchStats {
    pub name: String,
    pub runs: u64,
    pub avg_us: f64,
    pub p50_us: f64,
    pub p95_us: f64,
    pub min_us: f64,
    pub max_us: f64,
    pub par_stages: Vec<String>,  // v53.2.0: par ブロック内 stage 名
}
```

`compute_bench_stats` の返却値に `par_stages: vec![]` を追加:
```rust
BenchStats { name: name.to_string(), runs: n as u64, avg_us, p50_us, p95_us, min_us, max_us, par_stages: vec![] }
```

**注意**: `v176000_tests::bench_json_output`（driver.rs 行 32061）も `BenchStats { ... }` リテラルを直接構築している。
`par_stages` フィールドを追加すると Rust の struct literal 網羅性チェックによりコンパイルエラーになるため、
`bench_json_output` のリテラルにも `par_stages: vec![]` を追加する:
```rust
let stats = vec![BenchStats {
    name: "my bench".to_string(),
    // ... 既存フィールド ...
    par_stages: vec![],  // 追加
}];
```

`bench_stats_to_compare_json` は `BenchStats` フィールドを直接参照しないため変更不要。

---

## ステップ 4: `bench_stats_to_json` 更新

```rust
serde_json::json!({
    "name":       s.name,
    "runs":       s.runs,
    "avg_us":     s.avg_us,
    "p50_us":     s.p50_us,
    "p95_us":     s.p95_us,
    "min_us":     s.min_us,
    "max_us":     s.max_us,
    "par_stages": s.par_stages,  // v53.2.0 追加
})
```

---

## ステップ 5: `cmd_bench` — `par_stages` 後付け

`cmd_bench` 内の bench case 処理ループで、`compute_bench_stats` 後に `par_stages` を付加:

```rust
match exec_bench_case_timed(prog, desc, opts.runs, opts.warmup) {
    Ok(timings) => {
        let mut stats = compute_bench_stats(desc, timings);
        stats.par_stages = collect_par_stage_names(prog);  // v53.2.0 追加
        all_stats.push(stats);
    }
    // ...
}
```

---

## ステップ 6: `driver.rs` — `v53200_tests` 追加

`v53100_tests` モジュールの直前に `v53200_tests` を追加:

```rust
// -- v53200_tests (v53.2.0) -- bench × par 統合 --
#[cfg(test)]
mod v53200_tests {
    #[test]
    fn bench_par_stage_individual() {
        use crate::driver::collect_par_stage_names;
        use crate::frontend::parser::Parser;
        let source = r#"
stage Enrich: Int -> Int = |n| { n }
stage Validate: Int -> Int = |n| { n }
seq pipeline = par [Enrich, Validate] |> Merge.ordered
"#;
        let program = Parser::parse_str(source, "t.fav").expect("parse");
        let names = collect_par_stage_names(&program);
        assert!(names.contains(&"Enrich".to_string()), "must detect Enrich");
        assert!(names.contains(&"Validate".to_string()), "must detect Validate");
    }

    #[test]
    fn bench_par_stage_total() {
        use crate::driver::{bench_stats_to_json, BenchStats};
        let stats = BenchStats {
            name: "parallel pipeline".to_string(),
            runs: 10,
            avg_us: 18900.0,
            p50_us: 18800.0,
            p95_us: 19500.0,
            min_us: 18000.0,
            max_us: 20000.0,
            par_stages: vec!["Enrich".to_string(), "Validate".to_string()],
        };
        let json = bench_stats_to_json(&[stats]);
        assert!(json.contains("par_stages"), "JSON must include par_stages field");
        assert!(json.contains("Enrich"), "JSON must include Enrich in par_stages");
        assert!(json.contains("Validate"), "JSON must include Validate in par_stages");
    }
}
```

---

## ステップ 7: `fav/Cargo.toml` バージョン更新

`version = "53.1.0"` → `version = "53.2.0"`

**注意**: v53100_tests にはバージョンピンテストが存在しないため、
空化が必要な既存テストはない。

---

## ステップ 8: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3167 passed, 0 failed

---

## ステップ 9: 後処理

- `CHANGELOG.md` に v53.2.0 エントリ追加
- `versions/current.md` を v53.2.0（3167 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.2.0 実績欄を COMPLETE に更新（推定 3161 → 実績 3167 に修正）
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）
