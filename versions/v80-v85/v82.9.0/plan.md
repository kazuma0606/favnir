# v82.9.0 実装計画 — 安定化・コードフリーズ

---

## 実装ステップ

### Step 1: `contracts_full_sprint_all_stable` テスト追加

`fav/src/driver.rs` 末尾に以下の統合確認テストを追加する。

v82.1〜v82.8 の各主要型・関数を代表例で呼び出し、panic なく動作することを確認:

- v82.1: `validate_io_contract` → `ContractValidationResult.valid == true`
- v82.2: `evaluate_sla` → `SlaStatus::Met`
- v82.3: `build_dependency_graph` → dependencies が空
- v82.4: `format_violation_report`（空 violations）→ 非空文字列が返る
- v82.5: `infer_field_type_from_str("Int")` → `ContractFieldType::Int`、`format_contract_as_toml` → 契約名を含む文字列
- v82.6: `ContractVersion::parse("1.0.0")` → `major == 1`
- v82.7: `cmd_verify_contract` → `io_result.valid == true`、`format_verify_result` → "PASS"（E2E フロー）
- v82.8: `ContractRegistry::new().list_all().is_empty() == true`

---

### Step 2: `registry_and_sla_integrated` テスト追加

`ContractRegistry` + `SlaContract` + `DependencyGraph` + `cmd_verify_contract` + `check_contract_compatibility` の連携シナリオ:

1. `IoContract` を 2 つ作成して `build_dependency_graph` → `format_dependency_graph`
2. `SlaContract` を作成して `evaluate_sla` → `SlaStatus::Met`
3. `ContractRegistryEntry` を作成して `ContractRegistry::new().register()` → `lookup`
4. `cmd_verify_contract` で IoContract を検証 → `io_result.valid == true`（input が空なので全フィールド存在）
5. `check_contract_compatibility` で `c1` と `c2`（ともに input/output 空）を比較 → `Compatible`

---

### Step 3: CHANGELOG 更新

`CHANGELOG.md` 先頭に v82.9.0 エントリを追加する。

---

### Step 4: テスト通過確認

`cargo test` を実行し 3,883 tests pass（+2）を確認する。

---

## 依存関係

```
（新型・新関数なし）
v82.1〜v82.8 の型をすべて使用する統合テスト
    └── v82900_tests
```

## 注意事項

- `test_framework.rs` への変更は**バグ修正が発生した場合のみ**行う
- 新機能・新型の追加は禁止
