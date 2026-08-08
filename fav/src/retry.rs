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
