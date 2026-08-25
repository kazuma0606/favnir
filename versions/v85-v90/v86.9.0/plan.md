# Plan: v86.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.9.0 エントリ追加

v86.8.0 エントリの直前（先頭）に v86.9.0 エントリを追加する。
`changelog_has_v86_9_0` テストが Step 3 の `cargo test` 前に通る必要があるため、
必ずテストモジュール追加（Step 2）より先に実施すること。

### Step 2: `fav/src/driver.rs` に `mod v86900_tests` 追加

`mod v86800_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v86900_tests {
    #[test]
    fn sap_master_data_business_partner_crud_covered() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(content.contains("fn business_partners"), "business_partner.fav should define business_partners");
        assert!(content.contains("fn business_partner_by_id"), "business_partner.fav should define business_partner_by_id");
        assert!(content.contains("fn create_business_partner"), "business_partner.fav should define create_business_partner");
        assert!(content.contains("fn update_business_partner"), "business_partner.fav should define update_business_partner");
    }

    #[test]
    fn sap_master_data_scenario1_pipeline_exists() {
        let path = std::path::Path::new("../infra/e2e-demo/sap-odata/pipeline.fav");
        assert!(path.exists(), "infra/e2e-demo/sap-odata/pipeline.fav should exist");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

ベース: v86.8.0 完了時点で 3969 tests。

```
cargo test 2>&1 | grep "test result"
```

期待: `3971 tests, 0 failures`

### Step 4: CI 事前確認

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
