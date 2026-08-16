# Roadmap v82.1.0 〜 v83.0.0 — Pipeline Contracts 1.0

Date: 2026-08-16
Status: 未着手（v82.0.0 完了後に開始）

マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)

---

## 前提

- 直前完了: v82.0.0「Data Quality 2.0 宣言」（tests = 3,853）
- 本スプリントは Quality-First Era の第 3 スプリント
- 目標: v83.0.0「Pipeline Contracts 1.0 宣言」（tests = 3,875）

### スプリントの性格

パイプライン間の入出力契約を型で定義し、`fav verify --contract` で検証する仕組みを構築する。
`IoContract`（インターフェース）/ `SlaContract`（SLA 遵守）/ `ContractRegistry`（チーム間共有）の 3 層構造。
`SlaContract` は Favnir 3.0 の `!Adaptive` / `!Cached` エフェクトと連携し、
「SLA を満たすための実行戦略を型で宣言する」形で実装する。
A（新機能）60% + B（統合）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v82.1.0 | `IoContract` / `ContractField` 型基盤 | 3853 + 2 = 3855 | 未着手 |
| v82.2.0 | `SlaContract`（SLA 遵守型） | 3855 + 2 = 3857 | 未着手 |
| v82.3.0 | パイプライン間契約依存（`ContractDependency`） | 3857 + 2 = 3859 | 未着手 |
| v82.4.0 | 契約違反の詳細レポート（`ContractViolation`） | 3859 + 2 = 3861 | 未着手 |
| v82.5.0 | スキーマから契約自動生成（`infer_contract`） | 3861 + 2 = 3863 | 未着手 |
| v82.6.0 | 契約バージョニング（`ContractVersion` / 後方互換チェック） | 3863 + 2 = 3865 | 未着手 |
| v82.7.0 | `fav verify --contract` コマンド強化 | 3865 + 2 = 3867 | 未着手 |
| v82.8.0 | 契約レジストリ（`ContractRegistry` / ローカルキャッシュ） | 3867 + 2 = 3869 | 未着手 |
| v82.9.0 | 安定化・コードフリーズ | 3869 + 2 = 3871 | 未着手 |
| v83.0.0 | Pipeline Contracts 1.0 宣言 ★クリーンアップ | 3871 + 4 = 3875 | 未着手 |

---

## v82.1.0 — `IoContract` / `ContractField` 型基盤

パイプラインの入出力インターフェースを型で定義する基盤を構築する。

**実装内容:**
- `ContractFieldType` enum（`Str` / `Int` / `Float` / `Bool` / `Nullable(Box<ContractFieldType>)` / `List(Box<ContractFieldType>)`）
- `ContractField` 構造体（`name: String`, `field_type: ContractFieldType`, `required: bool`）
- `IoContract` 構造体（`name: String`, `version: String`, `input: Vec<ContractField>`, `output: Vec<ContractField>`）
- `validate_io_contract(contract: &IoContract, actual_input: &[ContractField]) -> ContractValidationResult`
- `ContractValidationResult` 構造体（`valid: bool`, `errors: Vec<String>`）

**完了条件**: Rust テスト 2 件（3853 + 2 = 3855）
- `io_contract_validates_matching_fields`
- `io_contract_fails_on_missing_required_field`

---

## v82.2.0 — `SlaContract`（SLA 遵守型）

応答時間・スループット・可用性の SLA を型として宣言し、
Favnir 3.0 の `!Adaptive` / `!Cached` エフェクトと連携させる。

**実装内容:**
- `SlaTarget` 構造体（`max_latency_ms: u64`, `min_throughput_rps: f64`, `min_availability_pct: f64`）
- `SlaContract` 構造体（`name: String`, `target: SlaTarget`, `adaptive_strategy: Option<String>`, `cache_ttl_secs: Option<u64>`）
- `SlaStatus` enum（`Met` / `AtRisk(String)` / `Breached(String)`）
- `evaluate_sla(contract: &SlaContract, actual_latency_ms: u64, actual_rps: f64) -> SlaStatus`
- `format_sla_status(status: &SlaStatus) -> String`

**完了条件**: Rust テスト 2 件（3855 + 2 = 3857）
- `sla_contract_met_within_target`
- `sla_contract_breached_over_latency`

---

## v82.3.0 — パイプライン間契約依存（`ContractDependency`）

複数のパイプラインが「上流の契約出力を下流の入力として使う」依存関係を型で表現する。

**実装内容:**
- `ContractDependency` 構造体（`upstream: String`, `downstream: String`, `output_contract: String`）
- `DependencyGraph` 構造体（`dependencies: Vec<ContractDependency>`）
- `build_dependency_graph(contracts: &[IoContract]) -> DependencyGraph`
- `detect_circular_dependencies(graph: &DependencyGraph) -> Vec<Vec<String>>`
- `format_dependency_graph(graph: &DependencyGraph) -> String`

**完了条件**: Rust テスト 2 件（3857 + 2 = 3859）
- `dependency_graph_built_from_contracts`
- `circular_dependency_detected`

---

## v82.4.0 — 契約違反の詳細レポート（`ContractViolation`）

契約検証の失敗をどのフィールドがどう違反しているかまで詳細に報告する。

**実装内容:**
- `ViolationKind` enum（`TypeMismatch { expected: String, actual: String }` / `MissingField(String)` / `ExtraField(String)` / `NullNotAllowed(String)`）
- `ContractViolation` 構造体（`field: String`, `kind: ViolationKind`, `row_index: Option<usize>`）
- `ContractViolationReport` 構造体（`contract_name: String`, `violations: Vec<ContractViolation>`）
- `format_violation_report(report: &ContractViolationReport) -> String`
- `violation_severity(violation: &ContractViolation) -> RuleSeverity`

**完了条件**: Rust テスト 2 件（3859 + 2 = 3861）
- `violation_report_shows_type_mismatch`
- `violation_report_shows_missing_field`

---

## v82.5.0 — スキーマから契約自動生成（`infer_contract`）

実際のデータスキーマから `IoContract` を自動生成する。

**実装内容:**
- `infer_contract_from_schema(schema: &SchemaSnapshot, name: &str, version: &str) -> IoContract`
- `infer_field_type_from_str(type_name: &str) -> ContractFieldType`
- `merge_contracts(base: &IoContract, override_: &IoContract) -> IoContract`
- `format_contract_as_toml(contract: &IoContract) -> String`

**完了条件**: Rust テスト 2 件（3861 + 2 = 3863）
- `contract_inferred_from_schema`
- `contract_formatted_as_toml`

---

## v82.6.0 — 契約バージョニング（`ContractVersion` / 後方互換チェック）

契約のバージョンを管理し、後方互換性を自動チェックする。

**実装内容:**
- `ContractVersion` 構造体（`major: u32`, `minor: u32`, `patch: u32`）
- `ContractVersion::parse(s: &str) -> Result<ContractVersion, String>`
- `CompatibilityResult` enum（`Compatible` / `BackwardsCompatible(Vec<String>)` / `Breaking(Vec<String>)`）
- `check_contract_compatibility(old: &IoContract, new_: &IoContract) -> CompatibilityResult`
- `format_compatibility_result(result: &CompatibilityResult) -> String`

**完了条件**: Rust テスト 2 件（3863 + 2 = 3865）
- `contract_version_parsed`
- `breaking_change_detected_on_field_removal`

---

## v82.7.0 — `fav verify --contract` コマンド強化

既存の `fav verify --contract` を `IoContract` / `SlaContract` 対応に強化する。

**実装内容:**
- `VerifyContractOptions` 構造体（`contract_path: String`, `input_schema: Option<String>`, `strict: bool`）
- `cmd_verify_contract` 関数（`fav verify --contract` ハンドラ強化版）
- `ContractVerifyResult` 構造体（`io_result: ContractValidationResult`, `sla_result: Option<SlaStatus>`）
- `format_verify_result(result: &ContractVerifyResult) -> String`

**完了条件**: Rust テスト 2 件（3865 + 2 = 3867）
- `verify_contract_cmd_passes_valid_contract`
- `verify_contract_cmd_fails_breaking_change`

---

## v82.8.0 — 契約レジストリ（`ContractRegistry` / ローカルキャッシュ）

チーム間で契約を共有・検索・バージョン管理するローカルレジストリを作る。

**実装内容:**
- `ContractRegistryEntry` 構造体（`name: String`, `version: ContractVersion`, `contract: IoContract`, `registered_at: String`）
- `ContractRegistry` 構造体（`entries: Vec<ContractRegistryEntry>`）
- `ContractRegistry::register(entry: ContractRegistryEntry) -> ContractRegistry`
- `ContractRegistry::lookup(name: &str, version: Option<&str>) -> Option<&ContractRegistryEntry>`
- `ContractRegistry::list_all() -> Vec<&ContractRegistryEntry>`
- `format_registry_listing(registry: &ContractRegistry) -> String`

**完了条件**: Rust テスト 2 件（3867 + 2 = 3869）
- `contract_registry_register_and_lookup`
- `contract_registry_list_all`

---

## v82.9.0 — 安定化・コードフリーズ

v82.1〜v82.8 の全スプリント統合確認。バグ修正のみ。

**実装内容:**
- v82.1〜v82.8 の全テスト通過確認（`cargo test` 全 pass）
- `ContractRegistry` + `SlaContract` + `DependencyGraph` 連携確認
- `fav verify --contract` E2E 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3869 + 2 = 3871）
- `contracts_full_sprint_all_stable`
- `registry_and_sla_integrated`

---

## v83.0.0 — Pipeline Contracts 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「パイプライン間の約束が型になった。
>  `IoContract` がインターフェースを定義し、`SlaContract` が応答時間を保証し、
>  `ContractRegistry` がチームを繋ぐ。
>  Favnir のパイプラインは今、契約で安全に接続できる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `83.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- マスターロードマップの本スプリントを「完了」に更新

**完了条件**: `v83000_tests` 4 件（3871 + 4 = 3875）
- `cargo_toml_version_is_83_0_0`
- `changelog_has_v83_0_0`
- `milestone_has_pipeline_contracts`
- `readme_mentions_contract_registry`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v82.0.0（ベース） | 3,853 | — |
| v82.1.0 | 3,855 | +2 |
| v82.2.0 | 3,857 | +2 |
| v82.3.0 | 3,859 | +2 |
| v82.4.0 | 3,861 | +2 |
| v82.5.0 | 3,863 | +2 |
| v82.6.0 | 3,865 | +2 |
| v82.7.0 | 3,867 | +2 |
| v82.8.0 | 3,869 | +2 |
| v82.9.0 | 3,871 | +2 |
| v83.0.0（宣言） | 3,875 | +4 |

**本スプリント合計**: +22 tests（3,853 → 3,875）

---

## 参考リンク

- マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 前スプリント: [roadmap-v81.1-v82.0.md](roadmap-v81.1-v82.0.md)
- 次スプリント: [roadmap-v83.1-v84.0.md](roadmap-v83.1-v84.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
