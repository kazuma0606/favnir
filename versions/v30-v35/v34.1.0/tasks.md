# v34.1.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.0.0` であること
- [x] `benchmarks/v34.0.0.json` の `tests_passed` が 2536 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2536 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v341000_tests` が存在しないこと
- [x] v34.0.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_0_0` が v340000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_0_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v340000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `examples/real-world-etl/` が存在しないこと（新規作成対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.0.0` → `34.1.0` に更新
- [x] **T2** `examples/real-world-etl/fav.toml` — プロジェクト定義を新規作成
- [x] **T3** `examples/real-world-etl/src/types.fav` — Order / OrderStatus / ValidationError / LoadResult 型定義
- [x] **T4** `examples/real-world-etl/src/validators.fav` — validate_order / validate_all 実装
- [x] **T5** `examples/real-world-etl/src/stages.fav` — load_csv / write_postgres / sync_bigquery 実装
- [x] **T6** `examples/real-world-etl/src/notifications.fav` — notify_success / notify_failure 実装
- [x] **T7** `examples/real-world-etl/src/main.fav` — RealWorldEtl pipeline + main 実装
- [x] **T8** `examples/real-world-etl/data/orders_sample.csv` — ヘッダー行 + 5 行サンプル作成（`order_id` カラム含む）
- [x] **T9** `examples/real-world-etl/README.md` — 30 分で動かす手順書を作成（`30` を含むこと）
- [x] **T10** `fav/src/driver.rs` — `cargo_toml_version_is_34_0_0` をスタブ化（コメント付き）
- [x] **T11** `fav/src/driver.rs` — `v341000_tests`（5 件）を追加
        挿入位置: `v340000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T12** `CHANGELOG.md` — `[v34.1.0]` セクションを先頭に追記
- [x] **T13** `benchmarks/v34.1.0.json` — 新規作成
- [x] **T14** `versions/current.md` — 「最新安定版」欄を v34.1.0 に更新

---

## テスト確認

- [x] **T15** `fav check examples/real-world-etl/src/main.fav` — ��ラーなし（型チェック確認）
- [x] **T16** `cargo test --bin fav v341000 2>&1 | tail -8` — 5/5 PASS
- [x] **T17** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2541 passed = 2536 + 5、0 failures）

---

## 完了処理

- [x] **T18** `benchmarks/v34.1.0.json` の `tests_passed` を実測値で更新（2541 確定）
- [x] **T19** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.1.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.1.0"`
- [x] `cargo_toml_version_is_34_0_0` が空スタブになっていること（他テストは残存）
- [x] `fav check examples/real-world-etl/src/main.fav` — エラーなし
- [x] `cargo test --bin fav v341000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2541 件 = 2536 + 5、0 failures）
- [x] `examples/real-world-etl/fav.toml` が存在し `real-world-etl` を含むこと
- [x] `examples/real-world-etl/src/main.fav` が存在し `pipeline` または `main` を含むこと
- [x] `examples/real-world-etl/data/orders_sample.csv` が存在し `order_id` ヘッダーを含むこと
- [x] `examples/real-world-etl/README.md` が存在し `30` を含むこと
- [x] `CHANGELOG.md` に `[v34.1.0]` セクション
- [x] `benchmarks/v34.1.0.json` 存在かつ `tests_passed` が実測値（2541）
- [x] `versions/current.md` が v34.1.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v341000_tests` に `use super::*` が**ない**こと
- [x] `v341000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_34_0_0` が空スタブになっている��と（コメン���付き）
- [x] `real_world_etl_fav_toml_exists` で `src.contains("real-world-etl")` を assert していること
- [x] `real_world_etl_sample_data_exists` で `src.contains("order_id")` を assert していること
- [x] 挿入位置が `v340000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.1.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.1.0 に更新されていること
- [x] `examples/real-world-etl/src/` に 5 ファイルすべてが存在すること
  （types.fav / validators.fav / stages.fav / notifications.fav / main.fav）
- [x] `examples/real-world-etl/` 配下が合計 8 ファイル（fav.toml + src/ 5 ファイル + data/orders_sample.csv + README.md）で構成されていること
- [x] `validators.fav` に `import runes/csv` が**ない**こと（csv import は stages.fav のみ）
