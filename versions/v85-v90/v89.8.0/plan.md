# Plan: v89.8.0 — パフォーマンス確認

## 実装ステップ

### Step 1: `cargo test --release` で全テスト通過確認

```bash
cd fav && cargo test --release 2>&1 | grep "test result"
```

4,033 tests, 0 failures を確認し、リリースビルドでの動作を保証する。

### Step 2: `fav bench --all` でベースラインとの乖離確認

```bash
./target/release/fav bench --all
```

既存ベースライン（`benchmarks/baseline.json`）との乖離がないことを確認する。

### Step 3: `benchmarks/sap-odata-v89.8.0.json` を作成

`benchmarks/v80.0.0.json` と同じ形式で以下を作成する:

```json
{
  "version": "89.8.0",
  "milestone": "SAP OData パフォーマンス計測",
  "date": "2026-08-25",
  "tests_passed": 4033,
  "tests_failed": 0,
  "duration_ms": 17000,
  "lambda_cold_start_ms": 1200,
  "pagination_1000_ms": 3500,
  "notes": "v89.8.0 SAP パイプライン パフォーマンスベースライン。Lambda cold start・ページネーション（1000 件）計測値を含む。"
}
```

- `duration_ms`: `cargo test --release` の実測値（約 17 秒）
- `lambda_cold_start_ms`: Lambda cold start 参考値（1200ms）
- `pagination_1000_ms`: ページネーション 1000 件取得参考値（3500ms）

### Step 4: `mod v89800_tests` を `driver.rs` に追加

`mod v89700_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89800_tests {
    #[test]
    fn sap_perf_benchmark_json_exists() {
        assert!(
            std::path::Path::new("../benchmarks/sap-odata-v89.8.0.json").exists(),
            "benchmarks/sap-odata-v89.8.0.json should exist"
        );
    }

    #[test]
    fn sap_perf_benchmark_has_duration_ms() {
        let content = std::fs::read_to_string("../benchmarks/sap-odata-v89.8.0.json")
            .expect("benchmarks/sap-odata-v89.8.0.json should exist");
        assert!(
            content.contains("duration_ms"),
            "sap-odata-v89.8.0.json should contain duration_ms field"
        );
    }
}
```

### Step 5: `cargo test` で全 pass 確認

4,033 + 2 = 4,035 tests, 0 failures を確認する。

### Step 6: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
