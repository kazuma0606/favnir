# v21.3.0 — テストカバレッジ HTML / LCOV タスク

## ステータス: DONE

---

## タスク一覧

### T1: `fav/src/coverage/mod.rs` — 新規モジュール作成

- [x] **事前確認**: `ls fav/src/coverage/ 2>/dev/null || echo "not exists"` でモジュール未存在を確認
- [x] **事前確認**: `grep -n "mod incremental\|mod pushdown" fav/src/lib.rs | head -5` で mod 追加箇所を確認
- [x] `fav/src/coverage/` ディレクトリを作成
- [x] `fav/src/coverage/mod.rs` を新規作成
  - [x] `CoverageFileStat` struct（path / covered / total / fn_covered / fn_total / line_hits / fn_hits）
  - [x] `CoverageSummary` struct（files: Vec<CoverageFileStat>）と `total_covered()` / `total_lines()` / `overall_pct()` メソッド
  - [x] `format_coverage_summary_console(summary, threshold) -> String`（`Coverage: XX.X%` + `✓`/`✗` 行）
  - [x] `generate_coverage_html(summary, source_map) -> String`（インライン CSS、行ハイライト、`covered`/`uncovered`/`nonexec` クラス）
  - [x] `HTML_HEADER` / `HTML_FOOTER` 定数（外部依存なし）
  - [x] `generate_lcov(summary) -> String`（`TN:`/`SF:`/`FN:`/`FNDA:`/`DA:`/`end_of_record`）
- [x] `driver.rs` の `is_executable_line` を `pub(crate)` に昇格（`fn is_executable_line` → `pub(crate) fn is_executable_line`）
- [x] `fav/src/lib.rs` に `#[cfg(not(target_arch = "wasm32"))] pub mod coverage;` を追加
- [x] `fav/src/main.rs` に `#[cfg(not(target_arch = "wasm32"))] mod coverage;` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/driver.rs` — `cmd_test` 更新

- [x] **事前確認**: `grep -n "coverage_report_dir\|format_coverage_report\|full_report" fav/src/driver.rs | head -10` で既存 coverage 出力を確認
- [x] `pub use crate::coverage::{CoverageFileStat, CoverageSummary, format_coverage_summary_console, generate_coverage_html, generate_lcov}` を追加（`#[cfg(not(target_arch = "wasm32"))]` ガード付き）
- [x] `cmd_test` シグネチャに `coverage_html: bool` / `coverage_lcov: bool` 引数を追加
  - 注意: シグネチャ変更後は `main.rs` 側（T3）も同時に更新しないとコンパイルエラー。T2 の `cargo check` は T3 完了後に実施する。
- [x] `cmd_test` 内の `if coverage { ... }` ブロックを更新:
  - [x] `CoverageSummary` を組み立てる（`is_executable_line` 流用、`line_hits` を格納）
  - [x] `format_coverage_summary_console` でコンソールサマリーを出力（既存テキストレポートも維持）
  - [x] `coverage_html` が true かつ `coverage_report_dir` が Some のとき `index.html` を書き出す
  - [x] `coverage_lcov` が true かつ `coverage_report_dir` が Some のとき `lcov.info` を書き出す
- [x] 既存の `coverage.txt` 書き出しは後方互換のために維持
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/main.rs` — CLI 更新

- [x] **事前確認**: `grep -n "\-\-coverage\|\-\-coverage-report" fav/src/main.rs | head -10` で既存パースを確認
- [x] ヘルプ文言を `--coverage [--html] [--lcov] [--coverage-report <dir>]` に更新
- [x] `"--html"` / `"--lcov"` 引数パースを追加
- [x] `cmd_test` 呼び出しに `coverage_html` / `coverage_lcov` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "21.2.0"` → `"21.3.0"` に変更
- [x] **事前確認**: `grep -n "mod v212000_tests\|version_is_21_2_0" fav/src/driver.rs` で行番号を確認
- [x] `v212000_tests::version_is_21_2_0` に `#[ignore]` を追加
- [x] `cargo test v212000` — `version_is_21_2_0` が ignore されること
- [x] `cargo build` でコンパイルエラー 0

---

### T5: `CHANGELOG.md` + `site/content/docs/tools/coverage.mdx`

- [x] `CHANGELOG.md` の先頭に v21.3.0 エントリを追加（plan.md T5 の内容に従う）
  - [x] `### Added` — `--html` / `--lcov` / coverage モジュール / MDX
- [x] **事前確認**: `ls site/content/docs/tools/` で `coverage.mdx` が既存かどうかを確認
- [x] `site/content/docs/tools/coverage.mdx` を新規作成
  - [x] `--coverage --html` / `--coverage --lcov` の使い方
  - [x] コンソールサマリー出力例
  - [x] GitHub Actions 連携例（lcov → coveralls）

---

### T6: `fav/src/driver.rs` — `v213000_tests` 追加

- [x] `v212000_tests::version_is_21_2_0` に `#[ignore]` が付いていること（T4 で実施済み）
- [x] `v213000_tests` モジュールを追加（plan.md T6 の内容に従う）
  - [x] `version_is_21_3_0` — Cargo.toml に `"21.3.0"` が含まれる
  - [x] `coverage_html_contains_structure` — HTML に `<!DOCTYPE html>`・`<table>`・カバレッジ率
  - [x] `coverage_html_highlights_covered_lines` — `covered`/`uncovered` クラスが含まれる
  - [x] `coverage_lcov_format` — LCOV に `SF:`/`DA:`/`end_of_record` が含まれる
  - [x] `coverage_summary_line_format` — サマリーに `Coverage:` と `✓` が含まれる
- [x] 各テストに `#[cfg(not(target_arch = "wasm32"))]` ガードを付与（モジュールレベル）
- [x] `cargo test v213000` — 5/5 PASS を確認
- [x] `cargo test` — リグレッションなし（exit 0）を確認

---

## テスト（v213000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_3_0` | Cargo.toml に `"21.3.0"` が含まれる |
| `coverage_html_contains_structure` | HTML に `<!DOCTYPE html>`・`<table>`・カバレッジ率が含まれる |
| `coverage_html_highlights_covered_lines` | `covered`/`uncovered` クラスが行ハイライトに使われる |
| `coverage_lcov_format` | LCOV に `SF:`/`DA:`/`end_of_record` が含まれる |
| `coverage_summary_line_format` | コンソールサマリーに `Coverage:` と `✓` が含まれる |

---

## 完了条件チェックリスト

- [x] `fav test --coverage --html src/` で `coverage/index.html` が生成される
- [x] `fav test --coverage --lcov src/` で `coverage/lcov.info` が生成される
- [x] コンソールに `Coverage: XX.X% (N/M lines)` のサマリー行が表示される
- [x] HTML にファイル名・カバレッジ率・行ハイライト（covered/uncovered クラス）が含まれる
- [x] LCOV に `SF:` / `FN:` / `DA:` / `end_of_record` が含まれる
- [x] 既存の `--coverage` / `--coverage-report` の動作がリグレッションしない（`cargo test coverage` PASS）
- [x] `cargo test v213000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（exit 0）
- [x] `CHANGELOG.md` に v21.3.0 エントリが追加されている
- [x] `fav/Cargo.toml` version が `21.3.0`
- [x] `site/content/docs/tools/coverage.mdx` が存在する

---

## 優先度

```
T1（coverage/mod.rs — HTML / LCOV / summary ロジック）  ← 最初
T2（driver.rs — cmd_test 更新）                         ← T1 完了後
T3（main.rs — CLI 更新）                                ← T2 完了後
T4（Cargo.toml バージョン）                             ← T1 と並列可
T5（CHANGELOG + MDX）                                   ← T3 完了後
T6（driver.rs テスト）                                  ← T1 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `cmd_test` シグネチャ変更でコンパイルエラー | `cmd_test` 呼び出し箇所（driver.rs + main.rs）を同時に更新 |
| `is_executable_line` が `pub` でない | `driver.rs` 内で `is_executable_line` を `pub(crate)` に変更するか、coverage モジュールに同等ロジックをコピー |
| HTML が大きすぎてテストが遅い | テスト用ソースは1〜2行の最小サンプルを使用 |
| LCOV の行番号が実行時のカバレッジ番号（u32）と異なる | `line_hits: Vec<(u32, bool)>` に実際の行番号を格納。`is_executable_line` フィルタを通した行のみ登録 |
| WASM ビルドで `mod coverage` が引き込まれる | `lib.rs` / `main.rs` の `mod coverage` に `#[cfg(not(target_arch = "wasm32"))]` を付与 |
