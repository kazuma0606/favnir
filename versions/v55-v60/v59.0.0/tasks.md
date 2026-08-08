# v59.0.0 Tasks — Governance & Deployment 2.0 宣言 ★クリーンアップ

Date: 2026-07-29
Status: COMPLETE（2026-07-29）— 3308 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3304 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"58.9.0"` であることを確認
- [x] `MILESTONE.md` に `"Governance & Deployment 2.0"` がまだ含まれていないことを確認
- [x] `README.md` に `"Governance & Deployment 2.0"` がまだ含まれていないことを確認
- [x] `grep -c '58\.9\.0' fav/src/driver.rs` でローリング文字列件数を確認（5 件のはず）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "58.9.0"` → `"59.0.0"`

---

## T2: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭に v59.0.0 エントリを挿入
  - `"Governance & Deployment 2.0"` を含む（テストで検証）
  - ロードマップの宣言文を引用
  - v58.1〜v58.9 の達成内容一覧を記載

---

## T3: README.md 更新

- [x] `README.md` に `"Governance & Deployment 2.0"` を含む言及を追加
  - マイルストーン進捗テーブルへの v59.0.0 行の追記 など

---

## T4: CHANGELOG.md 更新（テストモジュール追加の前に実施）

- [x] `CHANGELOG.md` に v59.0.0 エントリを追加（`"v59.0.0"` を含める）
  - **注意**: T5 の `include_str!` テストが v59.0.0 を参照するため、T5 の前に追記必須

---

## T5: driver.rs テストモジュール追加

- [x] main.rs を変更していないことを確認（宣言専用バージョン）
- [x] `v59000_tests` モジュールを v58900_tests の直前に挿入
  - **注意**: T2〜T4（MILESTONE.md・README.md・CHANGELOG.md 更新）を先に行うこと
  - [x] `cargo_toml_version_is_59_0_0`: `include_str!("../Cargo.toml")` が `version = "59.0.0"` を含む（ローリングチェック）
  - [x] `changelog_has_v59_0_0`: `include_str!("../../CHANGELOG.md")` が `"v59.0.0"` を含む
  - [x] `milestone_has_governance_deployment2`: `include_str!("../../MILESTONE.md")` が `"Governance & Deployment 2.0"` を含む
  - [x] `readme_mentions_governance_deployment2`: `include_str!("../../README.md")` が `"Governance & Deployment 2.0"` を含む
  - [x] `use super::*` は不要（`include_str!` のみ使用）

---

## T6: driver.rs ローリングチェック更新

- [x] `version = \"58.9.0\"` → `\"59.0.0\"` に一括更新（5 件、`replace_all`）
- [x] failure メッセージ `"Cargo.toml version should be 58.9.0"` → `"59.0.0"` に更新（5 件）
  - `cargo_toml_version_is_58_0_0` 用（メッセージ末尾なし）
  - `cargo_toml_version_is_57_9_0` 用（メッセージ末尾なし）
  - `cargo_toml_version_is_57_0_0` 用（`rolling check from v57.0.0` 付き）
  - `cargo_toml_version_is_56_9_0` 用（`rolling check from v56.9.0` 付き）
  - `cargo_toml_version_is_56_3_0` 用（メッセージ末尾なし）

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `cargo_toml_version_is_59_0_0` pass を確認
- [x] `changelog_has_v59_0_0` pass を確認
- [x] `milestone_has_governance_deployment2` pass を確認
- [x] `readme_mentions_governance_deployment2` pass を確認
- [x] 総テスト数 **3308** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T8: ★クリーンアップ

- [x] テスト全通過を確認してから `cargo clean` を実行

---

## T9: 事後処理

- [x] `versions/current.md` を v59.0.0 / 3308 tests に更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v59.0.0 実績欄を更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の「直前完了」欄を実績値に更新（code-review でテスト数が増加した場合）
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.1.0 ベース数を実績値に合わせて修正（code-review でテスト数が増加した場合）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

- [BUG][実装時] v58900_tests::cargo_toml_version_is_58_9_0 の更新漏れ → 6件目の rolling check として更新（spec の「5件」カウントが不正確だった）
- [LOW] v58900_tests::cargo_toml_version_is_58_9_0 に rolling check コメントがなかった → 追記（他モジュールとの一貫性確保）

最終テスト数: 3308 tests passed, 0 failed（code-review 対応で変化なし）

---

Status: COMPLETE（2026-07-29）— 3308 tests passed, 0 failed
