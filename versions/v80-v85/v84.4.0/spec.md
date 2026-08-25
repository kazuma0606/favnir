# spec: v84.4.0 — 契約統合ショーケース（`fav verify --contract` E2E）

## Background

> **テスト数注記**: ロードマップ計画値は 3,903/3,905 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,915 tests**（v84.3.0 完了時点）。
> v84.4.0 完了目標は **3,917 tests**（+2）。

v84.3.0 でショーケースに Data Quality 2.0（QualityCheck / QualityGate / AnomalyDetector）
を統合した。v84.4.0 では Sprint 3「Pipeline Contracts 1.0」の機能
（IoContract / SlaContract / ContractDependency / ContractRegistry）を統合し、
`contract.fav` を完成版に更新してショーケースが型付き契約検証を示すことを確認する。

## Goals

1. `infra/e2e-demo/favnir4-showcase/contract.fav` を完成版に更新する
   - `SlaContract` 型宣言を追加（`SlaTarget` + optional フィールドを含む）
   - `ContractDependency` 宣言を追加（upstream / downstream / output_contract）
2. `infra/e2e-demo/favnir4-showcase/pipeline.fav` に契約統合セクションを追加する
   - `ContractRegistry` 登録フロー（`IoContract` + `ContractRegistryEntry` を登録）
3. Rust テスト 2 件でショーケースの内容を検証する
   - `showcase_contract_verified` — contract.fav に SlaContract / ContractDependency が含まれること
   - `showcase_contract_registry_registered` — pipeline.fav に ContractRegistry / IoContract が含まれること

## Syntax / API Examples（実際の型定義に基づく）

### contract.fav への追加（SlaContract + ContractDependency）

```favnir
-- SlaContract: SLA 目標値を型として宣言（Sprint 3 v82.2.0）
type SlaContract {
    name: String,
    target: SlaTarget,
    adaptive_strategy: Option<String>,
    cache_ttl_secs: Option<Int>,
}

-- ContractDependency: パイプライン間の依存エッジ（Sprint 3 v82.3.0）
type ContractDependency {
    upstream: String,
    downstream: String,
    output_contract: String,
}
```

### pipeline.fav への追加（ContractRegistry 登録セクション）

```favnir
-- ── 契約統合セクション（Sprint 3: Pipeline Contracts 1.0）──────────────

fn showcase_contract_registry(ctx: AppCtx) -> Result<ContractRegistry, String> {
    -- ContractRegistry: IoContract を登録して共有レジストリを構築
    bind registry <- ContractRegistry { entries: List.empty() }
    bind contract <- IoContract {
        name: "Favnir4ShowcaseContract",
        version: "1.0.0",
        input: List.empty(),
        output: List.empty(),
    }
    bind entry <- ContractRegistryEntry {
        name: "showcase",
        version: ContractVersion { major: 1, minor: 0, patch: 0 },
        contract: contract,
    }
    Result.ok(registry.register(entry))
}
```

### v84400_tests（Rust テスト）

```rust
#[cfg(test)]
mod v84400_tests {
    #[test]
    fn showcase_contract_verified() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/contract.fav");
        assert!(content.contains("SlaContract"),       "contract.fav should include SlaContract");
        assert!(content.contains("ContractDependency"), "contract.fav should include ContractDependency");
    }

    #[test]
    fn showcase_contract_registry_registered() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("ContractRegistry"), "pipeline.fav should include ContractRegistry");
        assert!(content.contains("IoContract"),       "pipeline.fav should include IoContract");
    }
}
```

## 実際の型定義（参照）

以下は Rust 実装シグネチャ。Favnir 側では `Vec<T>` → `List<T>`、`u64`/`u32` → `Int`、`Option<T>` → `Option<T>`（同名）に読み替える。

| 型 / 関数 | Rust シグネチャ |
|---|---|
| `IoContract` | `name: String`, `version: String`, `input: Vec<ContractField>`, `output: Vec<ContractField>` |
| `ContractField` | `name: String`, `field_type: ContractFieldType`, `required: bool` |
| `SlaContract` | `name: String`, `target: SlaTarget`, `adaptive_strategy: Option<String>`, `cache_ttl_secs: Option<u64>` |
| `SlaTarget` | `max_latency_ms: u64`, `min_throughput_rps: f64`, `min_availability_pct: f64` |
| `ContractDependency` | `upstream: String`, `downstream: String`, `output_contract: String` |
| `ContractRegistry` | `entries: Vec<ContractRegistryEntry>` |
| `ContractRegistryEntry` | `name: String`, `version: ContractVersion`, `contract: IoContract` |
| `ContractVersion` | `major: u32`, `minor: u32`, `patch: u32` |
| `ContractRegistry::register` | `(&self, entry: ContractRegistryEntry) -> ContractRegistry` |

## Success Criteria

- `infra/e2e-demo/favnir4-showcase/contract.fav` に `SlaContract`・`ContractDependency` が含まれること
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `ContractRegistry`・`IoContract` が含まれること
- `cargo test` が 3,917 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル更新のみ）

## Files to Modify / Create

### 更新
- `infra/e2e-demo/favnir4-showcase/contract.fav` — SlaContract + ContractDependency 型宣言を追加
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — 契約統合セクションを末尾に追加

### 追記
- `fav/src/driver.rs` — `v84400_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.4.0 エントリ追加

### パス起点（v84.1.0 から踏襲）

`v84400_tests` は `include_str!("../../infra/...")` を使用。
パス起点は `fav/src/driver.rs`（`fav/src/`）。`driver.rs` 移動時はパスを更新すること。
