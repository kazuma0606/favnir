# v76.8.0 タスクリスト — Provenance contracts

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.7.0` であることを確認
- [x] `cargo test` が全 pass（3728 tests）であることを確認（v76.8.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.8.0: Provenance contracts ---` コメントを追加する
- [x] `PiiPolicy` enum を追加する（MustBeMasked / AllowRaw / MustBeAbsent、PartialEq 付き）
- [x] `ProvenanceContract` 構造体を追加する（allowed_sources: Vec<DataSourceType>, pii_policy: PiiPolicy）
- [x] `validate_provenance_contract(contract: &ProvenanceContract, tag: &ProvenanceTag) -> Result<(), String>` を追加する
  - `allowed_sources` 非空かつ `tag.source.source_type` が含まれない → `Err("source type not allowed: ...")`
  - `MustBeMasked` かつ `tag.pii=true` → `Err("pii policy violated: MustBeMasked ...")`
  - `MustBeAbsent` かつ `tag.pii=true` → `Err("pii policy violated: MustBeAbsent ...")`
  - `AllowRaw` → 常に Ok
  - 全チェック通過 → `Ok(())`
- [x] `cargo test` で既存 3728 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.8.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v768000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `provenance_contract_source_violation` テストを実装する
  - Api ソースに Snowflake/S3 のみ許可コントラクト → Err（"source" を含む）
  - allowed_sources 空コントラクト → Ok（ソースチェックスキップ）
- [x] `provenance_contract_pii_violation` テストを実装する
  - pii=true + MustBeMasked → Err（"pii" を含む）
  - pii=true + MustBeAbsent → Err
  - pii=true + AllowRaw → Ok
- [x] `cargo test v768000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.7.0"` → `"76.8.0"` に変更する
- [x] `driver.rs` 内の `76.7.0` バージョン文字列アサーションを `76.8.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.8.0 に更新する
- [x] 「次に切る版」を v76.9.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3730 tests）
- [x] `cargo test v768000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.8.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.8.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v76.8.0 であることを確認する

---

## 完了チェックリスト

- [x] `versions/current.md` の「進行中バージョン」が v76.8.0 であることを確認する

- [x] 全タスク（T0〜T6）が完了している
- [x] `provenance_contract_source_violation` が pass
- [x] `provenance_contract_pii_violation` が pass
- [x] テスト総数: 3730（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_8_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.8.0 エントリ追加自体は T2 で必須
