# v68.6.0 — Cost-Aware Scheduling

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

AI パイプラインの実行コストを実行前に見積もり、最適化提案を出す。
LLM API 呼び出し・ベクトル DB クエリ・コンピュートを統合してコスト計算する。
v68.6.0 はスタブ実装。実際のプロバイダー API 呼び出し・料金取得は将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/cost_estimate.rs` — 新規作成
  - `pub fn cmd_cost_estimate(src: &str, provider: &str, scale: &str) -> String`
    - `"Cost Estimate"` / `"TOTAL"` / `"--scale"` を含む出力（`cost_estimate_ai_pipeline` テスト要件）
    - `"Optimizations"` / `"バッチサイズ"` / `"-55%"` を含む出力（`cost_optimize_batch_size` テスト要件）
    - 出力末尾は `[stub] Would calculate costs for: <src>`（実際の計算なし）
- `fav/src/main.rs` — `mod cost_estimate;` 追加 + `Some("cost-estimate")` アーム追加
  - `fav cost-estimate pipeline.fav --provider aws --scale 1M-rows` 形式
  - `--provider <value>` と `--scale <value>` を解析
  - `src` 検出時は `provider` / `scale` の値を除外（誤検出防止）
  - `src` 省略時デフォルト: `"pipeline.fav"`
  - `provider` 省略時デフォルト: `"aws"`、`scale` 省略時デフォルト: `"1M-rows"`
- `fav/src/driver.rs` — `v68600_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.6.0 はスタブ実装のため将来フェーズとする。

- 実際のプロバイダー API 料金テーブル取得（OpenAI / Anthropic / Cohere）: 将来フェーズ
- ベクトル DB コスト計算（Pinecone / Weaviate / pgvector）: 将来フェーズ
- コンピュートコスト計算（AWS ECS / Lambda / GCP Cloud Run）: 将来フェーズ
- 最適化提案の実際のロジック（バッチサイズ自動調整・Spot 活用）: 将来フェーズ
- `--provider gcp` / `--provider azure` の実際の料金差分: 将来フェーズ

## コマンド設計

```
fav cost-estimate pipeline.fav --provider aws --scale 1M-rows
fav cost-estimate pipeline.fav --provider gcp --scale 500K-rows
fav cost-estimate pipeline.fav
```

- `Some("cost-estimate")` は新しいサブコマンドアーム（`Some("run")` とは別）
- `--provider <value>` の次引数を取得、省略時は `"aws"`
- `--scale <value>` の次引数を取得、省略時は `"1M-rows"`
- `src` 検出: `args[0]` = `"fav"`、`args[1]` = `"cost-estimate"` を `skip(2)` でスキップ後、`provider` / `scale` の値（`"aws"` / `"1M-rows"` 等）は `-` で始まらないため明示的に除外
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `cost_estimate_ai_pipeline` | `cmd_cost_estimate` が `"Cost Estimate"` / `"TOTAL"` / `"--scale"` を含む |
| `cost_optimize_batch_size` | `cmd_cost_estimate` が `"Optimizations"` / `"バッチサイズ"` / `"-55%"` を含む |

ベーステスト: 3529 → 目標: **3531**

> 各テストは `crate::cost_estimate::cmd_cost_estimate("pipeline.fav", "aws", "1M-rows")` を直接呼び出す。各キーワードは個別の `assert!` で検証する（失敗時の診断性確保）。
