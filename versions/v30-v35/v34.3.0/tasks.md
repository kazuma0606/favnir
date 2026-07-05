# v34.3.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.2.0` であること
- [x] `benchmarks/v34.2.0.json` の `tests_passed` が 2546 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2546 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v343000_tests` が存在しないこと
- [x] v34.2.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_2_0` が v342000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_2_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v342000` が 5/5 PASS であること（前バージョン 5 件 PASS を確認）
- [x] `benchmarks/real-world/` が存在しないこと（新規作成対象）
  ```bash
  ls benchmarks/real-world/ 2>/dev/null || echo "does not exist"
  # does not exist であることを確認
  ```
- [x] `site/content/docs/bench/index.mdx` に dbt 比較セクション見出しが含まれていないこと（追記対象）
  ```bash
  grep "## dbt との比較" site/content/docs/bench/index.mdx | wc -l
  # 0 であることを確認（脚注に "dbt" が 1 件あるのは正常）
  ```

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.2.0` → `34.3.0` に更新
- [x] **T2** `benchmarks/real-world/favnir.json` — Favnir 実測データを新規作成
- [x] **T3** `benchmarks/real-world/python_pandas.json` — Python pandas 実測データを新規作成
- [x] **T4** `benchmarks/real-world/apache_spark.json` — Apache Spark 実測データを新規作成
- [x] **T5** `site/content/docs/bench/index.mdx` — 履歴テーブルに v34.1〜v34.3 行を先頭追加
- [x] **T6** `site/content/docs/bench/index.mdx` — dbt 比較セクションを計測環境脚注の後に追加
- [x] **T7** `fav/src/driver.rs` — `cargo_toml_version_is_34_2_0` をスタブ化
- [x] **T8** `fav/src/driver.rs` — `v343000_tests`（5 件）を追加
        挿入位置: `v342000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T9** `CHANGELOG.md` — `[v34.3.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v34.3.0.json` — 新規作成（`tests_passed`: 2551 実測値）
- [x] **T11** `versions/current.md` — 「最新安定版」欄を v34.3.0 に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v343000 2>&1 | tail -8` — 5/5 PASS
- [x] **T13** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2551 passed、0 failures）

---

## 完了処理

- [x] **T14** `benchmarks/v34.3.0.json` の `tests_passed` を実測値で更新（2551 確定）
- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.3.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.3.0"`
- [x] `cargo_toml_version_is_34_2_0` が空スタブになっていること（他テストは残存）
- [x] `cargo test --bin fav v343000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2551 passed、0 failures）
- [x] `benchmarks/real-world/favnir.json` が存在し `"tool": "favnir"` を含むこと（`"favnir"` で検証）
- [x] `benchmarks/real-world/python_pandas.json` が存在し `"tool": "python_pandas"` を含むこと
- [x] `benchmarks/real-world/apache_spark.json` が存在し `"tool": "apache_spark"` を含むこと
- [x] `site/content/docs/bench/index.mdx` に `dbt` 言及があること
- [x] `site/content/docs/bench/index.mdx` の履歴テーブルに `v34.3.0` 行があること
- [x] `CHANGELOG.md` に `[v34.3.0]` セクション
- [x] `benchmarks/v34.3.0.json` 存在かつ `tests_passed` が実測値（2551）
- [x] `versions/current.md` が v34.3.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v343000_tests` に `use super::*` が**ない**こと
- [x] `v343000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_34_2_0` が空スタブになっていること（コメント付き）
- [x] `real_world_bench_favnir_exists` で `src.contains("favnir")` を assert していること
- [x] `real_world_bench_python_pandas_exists` で `src.contains("python_pandas")` を assert していること
- [x] `real_world_bench_apache_spark_exists` で `src.contains("apache_spark")` を assert していること
- [x] `bench_page_has_dbt_comparison` で `src.contains("dbt")` を assert していること
- [x] 挿入位置が `v342000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.3.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.3.0 に更新されていること
- [x] `benchmarks/real-world/` 3 ファイルの `"tool"` フィールドがそれぞれ `"favnir"` / `"python_pandas"` / `"apache_spark"` であること
- [x] `benchmarks/real-world/favnir.json` の計測値が bench/index.mdx の v34.2 比較数値と整合していること
- [x] bench/index.mdx の `dbt` 比較セクションが計測環境脚注の**後**に配置されていること
