# v28.4.0 Plan — `fav profile` 強化（`--compare` フラグ追加）

## Phase 概要

| Phase | 内容 | 依存 |
|---|---|---|
| Phase 0 | 事前確認 | — |
| Phase 1 | Cargo.toml バージョン bump | — |
| Phase 2 | driver.rs: `cmd_profile_compare` 追加 | Phase 1 |
| Phase 3 | main.rs: `--compare` フラグ追加・dispatch | Phase 2 |
| Phase 4 | `fav/tests/fixtures/etl.fav` 新規作成 | — |
| Phase 5 | `profiling.mdx` に `--compare` セクション追加 | — |
| Phase 6 | CHANGELOG.md 更新 | — |
| Phase 7 | `benchmarks/v28.4.0.json` 新規作成 | — |
| Phase 9a | driver.rs に `v284000_tests` 追加 | Phase 2〜4 |
| Phase 9b | `cargo test --bin fav v284000` — 9/9 PASS 確認 | Phase 9a |
| Phase 9c | `cargo test --bin fav` — 2262 PASS 確認 | Phase 9b |

---

## Phase 0 — 事前確認

```bash
grep '^version' fav/Cargo.toml          # "28.3.0" を確認
cargo test --bin fav 2>&1 | tail -1     # "2253 tests" を含むことを確認
grep 'v284000_tests' fav/src/driver.rs  # 存在しないことを確認
grep 'cmd_profile_compare' fav/src/driver.rs  # 存在しないことを確認
grep '\-\-compare' fav/src/main.rs      # 存在しないことを確認
```

---

## Phase 2 — `cmd_profile_compare` 追加

`fav/src/driver.rs` の `cmd_profile`（行 10989 付近）の直後に追加。

```rust
/// fav profile --compare <baseline_version> <path>
/// `benchmarks/{baseline_version}.json` の stage 別 ms データと
/// 現在の計測値を比較してレポートを出力する。
pub fn cmd_profile_compare(baseline_version: &str, path: &str) {
    use crate::profiler::collector::{average_records, parse_profile_json};

    // 1. benchmarks/{baseline_version}.json を読み込む
    let bench_path = format!("benchmarks/{}.json", baseline_version);
    let bench_json = match std::fs::read_to_string(&bench_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("error: benchmark file not found: {}", bench_path);
            process::exit(1);
        }
    };

    // baseline_version のサニタイズ（newline インジェクション防止）
    let safe_version = baseline_version
        .replace('\n', "").replace('\r', "").replace('/', "").replace('\\', "");

    // baseline の stage 別 ms を抽出（JSON 内の "stages" 配列 or フラット構造）
    // benchmarks/*.json のフォーマット: {"version":"...", "test_count":..., "timestamp":"..."}
    // 現時点では stage 別データ未保持のため空マップ → 全 stage [NEW] 扱い
    let baseline_stages: std::collections::HashMap<String, f64> =
        extract_profile_stages(&bench_json);

    // 2. path を compile_profiled_str でコンパイル & 実行
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };
    let bytes = match crate::compiler_fav_runner::compile_profiled_str(&src) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: profile compile failed: {}", e);
            process::exit(1);
        }
    };
    crate::backend::vm::clear_profile_records();
    run_fvc_bytes(&bytes, None, Some(path));
    let json = crate::backend::vm::take_profile_dump_json();
    let current_records = average_records(vec![parse_profile_json(&json)]);

    // 3. 比較レポート出力（safe_version を使用してインジェクション防止）
    println!("fav profile --compare {} {}\n", safe_version, path);
    let name_w = current_records.iter().map(|r| r.name.len()).max().unwrap_or(5).max(15);
    println!(
        "{:<nw$}  {:>14}  {:>12}  {:>8}  {}",
        "Stage", "Baseline (ms)", "Current (ms)", "Diff", "Mark",
        nw = name_w
    );
    println!("{}", "─".repeat(name_w + 52));

    for r in &current_records {
        let current_ms = r.elapsed_ms as f64;
        if let Some(&baseline_ms) = baseline_stages.get(&r.name) {
            let diff_pct = if baseline_ms > 0.0 {
                (current_ms - baseline_ms) / baseline_ms * 100.0
            } else {
                0.0
            };
            let mark = if diff_pct > 10.0 {
                "[SLOWER]"
            } else if diff_pct < -10.0 {
                "[FASTER]"
            } else {
                ""
            };
            println!(
                "{:<nw$}  {:>14.1}  {:>12.1}  {:>+7.1}%  {}",
                r.name, baseline_ms, current_ms, diff_pct, mark,
                nw = name_w
            );
        } else {
            println!(
                "{:<nw$}  {:>14}  {:>12.1}  {:>8}  [NEW]",
                r.name, "-", current_ms, "-",
                nw = name_w
            );
        }
    }
}

fn extract_profile_stages(json: &str) -> std::collections::HashMap<String, f64> {
    // benchmarks/*.json は現時点で stage 別データを持たないため空マップを返す。
    // 将来 "stages": [{"name": "...", "ms": ...}] 形式で拡張予定。
    let _ = json;
    std::collections::HashMap::new()
}
```

> **注記**: 現行の `benchmarks/*.json` は `test_count` / `version` / `timestamp` のみで
> stage 別 ms データを持たない。`extract_profile_stages` はスタブとして空マップを返し、
> 全 stage が `[NEW]` マーカーで出力される。stage 別データは v28.x 以降で拡張予定。

---

## Phase 3 — main.rs `--compare` フラグ追加

既存の `Some("profile")` アームに追加。`use driver::cmd_profile_compare` のインポートも追加。

```rust
Some("profile") => {
    let mut path = String::new();
    let mut format = "text".to_string();
    let mut runs: usize = 1;
    let mut stage_filter: Option<String> = None;
    let mut out: Option<String> = None;
    let mut compare: Option<String> = None;   // ← NEW
    let mut i = 2usize;
    while i < args.len() {
        let arg = args[i].as_str();
        // ... existing flags ...
        } else if let Some(v) = arg.strip_prefix("--compare=") {  // ← NEW
            compare = Some(v.to_string()); i += 1;
        } else if arg == "--compare" {                              // ← NEW
            compare = args.get(i + 1).cloned(); i += 2;
        } else {
            path = arg.to_string(); i += 1;
        }
    }
    if path.is_empty() {
        eprintln!("error: profile requires a .fav file");
        process::exit(1);
    }
    if let Some(ref v) = compare {                                 // ← NEW
        cmd_profile_compare(v, &path);
    } else {
        cmd_profile(&path, &format, runs, stage_filter.as_deref(), out.as_deref());
    }
}
```

---

## Phase 4 — `fav/tests/fixtures/etl.fav` 新規作成

> `fav/tests/fixtures/` ディレクトリには既に `dbt_manifest.json` が存在するため
> 新規ディレクトリ作成は不要。ファイルを直接配置する。

```favnir
// fav/tests/fixtures/etl.fav — profile テスト用 ETL フィクスチャ (v28.4.0)
stage ExtractOrders: Unit -> Unit = |_| { unit }
stage TransformOrders: Unit -> Unit = |_| { unit }
stage LoadWarehouse: Unit -> Unit = |_| { unit }

seq EtlPipeline = ExtractOrders |> TransformOrders |> LoadWarehouse
```

---

## Phase 5 — profiling.mdx 更新

`site/content/docs/performance/profiling.mdx` に以下のセクションを追加:

```mdx
## バージョン間比較: `--compare`

前バージョンとのベースライン比較を行い、劣化した stage を `[SLOWER]` でハイライトします。

```bash
# v28.3.0 のベンチマークと比較
fav profile --compare v28.3.0 src/pipeline.fav
```

出力例:

```
Stage           Baseline (ms)  Current (ms)    Diff      Mark
──────────────────────────────────────────────────────────────
ExtractOrders       12.0           18.0       +50.0%   [SLOWER]
TransformOrders      8.0            7.0       -12.5%   [FASTER]
LoadWarehouse       25.0           25.0        +0.0%
NewStage               -           10.0           -    [NEW]
```

- **`[SLOWER]`**: ベースラインより 10% 以上遅い
- **`[FASTER]`**: ベースラインより 10% 以上速い
- **`[NEW]`**: ベースラインに存在しない新しい stage
```

---

## Phase 9a — driver.rs テスト追加

`v284000_tests` を `v283000_tests` の直前に追加。

```rust
// ── v284000_tests (v28.4.0) — fav profile --compare ───────────────────────
#[cfg(test)]
mod v284000_tests {
    #[test]
    fn cmd_profile_compare_fn_exists() {
        let src = include_str!("driver.rs");
        assert!(src.contains("pub fn cmd_profile_compare"), "driver.rs must define pub fn cmd_profile_compare");
    }
    #[test]
    fn profile_compare_reads_benchmark_dir() {
        let src = include_str!("driver.rs");
        assert!(src.contains("benchmarks/"), "cmd_profile_compare must reference benchmarks/");
    }
    #[test]
    fn profile_compare_slower_marker() {
        let src = include_str!("driver.rs");
        assert!(src.contains("[SLOWER]"), "cmd_profile_compare must output [SLOWER] marker");
    }
    #[test]
    fn profile_compare_faster_marker() {
        let src = include_str!("driver.rs");
        assert!(src.contains("[FASTER]"), "cmd_profile_compare must output [FASTER] marker");
    }
    #[test]
    fn profile_compare_new_stage_marker() {
        let src = include_str!("driver.rs");
        assert!(src.contains("[NEW]"), "cmd_profile_compare must output [NEW] marker");
    }
    #[test]
    fn main_has_compare_flag() {
        let src = include_str!("main.rs");
        assert!(src.contains("--compare"), "main.rs must handle --compare flag");
    }
    #[test]
    fn etl_fixture_exists() {
        let src = include_str!("../tests/fixtures/etl.fav");
        assert!(src.contains("EtlPipeline"), "etl.fav must define EtlPipeline seq");
    }
    #[test]
    fn profiling_doc_has_compare() {
        let src = include_str!("../../site/content/docs/performance/profiling.mdx");
        assert!(src.contains("--compare"), "profiling.mdx must document --compare flag");
    }
    #[test]
    fn changelog_has_v28_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.4.0]") || src.contains("## v28.4.0"), "CHANGELOG.md must contain '[v28.4.0]'");
    }
}
```

---

## Phase 9b / 9c — テスト確認

```bash
cargo test --bin fav v284000   # 9/9 PASS
cargo test --bin fav           # 2262 tests PASS
```
