# v19.8.0 — プロファイリング強化（フレームグラフ） タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/Cargo.toml` — inferno 追加

- [x] native-only deps に `inferno = "0.11"` を追加
  （`features = ["flamegraph"]` は存在しないためデフォルト features を使用）

---

### T2: `fav/src/profiler/` モジュール作成

- [x] `profiler/mod.rs` — `pub mod collector; pub mod flamegraph; pub mod report;`
- [x] `profiler/collector.rs`:
  - `StageRecord { name, elapsed_ms }`
  - `parse_profile_json(json) -> Vec<StageRecord>`（`[{"name": ..., "ms": ...}]` 形式）
  - `to_folded_stacks(records) -> Vec<String>`（`"pipeline;<name> <ms>"` 形式）
  - `average_records(runs) -> Vec<StageRecord>`（N 回の平均）
- [x] `profiler/flamegraph.rs`:
  - `generate_svg(folded) -> Result<Vec<u8>, String>`
  - `inferno::flamegraph::from_lines(&mut opts, iter, &mut buf)` 使用
  - 空 folded の場合は最小限の SVG を返す
- [x] `profiler/report.rs`:
  - `format_text_report(records, label) -> String`（HOT PATH マーカー付き）
  - `format_json_report(records) -> String`（`pct` フィールド付き）

---

### T3: `fav/src/lib.rs` + `src/main.rs` — `mod profiler` 登録

- [x] `lib.rs`: `#[cfg(not(target_arch = "wasm32"))] pub mod profiler;`
- [x] `main.rs`: `mod profiler;`

---

### T4: `fav/src/driver.rs` — `cmd_profile` 拡張

- [x] シグネチャ変更:
  `cmd_profile(path, format, runs, stage_filter: Option<&str>, out: Option<&str>)`
- [x] `runs` 回ループ + `average_records` で平均化
- [x] `stage_filter` によるフィルタ（`records.retain`）
- [x] `"flamegraph"` → `generate_svg` → ファイル書き込み
- [x] `"json"` → `format_json_report`
- [x] `"text"` → `format_text_report`
- [x] その他（旧 "table"）→ `render_profile_table` フォールバック

---

### T5: `fav/src/main.rs` — `profile` コマンド引数パース拡張

- [x] `--format=<text|flamegraph|json>` / `--format <value>` 対応
- [x] `--runs=N` / `--runs N` 対応
- [x] `--stage=<name>` / `--stage <name>` 対応
- [x] `--out=<path>` / `--out <path>` 対応
- [x] `cmd_profile` の新シグネチャに合わせて呼び出し更新

---

### T6: `fav/src/driver.rs` — `v198000_tests` 追加（5件）

- [x] `v198000_tests` モジュール追加（`#[cfg(not(target_arch = "wasm32"))]`）
- [x] `version_is_19_7_0` に `#[ignore]` 追加
- [x] `cargo test v198000` — 5/5 PASS 確認

---

### T7: `fav/Cargo.toml` — バージョン更新

- [x] `version = "19.7.0"` → `"19.8.0"`

---

### T8: `site/content/docs/tools/profiling.mdx`（新規）

- [x] text / flamegraph / json 各フォーマットの使い方
- [x] `--runs=N` / `--stage=<name>` / `--out=<path>` オプション説明
- [x] CI 統合例（JSON で遅い stage を検出）
- [x] プロファイリングの仕組み説明

---

## テスト（v198000_tests、5件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_19_8_0` | Cargo.toml に `"19.8.0"` が含まれる | PASS |
| `profile_flamegraph_generates_svg` | `generate_svg` が `<svg` を含むバイト列を返す | PASS |
| `profile_text_output` | `format_text_report` が stage 名・ms・% を含む | PASS |
| `profile_json_output` | `format_json_report` が `pct` 付き valid JSON を返す | PASS |
| `profile_hot_path_detected` | Transform(210ms) 行に "HOT PATH" が付く | PASS |

---

## 完了条件チェックリスト

- [x] `inferno = "0.11"` が native-only deps に追加されている
- [x] `src/profiler/collector.rs`（StageRecord / parse_profile_json / to_folded_stacks / average_records）
- [x] `src/profiler/flamegraph.rs`（generate_svg）
- [x] `src/profiler/report.rs`（format_text_report / format_json_report）
- [x] `src/lib.rs` に `pub mod profiler` が追加されている
- [x] `cmd_profile` が `--format` / `--runs` / `--stage` / `--out` に対応している
- [x] `cargo test v198000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（1741 passed, 0 failed）
- [x] `site/content/docs/tools/profiling.mdx` が存在する

---

## 実装ノート

- `inferno 0.11` に `flamegraph` feature は存在しない → デフォルト features を使用
- `from_lines` シグネチャ: `from_lines(&mut Options, impl IntoIterator<Item = &str>, impl Write) -> quick_xml::Result<()>`
- 空の folded スタックを渡すと inferno がエラーになるため、空の場合は最小限の SVG を直接返す
- `#[cfg(not(target_arch = "wasm32"))]` を `lib.rs` の `mod profiler` と `v198000_tests` に付与
