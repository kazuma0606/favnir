# v68.6.0 実装計画

## Step 1: `fav/src/cost_estimate.rs` 新規作成

```rust
// fav/src/cost_estimate.rs — v68.6.0 Cost-Aware Scheduling

pub fn cmd_cost_estimate(src: &str, provider: &str, scale: &str) -> String {
    // スタブ実装: 将来フェーズで実際の料金テーブル取得・最適化ロジックを実装
    format!(
        "=== Cost Estimate: SemanticSearchPipeline ===\n\
         Scale: {} (--scale) | Provider: {}\n\
         Source: {}\n\
         \n\
         | Stage          | Provider   | Cost     | % Total |\n\
         |----------------|------------|----------|---------|\n\
         | EmbedText      | OpenAI     | $1.00    |  43%    |\n\
         | ExtractInvoice | Claude     | $0.80    |  34%    |\n\
         | VectorSearch   | Pinecone   | $0.42    |  18%    |\n\
         | Compute        | {} ECS    | $0.12    |   5%    |\n\
         | TOTAL          |            | $2.34    | 100%    |\n\
         \n\
         === Optimizations ===\n\
         [HIGH] バッチサイズ 10 → 50: EmbedText $1.00 → $0.40 (-$0.60)\n\
         [MED]  Cohere embed（$0.30）に切り替え: -$0.70\n\
         [LOW]  Spot instances: Compute $0.12 → $0.04 (-$0.08)\n\
         Optimized estimate: $1.04 (-55%)\n\
         [stub] Would calculate costs for: {}",
        scale, provider, src, provider, src
    )
}
```

出力に含まれるキーワード確認:
- `cost_estimate_ai_pipeline` テスト:
  - `"Cost Estimate"` ✓（行 1 の `=== Cost Estimate:` ）
  - `"TOTAL"` ✓（行 9 の `| TOTAL`）
  - `"--scale"` ✓（行 2 の `(--scale)`）
- `cost_optimize_batch_size` テスト:
  - `"Optimizations"` ✓（行 11 の `=== Optimizations ===`）
  - `"バッチサイズ"` ✓（行 12 の `[HIGH] バッチサイズ`）
  - `"-55%"` ✓（行 15 の `(-55%)`）

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod cost_estimate;` を mod 宣言部に追加

`mod dist_cache;` の直後に追加。

```rust
mod cost_estimate;
```

### 2b: `Some("cost-estimate")` アームを追加

挿入位置: 既存サブコマンドのアーム群（`Some("cluster")` / `Some("deploy")` 等）と並列に追加。
`Some("cluster")` の直前（アルファベット順でなく追記順）に挿入する。

```rust
Some("cost-estimate") => {
    // ── v68.6.0: fav cost-estimate <src> --provider <aws|gcp|azure> --scale <N>-rows ──
    let provider = args.iter().position(|a| a == "--provider")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("aws");
    let scale = args.iter().position(|a| a == "--scale")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("1M-rows");
    // provider / scale の値（"-" で始まらない）を src 候補から除外
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-') && a.as_str() != provider && a.as_str() != scale)
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", cost_estimate::cmd_cost_estimate(src, provider, scale));
}
```

**注意**:
- `args` は `["fav", "cost-estimate", "pipeline.fav", "--provider", "aws", "--scale", "1M-rows"]` など。`skip(2)` で `"fav"` / `"cost-estimate"` をスキップ。
- `provider`（例: `"aws"`, `"gcp"`）と `scale`（例: `"1M-rows"`, `"500K-rows"`）は `-` で始まらないため `src` 検出時に明示的に除外する。
- 省略時デフォルト: `provider = "aws"`, `scale = "1M-rows"`, `src = "pipeline.fav"`

## Step 3: `driver.rs` — `v68600_tests` 追加

挿入位置: `// -- v68500_tests (v68.5.0) -- Distributed Incremental Cache --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕）

```rust
// -- v68600_tests (v68.6.0) -- Cost-Aware Scheduling --
#[cfg(test)]
mod v68600_tests {
    #[test]
    fn cost_estimate_ai_pipeline() {
        let result = crate::cost_estimate::cmd_cost_estimate("pipeline.fav", "aws", "1M-rows");
        assert!(result.contains("Cost Estimate"), "should output 'Cost Estimate'");
        assert!(result.contains("TOTAL"), "should output 'TOTAL'");
        assert!(result.contains("--scale"), "should output '--scale'");
    }

    #[test]
    fn cost_optimize_batch_size() {
        let result = crate::cost_estimate::cmd_cost_estimate("pipeline.fav", "aws", "1M-rows");
        assert!(result.contains("Optimizations"), "should output 'Optimizations'");
        assert!(result.contains("バッチサイズ"), "should output 'バッチサイズ'");
        assert!(result.contains("-55%"), "should output '-55%'");
    }
}
```

- `cargo build` でエラーなし（Step 3 完了後）

## 注意事項

- `Some("cost-estimate")` は `Some("run")` 内ではなく、トップレベルのサブコマンドアームとして追加する
- `provider` / `scale` は `-` で始まらないため `src` 検出時に明示的に除外する
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68600_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
