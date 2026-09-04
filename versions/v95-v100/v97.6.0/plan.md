# Plan: v97.6.0 — E2E デモ（発注 → 承認 → SAP 反映）

## 実装ステップ

### Step 1: `infra/e2e-demo/sap-odata/workflow_demo/README.md` 新規作成

内容:
- デモの概要（発注書作成 → 承認フロー起動 → 承認完了 → SAP 反映）
- 前提条件（fav CLI インストール済み、pipeline_workflow.fav の存在）
- 実行手順（`bash run.sh`）

### Step 2: `infra/e2e-demo/sap-odata/workflow_demo/run.sh` 新規作成

内容:
- `#!/usr/bin/env bash` + `set -euo pipefail`
- `fav run ../pipeline_workflow.fav` を実行する最小限のスクリプト

### Step 3: `fav/src/driver.rs` に `mod v97600_tests` を追加

`mod v97500_tests` の直後に追加:

```rust
#[cfg(test)]
mod v97600_tests {
    #[test]
    fn workflow_demo_readme_exists() {
        let _ = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/workflow_demo/README.md",
        )
        .expect("infra/e2e-demo/sap-odata/workflow_demo/README.md should exist (v97.6.0)");
    }
    #[test]
    fn workflow_demo_run_sh_has_fav_run() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/workflow_demo/run.sh",
        )
        .expect("infra/e2e-demo/sap-odata/workflow_demo/run.sh should exist (v97.6.0)");
        assert!(
            content.contains("fav run"),
            "run.sh should invoke fav run"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

テスト数: 4,223 + 2 = 4,225

### Step 5: `CHANGELOG.md` に v97.6.0 エントリを追加

先頭に追加。

### Step 6: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v97.6.0` に更新
- 最新安定版を `v97.6.0 — 4,225 tests` に更新

### Step 7: CI 事前確認（Clippy / Self-fmt）

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
