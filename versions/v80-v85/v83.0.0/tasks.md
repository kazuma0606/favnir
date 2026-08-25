# v83.0.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,883 tests pass、0 failures であることを確認する（前提: v82.9.0 完了済み）

## T1: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v83.0.0 エントリを追加する
  - Pipeline Contracts 1.0 宣言文・クリーンアップ内容を記載する

## T2: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version = "82.0.0"` を `version = "83.0.0"` に変更する
- [x] 旧バージョン確認テスト（`cargo_toml.contains("version = ...")` 等）を `83.0.0` に一括更新する

## T3: `MILESTONE.md` 更新

- [x] `MILESTONE.md` に Pipeline Contracts 1.0 達成宣言を追加する
  - "Pipeline Contracts" という文字列を含む宣言文を追加する
  - 宣言日付・達成テスト数（3,887）を記載する

## T4: `README.md` 更新

- [x] `README.md` に `ContractRegistry` への言及を追加する
  - Pipeline Contracts 1.0 の概要と `ContractRegistry` の説明を追加する

## T5: `v83000_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾付近に `#[cfg(test)] mod v83000_tests` を追加する（`use super::*` 不要）
  - `cargo_toml_version_is_83_0_0`
  - `changelog_has_v83_0_0`
  - `milestone_has_pipeline_contracts`
  - `readme_mentions_contract_registry`

## T6: テスト通過確認

- [x] `cargo test` が 3,887 tests pass（+4）、0 failures であることを確認する

## T7: `cargo clean` 実施

- [x] `cargo clean` を実行して build artifacts をクリアする（10.8 GiB 削除）

## T8: `versions/current.md` 更新

- [x] 最新安定版を v83.0.0（3,887 tests）に更新する
- [x] 進行中バージョンを v83.1.0〜v84.0.0 に更新する

## T9: `roadmap-v80.1-v85.0.md` 更新

- [x] Sprint 3 テーブルのテスト数を drift 補正後の実績値に更新する（3,865〜3,887）
- [x] 次スプリントロードマップ `roadmap-v83.1-v84.0.md` の確認（未作成 — v84.0.0 スプリント計画時に作成予定）

## T10: 最終確認（CI チェック）

- [x] `cargo build` 実施（cargo clean 後の再ビルド）
- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 実装メモ

- 旧 `cargo_toml_version_is_*` テストは全て `"82.0.0"` をチェックしていたため、
  `82.0.0` → `83.0.0` の一括置換（変数名: `cargo_toml` / `cargo` / `content` / `src` / `toml` 等）が必要だった。

## code-reviewer 対応

- [x] [LOW] 旧テストのアサーションメッセージに古いバージョン文字列（`"75.1.0"` / `"70.0.0"` / `"67.0.0"` / `"73.7.0"` / `"81.0.0"`）が残存していたものを `"83.0.0"` に一括更新
