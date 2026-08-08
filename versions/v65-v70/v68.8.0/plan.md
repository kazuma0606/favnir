# v68.8.0 実装計画

## Step 1: `fav/src/dist_otel.rs` 新規作成

```rust
// fav/src/dist_otel.rs — v68.8.0 Distributed Observability

pub fn cmd_dist_otel(src: &str, otel_endpoint: &str) -> String {
    // スタブ実装: 将来フェーズで実際の OTel Collector 送信・span 記録を実装
    format!(
        "[otel] Tracing enabled → {}\n\
         [otel] --otel-endpoint: {}\n\
         [trace] Pipeline: semantic-search-pipeline (trace_id: a3f2b1c9...)\n\
         [span] LoadDocs:      2ms   worker-1\n\
         [span] EmbedText[0]:  1240ms worker-1 | LLM: openai/text-embedding-3-small\n\
         [span] EmbedText[1]:  1238ms worker-2 | LLM: openai/text-embedding-3-small\n\
         [span] VectorStore:   45ms  worker-1  | VectorDB: pinecone/prod\n\
         [span] SemanticSearch: 23ms  worker-3 | VectorDB: pinecone/prod\n\
         [otel] Trace exported to Tempo. View: http://grafana:3000/d/favnir-ai (Grafana)\n\
         [stub] Would export trace to: {}",
        otel_endpoint, otel_endpoint, otel_endpoint
    )
}
```

出力に含まれるキーワード確認:
- `distributed_otel_trace` テスト:
  - `"--otel-endpoint"` ✓（行 2 の `[otel] --otel-endpoint:`）
  - `"trace_id"` ✓（行 3 の `trace_id: a3f2b1c9...`）
  - `"span"` ✓（行 4〜8 の `[span]`）
- `distributed_latency_breakdown` テスト:
  - `"LLM"` ✓（行 5〜6 の `| LLM:`）
  - `"VectorDB"` ✓（行 7〜8 の `| VectorDB:`）
  - `"Grafana"` ✓（行 9 の `Grafana`）

`format!` プレースホルダー確認:
- `{}` 計 3 個（otel_endpoint / otel_endpoint / otel_endpoint）、引数 3 個（otel_endpoint, otel_endpoint, otel_endpoint）— 一致

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod dist_otel;` を mod 宣言部に追加

`mod ai_routing;` の直後に追加。

```rust
mod dist_otel;
```

### 2b: `Some("run")` アームに `--otel-endpoint` ブランチを追加

挿入位置: `--distributed-cache` ブランチ（v68.5.0）の直後・`--env` ブランチの前

```rust
// ── v68.8.0: fav run --otel-endpoint <url> ───────────────────────────
// 注意: --checkpoint/--retry-policy/--distributed-cache と同時指定した場合は先行ブランチが優先される。
// otel_endpoint は http://... 等 '-' で始まらないためインデックスベースで除外する。
if args.iter().any(|a| a == "--otel-endpoint") {
    let otel_idx = args.iter().position(|a| a == "--otel-endpoint");
    let otel_endpoint = match otel_idx
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .filter(|v| !v.starts_with('-'))
    {
        Some(v) => v,
        None => {
            eprintln!("error: --otel-endpoint requires a URL (e.g. http://tempo:4317)");
            std::process::exit(1);
        }
    };
    let mut skip_indices = std::collections::HashSet::new();
    // any() チェック後のため otel_idx が None になることは非到達だが防衛的チェックとして残す
    if let Some(i) = otel_idx { skip_indices.insert(i + 1); }
    let src = args.iter().enumerate().skip(2)
        .find(|(i, a)| !a.starts_with('-') && !skip_indices.contains(i))
        .map(|(_, s)| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", dist_otel::cmd_dist_otel(src, otel_endpoint));
    return;
}
```

**注意**:
- `otel_endpoint`（例: `"http://tempo:4317"`）は `-` で始まらないため、インデックスベース除外を使用する
- URL 未指定時（値なし・`-` で始まる）は `eprintln!` + `process::exit(1)`（v68.5.0 [MED] 修正と同じパターン）
- 先行ブランチとの同時指定時は先行ブランチが優先される（コメントで明記）

## Step 3: `driver.rs` — `v68800_tests` 追加

挿入位置: `// -- v68700_tests (v68.7.0) -- Multi-Cloud AI Routing --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕）

```rust
// -- v68800_tests (v68.8.0) -- Distributed Observability --
#[cfg(test)]
mod v68800_tests {
    #[test]
    fn distributed_otel_trace() {
        let result = crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317");
        assert!(result.contains("--otel-endpoint"), "should output '--otel-endpoint'");
        assert!(result.contains("trace_id"), "should output 'trace_id'");
        assert!(result.contains("span"), "should output 'span'");
    }

    #[test]
    fn distributed_latency_breakdown() {
        let result = crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317");
        assert!(result.contains("LLM"), "should output 'LLM'");
        assert!(result.contains("VectorDB"), "should output 'VectorDB'");
        assert!(result.contains("Grafana"), "should output 'Grafana'");
    }
}
```

- `cargo build` でエラーなし（Step 3 完了後）

## 注意事項

- `--otel-endpoint` は `Some("run")` 内のブランチとして追加（新サブコマンドではない）
- URL 未指定時のエラー終了パターンは `--distributed-cache`（v68.5.0 修正済み）と同一にする
- インデックスベース src 除外は v68.6.0 以降の標準パターン（`HashSet` + `enumerate`）
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68800_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
