# v68.8.0 — Distributed Observability

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

分散実行中のパイプラインを OpenTelemetry でエンドツーエンドトレースする。
LLM 呼び出し・ベクトル DB クエリのレイテンシを統合ダッシュボードで可視化する。
v68.8.0 はスタブ実装。実際の OTel Collector 送信・Grafana ダッシュボード書き込みは将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/dist_otel.rs` — 新規作成
  - `pub fn cmd_dist_otel(src: &str, otel_endpoint: &str) -> String`
    - `"--otel-endpoint"` / `"trace_id"` / `"span"` を含む出力（`distributed_otel_trace` テスト要件）
    - `"LLM"` / `"VectorDB"` / `"Grafana"` を含む出力（`distributed_latency_breakdown` テスト要件）
    - 注意: ロードマップ出力例（行 357）は `| DB: pinecone/prod` と省略しているが、テストキーワードは `"VectorDB"` であるため実装は `| VectorDB: pinecone/prod` とする
    - 出力末尾は `[stub] Would export trace to: <otel_endpoint>`（実際の送信なし）
- `fav/src/main.rs` — `mod dist_otel;` 追加 + `Some("run")` アームに `--otel-endpoint` ブランチ追加
  - `--otel-endpoint <url>` フラグが存在する場合に `cmd_dist_otel(src, otel_endpoint)` を呼び出して `return`
  - `otel_endpoint` は `--otel-endpoint` の直後の引数から取得。値がない・`-` で始まる場合はエラー終了
  - `src` 検出時は `otel_endpoint` の値をインデックスベースで除外（誤検出防止）
    - `args[0]="fav"`、`args[1]="run"` を `skip(2)` でスキップ
  - `src` 省略時デフォルト: `"pipeline.fav"`
  - 挿入位置: `--distributed-cache` ブランチの直後・`--env` ブランチの前
  - `--checkpoint` / `--retry-policy` / `--distributed-cache` と同時指定した場合は先行ブランチが優先される（ブランチ順による暫定仕様）
- `fav/src/driver.rs` — `v68800_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.8.0 はスタブ実装のため将来フェーズとする（v69.0.0 以降でロードマップ追加予定）。

- 実際の OTel Collector への trace 送信: 将来フェーズ
- 分散トレース: 各ステージを span として記録（parent/child 関係）: 将来フェーズ
- LLM span: モデル名・プロンプトトークン数・コスト・レイテンシ: 将来フェーズ
- VectorDB span: インデックス名・クエリ次元・top_k・レイテンシ: 将来フェーズ
- Grafana ダッシュボード定義（`infra/monitoring/favnir-ai-dashboard.json`）: 将来フェーズ
- Prometheus メトリクス統合（既存 v29.x の OTel Rune との連携）: 将来フェーズ

## コマンド設計

```
fav run pipeline.fav --otel-endpoint http://tempo:4317
fav run pipeline.fav --cluster workers.yaml --otel-endpoint http://tempo:4317
```

- `--otel-endpoint <url>` は `Some("run")` の `--distributed-cache` ブランチの直後に挿入
- `otel_endpoint` は `--otel-endpoint` の直後の引数から取得。値がない・`-` で始まる場合は `eprintln!` + `process::exit(1)`（他の値必須フラグと挙動を統一）
- `src` 検出: `--otel-endpoint` フラグ値インデックス（i+1）を `HashSet` に収集してインデックスベースで除外
- `src` 省略時デフォルト: `"pipeline.fav"`
- `--checkpoint` / `--retry-policy` / `--distributed-cache` と同時指定した場合は先行ブランチが優先される

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `distributed_otel_trace` | `cmd_dist_otel` が `"--otel-endpoint"` / `"trace_id"` / `"span"` を含む |
| `distributed_latency_breakdown` | `cmd_dist_otel` が `"LLM"` / `"VectorDB"` / `"Grafana"` を含む |

ベーステスト: 3533 → 目標: **3535**

> 各テストは `crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317")` を直接呼び出す。各キーワードは個別の `assert!` で検証する（失敗時の診断性確保）。
