# v68.7.0 実装計画

## Step 1: `fav/src/ai_routing.rs` 新規作成

```rust
// fav/src/ai_routing.rs — v68.7.0 Multi-Cloud AI Routing

pub fn cmd_ai_routing(src: &str, env: &str) -> String {
    // スタブ実装: 将来フェーズで toml.rs [ai] セクションパース・プロバイダー抽象化を実装
    format!(
        "[ai] Routing config loaded from fav.toml [ai] section\n\
         [ai] --env: {} | source: {}\n\
         [ai] llm_provider: anthropic (prod) / ollama-local (dev) / mock (test)\n\
         [ai] embed_provider: openai (prod) / ollama-local (dev) / mock (test)\n\
         [ai] vector_db: pinecone (prod) / qdrant-local (dev) / in-memory (test)\n\
         [routing] Applying {} profile: dev → ollama-local, test → mock, prod → anthropic\n\
         [stub] Would apply AI routing (source: {})",
        env, src, env, src
    )
}
```

出力に含まれるキーワード確認:
- `multi_cloud_ai_routing` テスト:
  - `"[ai]"` ✓（行 1 の `[ai] Routing config`）
  - `"llm_provider"` ✓（行 3 の `[ai] llm_provider:`）
  - `"--env"` ✓（行 2 の `[ai] --env:`）
- `ai_provider_local_fallback` テスト:
  - `"ollama-local"` ✓（行 3・4 の `ollama-local (dev)`）
  - `"mock"` ✓（行 3・4 の `mock (test)`）
  - `"in-memory"` ✓（行 5 の `in-memory (test)`）

`format!` プレースホルダー確認:
- `{}` 計 4 個（env / src / env / src）、引数 4 個（env, src, env, src）— 一致

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod ai_routing;` を mod 宣言部に追加

`mod cost_estimate;` の直後に追加。

```rust
mod ai_routing;
```

### 2b: `Some("ai-routing")` アームを追加

挿入位置: 既存サブコマンドアーム群（`Some("cost-estimate")` の近く）に追記。

```rust
Some("ai-routing") => {
    // ── v68.7.0: fav ai-routing <src> --env <dev|prod|test> ──
    // args[0]="fav", args[1]="ai-routing" を skip(2) でスキップ
    // env 値（"dev"/"test"/"prod" 等）は "-" で始まらないためインデックスベースで除外する
    let env_idx = args.iter().position(|a| a == "--env");
    let env = env_idx
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("prod");
    let mut skip_indices = std::collections::HashSet::new();
    if let Some(i) = env_idx { skip_indices.insert(i + 1); }
    let src = args.iter().enumerate().skip(2)
        .find(|(i, a)| !a.starts_with('-') && !skip_indices.contains(i))
        .map(|(_, s)| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", ai_routing::cmd_ai_routing(src, env));
}
```

**注意**:
- `env` 値（`"dev"`, `"test"`, `"prod"`）は `-` で始まらないためインデックスベース除外を使用（v68.6.0 の [MED] 修正と同じパターン）
- `Some("run")` 内の既存 `--env` ブランチとは別アーム（競合なし）
- 省略時デフォルト: `env = "prod"`, `src = "pipeline.fav"`

## Step 3: `driver.rs` — `v68700_tests` 追加

挿入位置: `// -- v68600_tests (v68.6.0) -- Cost-Aware Scheduling --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕）

```rust
// -- v68700_tests (v68.7.0) -- Multi-Cloud AI Routing --
#[cfg(test)]
mod v68700_tests {
    #[test]
    fn multi_cloud_ai_routing() {
        let result = crate::ai_routing::cmd_ai_routing("pipeline.fav", "dev");
        assert!(result.contains("[ai]"), "should output '[ai]'");
        assert!(result.contains("llm_provider"), "should output 'llm_provider'");
        assert!(result.contains("--env"), "should output '--env'");
    }

    #[test]
    fn ai_provider_local_fallback() {
        let result = crate::ai_routing::cmd_ai_routing("pipeline.fav", "dev");
        assert!(result.contains("ollama-local"), "should output 'ollama-local'");
        assert!(result.contains("mock"), "should output 'mock'");
        assert!(result.contains("in-memory"), "should output 'in-memory'");
    }
}
```

- `cargo build` でエラーなし（Step 3 完了後）

## 注意事項

- `Some("ai-routing")` は `Some("run")` 内ではなく、トップレベルのサブコマンドアームとして追加する
- `--env` 値のインデックスベース除外は v68.6.0 の `HashSet` パターンを踏襲する
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68700_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
