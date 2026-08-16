# v73.2.0 タスクリスト — データ品質スコアリング

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.1.0` であることを確認
- [x] `cargo test` が 3648 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v731000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v732000_tests` が未存在であることを確認
- [x] `driver.rs` 内の `"73.1.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: 構造体追加（`QualityDimension` / `QualityReport`）

- [x] `QualityDimension { name: String, score: u32, detail: String }` を追加した
- [x] `QualityReport { overall_score: u32, dimensions: Vec<QualityDimension>, recommendations: Vec<String> }` を追加した
- [x] 全フィールドが `pub` であることを確認
- [x] `cargo build` でエラーがないことを確認

---

## T2: `compute_quality_report` 追加

- [x] `pub fn compute_quality_report(rows: &[Vec<Option<String>>]) -> QualityReport` を実装した
  - Completeness: null セル割合からスコア算出
  - Validity: 全 Some の行の割合からスコア算出
  - Consistency / Freshness / Referential: スタブスコア（78 / 92 / 95）
  - overall_score: 5 次元平均
  - recommendations: Completeness < 95 → "null checks"、Validity < 90 → "field validators"
- [x] `cargo build` でエラーがないことを確認

---

## T3: `format_quality_report` 追加

- [x] `pub fn format_quality_report(report: &QualityReport) -> String` を実装した
  - ヘッダー `"Favnir Data Quality Report"` を含む
  - `"Overall Score: {}/100"` を含む
  - 各次元を表形式で出力
  - Recommendations セクションを含む
- [x] `cargo build` でエラーがないことを確認

---

## T3.5: `cmd_quality_report` スタブ追加

- [x] `pub fn cmd_quality_report(path: &str) -> String` を実装した
  - 将来の .fav 解析用スタブ（path は未使用、空 rows でレポートを返す）
  - 内部で `compute_quality_report` + `format_quality_report` を呼ぶ
- [x] `cargo build` でエラーがないことを確認

---

## T4: `v732000_tests` モジュール追加

- [x] `v731000_tests` モジュールの直後に `v732000_tests` モジュールを追加した
- [x] `use super::{compute_quality_report, format_quality_report}` を追加した
- [x] `quality_report_completeness_score` テストを実装した
  - 1000 行中 58 件 null → Completeness >= 90 を assert
  - overall_score > 0 を assert
- [x] `quality_report_recommendations` テストを実装した
  - null 多めデータ → recommendations が空でないことを assert
  - recommendations に "null" が含まれることを assert
  - `format_quality_report` の出力に "Favnir Data Quality Report" が含まれることを assert
- [x] `cargo test v732000` で 2 件 pass することを確認

---

## T5: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.1.0"` → `version = "73.2.0"` に変更した
- [x] `driver.rs` 内の `version = \"73.1.0\"` を `version = \"73.2.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 73.1.0"` を `"73.2.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 73.1.0"` を `"73.2.0"` に replace_all した
- [x] 残存 `73.1.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `grep "73.1.0" driver.rs` で意図的保持分以外がゼロ件であることを確認
- [x] `cargo build` 後に `fav/Cargo.lock` が `version = "73.2.0"` を含むことを確認

---

## T6: 部分テスト確認

- [x] `cargo test v732000` で 2 件 pass することを確認

---

## T7: 全体テスト確認

- [x] `cargo test` 全体で 3650 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [x] `## [v73.2.0]` エントリを先頭に追加した
  - Added: `QualityDimension` / `QualityReport` / `compute_quality_report` / `format_quality_report`
  - Tests: 2 件、合計テスト数 3650（+2）

---

## T9: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.2.0)` に更新した
- [x] 「進行中バージョン」を `v73.2.0` に更新した
- [x] 「次に切る版」を `v73.3.0` に更新した

---

## T10: 最終確認（T8・T9 完了後）

- [x] `cargo test v732000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3650 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.2.0` であることを確認
- [x] `QualityDimension` / `QualityReport` が pub で存在することを確認
- [x] `compute_quality_report` / `format_quality_report` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.2.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.2.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [BUG] | `consistency_score` のみ `total_rows == 0` 分岐があり他スタブと挙動非対称 | `consistency_score = 78u32` 固定に統一（スタブは全部定数） |
| [STYLE] | `cmd_quality_report(path: &str)` の `let _ = path;` が慣用形でない | `_path: &str` に変更 |

---

## スコープ外（明示的除外）

- `fav quality report` CLI コマンドの .fav ファイル入力解析（将来バージョン）
- CSV / Parquet 実ファイル読み込み（v74.x 以降）
- Consistency / Freshness / Referential の実スコアリング（スタブ、将来実装）
- WASM / サイト MDX 更新（v74.x 以降）
