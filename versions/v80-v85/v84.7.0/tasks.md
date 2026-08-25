# v84.7.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,921 tests, 0 failures を確認する（前提: v84.6.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84600_tests` が存在することを確認する（v84.6.0 完了済みの証拠）

## T1: `CONTRIBUTING.md` に v4 対応セクションを追加

- [x] 既存の `CONTRIBUTING.md` を読み、末尾に「Favnir 4.0 機能の追加手順」セクションを追加する
  - `QualityRule` 追加手順（test_framework.rs / driver.rs / docs/v4/data-quality.mdx の 3 ステップ）を含める
  - `IoContract` 追加手順（test_framework.rs / pipeline.fav / docs/v4/pipeline-contracts.mdx の 3 ステップ）を含める

## T2: `.github/ISSUE_TEMPLATE/quality-feedback.md` を新規作成

- [x] `.github/ISSUE_TEMPLATE/quality-feedback.md` を新規作成する
  - front matter（name / about / title / labels / assignees）を含める
  - フィードバック種別チェックボックス 4 項目を含める（QualityRule の誤動作・QualityGate の閾値問題・AnomalyDetector の検知精度・その他）
  - 再現手順・期待する動作・実際の動作・環境セクションを含める

## T3: `SECURITY.md` を v4 対応に更新

- [x] `SECURITY.md` のサポートバージョンテーブルを確認し、v84.x（Favnir 4.0）を追加する

## T4: `CODE_OF_CONDUCT.md` を確認

- [x] `CODE_OF_CONDUCT.md` を読み、内容が最新であることを確認する（変更不要なら確認のみ）

## T5: `fav/src/driver.rs` に `v84700_tests` を追加

- [x] `mod v84600_tests { ... }` の直後に `#[cfg(test)] mod v84700_tests { ... }` を追加する
- [x] `oss_contributing_v4_exists` テストを実装する
  - `include_str!("../../CONTRIBUTING.md")` で内容を読み込む（パス起点: `fav/src/`）
  - `"QualityRule"` が含まれること（メッセージ付き）
- [x] `oss_issue_template_quality_exists` テストを実装する
  - `Path::new("../.github/ISSUE_TEMPLATE/quality-feedback.md").exists()` を確認（パス起点: `fav/`）
  - メッセージ付き

## T6: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,923 tests, 0 failures（+2）であることを確認する

## T7: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.7.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
