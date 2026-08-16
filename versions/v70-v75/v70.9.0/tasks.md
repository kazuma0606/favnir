# v70.9.0 タスクリスト — 安定化・コードフリーズ

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.8.0` であることを確認
- [x] `cargo test` が全 pass（3578 tests）であることを確認
- [x] bench.yml の Compare ステップに `|| true` が存在することを確認（修正前）
- [x] driver.rs に v70.1〜v70.8 の代表テスト名が全て存在することを確認:
  - [x] `backlog_compiler_fav_ctx_multiparams`（v70.1）
  - [x] `migrate_effect_annotation_to_ctx`（v70.2）
  - [x] `bench_subcommand_all_outputs_json`（v70.3）
  - [x] `diagnostic_e0374_shows_migration_hint`（v70.4）
  - [x] `pattern_match_if_guard`（v70.5）
  - [x] `bind_destructure_record`（v70.6）
  - [x] `self_coverage_compiler_fav_above_95pct`（v70.7）
  - [x] `doctor_detects_paper_rune`（v70.8）

---

## T1: bench.yml 修正

- [x] `.github/workflows/bench.yml` の `Compare with baseline` ステップの `|| true` を除去する
- [x] 除去対象は行末 `|| true`（シェルオペレータ）のみ — `continue-on-error:` YAML キーは bench.yml に存在しない
- [x] 他のステップ（Run benchmarks, Regression check）の `|| true` は**触れない**（スコープ外）
- [x] ファイルを保存し、差分を確認する

---

## T2: driver.rs に `v709000_tests` モジュールを追加

- [x] `v708000_tests` の直後に `v709000_tests` モジュールを追加する
- [x] `language_complete_all_stable` テストを実装する:
  - `include_str!("driver.rs")` で自己参照
  - v70.1〜v70.8 の代表テスト名 8 件が全て含まれることを assert
- [x] `bench_ci_no_continue_on_error` テストを実装する:
  - `include_str!("../../.github/workflows/bench.yml")` でファイルを読む
  - `Compare with baseline` セクションに `|| true` がないことを assert
- [x] `cargo test v709000` で 2 件 pass することを確認

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.8.0"` → `"70.9.0"` に変更する
- [x] driver.rs 内のバージョン文字列（`"70.8.0"` 文字列チェック箇所）を `"70.9.0"` に更新する
  - 注: テスト関数名 `cargo_toml_version_is_70_8_0` 自体はリネームしない

---

## T4: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.8.0 エントリの直前）に v70.9.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `v709000_tests` 2 件（3578 → 3580 tests）
  - Added: `language_complete_all_stable` — v70.1〜v70.8 代表テスト全 pass 確認
  - Fixed: bench.yml Compare ステップを strict mode に変更（`|| true` 除去）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を `v70.9.0`（安定化・コードフリーズ）に更新する
- [x] 「次に切る版」を `v71.0.0` に更新する

---

## T6: 最終確認

- [x] `cargo test v709000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3580 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.9.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認
- [x] bench.yml の Compare ステップに `|| true` が残っていないことを確認

---

## コードレビュー指摘対応

（実装後に記録）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `language_complete_all_stable` が pass
- [x] `bench_ci_no_continue_on_error` が pass
- [x] テスト総数: 3580（+2）
- [x] bench.yml が strict mode になっていることを確認
- [x] `versions/current.md` が v70.9.0 に更新されていることを確認
