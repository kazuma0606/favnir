# v59.0.0 Spec — Governance & Deployment 2.0 宣言 ★クリーンアップ

Date: 2026-07-29
Status: 設計中

---

## 概要

v58.1〜v58.9 で実装した Governance & Deployment 機能全体を宣言する**マイルストーンバージョン**。
`MILESTONE.md`・`README.md` に宣言文を追加し、4 件の Rust テストで自動検証する。
実装完了後に `cargo clean`（★クリーンアップ）を実施して v59.1 スプリントに備える。

---

## 変更するファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | バージョン `59.0.0` |
| `MILESTONE.md` | `"Governance & Deployment 2.0"` 宣言エントリを先頭に追加 |
| `README.md` | `"Governance & Deployment 2.0"` の言及を追加（v59.0 達成欄） |
| `CHANGELOG.md` | v59.0.0 エントリ追加（`"v59.0.0"` を含む） |
| `fav/src/driver.rs` | v59000_tests 追加 + ローリングチェック更新（6 件） |
| `versions/current.md` | 最新安定版を v59.0.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v59.0.0 実績欄に完了記録 |

> main.rs の変更はなし（宣言専用バージョン）。

---

## テスト

`v59000_tests` モジュールを `v58900_tests` の直前に挿入（4 件）:

| テスト名 | assert 内容 | include_str! パス |
|---|---|---|
| `cargo_toml_version_is_59_0_0` | `version = "59.0.0"` を含む（ローリングチェック） | `"../Cargo.toml"` |
| `changelog_has_v59_0_0` | `"v59.0.0"` を含む | `"../../CHANGELOG.md"` |
| `milestone_has_governance_deployment2` | `"Governance & Deployment 2.0"` を含む | `"../../MILESTONE.md"` |
| `readme_mentions_governance_deployment2` | `"Governance & Deployment 2.0"` を含む | `"../../README.md"` |

- `cargo_toml_version_is_59_0_0` は v58000_tests の `cargo_toml_version_is_58_0_0` と同じ**ローリングチェック**パターン（関数名は凍結、assertion は次バージョンから更新）
- `use super::*` 不要（`include_str!` のみ）

**実際のベース**: 3304（v58.9.0 実績値）
**完了条件**: 3304 + 4 = **3308 tests passed, 0 failed**

---

## ローリングチェック更新

v58000_tests の既存ローリングアサーション 5 件（`version = "58.9.0"`）を `"59.0.0"` に更新。
failure メッセージ 5 件も同様に `"59.0.0"` に更新。

**注意**: v59000_tests の `cargo_toml_version_is_59_0_0` も初回から `"59.0.0"` を assert するため、
更新後のローリングチェック総数は **6 件**（既存 5 + 新規 1）となる。

既存 5 件の内訳:
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0`）
- `v56900_tests::cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0`）
- `v56300_tests::cargo_toml_version_is_56_3_0`

---

## MILESTONE.md 要件

宣言エントリを **先頭（v58.0.0 エントリの前）** に挿入:

- `"Governance & Deployment 2.0"` という文字列（テストで検証）
- v59.0 の宣言文テキスト（ロードマップ記載の引用文をそのまま使用）
- v58.1〜v58.9 の達成内容一覧

---

## README.md 要件

- `"Governance & Deployment 2.0"` という文字列を追加（テストで検証）
- v59.0 達成欄の更新（例: マイルストーン進捗テーブルに v59.0.0 行を追記）

---

## ★クリーンアップ

テスト全通過後に `cargo clean` を実施。
次スプリント（v59.1〜v60.0）に向けてビルドキャッシュを初期化する。
