# Plan: v97.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 1: `fav/src/driver.rs` に `mod v97900_tests` を追加

`mod v97800_tests` の直後に追加（`std::fs::read_to_string` パターン）:

```rust
#[cfg(test)]
mod v97900_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_workflow_mdx_has_iflow_client() {
        let content = std::fs::read_to_string(
            "../site/content/docs/guides/sap-workflow.mdx",
        )
        .expect("site/content/docs/guides/sap-workflow.mdx should exist");
        assert!(
            content.contains("IFlowClient"),
            "sap-workflow.mdx should document IFlowClient (v97.5.0 ↔ v97.8.0 coherence)"
        );
    }
    #[test]
    fn mock_fav_has_impl_approval_client() {
        let content = std::fs::read_to_string("../runes/sap-odata/mock.fav")
            .expect("runes/sap-odata/mock.fav should exist");
        assert!(
            content.contains("impl ApprovalClient for MockWorkflowClient"),
            "mock.fav should implement ApprovalClient for MockWorkflowClient (v97.3.0 ↔ v97.7.0 coherence)"
        );
    }
}
```

### Step 2: `cargo test` で全 pass 確認

テスト数: 4,229 + 2 = 4,231

### Step 3: `CHANGELOG.md` に v97.9.0 エントリを追加

先頭に追加。

### Step 4: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v97.9.0` に更新
- 最新安定版を `v97.9.0 — 4,231 tests` に更新

### Step 5: CI 事前確認（Clippy / Self-fmt）

CHANGELOG / current.md 更新完了後に実施する（`cargo test` 再実行は不要）。

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
