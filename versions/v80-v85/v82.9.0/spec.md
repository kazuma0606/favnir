# v82.9.0 — 安定化・コードフリーズ

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 9 版（最終安定化）。
v82.1.0〜v82.8.0 で構築した以下の型・関数を統合確認し、バグ修正のみ受け入れる。

| バージョン | 主な成果 |
|---|---|
| v82.1.0 | `IoContract` / `ContractField` 型基盤 |
| v82.2.0 | `SlaContract` / `evaluate_sla` |
| v82.3.0 | `DependencyGraph` / `build_dependency_graph` |
| v82.4.0 | `ContractViolation` / `format_violation_report` |
| v82.5.0 | `infer_contract_from_schema` / `merge_contracts` |
| v82.6.0 | `ContractVersion` / `check_contract_compatibility` |
| v82.7.0 | `cmd_verify_contract` / `format_verify_result` |
| v82.8.0 | `ContractRegistry` / `format_registry_listing` |

本バージョンでは**新機能を追加しない**。
2 件の統合確認テストを追加し、全スプリントの型・関数が連携して動作することを保証する。

---

## Goals

1. `contracts_full_sprint_all_stable` テストを追加する
   - v82.1〜v82.8 の各主要型・関数（代表例）を順に呼び出し、例外・panic なく動作することを確認する
   - `cargo test` が 3,883 tests（+2）pass であることが合格条件

2. `registry_and_sla_integrated` テストを追加する
   - `ContractRegistry` + `SlaContract` + `DependencyGraph` の連携シナリオ:
     1. `IoContract` を 2 つ作成して `build_dependency_graph` で依存グラフを構築する
     2. `SlaContract` を作成して `evaluate_sla` で SLA を評価する
     3. `ContractRegistry` に登録して `lookup` で取得する
     4. `cmd_verify_contract` で IoContract を検証する
     5. `check_contract_compatibility` で後方互換性を確認する
   - すべてのステップが期待通りに動作することを確認する

3. バグ修正のみ受け入れる（新機能追加なし）

---

## API Examples（Rust テストコード）

### `contracts_full_sprint_all_stable`

```rust
// v82.1: IoContract / ContractField
let field = ContractField { name: "id".into(), field_type: ContractFieldType::Int, required: true };
let contract = IoContract { name: "orders".into(), version: "1.0.0".into(), input: vec![field.clone()], output: vec![] };
let result = validate_io_contract(&contract, &[field.clone()]);
assert!(result.valid);

// v82.2: SlaContract / evaluate_sla
let sla = SlaContract {
    name: "orders-sla".into(),
    target: SlaTarget { max_latency_ms: 200, min_throughput_rps: 100.0, min_availability_pct: 99.9 },
    adaptive_strategy: None,
    cache_ttl_secs: None,
};
assert!(matches!(evaluate_sla(&sla, 100, 150.0), SlaStatus::Met));

// v82.3: DependencyGraph
let graph = build_dependency_graph(&[contract.clone()]);
assert!(graph.dependencies.is_empty()); // 1 契約では依存なし

// v82.4: ContractViolation / format_violation_report
let report = ContractViolationReport { contract_name: "orders".into(), violations: vec![] };
let txt = format_violation_report(&report);
assert!(!txt.is_empty());

// v82.5: infer_field_type_from_str / format_contract_as_toml
let ft = infer_field_type_from_str("Int");
assert!(matches!(ft, ContractFieldType::Int));
let toml_str = format_contract_as_toml(&contract);
assert!(toml_str.contains("orders"));

// v82.6: ContractVersion / check_contract_compatibility
let version = ContractVersion::parse("1.0.0").unwrap();
assert_eq!(version.major, 1);

// v82.7: cmd_verify_contract / format_verify_result（E2E フロー確認）
let options = VerifyContractOptions { contract_path: "c.toml".into(), input_schema: None, strict: false };
let vr = cmd_verify_contract(&options, &contract, &[field.clone()], None);
assert!(vr.io_result.valid);
let vr_str = format_verify_result(&vr);
assert!(vr_str.contains("PASS")); // ← `fav verify --contract` のフルフロー（options → cmd → format）を確認

// v82.8: ContractRegistry
let registry = ContractRegistry::new();
assert!(registry.list_all().is_empty());
```

### `registry_and_sla_integrated`

```rust
// IoContract 2 つを ContractRegistry に登録して連携確認
let c1 = IoContract { name: "orders".into(), version: "1.0.0".into(), input: vec![], output: vec![] };
let c2 = IoContract { name: "payments".into(), version: "1.0.0".into(), input: vec![], output: vec![] };

// DependencyGraph 構築
let graph = build_dependency_graph(&[c1.clone(), c2.clone()]);
let fmt = format_dependency_graph(&graph);
assert!(fmt.contains("dependencies"));

// SLA 評価
let sla = SlaContract {
    name: "orders-sla".into(),
    target: SlaTarget { max_latency_ms: 200, min_throughput_rps: 100.0, min_availability_pct: 99.9 },
    adaptive_strategy: None,
    cache_ttl_secs: None,
};
let sla_status = evaluate_sla(&sla, 100, 150.0);
assert!(matches!(sla_status, SlaStatus::Met));

// ContractRegistry への登録と lookup
let v1 = ContractVersion::parse("1.0.0").unwrap();
let entry = ContractRegistryEntry {
    name: "orders".into(), version: v1, contract: c1.clone(),
    registered_at: "2026-08-20T00:00:00Z".into(),
};
let registry = ContractRegistry::new().register(entry);
let found = registry.lookup("orders", Some("1.0.0"));
assert!(found.is_some());

// cmd_verify_contract + format_verify_result（`fav verify --contract` E2E フロー）
let options = VerifyContractOptions { contract_path: "c.toml".into(), input_schema: None, strict: false };
let vr = cmd_verify_contract(&options, &c1, &[], None);
assert!(vr.io_result.valid); // input が空なので全フィールド存在
let vr_str = format_verify_result(&vr);
assert!(vr_str.contains("PASS"));

// check_contract_compatibility
let compat = check_contract_compatibility(&c1, &c2);
assert!(matches!(compat, CompatibilityResult::Compatible));
```

---

## Success Criteria

- `cargo test` 全 pass（3,883 tests = 3,881 + 2）※ drift 補正後
- 新規テスト 2 件（`v82900_tests` モジュール）:
  - `contracts_full_sprint_all_stable`
  - `registry_and_sla_integrated`
- バグ修正以外の変更なし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `#[cfg(test)] mod v82900_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.9.0 エントリを先頭に追加 |

`fav/src/test_framework.rs` への変更は**なし**（バグ修正が発生した場合のみ）。
