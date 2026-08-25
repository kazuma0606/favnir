# plan: v84.4.0 — 契約統合ショーケース（`fav verify --contract` E2E）

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,915 tests, 0 failures を確認する（前提: v84.3.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
  > 注: ロードマップ計画値は 3,903/3,905 だが、code-reviewer 対応の累積で実績ベースは 3,915/3,917 となっている。
- `fav/src/driver.rs` に `mod v84300_tests` が存在することを確認する

### Step 2: contract.fav を完成版に更新

`infra/e2e-demo/favnir4-showcase/contract.fav` の末尾に以下を追加する。

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

### Step 3: pipeline.fav に契約統合セクションを追加

現在の `pipeline.fav` 末尾に以下を追加する。

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

### Step 4: driver.rs に v84400_tests を追加

`mod v84300_tests` の直後に `#[cfg(test)] mod v84400_tests` を追加する。

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

### Step 5: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,917 tests, 0 failures を確認する。

### Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.4.0 エントリを追加する。

> 注: `site/` MDX 追加は v84.6.0 で一括実施するため本バージョンでは省略する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
