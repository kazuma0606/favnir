# Plan: v89.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 1: 着手前確認

```bash
cd fav && cargo test 2>&1 | grep "test result"
```

4,035 tests, 0 failures を確認する。

### Step 2: `runes/sap-odata/` ディレクトリの存在確認

```bash
ls ../runes/sap-odata/
```

`sap_integration_rune_registry_deployed` テストが参照するディレクトリが存在することを確認する。
存在しない場合は空ディレクトリを作成する:

```bash
mkdir -p ../runes/sap-odata
```

### Step 3: `mod v89900_tests` を `driver.rs` に追加

`mod v89800_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89900_tests {
    #[test]
    fn sap_all_four_scenarios_in_pipeline() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("sync_business_partners"),  "Scenario 1 missing");
        assert!(content.contains("daily_sales_report"),      "Scenario 2 missing");
        assert!(content.contains("check_stock_vs_orders"),   "Scenario 3 missing");
        assert!(content.contains("outstanding_payables"),    "Scenario 4 missing");
    }

    #[test]
    fn sap_integration_rune_registry_deployed() {
        assert!(
            std::path::Path::new("../runes/sap-odata").exists(),
            "runes/sap-odata/ should exist (Rune Registry deployment artifact)"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

4,035 + 2 = 4,037 tests, 0 failures を確認する。

### Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
