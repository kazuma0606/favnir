# v19.8.0 — プロファイリング強化（フレームグラフ） タスク

## ステータス: TODO

---

## タスク一覧

### T1: `fav/Cargo.toml` — inferno 追加

- [ ] `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追加:
  ```toml
  inferno = { version = "0.11", default-features = false, features = ["flamegraph"] }
  ```
  **注意:** `features = ["flamegraph"]` が使えない場合（features 名が違う場合）は
  `inferno = "0.11"` でデフォルト features を使う。

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/profiler/` モジュール作成

#### `fav/src/profiler/mod.rs`

- [ ] 以下の内容で作成:
  ```rust
  // v19.8.0 — フレームグラフ生成・プロファイリングレポート
  pub mod collector;
  pub mod flamegraph;
  pub mod report;
  ```

#### `fav/src/profiler/collector.rs`

- [ ] `StageRecord` struct を定義:
  ```rust
  #[derive(Debug, Clone)]
  pub struct StageRecord {
      pub name: String,
      pub elapsed_ms: i64,
  }
  ```

- [ ] `parse_profile_json(json: &str) -> Vec<StageRecord>` を実装:
  - `take_profile_dump_json()` の出力（`[{"name": ..., "ms": ...}]`）をパース
  ```rust
  #[derive(serde::Deserialize)]
  struct Raw { name: String, ms: i64 }
  ```

- [ ] `to_folded_stacks(records: &[StageRecord]) -> Vec<String>` を実装:
  - inferno の折り畳みスタック形式に変換
  - 各エントリ: `"pipeline;<stage_name> <elapsed_ms>"`
  ```rust
  records.iter().map(|r| format!("pipeline;{} {}", r.name, r.elapsed_ms.max(1))).collect()
  ```

- [ ] `average_records(runs: Vec<Vec<StageRecord>>) -> Vec<StageRecord>` を実装:
  - N 回の実行結果を同名 stage で平均化
  - 先頭 run の stage 順序を維持

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

#### `fav/src/profiler/flamegraph.rs`

- [ ] `generate_svg(folded: &[String]) -> Result<Vec<u8>, String>` を実装:
  ```rust
  pub fn generate_svg(folded: &[String]) -> Result<Vec<u8>, String> {
      use inferno::flamegraph;
      let mut opts = flamegraph::Options::default();
      let mut svg_buf = Vec::new();
      flamegraph::from_lines(
          &mut opts,
          folded.iter().map(|s| s.as_str()),
          &mut svg_buf,
      )
      .map_err(|e| format!("flamegraph error: {e}"))?;
      Ok(svg_buf)
  }
  ```

  **注意:** `inferno::flamegraph::from_lines` のシグネチャを実際に確認してから実装すること。
  バージョンによっては `lines` 引数の型が異なる場合がある。
  `cargo doc --open` または `cargo build` エラーメッセージで確認。

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

#### `fav/src/profiler/report.rs`

- [ ] `format_text_report(records: &[StageRecord], label: &str) -> String` を実装:
  - ヘッダー行: `Profile: <label> (<total_ms>ms total)`
  - 各 stage: `{name:<width>}  {ms:>10}ms  {pct:>6.1}%  [HOT PATH]`
  - 最も elapsed_ms が大きい stage に `*** HOT PATH ***` を付与
  - `total_ms = 0` の場合は "%" を 0.0% として扱う（ゼロ除算防止）

- [ ] `format_json_report(records: &[StageRecord]) -> String` を実装:
  - 出力: `[{"stage": ..., "ms": ..., "pct": ...}]`
  - `pct = elapsed_ms as f64 / total_ms as f64 * 100.0`（rounded to 1 decimal）

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/lib.rs` — `mod profiler` 登録

- [ ] `lib.rs` に追加:
  ```rust
  #[cfg(not(target_arch = "wasm32"))]
  pub mod profiler;
  ```
  **注意:** `inferno` は native-only dependency のため `#[cfg(not(target_arch = "wasm32"))]` が必要。

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/driver.rs` — `cmd_profile` 拡張

- [ ] `cmd_profile` のシグネチャを変更:
  ```rust
  // 旧: pub fn cmd_profile(path: &str, out_fmt: &str)
  // 新:
  pub fn cmd_profile(
      path: &str,
      format: &str,      // "flamegraph" | "text" | "json" (デフォルト: "text")
      runs: usize,       // 実行回数（デフォルト: 1）
      stage_filter: Option<&str>,  // --stage=<name> フィルタ
      out: Option<&str>, // --out=<path> SVG 出力先（デフォルト: "flamegraph.svg"）
  )
  ```

- [ ] 実装フロー:
  1. ソース読み込み → `compile_profiled_str`
  2. `runs` 回ループ: `clear_profile_records()` → `run_fvc_bytes` → `parse_profile_json`
  3. `average_records(all_runs)` で平均化
  4. `stage_filter` があれば `records.retain(|r| r.name == stage_filter)` でフィルタ
  5. `format` に応じて出力:
     - `"flamegraph"` → `to_folded_stacks` → `generate_svg` → ファイル書き込み
     - `"text"` → `format_text_report` → println
     - `"json"` → `format_json_report` → println
     - その他（旧 "table"）→ `render_profile_table` にフォールバック

  ```rust
  use crate::profiler::collector::{average_records, parse_profile_json, to_folded_stacks};
  use crate::profiler::flamegraph::generate_svg;
  use crate::profiler::report::{format_json_report, format_text_report};
  ```

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/main.rs` — `profile` コマンドの引数パース拡張

- [ ] `Some("profile")` ブランチを以下のように更新:
  ```rust
  Some("profile") => {
      let mut path = String::new();
      let mut format = "text".to_string();   // 旧: out_fmt
      let mut runs: usize = 1;
      let mut stage_filter: Option<String> = None;
      let mut out: Option<String> = None;
      let mut i = 2usize;
      while i < args.len() {
          match args[i].as_str() {
              s if s.starts_with("--format=") => {
                  format = s.trim_start_matches("--format=").to_string();
                  i += 1;
              }
              "--format" => {
                  format = args.get(i + 1).cloned().unwrap_or_default();
                  i += 2;
              }
              s if s.starts_with("--runs=") => {
                  runs = s.trim_start_matches("--runs=").parse().unwrap_or(1);
                  i += 1;
              }
              "--runs" => {
                  runs = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1);
                  i += 2;
              }
              s if s.starts_with("--stage=") => {
                  stage_filter = Some(s.trim_start_matches("--stage=").to_string());
                  i += 1;
              }
              "--stage" => {
                  stage_filter = args.get(i + 1).cloned();
                  i += 2;
              }
              s if s.starts_with("--out=") => {
                  out = Some(s.trim_start_matches("--out=").to_string());
                  i += 1;
              }
              "--out" => {
                  out = args.get(i + 1).cloned();
                  i += 2;
              }
              other => {
                  path = other.to_string();
                  i += 1;
              }
          }
      }
      if path.is_empty() {
          eprintln!("error: profile requires a .fav file");
          process::exit(1);
      }
      cmd_profile(&path, &format, runs, stage_filter.as_deref(), out.as_deref());
  }
  ```

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `v198000_tests` 追加（5件）

- [ ] `v197000_tests` の直後に以下のモジュールを追加:

```rust
// ── v198000_tests (v19.8.0) — フレームグラフ生成 ─────────────────────────────
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v198000_tests {
    use crate::profiler::collector::{StageRecord, parse_profile_json, to_folded_stacks};
    use crate::profiler::flamegraph::generate_svg;
    use crate::profiler::report::{format_json_report, format_text_report};

    fn sample_records() -> Vec<StageRecord> {
        vec![
            StageRecord { name: "LoadCsv".into(),   elapsed_ms: 45  },
            StageRecord { name: "Transform".into(),  elapsed_ms: 210 },
            StageRecord { name: "Save".into(),       elapsed_ms: 30  },
        ]
    }

    #[test]
    fn version_is_19_8_0() {
        assert!(
            include_str!("../Cargo.toml").contains("19.8.0"),
            "Cargo.toml should contain version 19.8.0"
        );
    }

    #[test]
    fn profile_flamegraph_generates_svg() {
        let records = sample_records();
        let folded = to_folded_stacks(&records);
        assert!(!folded.is_empty(), "folded stacks should not be empty");
        let svg = generate_svg(&folded).expect("generate_svg should succeed");
        let has_svg_tag = svg.windows(4).any(|w| w == b"<svg");
        assert!(has_svg_tag, "output should contain <svg tag, got {} bytes", svg.len());
    }

    #[test]
    fn profile_text_output() {
        let report = format_text_report(&sample_records(), "test.fav");
        assert!(report.contains("Transform"), "report should contain stage name");
        // Transform = 210ms / 285ms total ≈ 73.7%
        assert!(
            report.contains("73") || report.contains("74"),
            "report should show ~74% for Transform: {report}"
        );
    }

    #[test]
    fn profile_json_output() {
        let json = format_json_report(&sample_records());
        assert!(json.contains("\"pct\""),       "json should have pct field");
        assert!(json.contains("\"stage\""),     "json should have stage field");
        assert!(json.contains("\"Transform\""), "json should have stage name");
        // パース可能な JSON であることを確認
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("format_json_report should produce valid JSON");
        assert!(parsed.is_array(), "json report should be an array");
    }

    #[test]
    fn profile_hot_path_detected() {
        let report = format_text_report(&sample_records(), "test.fav");
        assert!(
            report.contains("HOT PATH"),
            "slowest stage should be marked as HOT PATH: {report}"
        );
        // Transform (210ms) が最も遅いので HOT PATH マーカーが付くはず
        let transform_line = report.lines()
            .find(|l| l.contains("Transform"))
            .unwrap_or("");
        assert!(
            transform_line.contains("HOT PATH"),
            "Transform line should have HOT PATH marker: '{transform_line}'"
        );
    }
}
```

- [ ] `cargo test v198000` — 5/5 PASS を確認

---

### T7: `fav/Cargo.toml` — バージョン更新

- [ ] `version = "19.7.0"` → `"19.8.0"` に変更
- [ ] 既存の `version_is_19_7_0` テストに `#[ignore]` を追加

---

### T8: `site/content/docs/tools/profiling.mdx`（新規）

- [ ] 以下の内容を含む MDX を作成:
  - プロファイリングの概要（stage 別タイミング計測）
  - `fav profile --format=flamegraph` の使い方と flamegraph.svg の確認方法
  - `fav profile --format=text` の出力例（HOT PATH 説明）
  - `fav profile --format=json` の出力例（CI での使い方）
  - `--runs=N` で複数回実行して平均を取る方法
  - `--stage=<name>` で特定 stage のみ確認する方法

---

## テスト（v198000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_8_0` | Cargo.toml に `"19.8.0"` が含まれる |
| `profile_flamegraph_generates_svg` | `generate_svg` が `<svg` を含むバイト列を返す |
| `profile_text_output` | `format_text_report` が stage 名・ms・% を含む文字列を返す |
| `profile_json_output` | `format_json_report` が `pct` フィールドを含む valid JSON を返す |
| `profile_hot_path_detected` | 最も遅い stage（Transform: 210ms）の行に "HOT PATH" が付く |

---

## 完了条件チェックリスト

- [ ] `inferno = "0.11"` が native-only deps に追加されている
- [ ] `src/profiler/collector.rs`（StageRecord / parse_profile_json / to_folded_stacks / average_records）
- [ ] `src/profiler/flamegraph.rs`（generate_svg）
- [ ] `src/profiler/report.rs`（format_text_report / format_json_report）
- [ ] `src/lib.rs` に `pub mod profiler` が追加されている
- [ ] `cmd_profile` が `--format` / `--runs` / `--stage` / `--out` に対応している
- [ ] `cargo test v198000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/profiling.mdx` が存在する

---

## 優先度

```
T1（Cargo.toml — inferno）    ← 最初
    ↓
T2（profiler/ モジュール）    ← T1 完了後
    ↓
T3（lib.rs 登録）             ← T2 完了後すぐ（build 確認のため）
    ↓
T4（driver.rs — cmd_profile 拡張）  ← T2/T3 完了後
    ↓
T5（main.rs — 引数パース拡張）      ← T4 と並列可
    ↓
T6（v198000_tests）           ← T2〜T5 完了後
    ↓
T7（Cargo.toml バージョン）   ← T6 と並列可
    ↓
T8（ドキュメント）            ← T6 完了後
```

---

## 重要な技術ノート

### inferno の API 確認方法

実装前に以下で API を確認:
```bash
cd fav && cargo doc -p inferno --open 2>/dev/null || cargo build 2>&1 | head -30
```

v0.11 の `flamegraph::from_lines` シグネチャ（推定）:
```rust
pub fn from_lines<'a>(
    opt: &mut Options,
    lines: impl IntoIterator<Item = &'a str>,
    writer: impl Write,
) -> Result<(), Error>
```

`Error` は `std::fmt::Display` を実装しているので `.to_string()` でメッセージ取得可能。

### `#[cfg(not(target_arch = "wasm32"))]` の適用

`profiler` モジュールは `inferno` に依存するため wasm32 ビルドでは除外する:
- `lib.rs`: `#[cfg(not(target_arch = "wasm32"))] pub mod profiler;`
- `v198000_tests`: `#[cfg(not(target_arch = "wasm32"))]` を mod に付与

### `cmd_profile` 旧シグネチャとの互換

`main.rs` の呼び出し箇所（`cmd_profile(&path, &out_fmt)` → 新シグネチャ）を必ず更新する。
旧 `out_fmt = "table"` は `format = "text"` へのフォールバックとして扱う。

### `parse_profile_json` のデシリアライズ形式

`take_profile_dump_json()` の出力形式（vm.rs L888-895）:
```json
[{"name": "StageName", "ms": 42}]
```
この形式に合わせて `Raw { name: String, ms: i64 }` を定義する。
