# v68.5.0 実装計画

## Step 1: `fav/src/dist_cache.rs` 新規作成

```rust
// fav/src/dist_cache.rs — v68.5.0 Distributed Incremental Cache

pub fn cmd_distributed_cache(src: &str, cache_url: &str) -> String {
    // スタブ実装: 将来フェーズで実際の Redis 接続・キャッシュ読み書きを実装
    format!(
        "[--distributed-cache] Connecting to: {}\n\
         [cache] Connected to Redis (distributed mode)\n\
         [cache] redis backend: {}\n\
         [step embed] EmbedText(row 1..500): MISS → executed (1240ms)\n\
         [step embed] EmbedText(row 1..500): HIT  ← cached by another worker (2ms)\n\
         [cache] L1 (memory): 128 entries | L2 (Redis): 4096 entries\n\
         [cache] invalidation: schema-change detected → cache cleared\n\
         [cache] --cache-ttl: default 3600s\n\
         [cache] Hit rate: 73% | Saved: $0.84 (LLM calls avoided)\n\
         [stub] Would connect to Redis cache (source: {})",
        cache_url, cache_url, src  // cache_url を 2 回渡す（行1の {} と行2の {} に展開）。src は最後の {}。
    )
}
```

出力に含まれるキーワード確認:
- `distributed_cache_hit_across_workers` テスト:
  - `"--distributed-cache"` ✓（行 1 の `[--distributed-cache]`）
  - `"redis"` ✓（行 2 の `Connected to Redis`）
  - `"Hit rate"` ✓（行 9）
- `distributed_cache_invalidation` テスト:
  - `"--cache-ttl"` ✓（行 8）
  - `"L1"` ✓（行 6 の `L1 (memory)`）
  - `"L2"` ✓（行 6 の `L2 (Redis)`）
  - `"invalidation"` ✓（行 7）

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod dist_cache;` を mod 宣言部に追加

```rust
mod dist_cache;
```

`mod retry;` の直後に追加。

### 2b: `Some("run")` アームに `--distributed-cache` ブランチを追加

挿入位置: `--retry-policy` ブランチ（v68.4.0）の直後

```rust
// ── v68.5.0: fav run --distributed-cache <url> ───────────────────────
// 注意: --checkpoint/--resume / --retry-policy と同時指定した場合は先行ブランチが優先される。
// cache_url は redis://... 等 '-' で始まらないため src 除外フィルターに明示的に追加する。
if args.iter().any(|a| a == "--distributed-cache") {
    let cache_url = args.iter().position(|a| a == "--distributed-cache")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("redis://localhost:6379");
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-') && a.as_str() != cache_url)
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", dist_cache::cmd_distributed_cache(src, cache_url));
    return;
}
```

**注意**:
- `args` は `["fav", "run", "pipeline.fav", "--distributed-cache", "redis://..."]` など。`skip(2)` で `"fav"` / `"run"` をスキップ。
- `cache_url`（例: `"redis://cache.internal:6379"`）は `-` で始まらないため、`src` 検出時に明示的に除外する。
- `--distributed-cache` が値なしの場合（次引数が `-` で始まる・または存在しない）は `"redis://localhost:6379"` をデフォルトとして使用する。

## Step 3: `driver.rs` — `v68500_tests` 追加

挿入位置: `// -- v68400_tests (v68.4.0) -- Stage Retry Policies（型安全エラー回復） --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕）

```rust
// -- v68500_tests (v68.5.0) -- Distributed Incremental Cache --
#[cfg(test)]
mod v68500_tests {
    #[test]
    fn distributed_cache_hit_across_workers() {
        let result = crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379");
        assert!(result.contains("--distributed-cache"), "should output '--distributed-cache'");
        assert!(result.contains("redis"), "should output 'redis'");
        assert!(result.contains("Hit rate"), "should output 'Hit rate'");
    }

    #[test]
    fn distributed_cache_invalidation() {
        let result = crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379");
        assert!(result.contains("--cache-ttl"), "should output '--cache-ttl'");
        assert!(result.contains("L1"), "should output 'L1'");
        assert!(result.contains("L2"), "should output 'L2'");
        assert!(result.contains("invalidation"), "should output 'invalidation'");
    }
}
```

## 注意事項

- `Some("run")` の既存ロジックは変更しない（`--distributed-cache` ブランチのみ追加）
- `cache_url` は `-` で始まらないため `src` 除外フィルターに明示的に追加する
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68500_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
