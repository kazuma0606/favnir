# v19.8.0 実装計画 — プロファイリング強化（フレームグラフ）

## 前提確認

### 既存の実装（v9.9.0）
- `backend/vm.rs`: `PROFILE_RECORDS: RefCell<Vec<(String, i64)>>` — stage 名 + elapsed ms
- `backend/vm.rs`: `clear_profile_records()` / `take_profile_dump_json()` — JSON 文字列返却
- `backend/vm.rs`: `"Env.profile_timed_raw"` builtin — stage 呼び出し時間計測
- `driver.rs`: `cmd_profile(path, out_fmt)` — "json" か table 形式で出力
- `driver.rs`: `render_profile_table(json)` — stage 別 ms 表（既存の text 相当）
- `main.rs`: `Some("profile")` ブランチ（`--format` / `--runs` / `--stage` は未実装）

### 追加依存
- `inferno = "0.11"` — flamegraph SVG 生成（native-only deps に追加）

### 注意事項
- `inferno` の API:
  ```rust
  inferno::flamegraph::from_lines(&mut opts, lines_iter, &mut writer) -> Result<(), Error>
  ```
  `lines_iter` は `Iterator<Item = &str>`（折り畳みスタック形式）
- 折り畳みスタック形式: `"親;子 カウント"` — カウントは ms をそのまま使用
- `inferno::flamegraph::Options::default()` でデフォルト設定

## 実装順序

```
T1: Cargo.toml — inferno 追加
    ↓
T2: src/profiler/mod.rs + collector.rs — StageRecord / parse_profile_json / to_folded_stacks
    ↓
T3: src/profiler/flamegraph.rs — generate_svg
    ↓
T4: src/profiler/report.rs — format_text_report / format_json_report
    ↓
T5: src/lib.rs + src/main.rs — mod profiler 登録
    ↓
T6: driver.rs — cmd_profile 拡張（--format / --runs / --stage / --out）
    ↓
T7: v198000_tests 追加（5件）
    ↓
T8: Cargo.toml バージョン更新（19.7.0 → 19.8.0）
    ↓
T9: site/content/docs/tools/profiling.mdx 作成
```

## 各タスクの詳細

### T1: Cargo.toml

```toml
# native-only deps に追加
inferno = { version = "0.11", default-features = false, features = ["flamegraph"] }
```

### T2: src/profiler/collector.rs

```rust
#[derive(Debug, Clone)]
pub struct StageRecord {
    pub name: String,
    pub elapsed_ms: i64,
}

pub fn parse_profile_json(json: &str) -> Vec<StageRecord> {
    #[derive(serde::Deserialize)]
    struct Raw { name: String, ms: i64 }
    let raw: Vec<Raw> = serde_json::from_str(json).unwrap_or_default();
    raw.into_iter().map(|r| StageRecord { name: r.name, elapsed_ms: r.ms }).collect()
}

pub fn to_folded_stacks(records: &[StageRecord]) -> Vec<String> {
    records.iter().map(|r| {
        format!("pipeline;{} {}", r.name, r.elapsed_ms.max(1))
    }).collect()
}

pub fn average_records(runs: Vec<Vec<StageRecord>>) -> Vec<StageRecord> {
    if runs.is_empty() { return vec![]; }
    let n = runs.len() as i64;
    // 先頭 run を基準に同名 stage の平均を計算
    let names: Vec<String> = runs[0].iter().map(|r| r.name.clone()).collect();
    names.into_iter().map(|name| {
        let total: i64 = runs.iter()
            .flat_map(|run| run.iter().filter(|r| r.name == name))
            .map(|r| r.elapsed_ms)
            .sum();
        StageRecord { name, elapsed_ms: total / n }
    }).collect()
}
```

### T3: src/profiler/flamegraph.rs

```rust
pub fn generate_svg(folded: &[String]) -> Result<Vec<u8>, String> {
    use inferno::flamegraph;
    let mut opts = flamegraph::Options::default();
    let mut svg_buf = Vec::new();
    flamegraph::from_lines(
        &mut opts,
        folded.iter().map(|s| s.as_str()),
        &mut svg_buf,
    ).map_err(|e| format!("flamegraph error: {e}"))?;
    Ok(svg_buf)
}
```

### T4: src/profiler/report.rs

```rust
pub fn format_text_report(records: &[StageRecord], label: &str) -> String {
    // 合計時間、最大 stage（HOT PATH）、表形式
}

pub fn format_json_report(records: &[StageRecord]) -> String {
    // [{"stage": ..., "ms": ..., "pct": ...}]
}
```

HOT PATH ロジック:
```rust
let max_ms = records.iter().map(|r| r.elapsed_ms).max().unwrap_or(0);
let hot_marker = if r.elapsed_ms == max_ms { "  *** HOT PATH ***" } else { "" };
```

### T5: lib.rs + main.rs

`src/lib.rs` に:
```rust
pub mod profiler;
```

`src/main.rs` に（テスト用 cfg なし — lib を通じて使用）:
`driver.rs` 内で `use crate::profiler::*` ではなく `crate::profiler::...` でアクセス。

実際には `driver.rs` は `src/bin/../` ではなく `src/main.rs` のソースから `lib.rs` 経由で参照するため、`lib.rs` への `pub mod profiler;` 追加のみで driver.rs から `crate::profiler::...` としてアクセスできる。

### T6: driver.rs — cmd_profile 拡張

新しいシグネチャ:
```rust
pub fn cmd_profile(path: &str, format: &str, runs: usize, stage_filter: Option<&str>, out: Option<&str>)
```

フロー:
1. `runs` 回ループ: `clear_profile_records()` → `run_fvc_bytes` → `take_profile_dump_json()` → `parse_profile_json`
2. `average_records(all_runs)` で平均化
3. `stage_filter` があれば records をフィルタ
4. `format` に応じて出力:
   - `"flamegraph"` → `to_folded_stacks` → `generate_svg` → ファイル書き込み（`out` or `"flamegraph.svg"`）
   - `"text"` → `format_text_report`
   - `"json"` → `format_json_report`

### T7: v198000_tests

```rust
mod v198000_tests {
    use crate::profiler::collector::{StageRecord, to_folded_stacks, parse_profile_json};
    use crate::profiler::flamegraph::generate_svg;
    use crate::profiler::report::{format_text_report, format_json_report};

    fn sample_records() -> Vec<StageRecord> {
        vec![
            StageRecord { name: "LoadCsv".into(), elapsed_ms: 45 },
            StageRecord { name: "Transform".into(), elapsed_ms: 210 },
            StageRecord { name: "Save".into(), elapsed_ms: 30 },
        ]
    }

    #[test] fn version_is_19_8_0() { ... }
    #[test] fn profile_flamegraph_generates_svg() {
        let folded = to_folded_stacks(&sample_records());
        let svg = generate_svg(&folded).expect("generate_svg");
        assert!(svg.windows(4).any(|w| w == b"<svg"), "SVG should contain <svg");
    }
    #[test] fn profile_text_output() {
        let report = format_text_report(&sample_records(), "test.fav");
        assert!(report.contains("Transform"), "should contain stage name");
        assert!(report.contains("73"), "should contain ~74% for Transform");
    }
    #[test] fn profile_json_output() {
        let json = format_json_report(&sample_records());
        assert!(json.contains("\"pct\""), "json should have pct field");
        assert!(json.contains("\"Transform\""), "json should have stage name");
    }
    #[test] fn profile_hot_path_detected() {
        let report = format_text_report(&sample_records(), "test.fav");
        assert!(report.contains("HOT PATH"), "slowest stage should be marked HOT PATH");
        // Transform(210ms) が最大なので HOT PATH マーカーが付くはず
        let transform_line = report.lines()
            .find(|l| l.contains("Transform"))
            .unwrap_or("");
        assert!(transform_line.contains("HOT PATH"), "Transform should be HOT PATH");
    }
}
```

### T8: Cargo.toml

`version = "19.7.0"` → `"19.8.0"`

### T9: ドキュメント

`site/content/docs/tools/profiling.mdx` — フレームグラフ生成ガイド

## 注意事項

### inferno の `from_lines` API

`inferno 0.11` では:
```rust
use inferno::flamegraph;
flamegraph::from_lines(&mut opts, lines, writer) -> Result<(), Error>
```

`lines` は `impl IntoIterator<Item = &str>` 相当。
空の folded スタックを渡すと空 SVG（または空バイト）が返る場合があるため、
テストでは空でないレコードを渡すこと。

### `default-features = false` の効果

`inferno 0.11` のデフォルト features には `collapse` ツール（perf/dtrace 等のパーサー）が含まれる。
`features = ["flamegraph"]` のみで `from_lines` が使える。
ただし、features 名が正確かどうかは `cargo add` で確認すること。
不確実な場合は `default-features` なし（全 features）でもよい。

### `cmd_profile` のシグネチャ変更

`main.rs` の `Some("profile")` ブランチで引数パースを拡張する。
旧シグネチャ `cmd_profile(path, out_fmt)` を新シグネチャに変更するため、
`main.rs` の呼び出し箇所も同時に更新する。
