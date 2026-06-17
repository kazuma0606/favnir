# v19.8.0 仕様書 — プロファイリング強化（フレームグラフ）

Date: 2026-06-17

## 概要

現在の `fav profile`（stage 別実行時間のみ）を、フレームグラフ SVG 生成・
テキスト詳細レポート・JSON 出力まで拡張する。ボトルネックを視覚的に特定できるようにする。

## 現状（v9.9.0 で実装済み）

```bash
fav profile src/pipeline.fav
# 出力: stage 別 ms 表（table 形式）
# Stage        Time (ms)     %
# ──────────────────────────
# LoadCsv            45   15%
# Transform         210   74%   ← 遅いのはわかるが、何が？
# Save               30   11%
```

実装: `PROFILE_RECORDS: RefCell<Vec<(String, i64)>>`（stage 名 + elapsed ms）
`take_profile_dump_json()` → JSON 文字列、`cmd_profile(path, out_fmt)`

## 目標（v19.8.0 以降）

```bash
# フレームグラフ SVG 生成
fav profile --format=flamegraph src/pipeline.fav
# → flamegraph.svg（ブラウザで確認）

# テキスト詳細レポート（HOT PATH 表示）
fav profile --format=text src/pipeline.fav

# JSON（CI/外部ツール向け）
fav profile --format=json src/pipeline.fav > profile.json

# 複数回実行して平均を取る
fav profile --format=text --runs=5 src/pipeline.fav

# 特定 stage のみフィルタ
fav profile --format=text --stage=Transform src/pipeline.fav
```

## CLI

```
fav profile [options] <file>

オプション:
  --format=<flamegraph|text|json>  出力形式（デフォルト: text）
  --runs=<N>                        実行回数（平均を計算、デフォルト: 1）
  --stage=<name>                    特定 stage のみ表示
  --out=<path>                      出力ファイルパス（flamegraph 用）
```

## テキストレポート形式

```
Profile: pipeline.fav (1 run, 285ms total)

Stage            Time (ms)   %
────────────────────────────────────
LoadCsv               45   15.8%
Transform            210   73.7%  *** HOT PATH ***
Save                  30   10.5%
────────────────────────────────────
Total                285  100.0%
```

- 最も時間がかかった stage に `*** HOT PATH ***` を付与
- `--runs=N` 時は N 回の平均値を表示

## フレームグラフ形式

```
入力（折り畳みスタック形式）:
  "pipeline;LoadCsv 45"
  "pipeline;Transform 210"
  "pipeline;Save 30"

↓ inferno::flamegraph::from_lines() で SVG 生成

出力: flamegraph.svg（ブラウザで開ける SVG）
```

`inferno 0.11` の `inferno::flamegraph::from_lines()` を使用。

## JSON 形式

```json
[
  { "stage": "LoadCsv",   "ms": 45,  "pct": 15.8 },
  { "stage": "Transform", "ms": 210, "pct": 73.7 },
  { "stage": "Save",      "ms": 30,  "pct": 10.5 }
]
```

既存の `take_profile_dump_json()` は `[{"name": ..., "ms": ...}]` 形式。
`pct` フィールドを追加した拡張 JSON を新しいフォーマットとして定義する。

## 実装計画

### 新規モジュール: `fav/src/profiler/`

```
src/profiler/
  mod.rs         — pub use, モジュール宣言
  collector.rs   — StageRecord, parse_profile_json, to_folded_stacks
  flamegraph.rs  — generate_svg(folded: &[String]) -> Result<Vec<u8>, String>
  report.rs      — format_text_report, format_json_report
```

### profiler/collector.rs

```rust
#[derive(Debug, Clone)]
pub struct StageRecord {
    pub name: String,
    pub elapsed_ms: i64,
}

/// Parse the JSON output from take_profile_dump_json() into StageRecord vec.
pub fn parse_profile_json(json: &str) -> Vec<StageRecord>

/// Convert records to inferno folded stack format.
/// Format: "pipeline;<stage_name> <weight>"
pub fn to_folded_stacks(records: &[StageRecord]) -> Vec<String>

/// Average N sets of records (for --runs=N).
pub fn average_records(runs: Vec<Vec<StageRecord>>) -> Vec<StageRecord>
```

### profiler/flamegraph.rs

```rust
/// Generate flamegraph SVG bytes from folded stack lines.
pub fn generate_svg(folded: &[String]) -> Result<Vec<u8>, String> {
    use inferno::flamegraph;
    let mut opts = flamegraph::Options::default();
    let mut svg_buf = Vec::new();
    flamegraph::from_lines(
        &mut opts,
        folded.iter().map(|s| s.as_str()),
        &mut svg_buf,
    ).map_err(|e| format!("flamegraph: {e}"))?;
    Ok(svg_buf)
}
```

### profiler/report.rs

```rust
/// Format a human-readable text report with HOT PATH marker.
pub fn format_text_report(records: &[StageRecord], label: &str) -> String

/// Format an enhanced JSON report with pct field.
pub fn format_json_report(records: &[StageRecord]) -> String
```

### driver.rs の変更

`cmd_profile` を拡張:
- `--format=flamegraph/text/json`（旧 `out_fmt` から変更）
- `--runs=N`: N 回実行して `average_records` で平均
- `--stage=<name>`: records をフィルタ
- `--out=<path>`: SVG 出力先（デフォルト: `flamegraph.svg`）

### Cargo.toml への追加

```toml
inferno = { version = "0.11", default-features = false, features = ["flamegraph"] }
```

`default-features = false` で `collapse` 機能を除外して依存を軽量化。

## テスト（v198000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_8_0` | Cargo.toml に `"19.8.0"` が含まれる |
| `profile_flamegraph_generates_svg` | `generate_svg` が `<svg` を含むバイト列を返す |
| `profile_text_output` | `format_text_report` が stage 名・ms・% を含む文字列を返す |
| `profile_json_output` | `format_json_report` が `pct` フィールドを含む JSON を返す |
| `profile_hot_path_detected` | 最も遅い stage に "HOT PATH" が付く |

## 完了条件

- [ ] `src/profiler/` モジュール（collector / flamegraph / report）が作成されている
- [ ] `inferno = "0.11"` が native-only deps に追加されている
- [ ] `generate_svg` が有効な SVG を生成する
- [ ] `format_text_report` が HOT PATH 表示付きレポートを生成する
- [ ] `format_json_report` が `pct` 付き JSON を生成する
- [ ] `cmd_profile` が `--format=flamegraph/text/json` / `--runs=N` / `--stage=<name>` に対応する
- [ ] `cargo test v198000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/profiling.mdx` が存在する
