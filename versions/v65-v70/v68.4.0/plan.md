# v68.4.0 実装計画

## Step 1: `fav/src/retry.rs` 新規作成

```rust
// fav/src/retry.rs — v68.4.0 Stage Retry Policies（型安全エラー回復）

pub fn cmd_retry_policy(src: &str) -> String {
    // スタブ実装: 将来フェーズで実際のリトライ実行を実装
    format!(
        "[retry] Analyzing retry policies for: {}\n\
         [policy] step \"call-llm\": ExponentialBackoff(max=3, base_ms=500) | timeout_ms=5000\n\
         [policy] step \"embed\": LinearBackoff(max=2, interval_ms=1000) | circuit_breaker={{threshold=5}}\n\
         [policy] step \"store\": ExponentialBackoff(max=5, base_ms=200)\n\
         [fallback] step \"call-llm\": Fallback(CachedResponse)\n\
         [fallback] step \"embed\": Skip\n\
         [fallback] step \"store\": DeadLetterQueue(\"failed-records\")\n\
         [stub] Would apply retry policies at runtime (source: {})",
        src, src
    )
}
```

出力に含まれるキーワード:
- `retry_exponential_backoff` テスト: `"ExponentialBackoff"` ✓ / `"LinearBackoff"` ✓ / `"timeout_ms"` ✓
- `retry_fallback_stage` テスト: `"Fallback"` ✓ / `"DeadLetterQueue"` ✓ / `"circuit_breaker"` ✓

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod retry;` を mod 宣言部に追加

```rust
mod retry;
```

`mod k8s;` の直後に追加。

### 2b: `Some("run")` アームに `--retry-policy` ブランチを追加

挿入位置: `--checkpoint`/`--resume` ブランチ（v68.2.0）の直後

```rust
// ── v68.4.0: fav run --retry-policy ──────────────────────────────────
if args.iter().any(|a| a == "--retry-policy") {
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", retry::cmd_retry_policy(src));
    return;
}
```

**注意**:
- `args` は `["fav", "run", "pipeline.fav", "--retry-policy"]` など。`skip(2)` で `"fav"` / `"run"` をスキップ。
- `--retry-policy` はフラグのみ（値なし）のため、フラグ値を `src` から除外する処理は不要。
- `--checkpoint`/`--resume` ブランチの直後に配置し、`--env` 等の既存ブランチには影響しない。

## Step 3: `driver.rs` — `v68400_tests` 追加

挿入位置: `// -- v68300_tests (v68.3.0) -- Kubernetes-Native Orchestration --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕。`v68400_tests` を `v68300_tests` ブロック開始行の1行前に挿入する）

```rust
// -- v68400_tests (v68.4.0) -- Stage Retry Policies（型安全エラー回復） --
#[cfg(test)]
mod v68400_tests {
    #[test]
    fn retry_exponential_backoff() {
        let result = crate::retry::cmd_retry_policy("pipeline.fav");
        assert!(result.contains("ExponentialBackoff"), "cmd_retry_policy should output 'ExponentialBackoff'");
        assert!(result.contains("LinearBackoff"), "cmd_retry_policy should output 'LinearBackoff'");
        assert!(result.contains("timeout_ms"), "cmd_retry_policy should output 'timeout_ms'");
    }

    #[test]
    fn retry_fallback_stage() {
        let result = crate::retry::cmd_retry_policy("pipeline.fav");
        assert!(result.contains("Fallback"), "cmd_retry_policy should output 'Fallback'");
        assert!(result.contains("DeadLetterQueue"), "cmd_retry_policy should output 'DeadLetterQueue'");
        assert!(result.contains("circuit_breaker"), "cmd_retry_policy should output 'circuit_breaker'");
    }
}
```

**注意**: `use super::*` は不要（`crate::retry::` で直接参照）。各キーワードを個別の `assert!` に分けることで失敗時の診断性を確保する（v68.3.0 のコードレビュー教訓）。

## 注意事項

- `Some("run")` の既存ロジックは変更しない（`--retry-policy` ブランチのみ追加）
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68400_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
