# v73.1.0 タスクリスト — データコントラクト

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.0.0` であることを確認
- [x] `cargo test` が 3646 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v73000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v731000_tests` が未存在であることを確認
- [x] `driver.rs` 内の `"73.0.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: 構造体追加（`DataContractField` / `DataContractSla` / `DataContract`）

- [x] `DataContractField { name: String, ty: String, nullable: bool }` を追加した
- [x] `DataContractSla { max_latency_ms: u64, min_throughput: u64, max_error_rate: f64 }` を追加した
- [x] `DataContract { name: String, input_fields: Vec<DataContractField>, output_fields: Vec<DataContractField>, sla: DataContractSla }` を追加した
- [x] 全フィールドが `pub` であることを確認
- [x] `cargo build` でエラーがないことを確認

---

## T2: `validate_contract_schema` 追加

- [x] `pub fn validate_contract_schema(contract: &DataContract, actual_input: &[(&str, &str)]) -> Result<(), String>` を実装した
  - `contract.input_fields` の各フィールドが `actual_input` に存在するか確認
  - フィールド不在 → `Err("schema mismatch: field '...' missing in actual input")`
  - 型不一致 → `Err("schema mismatch: field '...' expected type '...', got '...'")`
  - 全一致 → `Ok(())`
- [x] `cargo build` でエラーがないことを確認

---

## T3: `check_sla_compliance` 追加

- [x] `pub fn check_sla_compliance(sla: &DataContractSla, actual_latency_ms: u64, actual_throughput: u64, actual_error_rate: f64) -> Result<(), String>` を実装した
  - `actual_latency_ms > sla.max_latency_ms` → `Err("SLA violation: latency ...")`
  - `actual_throughput < sla.min_throughput` → `Err("SLA violation: throughput ...")`
  - `actual_error_rate > sla.max_error_rate` → `Err("SLA violation: error rate ...")`
  - 全条件クリア → `Ok(())`
- [x] `cargo build` でエラーがないことを確認

---

## T4: `v731000_tests` モジュール追加

- [x] `v73000_tests` モジュールの直後に `v731000_tests` モジュールを追加した
- [x] `use super::{DataContract, DataContractField, DataContractSla, validate_contract_schema, check_sla_compliance}` を追加した
- [x] `data_contract_schema_mismatch_error` テストを実装した
  - 正しい入力スキーマ → `Ok(())` を assert
  - 型不一致（Float → Int）→ `Err` を assert、エラー文字列に `"amount"` が含まれることを assert
  - フィールド欠落（`amount` なし）→ `Err` を assert、エラー文字列に `"amount"` が含まれることを assert
- [x] `data_contract_sla_monitoring` テストを実装した
  - SLA 満足（latency 3000ms, throughput 1500, error 0.005）→ `Ok(())` を assert
  - レイテンシ超過（6000ms）→ `Err` を assert
  - エラー文字列に `"latency"` が含まれることを assert
- [x] `cargo test v731000` で 2 件 pass することを確認

---

## T5: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.0.0"` → `version = "73.1.0"` に変更した
- [x] `driver.rs` 内の `version = \"73.0.0\"` を `version = \"73.1.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 73.0.0"` を `"73.1.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 73.0.0"` を `"73.1.0"` に replace_all した
- [x] 残存 `73.0.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `grep "73.0.0" driver.rs` で意図的保持分以外がゼロ件であることを確認
- [x] `cargo build` 後に `fav/Cargo.lock` が `version = "73.1.0"` を含むことを確認

---

## T6: 部分テスト確認

- [x] `cargo test v731000` で 2 件 pass することを確認

---

## T7: 全体テスト確認

- [x] `cargo test` 全体で 3648 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [x] `## [v73.1.0]` エントリを先頭に追加した
  - Added: `DataContractField` / `DataContractSla` / `DataContract` / `validate_contract_schema` / `check_sla_compliance`
  - Tests: 2 件、合計テスト数 3648（+2）

---

## T9: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.1.0)` に更新した
- [x] 「進行中バージョン」を `v73.1.0`（データコントラクト）に更新した
- [x] 「次に切る版」を `v73.2.0` に更新した

---

## T10: 最終確認（T8・T9 完了後）

- [x] `cargo test v731000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3648 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.1.0` であることを確認
- [x] `DataContract` / `DataContractSla` / `DataContractField` が pub で存在することを確認
- [x] `validate_contract_schema` / `check_sla_compliance` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.1.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.1.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [BUG] | `nullable` フィールドが validate_contract_schema で未使用（dead_code 警告リスク） | フィールドに「将来のランタイム NULL 検証用」コメントを追加 |
| [BUG] | SLA 境界値（等値）テストが欠落 | `check_sla_compliance(&sla, 5000, 1000, 0.01)` が `Ok` になるケースを追加 |
| [BUG] | throughput / error_rate 違反ケースが未テスト | `data_contract_sla_monitoring` に各違反 assert を追加 |
| [STYLE] | `validate_contract_schema` の open-world 仕様が不明瞭 | 関数直前に open-world コメントを追加 |
| [BUG] | `v73000_tests` の関数名と検証バージョンの乖離 | codebase パターン（replace_all で内容のみ更新、関数名は据え置き）のため変更なし |

---

## スコープ外（明示的除外）

- `contract` キーワードのパーサー統合（将来バージョン）
- WASM / サイト MDX 更新（v74.x 以降）
