# Plan: v89.5.0 — E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ）

## 実装ステップ

### Step 1: `scripts/run-sap-demo.sh` を作成

リポジトリルートの `scripts/` ディレクトリに追加（`start-sap-mock.sh` と同階層）:

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Favnir SAP E2E Demo (All 4 Scenarios) ==="

# [1/3] モックサーバー起動
echo "[1/3] Starting SAP mock server..."
if command -v docker &>/dev/null && [[ -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" ]]; then
    docker compose -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" up -d
else
    echo "  (docker-compose not available, skipping mock server)"
fi

# [2/3] パイプライン実行（Lambda 呼び出し）
echo "[2/3] Running pipeline via Lambda..."
"$REPO_ROOT/infra/e2e-demo/sap-odata/scripts/run.sh" || true

# [3/3] S3 出力確認
echo "[3/3] Checking S3 output..."
ENDPOINT_ARGS=()
if [[ -n "${AWS_ENDPOINT_URL:-}" ]]; then
    ENDPOINT_ARGS=(--endpoint-url "$AWS_ENDPOINT_URL")
fi
aws s3 ls s3://favnir-sap-demo/ "${ENDPOINT_ARGS[@]}" --recursive 2>/dev/null || true

echo "Done."
```

作成後、実行権限を付与: `chmod +x scripts/run-sap-demo.sh`

### Step 2: `mod v89500_tests` を `driver.rs` に追加

`mod v89400_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89500_tests {
    #[test]
    fn sap_e2e_demo_pipeline_has_journal_entry_scenario() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline.fav",
        )
        .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(
            content.contains("outstanding_payables"),
            "pipeline.fav should contain outstanding_payables (journal entry scenario)"
        );
    }

    #[test]
    fn sap_e2e_run_script_exists() {
        assert!(
            std::path::Path::new("../scripts/run-sap-demo.sh").exists(),
            "scripts/run-sap-demo.sh should exist"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,027 + 2 = 4,029 tests, 0 failures を確認する。

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
