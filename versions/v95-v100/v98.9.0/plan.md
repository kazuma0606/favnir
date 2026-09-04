# Plan: v98.9.0 — 安定化・コードフリーズ

## 実装順序

### Step 1: driver.rs に mod v98900_tests を追加

`mod v98800_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98900_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_odata_rune_exports_kpi_alert() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/sap_odata.fav",
        )
        .expect("sap_odata.fav should exist");
        assert!(
            content.contains("KpiAlert"),
            "sap_odata.fav should re-export KpiAlert (v98.9.0 freeze check)"
        );
    }

    #[test]
    fn analytics_demo_run_script_exists() {
        std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/analytics_demo/run.sh",
        )
        .expect("analytics_demo/run.sh should exist (v98.9.0 freeze check)");
    }
}
```

---

### Step 2: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,253 tests, 0 failures

---

### Step 3: CHANGELOG.md に v98.9.0 エントリを追加

---

### Step 4: versions/current.md 更新

最新安定版を `v98.9.0` に更新（テスト数 4,253）。

---

### Step 5: CI 事前確認（コードフリーズ確認）

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
