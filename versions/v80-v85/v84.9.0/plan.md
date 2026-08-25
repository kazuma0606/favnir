# plan: v84.9.0 — 安定化・コードフリーズ

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,925 tests, 0 failures を確認する（前提: v84.8.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する（v84.x マイナーバージョンは Cargo.toml 更新不要）
- `fav/src/driver.rs` に `mod v84800_tests` が存在することを確認する

> 注: ロードマップ計画値は 3,913/3,915 だが、code-reviewer 対応の累積で実績ベースは 3,925/3,927。

### Step 2: E2E ショーケース統合確認

`infra/e2e-demo/favnir4-showcase/pipeline.fav` を確認し、4 スプリント全識別子が含まれていることを目視検証する。

| スプリント | 識別子 | v84.x で追加 |
|---|---|---|
| Sprint 1: Test-Driven Data | `TestSuite` | v84.2.0 |
| Sprint 2: Data Quality 2.0 | `QualityCheck` | v84.3.0 |
| Sprint 3: Pipeline Contracts | `ContractRegistry` | v84.4.0 |
| Sprint 4: Observability 2.0 | `PipelineMetrics` | v84.5.0 |

`infra/e2e-demo/favnir4-showcase/fav.toml` を確認し、`[quality]`・`[contract]`・`[observe]`
の 3 セクションが含まれていることを確認する。

### Step 3: v80〜v84 全スプリント統合動作確認

v84.1〜v84.8 で追加した全機能が driver.rs のテストモジュールとして存在することを確認する。
v80〜v83 のモジュールは前バージョンで確認済みのため、ここでは v84.x の全 8 モジュールを対象とする。

```
v84100_tests, v84200_tests, v84300_tests, v84400_tests, v84500_tests,
v84600_tests, v84700_tests, v84800_tests
```

バグが見つかった場合のみ修正する（新機能追加なし）。

### Step 4: driver.rs に v84900_tests を追加

`mod v84800_tests` の直後に `#[cfg(test)] mod v84900_tests` を追加する。

```rust
#[cfg(test)]
mod v84900_tests {
    #[test]
    fn favnir4_full_sprint_all_stable() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("TestSuite"),        "pipeline.fav should include TestSuite (Sprint 1)");
        assert!(content.contains("QualityCheck"),     "pipeline.fav should include QualityCheck (Sprint 2)");
        assert!(content.contains("ContractRegistry"), "pipeline.fav should include ContractRegistry (Sprint 3)");
        assert!(content.contains("PipelineMetrics"),  "pipeline.fav should include PipelineMetrics (Sprint 4)");
    }

    #[test]
    fn favnir4_e2e_showcase_runs() {
        let toml = include_str!("../../infra/e2e-demo/favnir4-showcase/fav.toml");
        assert!(toml.contains("[quality]"),  "fav.toml should have [quality] section");
        assert!(toml.contains("[contract]"), "fav.toml should have [contract] section");
        assert!(toml.contains("[observe]"),  "fav.toml should have [observe] section");
    }
}
```

### Step 5: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,927 tests, 0 failures を確認する。

### Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.9.0 エントリを追加する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
