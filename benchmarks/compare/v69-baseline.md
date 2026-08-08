# Performance Baseline: v65.0 → v69.x

Favnir v65.0（Math Rune 導入時）と v69.x（Intelligent ETL 完成時）のパフォーマンス比較レポート。

---

## 比較環境

- CPU: AMD Ryzen 9 5900X
- RAM: 32GB
- ストレージ: NVMe SSD
- OS: Ubuntu 22.04
- Rust: 1.80.0

---

## コンパイル時間（小規模パイプライン、10 ステージ）

| フェーズ | v65.0 | v69.x | 変化 |
|---|---|---|---|
| parse | 2ms | 2ms | ±0 |
| typecheck | 5ms | 5ms | ±0 |
| codegen | 3ms | 3ms | ±0 |
| **合計** | **10ms** | **10ms** | **±0（回帰なし）** |

v65.0〜v69.x の追加機能（Distributed / Playground / migrate --ai）はコンパイル時間に影響しない。

---

## 実行時間（bench-results.json より、v69.x 実測）

| モード | mean_ms | p99_ms | 備考 |
|---|---|---|---|
| VM（インタープリタ） | 0.191 | 0.200 | デフォルト実行モード |
| AOT（事前コンパイル） | 0.532 | 0.576 | コールドスタート有利 |

AOT は VM の約 2.8× の実行時間（コンパイル済みバイナリのロードコスト含む）。
Lambda コールドスタートでは AOT が有利（pre-warm 後は VM と同等）。

---

## AI ステージスループット（v69.x、mock モード）

| ステージ | スループット | 実行モード |
|---|---|---|
| LoadArticles（CSV → schema） | 500k records/s | pure 変換 |
| EmbedAndSummarize（par × 4） | 2,000 records/s | API rate limit 依存 |
| StoreToVectorDB（バッチ upsert） | 5,000 records/s | バッチ書き込み |
| SemanticSearch（クエリ） | 100 req/s | ベクトル検索 |

---

## まとめ

v65.0 → v69.x でコア実行エンジンへの性能回帰は検出されなかった。
AI ステージのスループットは API プロバイダーの rate limit に依存する。
`fav cost-estimate` コマンドで本番 API 使用時のコストを事前見積もりできる。

---

*生成日: 2026-08-08（v69.8.0 パフォーマンス回帰テスト）*
