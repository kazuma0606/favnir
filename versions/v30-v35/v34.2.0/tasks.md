# v34.2.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.1.0` であること
- [x] `benchmarks/v34.1.0.json` の `tests_passed` が 2541 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2541 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v342000_tests` が存在しないこと
- [x] v34.1.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_1_0` が v341000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_1_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v341000` が 5/5 PASS であること（前バージョン 5 件 PASS を確認）
- [x] `site/content/errors/index.mdx` が存在しないこと（新規作成対象）
- [x] `site/content/cookbook/` のファイル数が 32 であること（18 本追加予定）
  ```bash
  ls site/content/cookbook/ | wc -l
  # 32 であることを確認
  ```
- [x] 追加予定 18 本が既存 cookbook に存在しないこと
  ```bash
  ls site/content/cookbook/ | grep -E "postgres-etl|snowflake-load|duckdb-query|parquet-transform|avro-schema|iceberg-compaction|mongodb-etl|redis-cache-aside|elasticsearch-index|http-api-ingest|csv-validation|schema-evolution|data-quality-check|incremental-load|cron-trigger|secret-manager|jwt-auth|grpc-client"
  # 0 件であることを確認（重複なし）
  ```

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.1.0` → `34.2.0` に更新
- [x] **T2** `site/content/errors/index.mdx` — エラーコードリファレンスページを新規作成
        （フロントマター + `fav explain` 使い方 + カテゴリ別エラーコードテーブル）
- [x] **T3** `site/content/docs/bench/index.mdx` — Python pandas / Spark 比較セクションを追記
- [x] **T4〜T21** `site/content/cookbook/` — 18 本追加（以下 18 件）
  - [x] **T4**  `postgres-etl.mdx`
  - [x] **T5**  `snowflake-load.mdx`
  - [x] **T6**  `duckdb-query.mdx`
  - [x] **T7**  `parquet-transform.mdx`
  - [x] **T8**  `avro-schema.mdx`
  - [x] **T9**  `iceberg-compaction.mdx`
  - [x] **T10** `mongodb-etl.mdx`
  - [x] **T11** `redis-cache-aside.mdx`
  - [x] **T12** `elasticsearch-index.mdx`
  - [x] **T13** `http-api-ingest.mdx`
  - [x] **T14** `csv-validation.mdx`
  - [x] **T15** `schema-evolution.mdx`
  - [x] **T16** `data-quality-check.mdx`
  - [x] **T17** `incremental-load.mdx`
  - [x] **T18** `cron-trigger.mdx`
  - [x] **T19** `secret-manager.mdx`
  - [x] **T20** `jwt-auth.mdx`
  - [x] **T21** `grpc-client.mdx`
- [x] **T22** `fav/src/driver.rs` — `cargo_toml_version_is_34_1_0` をスタブ化（コメント付き）
- [x] **T23** `fav/src/driver.rs` — `v342000_tests`（5 件）を追加
        挿入位置: `v341000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T24** `CHANGELOG.md` — `[v34.2.0]` セクションを先頭に追記
- [x] **T25** `benchmarks/v34.2.0.json` — 新規作成（暫定 `tests_passed`: 2546）
- [x] **T26** `versions/current.md` — 「最新安定版」欄を v34.2.0 に更新

---

## テスト確認

- [x] **T27** `cargo test --bin fav v342000 2>&1 | tail -8` — 5/5 PASS
- [x] **T28** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2546 passed = 2541 + 5、0 failures）

---

## 完了処��

- [x] **T29** `benchmarks/v34.2.0.json` の `tests_passed` を実測値で更新（2546 確定）
- [x] **T30** `site/content/cookbook/` のファイル数が 50 であることを確認
  ```bash
  ls site/content/cookbook/ | wc -l
  # 50 であることを確認 ✓
  ```
- [x] **T31** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.2.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.2.0"`
- [x] `cargo_toml_version_is_34_1_0` が空スタブになっていること（他テストは残存）
- [x] `cargo test --bin fav v342000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2546 件 = 2541 + 5、0 failures）
- [x] `site/content/errors/index.mdx` が存在し `E0101` を含むこと
- [x] `site/content/docs/bench/index.mdx` が `pandas` を含むこと
- [x] `site/content/cookbook/postgres-etl.mdx` が存在すること
- [x] `site/content/cookbook/snowflake-load.mdx` が存在すること
- [x] `site/content/cookbook/` のファイル数が 50 であること
- [x] `CHANGELOG.md` に `[v34.2.0]` セクション
- [x] `benchmarks/v34.2.0.json` 存在かつ `tests_passed` が実測値（2546）
- [x] `versions/current.md` が v34.2.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v342000_tests` に `use super::*` が**ない**こと
- [x] `v342000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_34_1_0` が空スタブになっていること（コメント付き）
- [x] `errors_index_mdx_exists` で `src.contains("E0101")` を assert していること
- [x] `bench_page_has_python_comparison` で `src.contains("pandas") || src.contains("Python")` を assert していること
- [x] 挿入位置が `v341000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.2.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.2.0 に更新されていること
- [x] 新規 cookbook 18 本がすべて存在すること
- [x] 各 cookbook MDX にフロントマター（title / description）があること
- [x] `site/content/errors/index.mdx` に `fav explain` の使い方説明があること
