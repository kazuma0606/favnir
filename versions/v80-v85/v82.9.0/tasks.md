# v82.9.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,881 tests pass、0 failures であることを確認する（前提: v82.8.0 完了済み）

## T1: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.9.0 エントリを追加する

（site/ MDX / ドキュメント更新: 本バージョンは統合確認テストのみのため対象外）

## T2: `contracts_full_sprint_all_stable` テスト追加

- [x] `fav/src/driver.rs` 末尾の `v82900_tests` モジュールに `contracts_full_sprint_all_stable` を追加する
  - v82.1: `validate_io_contract` → `valid == true`
  - v82.2: `evaluate_sla` → `SlaStatus::Met`
  - v82.3: `build_dependency_graph` → `dependencies.is_empty()`
  - v82.4: `format_violation_report`（空 violations）→ 非空文字列
  - v82.5: `infer_field_type_from_str("Int")` → `ContractFieldType::Int`、`format_contract_as_toml` → 契約名を含む
  - v82.6: `ContractVersion::parse("1.0.0")` → `major == 1`
  - v82.7: `cmd_verify_contract` → `io_result.valid == true`、`format_verify_result` → "PASS"（E2E フロー）
  - v82.8: `ContractRegistry::new().list_all().is_empty()`

## T3: `registry_and_sla_integrated` テスト追加

- [x] `fav/src/driver.rs` 末尾の `v82900_tests` モジュールに `registry_and_sla_integrated` を追加する
  - 2 契約 → `build_dependency_graph` → 依存なし（input/output 空のため）を確認、`format_dependency_graph` が panic しないことを確認
  - `SlaContract` → `evaluate_sla` → `SlaStatus::Met`
  - `ContractVersion::parse("1.0.0")` を使って `ContractRegistryEntry` を構築
  - `ContractRegistry::new().register(entry)` → `lookup("orders", Some("1.0.0"))` → Some
  - `cmd_verify_contract`（input 空契約, actual 空） → `io_result.valid == true`、`format_verify_result` → "PASS"
  - `check_contract_compatibility`（ともに input/output 空） → `Compatible`

## T4: テスト通過確認

- [x] `cargo test` が 3,883 tests pass（+2）、0 failures であることを確認する

## T5: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 実装メモ

- `format_dependency_graph` は依存なし時に空文字列を返す（`join("\n")` の仕様）。
  spec.md のサンプルで `fmt.contains("dependencies")` とあったが、実際の出力形式と不一致のため `graph.dependencies.is_empty()` + `assert!(fmt.is_empty())` に変更。

## code-reviewer 対応

- [x] [LOW] `let _ = fmt;` → `assert!(fmt.is_empty(), ...)` に変更（意図を明示）
- [x] [LOW] `v82900_tests` を `v82800_tests` の後ろに移動（バージョン番号昇順に修正）
