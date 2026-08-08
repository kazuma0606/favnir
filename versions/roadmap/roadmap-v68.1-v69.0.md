# Roadmap v68.1.0 〜 v69.0.0 — Distributed Favnir

Date: 2026-08-04
Status: 未着手（v68.0.0 完了後に開始）

マスターロードマップ: [roadmap-v65.1-v70.0.md](roadmap-v65.1-v70.0.md)

---

## 前提

- 直前完了: v68.0.0「Developer Intelligence」（tests = 3519）
- 本スプリントは Phase 4「Distributed Favnir」の詳細計画
- 目標: v69.0.0「Distributed Favnir 宣言」（tests = 3541）

### 設計方針

**型安全な分散実行**: `par` キーワードを単一マシンからクラスタに拡張する。
ただし型システムは変わらない——分散していても型エラーは起きない。

**耐障害性の原則**: `--checkpoint` でステージを越えた状態を保存し、
失敗時は「失敗したステップから」再開する。最初からやり直しは不要。

**コスト透明性**: AI パイプラインの LLM 呼び出しコストを実行前に見積もる。
コスト超過の可能性を `fav cost-estimate` で事前に把握する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v68.1.0 | Multi-Node `par`（分散並列実行） | 3519 + 2 = 3521 | 完了 |
| v68.2.0 | Pipeline Checkpointing（耐障害性・再開） | 3521 + 2 = 3523 | 完了 |
| v68.3.0 | Kubernetes-Native Orchestration | 3523 + 2 = 3525 | 完了 |
| v68.4.0 | Stage Retry Policies（型安全エラー回復） | 3525 + 2 = 3527 | 完了 |
| v68.5.0 | Distributed Incremental Cache | 3527 + 2 = 3529 | 完了 |
| v68.6.0 | Cost-Aware Scheduling | 3529 + 2 = 3531 | 完了 |
| v68.7.0 | Multi-Cloud AI Routing | 3531 + 2 = 3533 | 完了 |
| v68.8.0 | Distributed Observability | 3533 + 2 = 3535 | 完了 |
| v68.9.0 | 安定化・コードフリーズ | 3535 + 2 = 3537 | 完了 |
| v69.0.0 | Distributed Favnir 宣言 ★クリーンアップ | 3537 + 4 = 3541 | 完了 |

---

## v68.1.0 — Multi-Node `par`（分散並列実行）

**概要**: `par` キーワードを単一マシンの並列実行からマルチノードクラスタに拡張。
`--cluster workers.yaml` で複数ワーカーに作業を分散する。

```favnir
// par が単一マシン → 複数ノードに拡張（シンタックスは変わらない）
pipeline DistributedEmbedding {
    step "load"   = seq   LoadDocs
    step "embed"  = par   [EmbedText, EmbedText, EmbedText, EmbedText] after "load"
    step "store"  = seq   VectorStore after "embed"
}
```

```bash
# workers.yaml で分散先を定義
$ fav run pipeline.fav --cluster workers.yaml --partition-by "row_id % 4"
[cluster] 4 workers: worker-1(192.168.1.10), worker-2(.11), worker-3(.12), worker-4(.13)
[step embed] Distributing 10000 rows across 4 workers...
[step embed] worker-1: 2500 rows ✓ (1240ms)
[step embed] worker-2: 2500 rows ✓ (1238ms)
[step embed] worker-3: 2500 rows ✓ (1245ms)
[step embed] worker-4: 2500 rows ✓ (1241ms)
[step embed] Merge: 10000 embeddings ✓
```

**実装内容**:

- `--cluster <workers.yaml>` フラグ — クラスタワーカー定義
- `workers.yaml` フォーマット: `{ workers: [{ host, port, cores }] }`
- パーティション戦略: `--partition-by <expr>`（行単位、ハッシュ、範囲）
- ワーカー間データ転送（シリアライズ/デシリアライズ）
- ワーカー障害時の自動リバランス（残ワーカーに再分配）
- `--cluster-monitor` フラグ — 各ワーカーの進捗をリアルタイム表示

**完了条件**: Rust テスト 2 件（3519 + 2 = **3521**）

```rust
// driver.rs mod v68100_tests
fn distributed_par_multi_node()    // --cluster / workers.yaml / "--partition-by" キーワードを含む
fn distributed_work_rebalance()    // 自動リバランス / "--cluster-monitor" キーワードを含む
```

---

## v68.2.0 — Pipeline Checkpointing（耐障害性・再開）

**概要**: パイプライン実行状態を checkpoint ファイルに保存し、失敗したステップから再開。
長時間の AI パイプライン（大規模埋め込み、LLM バッチ処理）で特に有効。

```bash
# チェックポイント付き実行
$ fav run pipeline.fav --checkpoint ./checkpoints/
[checkpoint] Saving state after each stage to ./checkpoints/
[step 1/4] LoadCsv     ✓ → ./checkpoints/step-1-loadcsv.ckpt
[step 2/4] EmbedText   ✓ → ./checkpoints/step-2-embedtext.ckpt
[step 3/4] Validate    FAILED (network timeout)
$ # 修正後に再開
$ fav run pipeline.fav --resume ./checkpoints/step-2-embedtext.ckpt
[resume] Resuming from step 3 (Validate) — skipping step 1, 2
[step 3/4] Validate    ✓ (retry成功)
[step 4/4] InsertDB    ✓
```

**実装内容**:

- `--checkpoint <dir>` フラグ — チェックポイント保存先ディレクトリ
- `--resume <file>` フラグ — チェックポイントから再開
- チェックポイントフォーマット: `.ckpt`（バイナリ、ステージ出力をシリアライズ）
- 部分的再開: 完了済みステージをスキップ
- TTL: `--checkpoint-ttl <hours>`（古いチェックポイントの自動削除）
- 整合性検証: チェックポイントのハッシュチェック（破損検出）

**完了条件**: Rust テスト 2 件（3521 + 2 = **3523**）

```rust
// driver.rs mod v68200_tests
fn checkpoint_save_restore()          // --checkpoint / ".ckpt" / "--resume" キーワードを含む
fn checkpoint_resume_mid_pipeline()   // "Resuming from step" / "--checkpoint-ttl" を含む
```

---

## v68.3.0 — Kubernetes-Native Orchestration

**概要**: Favnir パイプラインを Kubernetes で実行するためのマニフェスト生成。
`fav deploy --target kubernetes` で K8s CRD を自動生成。GPU ステージのリソース指定も対応。

```bash
$ fav deploy --target kubernetes pipeline.fav
[generate] Kubernetes manifests → ./k8s/

$ cat ./k8s/pipeline-semantic-search.yaml
```

```yaml
apiVersion: favnir.dev/v1
kind: Pipeline
metadata:
  name: semantic-search
  namespace: data-platform
spec:
  stages:
    - name: load
      image: favnir/runtime:68.0.0
      replicas: 1
    - name: embed
      image: favnir/runtime:68.0.0
      replicas: 4
      resources:
        requests: { memory: "2Gi", cpu: "2" }
        limits:   { memory: "4Gi", gpu: "1" }
    - name: store
      image: favnir/runtime:68.0.0
      replicas: 2
  checkpointing:
    enabled: true
    storageClass: standard
```

**実装内容**（v68.3.0 はスタブ実装。★ 印は将来フェーズ）:

- `cmd_deploy(src, target)` — `--target kubernetes` サポート（スタブ: 文字列出力のみ）
- K8s CRD 生成: `Pipeline` kind（`apiVersion: favnir.dev/v1`）（スタブ: ファイル書き込みなし）
- ★ ステージ別 replicas 設定（`par` ステージは並列数を replicas に変換）: 将来フェーズ
- ★ GPU リソース指定: `with { gpu: 1 }` → K8s `resources.limits`: 将来フェーズ
- ★ Helm チャート生成: `fav deploy --target kubernetes --helm`: 将来フェーズ
- ★ Argo Workflows 対応: `fav deploy --target argo`: 将来フェーズ

**完了条件**: Rust テスト 2 件（3523 + 2 = **3525**）

```rust
// driver.rs mod v68300_tests
fn k8s_pipeline_manifest_gen()  // "apiVersion: favnir.dev/v1" / "kind: Pipeline" を含む
fn k8s_stage_replicas()         // "replicas" / "resources" / "--target kubernetes" を含む
```

---

## v68.4.0 — Stage Retry Policies（型安全エラー回復）

**概要**: ステージレベルで型安全なリトライ・フォールバックポリシーを設定する。
LLM 呼び出しのタイムアウト・レート制限への対処を宣言的に記述する。

```favnir
pipeline ResilientPipeline {
    step "call-llm" = seq CallLLM with {
        retry: ExponentialBackoff(max: 3, base_ms: 500, max_ms: 10000),
        on_failure: Fallback(CachedResponse),
        timeout_ms: 5000
    }
    step "embed" = seq EmbedText with {
        retry: LinearBackoff(max: 2, interval_ms: 1000),
        on_failure: Skip,
        circuit_breaker: { threshold: 5, window_ms: 60000 }
    }
    step "store" = seq InsertDB with {
        retry: ExponentialBackoff(max: 5, base_ms: 200),
        on_failure: DeadLetterQueue("failed-records")
    }
}
```

**実装内容**:

- リトライポリシー: `ExponentialBackoff`, `LinearBackoff`, `FixedDelay`
- フォールバック: `Fallback(stage)`, `Skip`, `DeadLetterQueue(queue_name)`
- タイムアウト: `timeout_ms` — ステージレベルのタイムアウト
- サーキットブレーカー: `circuit_breaker: { threshold, window_ms }`（連続失敗でオープン）
- `with { ... }` 構文を parser / checker で正式サポート

**完了条件**: Rust テスト 2 件（3525 + 2 = **3527**）

```rust
// driver.rs mod v68400_tests
fn retry_exponential_backoff()  // "ExponentialBackoff" / "LinearBackoff" / "timeout_ms" を含む
fn retry_fallback_stage()       // "Fallback" / "DeadLetterQueue" / "circuit_breaker" を含む
```

---

## v68.5.0 — Distributed Incremental Cache

**概要**: 複数ワーカー間でコンパイルキャッシュ・ステージ実行キャッシュを共有。
同一入力の同一ステージは 2 回実行しない。コスト削減に直結。

```bash
$ fav run pipeline.fav --distributed-cache redis://cache.internal:6379
[cache] Connected to Redis (distributed mode)
[step embed] EmbedText(row 1..500): MISS → executed (1240ms)
[step embed] EmbedText(row 1..500): HIT  ← 別ワーカーのキャッシュを再利用 (2ms)
[cache] Hit rate: 73% | Saved: $0.84 (LLM calls avoided)
```

**実装内容**:

- `--distributed-cache <redis_url>` フラグ — Redis バックエンドの分散キャッシュ
- キャッシュキー: ステージ名 + 入力のハッシュ（SHA256）
- TTL 設定: `--cache-ttl <seconds>` / `--cache-ttl-per-stage <stage>=<seconds>`
- キャッシュ無効化: 入力スキーマ変更時に自動無効化
- コスト追跡: LLM 呼び出し回避による節約額を表示
- ローカルキャッシュとの併用: L1（メモリ） → L2（Redis）の 2 層キャッシュ

**完了条件**: Rust テスト 2 件（3527 + 2 = **3529**）

```rust
// driver.rs mod v68500_tests
fn distributed_cache_hit_across_workers()  // "--distributed-cache" / "redis" / "Hit rate" を含む
fn distributed_cache_invalidation()        // "--cache-ttl" / "L1" / "L2" / "invalidation" を含む
```

---

## v68.6.0 — Cost-Aware Scheduling（AI パイプラインコスト最適化）

**概要**: AI パイプラインの実行コストを実行前に見積もり、最適化提案を出す。
LLM API 呼び出し・ベクトル DB クエリ・コンピュートを統合してコスト計算。

```bash
$ fav cost-estimate pipeline.fav --provider aws --scale 1M-rows

=== Cost Estimate: SemanticSearchPipeline ===
Scale: 1,000,000 rows

| Stage        | Provider   | Cost     | % Total |
|--------------|------------|----------|---------|
| EmbedText    | OpenAI     | $1.00    |  43%    |
| ExtractInvoice| Claude    | $0.80    |  34%    |
| VectorSearch | Pinecone   | $0.42    |  18%    |
| Compute      | AWS ECS    | $0.12    |   5%    |
| TOTAL        |            | $2.34    | 100%    |

=== Optimizations ===
[HIGH] バッチサイズ 10 → 50: EmbedText $1.00 → $0.40 (-$0.60)
[MED]  Cohere embed（$0.30）に切り替え: -$0.70
[LOW]  Spot instances: Compute $0.12 → $0.04 (-$0.08)
Optimized estimate: $1.04 (-55%)
```

**実装内容**:

- `fav cost-estimate <pipeline.fav> --provider <aws|gcp|azure> --scale <N>-rows`
- コスト計算: LLM API 料金テーブル（OpenAI / Anthropic / Cohere）
- ベクトル DB コスト: Pinecone / Weaviate / pgvector の料金
- コンピュートコスト: AWS ECS / Lambda / GCP Cloud Run
- 最適化提案: バッチサイズ・モデル切り替え・Spot 活用

**完了条件**: Rust テスト 2 件（3529 + 2 = **3531**）

```rust
// driver.rs mod v68600_tests
fn cost_estimate_ai_pipeline()  // "Cost Estimate" / "TOTAL" / "--scale" キーワードを含む
fn cost_optimize_batch_size()   // "Optimizations" / "バッチサイズ" / "-55%" 的な削減率を含む
```

---

## v68.7.0 — Multi-Cloud AI Routing（LLM/VectorDB プロバイダー切り替え）

**概要**: 環境（本番/開発/テスト）に応じて LLM・ベクトル DB プロバイダーを型安全に切り替え。
コードを変更せず、`fav.toml` の設定でプロバイダーを変更できる。

```toml
# fav.toml
[ai]
llm_provider    = "anthropic"   # prod: Claude
embed_provider  = "openai"      # prod: text-embedding-3-small
vector_db       = "pinecone"    # prod: Pinecone

[ai.dev]
llm_provider    = "ollama-local"  # dev: 無料ローカル LLM
embed_provider  = "ollama-local"  # dev: nomic-embed-text
vector_db       = "qdrant-local"  # dev: ローカル Qdrant

[ai.test]
llm_provider    = "mock"          # test: モック LLM（決定論的）
embed_provider  = "mock"          # test: 固定ベクトルを返す
vector_db       = "in-memory"     # test: インメモリ VectorDB
```

**実装内容**:

- `fav.toml` の `[ai]` セクションパース（`toml.rs` 拡張）
- `fav run --env dev` フラグ — 環境別設定の切り替え
- プロバイダー抽象化: `LLMProvider` interface（anthropic / openai / ollama / mock）
- `VectorDBProvider` interface（pinecone / qdrant / pgvector / in-memory）
- コスト追跡: 本番プロバイダーのみコスト計算（dev/test は $0）

**完了条件**: Rust テスト 2 件（3531 + 2 = **3533**）

```rust
// driver.rs mod v68700_tests
fn multi_cloud_ai_routing()        // "[ai]" / "llm_provider" / "--env" キーワードを含む
fn ai_provider_local_fallback()    // "ollama-local" / "mock" / "in-memory" キーワードを含む
```

---

## v68.8.0 — Distributed Observability（AI パイプライン分散トレーシング）

**概要**: 分散実行中のパイプラインを OpenTelemetry でエンドツーエンドトレース。
LLM 呼び出し・ベクトル DB クエリのレイテンシを統合ダッシュボードで可視化。

```bash
$ fav run pipeline.fav --cluster workers.yaml --otel-endpoint http://tempo:4317

[otel] Tracing enabled → Tempo (http://tempo:4317)
[trace] Pipeline: semantic-search-pipeline (trace_id: a3f2...)
  [span] LoadDocs:     2ms    worker-1
  [span] EmbedText[0]: 1240ms  worker-1 | LLM: openai/text-embedding-3-small
  [span] EmbedText[1]: 1238ms  worker-2 | LLM: openai/text-embedding-3-small
  [span] VectorStore:  45ms   worker-1  | DB: pinecone/prod
  [span] SemanticSearch: 23ms  worker-3 | DB: pinecone/prod
[otel] Trace exported to Tempo. View: http://grafana:3000/d/favnir-ai
```

**実装内容**:

- `--otel-endpoint <url>` フラグ — OpenTelemetry Collector への送信先
- 分散トレース: 各ステージを span として記録（parent/child 関係を保持）
- LLM span: モデル名・プロンプトトークン数・コスト・レイテンシ
- VectorDB span: インデックス名・クエリ次元・top_k・レイテンシ
- Grafana ダッシュボード定義: `infra/monitoring/favnir-ai-dashboard.json`
- Prometheus メトリクス統合（既存 v29.x の OTel Rune と連携）

**完了条件**: Rust テスト 2 件（3533 + 2 = **3535**）

```rust
// driver.rs mod v68800_tests
fn distributed_otel_trace()        // "--otel-endpoint" / "trace_id" / "span" キーワードを含む
fn distributed_latency_breakdown() // "LLM" / "VectorDB" / "Grafana" キーワードを含む
```

---

## v68.9.0 — 安定化・コードフリーズ（Distributed Favnir 前調整）

**概要**: v68.1〜v68.8 の全機能が正常動作することを確認する安定化バージョン。
分散実行・チェックポイント・K8s・リトライ・キャッシュ・コスト計算の統合確認。

**確認内容**:

- `--cluster` / `--checkpoint` / `--distributed-cache` フラグが正常動作
- K8s マニフェスト生成が正しい YAML を出力
- `fav cost-estimate` がコスト見積もりを出力
- `site/content/docs/runtime/distributed.mdx` の作成

**完了条件**: Rust テスト 2 件（3535 + 2 = **3537**）

```rust
// driver.rs mod v68900_tests
fn distributed_all_stable()     // 分散機能フラグが出力にキーワードを含む
fn distributed_docs_complete()  // distributed.mdx が存在し "--cluster" を含む
```

---

## v69.0.0 — Distributed Favnir 宣言 ★クリーンアップ

**宣言文**:

> 「`par` がクラスタを越え、チェックポイントが失敗を無効にする。
>  Kubernetes が AI ステージのスケールを決め、
>  コスト見積もりが LLM 呼び出しの予算を守る。
>  型安全な AI パイプラインが、大規模でも壊れない。
>
>  これが Favnir v69.0 — Distributed Favnir の姿である。」

**タスク**:

- [ ] `fav/Cargo.toml` version を `"69.0.0"` に更新
- [ ] `MILESTONE.md` 先頭に v69.0.0「Distributed Favnir」エントリを追加
- [ ] `README.md` に v69.0.0 宣言文を追加
- [ ] `CHANGELOG.md` 先頭に v69.0.0 エントリを追加
- [ ] `v69000_tests` 4 件を `driver.rs` に追加
- [ ] `cargo clean` 実行（★クリーンアップ）
- [ ] `cargo test -j 8 -- --test-threads=8` で 3541 tests passed を確認

**完了条件**: `v69000_tests` 4 件（3537 + 4 = **3541**）

```rust
// driver.rs mod v69000_tests
fn cargo_toml_version_is_69_0_0()   // Cargo.toml に "version = \"69.0.0\"" を含む
fn changelog_has_v69_0_0()          // CHANGELOG.md に "v69.0.0" を含む
fn milestone_has_distributed()      // MILESTONE.md に "Distributed Favnir" を含む
fn readme_mentions_distributed()    // README.md に "Distributed Favnir" または "v69.0" を含む
```

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v68.0.0（ベース） | 3519 | — |
| v68.1.0 | 3521 | +2 |
| v68.2.0 | 3523 | +2 |
| v68.3.0 | 3525 | +2 |
| v68.4.0 | 3527 | +2 |
| v68.5.0 | 3529 | +2 |
| v68.6.0 | 3531 | +2 |
| v68.7.0 | 3533 | +2 |
| v68.8.0 | 3535 | +2 |
| v68.9.0 | 3537 | +2 |
| v69.0.0 | 3541 | +4 |
