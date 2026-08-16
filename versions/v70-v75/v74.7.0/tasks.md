# v74.7.0 タスクリスト — コミュニティ Rune 品質基準

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.6.0` であることを確認
- [x] `cargo test` が 3682 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v746000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v747000_tests` が未存在であることを確認

---

## T1: 構造体 + 関数を `driver.rs` に追加

- [x] `// --- v74.7.0: コミュニティ Rune 品質基準 ---` セクションコメントを追加した
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct RuneValidationItem` を追加した（name / passed / message）
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct RuneValidationReport` を追加した（rune_name / items / score）
- [x] `pub fn validate_rune_score(report: &RuneValidationReport) -> bool` を実装した
  - `report.score >= 80` なら `true`
- [x] `pub fn format_rune_validation_report(report: &RuneValidationReport) -> String` を実装した
  - passed なら "✓"、false なら "⚠" プレフィックス
  - 末尾に "Score: {score}/100 (Publish requires >= 80)" を追記
- [x] `cargo build` でエラーがないことを確認

---

## T2: `v747000_tests` モジュールを追加

- [x] `v746000_tests` の直後に `v747000_tests` モジュールを追加した
- [x] `use super::{RuneValidationItem, RuneValidationReport, validate_rune_score, format_rune_validation_report}` を追加した
- [x] `rune_validate_scoring` テストを実装した
  - 5 項目の `RuneValidationReport`（score=85）を構築し rune_name / items 数 / score を assert
  - `format_rune_validation_report` の出力に "✓" / "⚠" / "Score:" / "85" / "80" が含まれることを assert
- [x] `rune_validate_min_score_enforced` テストを実装した
  - score=100 / 80（ボーダー）で `validate_rune_score` が `true` を返すことを assert
  - score=79 / 0 で `validate_rune_score` が `false` を返すことを assert

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.6.0"` → `version = "74.7.0"` に変更した
- [x] `driver.rs` 内の `version = "74.6.0"` 参照を `version = "74.7.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.6.0` を `version should be 74.7.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.6.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.7.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v747000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3684 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.7.0]` エントリを先頭に追加した
  - Added: `RuneValidationItem` / `RuneValidationReport` / `validate_rune_score` / `format_rune_validation_report`
  - Tests: 2 件、合計テスト数 3684（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.7.0)` に更新した
- [x] 「進行中バージョン」を `v74.7.0` に更新した
- [x] 「次に切る版」を `v74.8.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v747000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3684 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.7.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.7.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.7.0` であることを確認

---

## スコープ外（明示的除外）

- `cmd_rune_validate(path)` 関数の実装（後続バージョンで対応）
- 実際のファイルシステム走査（rune.toml・実装ファイル・テスト・ドキュメントの読み込み）
- `fav publish rune` 時の自動 validate フック（後続バージョンで対応）
- `fav rune validate` の main.rs CLI エントリポイント（後続バージョンで対応）
- スコアリングアルゴリズムの自動計算（score は呼び出し元が設定）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）
