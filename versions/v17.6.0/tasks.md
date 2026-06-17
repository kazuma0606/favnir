# v17.6.0 — `fav bench` 統計強化 タスク

## ステータス: 完了

---

## タスク一覧

### T1: `BenchStats` 構造体 + `compute_bench_stats` 追加

- [x] `fav/src/driver.rs` に `pub struct BenchStats` を追加
  - フィールド: `name: String`, `runs: u64`, `avg_us: f64`, `p50_us: f64`, `p95_us: f64`, `min_us: f64`, `max_us: f64`
- [x] `pub fn compute_bench_stats(name: &str, mut timings_us: Vec<f64>) -> BenchStats` を追加
  - `timings_us` をソート
  - avg = 総和 / n
  - p50 = `timings_us[(n * 50).saturating_sub(1) / 100]`
  - p95 = `timings_us[(n * 95).saturating_sub(1) / 100]`
  - min = `timings_us[0]`, max = `timings_us[n - 1]`

### T2: `format_duration_us` + `bench_stats_to_json` 追加

- [x] `pub fn format_duration_us(us: f64) -> String` を追加
  - `< 1000.0` → `"{:.1}µs"`
  - `< 1_000_000.0` → `"{:.1}ms"`
  - それ以上 → `"{:.2}s"`
- [x] `pub fn bench_stats_to_json(results: &[BenchStats]) -> String` を追加
  - `serde_json::json!` で `{ "benchmarks": [...] }` 形式を返す

### T3: `BenchOpts` 構造体追加

- [x] `pub struct BenchOpts` を追加
  - フィールド: `file: Option<String>`, `filter: Option<String>`, `runs: u64`, `warmup: u64`, `json: bool`
- [x] `impl Default for BenchOpts` を追加（runs=100, warmup=5, json=false）

### T4: `exec_bench_case_timed` 追加

- [x] `fn exec_bench_case_timed(prog, description, runs, warmup) -> Result<Vec<f64>, String>` を追加
  - ウォームアップ: `warmup` 回 `VM::run`（時間計測なし）
  - 本計測: `runs` 回、各イテレーションを `std::time::Instant` で個別計測
  - 各イテレーションの µs 時間を `Vec<f64>` で返す
  - 旧 `exec_bench_case`（平均のみ）を置き換え

### T5: `cmd_bench` 更新

- [x] `cmd_bench` のシグネチャを `pub fn cmd_bench(opts: &BenchOpts)` に変更
- [x] 内部で `exec_bench_case_timed` を使うよう更新
- [x] 各ベンチの結果を `compute_bench_stats` で集計
- [x] `!opts.json` の場合: avg/p50/p95/min/max 形式で表示
- [x] `opts.json` の場合: `bench_stats_to_json` で JSON 出力

### T6: `main.rs` 更新

- [x] `Some("bench")` ブランチを `BenchOpts::default()` ベースに書き換え
- [x] `--runs` オプション追加（`opts.runs` に設定）
- [x] `--iters` は `--runs` と同義として維持（後方互換）
- [x] `--warmup` オプション追加（`opts.warmup` に設定）
- [x] `--json` フラグ追加（`opts.json = true`）
- [x] ヘルプテキスト更新
- [x] `cmd_bench(file, filter, iters)` → `cmd_bench(&opts)` に変更

### T7: テスト追加（`fav/src/driver.rs`）

- [x] `v176000_tests` モジュールを `driver.rs` に追加

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
        assert!(stats.p50_us >= 40.0 && stats.p50_us <= 60.0, "p50 should be around 50");
    }

    #[test]
    fn bench_filter_option() {
        let src = "bench \"transform rows\" { 1 + 1 }\nbench \"json parse\" { 2 + 2 }\n";
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

- [x] `cargo test v176000` — 5/5 PASS
- [x] `cargo test` — 1641 tests、リグレッションなし（v175000_tests の version チェックテストを削除）

### T8: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `17.6.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T9: ドキュメント

- [x] `site/content/docs/tools/bench.mdx` を新規作成
  - 基本構文（`bench "..." { }` ブロック）
  - 出力形式（avg/p50/p95/min/max）
  - オプション一覧（`--runs`・`--warmup`・`--filter`・`--json`）
  - ユースケース例

---

## 完了条件チェックリスト

- [x] `BenchStats` 構造体が定義されている
- [x] `compute_bench_stats` が avg/p50/p95/min/max を正しく返す
- [x] `format_duration_us` が µs/ms/s を自動選択する
- [x] `bench_stats_to_json` が `{ "benchmarks": [...] }` 形式を返す
- [x] `cmd_bench` が `BenchOpts` を受け取る
- [x] `--warmup N` オプションが動作する
- [x] `--json` フラグで JSON 出力が得られる
- [x] `cargo test v176000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（BenchStats + compute_bench_stats）
→ T2（format_duration_us + bench_stats_to_json）
→ T3（BenchOpts）
→ T4（exec_bench_case_timed）
→ T5（cmd_bench 更新）
→ T6（main.rs 更新）
→ T7（テスト）
→ T8（バージョン更新）
→ T9（ドキュメント）

T7 のテストは T1〜T3 が完了してから（compute_bench_stats / bench_stats_to_json のテストが依存）。
T5/T6 は並列実施可能（BenchOpts が確定した後）。

---

## 補足: serde_json 依存の確認

```toml
# Cargo.toml に既にあるか確認
[dependencies]
serde_json = "1"
```

既存依存として含まれているはず（JSON Rune 等で使用済み）。なければ追加が必要。
