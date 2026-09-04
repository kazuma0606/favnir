# Plan: v90.9.0 — 安定化・コードフリーズ

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（driver.rs テスト追加）
    ↓
Step 3（cargo test）
    ↓
Step 4（CHANGELOG 更新）
    ↓
Step 5（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `cargo test 2>&1 | grep "test result"` で 4,059 tests, 0 failures を確認する
- `infra/e2e-demo/sap-odata/pipeline.fav` に 4 シナリオ関数と `ctx.sap.` が含まれることを確認する
- `runes/sap-odata/mock.fav` が存在することを確認する

### Step 2: `driver.rs` に `mod v90900_tests` を追加

`mod v90800_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v90900_tests {
    use std::path::Path;

    #[test]
    fn sap_ctx_integration_smoke_all_scenarios() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline.fav",
        )
        .expect("pipeline.fav should exist");
        for name in &[
            "sync_business_partners",
            "daily_sales_report",
            "check_stock_vs_orders",
            "outstanding_payables",
            "ctx.sap.",
        ] {
            assert!(
                content.contains(name),
                "pipeline.fav should contain {}",
                name
            );
        }
    }

    #[test]
    fn sap_ctx_mock_client_in_rune_dir() {
        let path = Path::new("../runes/sap-odata/mock.fav");
        assert!(path.exists(), "runes/sap-odata/mock.fav should exist");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4,061 tests, 0 failures を確認する

### Step 4: `CHANGELOG.md` に v90.9.0 エントリを追加

- `## [v90.8.0]` の前に v90.9.0 エントリを追加する
- `安定化` / `コードフリーズ` / `4,061` が含まれることを確認する

### Step 5: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
