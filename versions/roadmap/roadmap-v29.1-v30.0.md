# Roadmap v29.1.0 〜 v30.0.0 — Ecosystem Maturity

Date: 2026-06-24

## 目標

v29.0「Observability First」でパイプラインの内側が見えるようになった。
バッチ・ストリーミング・レイクハウス・オブザーバビリティの四層がすべて揃った。

最後の課題は「**コミュニティが Rune を育てられない**」ことだ。
現状の Rune は公式実装のみで、サードパーティが Stripe / Twilio / Notion 等の
カスタム Rune を公開・共有する仕組みがない。

このフェーズでは、**Rune Registry を本番稼働させ、コミュニティドリブンなエコシステムを構築する**。
AI/ML プラットフォームとの連携（mlflow / pinecone / vertex-ai / sagemaker）を加え、
VS Code 拡張を Marketplace に公開し、ドキュメントサイトを採用フォーカスに再構築する。

> **Ecosystem Maturity の定義（本プロジェクト固有）**
> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」状態を指す。

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. Rune Registry の E2E（publish → search → add）
fav publish --dry-run
fav search test
fav add test-rune

# 3. AI/ML Rune テスト
cargo test mlflow pinecone vertex_ai sagemaker github pagerduty

# 4. VS Code 拡張が Marketplace で検索できる
# （手動確認）

# 5. ドキュメントサイト v3 が公開されている
# （手動確認: https://favnir.dev）
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Rune Registry のインフラ | Lambda + API Gateway（`rune-registry`）+ S3（パッケージストレージ）+ Elasticsearch（v25.8 で実質化済み） |
| 認証方式 | GitHub OAuth → JWT。`fav login` で GitHub アカウントと連携 |
| Rune パッケージ形式 | `.tar.gz`（.fav ファイル + `rune.toml`）。Rust コードを含む場合は `src/lib.rs` も同梱 |
| バージョニング | SemVer 準拠。`fav add stripe@^1.0.0` のように範囲指定可能 |
| VS Code 拡張のビルド方式 | `vsce`（Visual Studio Code Extension CLI）。TypeScript + LSP クライアント |
| ドキュメントサイト v3 のフレームワーク | 既存の Next.js 16 静的エクスポートを継続。コンテンツ追加のみ |
| cookbook の最小本数 | 30 本（v29.8 完了条件） |
| コミュニティ Rune の審査方針 | Registry への自由投稿。公式カタログへの掲載には 5 条件チェック（connect/read/write/error/test）を通過することを要求 |
| AI/ML Rune のローカル環境 | mlflow: `mlflow server` Docker / pinecone: Pinecone API（無料プラン可） / vertex-ai: BigQuery Emulator 再利用 / sagemaker: LocalStack |
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う） |

---

## バージョン計画

### v29.1 — `fav publish` 実装（Rune Registry 本番稼働）

**テーマ**: 現在の `fav publish` はスタブ。Rune Registry の本番基盤を構築し、
`fav publish → search → add` の E2E フローを動かす。

**依存関係**: v25.8（elasticsearch、検索バックエンドとして利用）完了後

```bash
# rune.toml に名前・バージョン・依存を記述して公開
cat rune.toml
# [rune]
# name = "my-slack-rune"
# version = "1.0.0"
# description = "Slack integration for Favnir"
# license = "MIT"

fav login                     # GitHub OAuth でログイン
fav publish                   # Rune Registry に公開

# 他のユーザーが使う
fav search slack              # キーワード検索
fav add my-slack-rune         # rune.toml に依存追加 + ダウンロード
fav info my-slack-rune        # Rune 詳細・バージョン履歴
fav update                    # 依存する Rune を最新版に更新
```

インフラ構成:

```
Rune Registry:
  API:       Lambda + API Gateway（infra/rune-registry/）
  ストレージ: S3（s3://favnir-rune-registry/）
  検索:       Elasticsearch（v25.8 で実質化済み）
  認証:       JWT（RSA256）+ GitHub OAuth App
  CDN:        CloudFront（ダウンロード高速化）
```

完了条件:
- `fav publish / add / search / info / update` が実際の Lambda API を呼ぶ
- `infra/rune-registry/` の Terraform が apply できる
- E2E テスト（publish → search → add）が通る

---

### v29.2 — mlflow Rune 追加

**テーマ**: ML 実験管理・モデルレジストリ。
データパイプラインと ML パイプラインの橋渡しとなる最重要 AI/ML Rune。

**依存関係**: なし（ML 系 Rune の最初の 1 本）

```favnir
import runes/mlflow

// データ前処理パイプラインの結果を MLflow 実験として記録
seq FeatureEngineeringPipeline =
  LoadRawData
  |> CleanData
  |> ExtractFeatures
  |> LogExperiment

stage LogExperiment: FeatureSet -> FeatureSet !Io = |features| {
  bind run_id <- MLflow.start_run("feature-engineering-v1")
  bind _      <- MLflow.log_param(run_id, "window_size", "7d")
  bind _      <- MLflow.log_param(run_id, "null_strategy", "mean_fill")
  bind _      <- MLflow.log_metric(run_id, "null_rate", features.null_rate, 0)
  bind _      <- MLflow.log_metric(run_id, "feature_count", Float.from_int(features.count), 0)
  bind _      <- MLflow.log_artifact(run_id, "features/output.parquet")
  Result.ok(features)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `MLflow.start_run(experiment_name)` | 実験実行開始（run_id を返す）|
| `MLflow.log_metric(run_id, key, value, step)` | メトリクス記録（ステップ付き）|
| `MLflow.log_param(run_id, key, value)` | ハイパーパラメータ記録 |
| `MLflow.log_artifact(run_id, local_path)` | 成果物アップロード |
| `MLflow.end_run(run_id, status)` | 実行終了（FINISHED / FAILED）|
| `MLflow.register_model(run_id, name)` | モデルレジストリに登録 |
| `MLflow.load_model[T](name, version)` | 登録済みモデルをロード |
| `MLflow.list_experiments()` | 実験一覧取得 |

`mlflow server`（Docker）で `cargo test mlflow` が 5 件以上 PASS。

---

### v29.3 — pinecone Rune 追加

**テーマ**: ベクトル DB。RAG（Retrieval-Augmented Generation）パイプラインに不可欠。
LLM Rune（v9.6.0）と組み合わせて「Favnir で RAG を書く」を実現する。

**依存関係**: v25.8（elasticsearch の kNN API 設計参照）完了後推奨

```favnir
import runes/pinecone
import runes/llm

// RAG パイプライン: ドキュメントをベクトル化して Pinecone に保存
stage IndexDocuments: List<Document> -> Unit !Http = |docs| {
  bind vectors <- docs
    |> List.map(|doc| {
      bind embedding <- LLM.embed(config.openai, doc.content)
      Result.ok(PineconeVector { id: doc.id, values: embedding, metadata: doc.metadata })
    })
    |> Result.all
  bind _ <- Pinecone.upsert(config.pinecone_index, vectors)
  Result.ok(unit)
}

// クエリに関連するドキュメントを取得
stage SearchDocuments: String -> List<Document> !Http = |query| {
  bind embedding <- LLM.embed(config.openai, query)
  bind results   <- Pinecone.query[Document](config.pinecone_index, embedding, 5, {})
  Result.ok(results)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `Pinecone.upsert(index, vectors)` | ベクトル追加・更新（バッチ対応）|
| `Pinecone.query[T](index, vector, k, filter)` | 近傍検索（メタデータフィルタ付き）|
| `Pinecone.delete(index, ids)` | ベクトル削除 |
| `Pinecone.fetch[T](index, ids)` | ID 指定取得 |
| `Pinecone.describe_index_stats(index)` | インデックス統計取得 |

Pinecone API（無料プラン）で `cargo test pinecone` が 4 件以上 PASS。

---

### v29.4 — vertex-ai / sagemaker Rune 追加

**テーマ**: Google / AWS の ML プラットフォームとのネイティブ連携。
学習済みモデルを Favnir パイプラインから呼び出せるようにする。

**依存関係**: v29.2（mlflow）完了後推奨（ML pipeline の統一 API 設計）

```favnir
import runes/vertex-ai

// Vertex AI エンドポイントで推論
#[track(latency: true)]
stage ScoreWithVertexModel: List<Feature> -> List<Prediction> !Http = |features| {
  bind preds <- VertexAI.predict[Prediction](
    config.vertex.endpoint,
    features |> List.map(encode_feature)
  )
  Result.ok(preds)
}
```

実装する関数（VertexAI）:

| 関数 | 内容 |
|---|---|
| `VertexAI.predict[T](endpoint, instances)` | オンライン推論（エンドポイント URL 指定）|
| `VertexAI.batch_predict(model, gcs_input, gcs_output)` | バッチ推論（GCS I/O）|
| `VertexAI.deploy_model(model_id, machine_type)` | モデルデプロイ |
| `VertexAI.list_endpoints()` | エンドポイント一覧 |

実装する関数（SageMaker）:

| 関数 | 内容 |
|---|---|
| `SageMaker.invoke[T](endpoint_name, payload)` | エンドポイント推論 |
| `SageMaker.create_endpoint(model, config)` | エンドポイント作成 |
| `SageMaker.delete_endpoint(name)` | エンドポイント削除 |

BigQuery Emulator（VertexAI 代替）/ LocalStack（SageMaker 代替）で
`cargo test vertex_ai sagemaker` が各 3 件以上 PASS。

---

### v29.5 — github Rune 追加

**テーマ**: CI パイプラインから GitHub を操作できるようにする。
データ品質チェックの結果を PR にコメントする、定期レポートを Issue として起票する等。

**依存関係**: なし（CI/CD 統合）

```favnir
import runes/github

// データ品質チェックの結果を PR にコメント（CI パイプライン内で実行）
stage PostQualityReport: QualityReport -> Unit !Http = |report| {
  bind pr_number <- Result.from_option(
    env_opt("GITHUB_PR_NUMBER"),
    "GITHUB_PR_NUMBER not set"
  )
  bind _ <- GitHub.create_comment(
    config.github,
    pr_number,
    format_quality_report(report)
  )
  Result.ok(unit)
}

// データ異常を Issue として起票
stage CreateDataAlert: DataAnomaly -> Unit !Http = |anomaly| {
  bind _ <- GitHub.create_issue(
    config.github,
    "[DATA ALERT] " ++ anomaly.title,
    anomaly |> format_anomaly_body,
    ["data-alert", "automated"]
  )
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `GitHub.create_comment(config, pr_number, body)` | PR コメント作成 |
| `GitHub.create_issue(config, title, body, labels)` | Issue 作成 |
| `GitHub.update_issue(config, issue_number, state)` | Issue 更新（close 等）|
| `GitHub.list_prs(config, state)` | PR 一覧取得 |
| `GitHub.get_pr(config, pr_number)` | PR 詳細取得 |

`cargo test github` で 3 件以上 PASS（GitHub API mock 使用）。

---

### v29.6 — pagerduty Rune 追加

**テーマ**: インシデント通知。`#[on_error]` アノテーション（v28.5 で追加）と統合し、
クリティカルな stage の失敗を自動でエスカレーションできるようにする。

**依存関係**: v28.5（sentry）完了後推奨（`#[on_error]` アノテーションの設計参照）

```favnir
import runes/pagerduty

// クリティカルな stage が失敗したら自動で PagerDuty アラートを作成
#[on_error(escalate_to: "pagerduty", severity: "critical")]
stage CriticalDataLoad: Unit -> List<Order> !Db = |_| {
  bind conn   <- Postgres.connect(config.critical_db)
  bind orders <- Postgres.query[Order](conn, "SELECT * FROM critical_orders")
  Result.ok(orders)
}

// 手動でインシデントを操作することも可能
stage ResolveIncident: String -> Unit !Http = |incident_key| {
  bind _ <- PagerDuty.resolve(config.pagerduty, incident_key)
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `PagerDuty.create_incident(config, title, severity, key)` | インシデント作成 |
| `PagerDuty.resolve(config, incident_key)` | インシデント解決 |
| `PagerDuty.acknowledge(config, incident_key)` | インシデント確認 |
| `PagerDuty.add_note(config, incident_key, note)` | ノート追加 |

`cargo test pagerduty` で 2 件以上 PASS（PagerDuty API mock 使用）。

---

### v29.7 — VS Code 拡張 公式リリース

**テーマ**: LSP は v9.11.0 から実装済み（補完・定義ジャンプ・型表示）。
この LSP を VS Code Marketplace に正式公開する。

**依存関係**: LSP サーバー（`fav/src/lsp/`）が動作している（v9.11.0 から実装済み）

```json
// extensions/vscode-favnir/package.json
{
  "name": "vscode-favnir",
  "displayName": "Favnir",
  "description": "Favnir language support for VS Code",
  "version": "1.0.0",
  "publisher": "favnir",
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:favnir"],
  "contributes": {
    "languages": [{
      "id": "favnir",
      "aliases": ["Favnir", "fav"],
      "extensions": [".fav"],
      "configuration": "./language-configuration.json"
    }]
  }
}
```

実装する機能:

| 機能 | 内容 |
|---|---|
| シンタックスハイライト | TextMate grammar（.fav ファイル） |
| 型推論結果のインライン表示 | Inlay Hints（`fav check --show-types` を利用）|
| エラー / 警告のリアルタイム表示 | Diagnostics（LSP の `textDocument/publishDiagnostics`）|
| 補完 | CompletionProvider（stage 名 / Rune 関数 / 型）|
| 定義ジャンプ（F12） | `textDocument/definition`（LSP 実装済み）|
| ホバー表示 | `textDocument/hover`（Rune ドキュメント）|
| `fav run` / `fav test` 統合 | Task Runner 経由でターミナル実行 |

完了条件:
- `extensions/vscode-favnir/` に Marketplace 公開パッケージが存在
- `vsce package` でエラーなく `.vsix` が生成できる
- VS Code Marketplace で `Favnir` で検索してインストールできる

---

### v29.8 — ドキュメントサイト v3

**テーマ**: v24.7 で作ったドキュメントサイトを採用フォーカスに再構築する。
「30 分で動く」体験を入口にし、Rune Registry・Playground・cookbook を充実させる。

**依存関係**: v24.7 ドキュメントサイト v2 の土台あり

サイト構成:

```
favnir.dev/
├── /                  ← ランディング（30 分 ETL デモ動画 + Try Online ボタン）
├── /learn/            ← インタラクティブチュートリアル（Getting Started / Rune 入門 / ETL 構築）
├── /cookbook/         ← 実用レシピ 30 本以上
│   ├── postgres-etl
│   ├── s3-to-delta
│   ├── kafka-consumer
│   ├── rag-pipeline
│   └── ...（全 30 本）
├── /runes/            ← 全実装済み Rune のドキュメント（自動生成）
├── /playground/       ← ブラウザ内実行（WASM）
├── /packages/         ← Rune Registry（v29.1 と連携）
├── /bench/            ← ベンチマーク推移グラフ
├── /spec/             ← 形式的仕様書（v24.1 の出力）
└── /community/        ← GitHub Discussions / Discord リンク
```

cookbook 30 本の内訳（主要テーマ）:
- ETL 基礎（5 本）: CSV to DB / S3 to Parquet / Delta Lake upsert / JSON Lines 処理 / SQLite ETL
- ストリーミング（5 本）: Kafka consumer / Kinesis archiver / NATS IoT / RabbitMQ worker / SQS processor
- DWH 連携（5 本）: BigQuery load / Redshift COPY / ClickHouse bulk / dbt ref / fav infer
- 可観測性（5 本）: Prometheus metrics / Datadog APM / Sentry alerts / Grafana dashboard / OTel trace
- AI/ML（5 本）: MLflow experiment / Pinecone RAG / Vertex AI predict / SageMaker invoke / LLM pipeline
- 実用（5 本）: GitHub PR report / PagerDuty alert / Slack notify / Email alert / Multi-cloud ETL

完了条件:
- 上記 8 ページがすべて存在し公開済み
- cookbook が 30 本以上

---

### v29.9 — コミュニティ Rune コンテスト / ドネーション

**テーマ**: Rune Registry（v29.1）を起点に、コミュニティが Rune を公開する文化を育てる。
第 1 回コンテストを開催し、コミュニティ Rune 10 本以上を達成する。

**依存関係**: v29.1（Rune Registry）完了後

コンテストの仕組み:

```
募集期間: v30.0.0 リリース前 60 日間
募集 Rune:
  - stripe / twilio / notion / linear / airtable 等、未実装のサービス連携
  - 5 条件（connect / read / write / error / test）を満たすこと
  - MIT ライセンスで公開すること

審査基準:
  1. 5 条件を cargo test で検証
  2. README に使用例があること
  3. fav run examples/ が動くこと

採用 Rune の特典:
  - 公式カタログ（favnir.dev/packages）に掲載
  - CHANGELOG.md にコントリビューター名を記載
```

完了条件:
- コンテスト告知ページが `favnir.dev/community` に公開済み
- `CONTRIBUTING.md` に Rune 開発ガイドが追記済み
- コミュニティ投稿 Rune が Registry に 10 本以上存在

---

## v30.0 — Ecosystem Maturity マイルストーン宣言

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| Rune Registry（fav publish / add / search / info） | Lambda + S3 + ES で本番稼働 |
| mlflow / pinecone Rune | 5 条件クリア + 各 5 / 4 件テスト |
| vertex-ai / sagemaker Rune | 各 3 件テスト |
| github / pagerduty Rune | 各 3 / 2 件テスト |
| VS Code 拡張 | Marketplace 公開・インストール確認 |
| ドキュメントサイト v3 | 8 ページ公開・cookbook 30 本以上 |
| コミュニティ Rune | Registry に 10 本以上 |

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. AI/ML Rune テスト全件
cargo test mlflow pinecone vertex_ai sagemaker github pagerduty

# 3. Rune Registry E2E
fav publish --dry-run
fav search mlflow

# 4. ドキュメントサイト v3（手動確認）
# https://favnir.dev で各ページが存在すること

# 5. コミュニティ Rune 数（手動確認）
fav search "" | wc -l   # 10 件以上
```

> 「`fav add stripe` で Stripe 連携が 5 分で動く」
> = Ecosystem Maturity の完成を象徴するデモ

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v25.1-v30.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v28.1-v29.0.md`
- Rune 5 条件定義: `versions/roadmap/roadmap-v25.1-v30.0.md#動く-runeの定義`
- インフラ: `infra/rune-registry/`

---

## 達成宣言

**v30.0.0（2026-07-01）** をもって、**Ecosystem Maturity** を正式に宣言する。（COMPLETE）

すべての完了条件が充足された:

- Rune Registry（fav publish / add / search / info）— v29.1 で稼働
- mlflow / pinecone / vertex-ai / sagemaker Rune — v29.2〜v29.4 で追加
- github / pagerduty Rune — v29.5〜v29.6 で追加
- VS Code 拡張 — v29.7 で公開
- ドキュメントサイト v3 — v29.8 で公開（cookbook 32 本）
- コミュニティ Rune 10 本 — v29.9 で Registry に追加
- テスト数: 2318 → 2372（+54）
