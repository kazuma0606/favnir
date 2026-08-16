# v75.9.0 タスクリスト — 安定化・コードフリーズ

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.8.0` であることを確認
- [x] `cargo test` が全 pass（3708 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.9.0 エントリを追加する
- [x] Tests セクション（2 件）を含める

---

## T2: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `v759000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `temporal_full_sprint_all_stable` テストを実装する
  - v75.1〜v75.8 の全 Temporal 型を網羅的に呼び出す
  - `check_freshness`、`is_in_range(ts, &TemporalRange { from_ts, to_ts })`、`format_as_of_query`、`apply_scd2_update`、`validate_temporal_join_config`、`apply_retention_check`、`check_stream_lag`、`validate_temporal_contract`、`cmd_time_travel`、`parse_time_travel_timestamp` を確認
  - `format_temporal_join_sql` が 3 引数（left_table, right_table, config）で呼ばれていることを確認する
- [x] `temporal_e2e_pipeline_valid` テストを実装する
  - `data_ts = 1735689600`、`now = data_ts + 60`
  - 鮮度チェック → タイムトラベルSQL → 保持チェック → ストリーム遅延 → コントラクト検証
- [x] `cargo test v759000` で 2 件が pass することを確認する

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.8.0"` → `"75.9.0"` に変更する
- [x] `driver.rs` 内の `75.8.0` バージョン文字列アサーションを `75.9.0` に一括更新（replace_all）

---

## T4: versions/current.md 更新

- [x] 「進行中バージョン」を v75.9.0 に更新する
- [x] 「次に切る版」を v76.0.0 に更新する

---

## T5: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3710 tests）
- [x] `cargo test v759000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.9.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.9.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T5）が完了している
- [x] `temporal_full_sprint_all_stable` が pass
- [x] `temporal_e2e_pipeline_valid` が pass
- [x] テスト総数: 3710（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（v76.0.0 宣言バージョンで行う）
- [x] MILESTONE.md 更新: 対象外（v76.0.0 で実施）
