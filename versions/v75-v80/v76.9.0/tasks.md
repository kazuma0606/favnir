# v76.9.0 タスクリスト — 安定化・コードフリーズ

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.8.0` であることを確認
- [x] `cargo test` が全 pass（3730 tests）であることを確認（v76.9.0 テスト追加前の状態、v76.8.0 完了時点の基準数）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.9.0 エントリを追加する
- [x] Tests セクション（2 件）を含める

---

## T2: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `v769000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `provenance_full_sprint_all_stable` テストを実装する
  - DataSource → ProvenanceTag → TracedData（map_traced）→ chain_provenance → provenance_to_openlineage → LineageGraph（format_lineage_dot）の連鎖を検証
- [x] `provenance_e2e_pipeline_valid` テストを実装する
  - generate_erasure_plan（pii=false → None）
  - validate_data_product（require_source_declared=true + pii_must_be_masked=true → Ok）
  - validate_provenance_contract（Snowflake + MustBeMasked → Ok）
  - provenance_to_openlineage + format_openlineage_json の出力検証
- [x] `cargo test v769000` で 2 件が pass することを確認する

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.8.0"` → `"76.9.0"` に変更する
- [x] `driver.rs` 内の `76.8.0` バージョン文字列アサーションを `76.9.0` に一括更新（`replace_all: true` で全件置換）

---

## T4: versions/current.md 更新

- [x] 「進行中バージョン」を v76.9.0 に更新する
- [x] 「次に切る版」を v77.0.0 に更新する

---

## T5: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3732 tests）
- [x] `cargo test v769000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.9.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.9.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v76.9.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T5）が完了している
- [x] `provenance_full_sprint_all_stable` が pass
- [x] `provenance_e2e_pipeline_valid` が pass
- [x] テスト総数: 3732（+2）
- [x] 新型・新関数の追加なし（コードフリーズ版）
- [x] site/ MDX 追加: 対象外（v77.0 で実施）
- [x] `changelog_has_v76_9_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
