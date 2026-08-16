# v76.7.0 タスクリスト — Data product 型

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.6.0` であることを確認
- [x] `cargo test` が全 pass（3726 tests）であることを確認（v76.7.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.7.0: Data product 型 ---` コメントを追加する
- [x] `DataProductSla` 構造体を追加する（freshness_minutes: u64）
- [x] `ProvenancePolicy` 構造体を追加する（require_source_declared: bool, pii_must_be_masked: bool）
- [x] `DataProduct` 構造体を追加する（name: String, owner: String, sla: DataProductSla, provenance_policy: ProvenancePolicy）
- [x] `validate_data_product(product: &DataProduct, tag: &ProvenanceTag) -> Result<(), String>` を追加する
  - `require_source_declared=true` かつ `tag.source.name.is_empty()` → `Err("source must be declared: ...")`
  - `pii_must_be_masked=true` かつ `tag.pii=true` → `Err("pii policy violated: ...")`
  - 両方満たせば `Ok(())`
- [x] `cargo test` で既存 3726 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.7.0 エントリを追加する
- [x] Added セクション（struct 3 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v767000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `data_product_validated` テストを実装する
  - source.name="crm"（非空）、pii=false、require_source_declared=true、pii_must_be_masked=true → Ok
- [x] `data_product_pii_policy_violated` テストを実装する
  - pii=true、pii_must_be_masked=true → Err、エラーメッセージに "pii" を含む
- [x] `cargo test v767000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.6.0"` → `"76.7.0"` に変更する
- [x] `driver.rs` 内の `76.6.0` バージョン文字列アサーションを `76.7.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.7.0 に更新する
- [x] 「次に切る版」を v76.8.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3728 tests）
- [x] `cargo test v767000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.7.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.7.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v76.7.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `data_product_validated` が pass
- [x] `data_product_pii_policy_violated` が pass
- [x] テスト総数: 3728（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_7_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.7.0 エントリ追加自体は T2 で必須
