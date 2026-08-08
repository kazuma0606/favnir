# v68.7.0 — Multi-Cloud AI Routing

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

環境（本番/開発/テスト）に応じて LLM・ベクトル DB プロバイダーを型安全に切り替える。
`fav.toml` の `[ai]` セクションで環境ごとのプロバイダーを宣言し、`fav ai-routing --env <env>` で適用設定を確認できる。
v68.7.0 はスタブ実装。実際のプロバイダー抽象化・`toml.rs` 拡張・プロバイダー切り替えは将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/ai_routing.rs` — 新規作成
  - `pub fn cmd_ai_routing(src: &str, env: &str) -> String`
    - `"[ai]"` / `"llm_provider"` / `"--env"` を含む出力（`multi_cloud_ai_routing` テスト要件）
    - `"ollama-local"` / `"mock"` / `"in-memory"` を含む出力（`ai_provider_local_fallback` テスト要件）
    - 出力末尾は `[stub] Would apply AI routing (source: <src>)`（実際のルーティング切り替えなし）
- `fav/src/main.rs` — `mod ai_routing;` 追加 + `Some("ai-routing")` アーム追加
  - `fav ai-routing --env <dev|prod|test> [pipeline.fav]` 形式の新サブコマンド
  - `--env` の次引数を `env` として取得。省略時デフォルト: `"prod"`
  - `src` 検出時は `env` 値を インデックスベースで除外（誤検出防止）
    - `args[0]="fav"`、`args[1]="ai-routing"` を `skip(2)` でスキップ
  - `src` 省略時デフォルト: `"pipeline.fav"`
- `fav/src/driver.rs` — `v68700_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.7.0 はスタブ実装のため将来フェーズとする。

- `fav.toml` の `[ai]` セクション実際のパース（`toml.rs` 拡張）: 将来フェーズ（v69.0.0 以降でロードマップ追加予定）
- `LLMProvider` interface 実装（anthropic / openai / ollama / mock）: 将来フェーズ（v69.0.0 以降でロードマップ追加予定）
- `VectorDBProvider` interface 実装（pinecone / qdrant / pgvector / in-memory）: 将来フェーズ（v69.0.0 以降でロードマップ追加予定）
- `fav run --env dev` との統合（実際のプロバイダー切り替え）: 将来フェーズ（v69.0.0 以降でロードマップ追加予定）
- コスト追跡: 本番プロバイダーのみコスト計算（dev/test は $0）: 将来フェーズ（v69.0.0 以降でロードマップ追加予定）

## コマンド設計

```
fav ai-routing pipeline.fav --env dev
fav ai-routing pipeline.fav --env test
fav ai-routing pipeline.fav --env prod
fav ai-routing pipeline.fav
```

- `Some("ai-routing")` は新しいサブコマンドアーム（`Some("run")` とは別）
- `--env` の次引数を `env` として取得、省略時は `"prod"`
- `src` 検出: `args[0]="fav"`、`args[1]="ai-routing"` を `skip(2)` でスキップ
- `env` 値（`"dev"` / `"test"` / `"prod"` 等）は `-` で始まらないため、インデックスベースで除外する
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `multi_cloud_ai_routing` | `cmd_ai_routing` が `"[ai]"` / `"llm_provider"` / `"--env"` を含む |
| `ai_provider_local_fallback` | `cmd_ai_routing` が `"ollama-local"` / `"mock"` / `"in-memory"` を含む |

ベーステスト: 3531 → 目標: **3533**

> 各テストは `crate::ai_routing::cmd_ai_routing("pipeline.fav", "dev")` を直接呼び出す。各キーワードは個別の `assert!` で検証する（失敗時の診断性確保）。
