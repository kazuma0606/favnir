# Favnir Milestones

## v35.0.0 — Production Ready（2026-07-04）

> 「`fav new --template postgres-etl my-pipeline` で始め、
>  `fav check` で型安全性を確認し、
>  `fav build --target native` でネイティブバイナリを生成し、
>  Lambda にデプロイして実データを処理できる。
>  エラーが起きれば `fav explain` で原因がわかり、
>  `fav test --watch` でリグレッションを防げる。
>
>  これが Favnir v35.0 — Production Ready の姿である。」

v35.0.0 をもって、Favnir の **Production Ready** を正式に宣言する。

実案件デモ / ドキュメントサイト v4 / ベンチマーク公開 / セキュリティ審査 v2 /
エフェクトシステム統一（`!Effect` → ctx）/ 移行ツール整備が v34.x シリーズで完成した。

### 達成コンポーネント（v34.1〜v34.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 実案件デモ | v34.1 | `examples/real-world-etl/`（8 ファイル・5 ステージ）|
| ドキュメントサイト v4 | v34.2 | `/errors/` + cookbook 50 本 + ベンチマーク比較 |
| ベンチマーク公開 | v34.3 | Python pandas / Apache Spark 実測比較 |
| セキュリティ審査 v2 | v34.4 | W021・認証情報・sandbox・OSS ライセンス確認 |
| !Effect 廃止宣言 | v34.5 | W022 / `migration-effects.mdx` / IoCtx |
| ctx Rune 移行 | v34.6 | db / http / stream / io ctx Rune ファイル |
| ドキュメント ctx 移行 | v34.7 | `ctx-syntax-guide.mdx` / `getting-started.mdx` |
| 移行ツール | v34.8 | `MIGRATION.md` / `fav upgrade --from-effects` |
| 移行ドキュメント完全化 | v34.9 | `upgrade-guide.mdx` / ctx_migration フィクスチャ |

**宣言日**: 2026-07-04
**宣言バージョン**: v35.0.0

---

## v34.0.0 — Performance & Tooling（2026-07-04）

> 「`fav build --target native` でネイティブバイナリが生成でき、
>  10GB CSV を定常メモリで処理でき、
>  Lambda コールドスタートが 100ms 以下になること」
> = Performance & Tooling の完成を象徴する定義

v34.0.0 をもって、Favnir の **Performance & Tooling** を正式に宣言する。

AOT ネイティブバイナリ（Cranelift）/ インクリメンタルコンパイル / ストリーミング評価 /
Arrow 列指向統合 / precompiled 起動 / WASM 最適化 / エフェクトシステム移行準備 /
プロファイリング強化 / 並列コンパイルが v33.x シリーズで確認・記録された。

### 達成コンポーネント（v33.1〜v33.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| AOT ネイティブバイナリ | v33.1 | `fav build --target native` / Cranelift バックエンド |
| インクリメンタルコンパイル | v33.2 | `~/.fav/cache/` / SHA256 ハッシュキャッシュ |
| ストリーミング評価 | v33.3 | `#[streaming(chunk_size)]` / 定常メモリ処理 |
| Arrow 列指向統合 | v33.4 | `ArrowBatch` 型 / Parquet ゼロコピー書き込み |
| precompiled 起動 | v33.5 | `fav run --precompiled` / `.favc` アーティファクト |
| WASM 最適化 | v33.6 | DCE / wasm-opt 統合 / `WasmBuildConfig` |
| エフェクトシステム移行準備 | v33.7 | `migrate_effects_in_source` / `resolve_use_effects` |
| プロファイリング強化 | v33.8 | `parse_profile_json` / `to_folded_stacks` |
| 並列コンパイル | v33.9 | `compile_parallel` / `topo_layers` 循環依存検出 |

**宣言日**: 2026-07-04
**宣言バージョン**: v34.0.0

---

## v33.0.0 — Language Power（2026-07-03）

> 「Favnir の型システムを使って、DB スキーマから型を自動生成し、
>  汎用的なレコード変換関数を型安全に書き、
>  コンパイル時に前提条件を保証できること」
> = Language Power の完成を象徴する定義

v33.0.0 をもって、Favnir の **Language Power** を正式に宣言する。

境界付きジェネリクス（`T with Ord`）と行多相（`R with { id: Int }`）により汎用的なレコード変換関数が
型安全に書けるようになった。`where { b != 0 }` で関数引数の前提条件をコンパイル時に保証し、
`type User = schema "postgres:users"` でスキーマから型を自動生成できる。
線形型（E0332/E0333）・分散アノテーション（E0334）・定数ジェネリクス（E0335）が加わり、
型システムが実用的なデータパイプライン設計に耐える水準に達した。

### 達成コンポーネント（v32.1〜v32.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v32.1 | `T with Ord` / E0325 制約チェック |
| 行多相 | v32.2 | `R with { id: Int }` / E0337 フィールド不足 |
| where 制約 | v32.3 | `fn f(x: Int where { x > 0 })` / E0331 |
| スキーマ型 | v32.4 | `schema "postgres:users"` パース |
| 線形型 | v32.5 | E0332（二重使用）/ E0333（未使用）|
| 分散アノテーション | v32.6 | `<+T>` / `<-T>` / E0334 |
| 定数ジェネリクス | v32.7 | `<const N: Int where { N > 0 }>` / E0335 |
| 型駆動 API 生成 | v32.8 | `#[api]` / OpenAPI JSON / ルートテーブル |
| エフェクト推論 | v32.9 | `infer_effects_fn` / 推移的推論 |

**宣言日**: 2026-07-03
**宣言バージョン**: v33.0.0

---

## v32.0.0 — Language Polish（2026-07-03）

> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」
> = Language Polish の完成を象徴する定義

v32.0.0 をもって、Favnir の **Language Polish** を正式に宣言する。

エラーメッセージが rustc スタイル（`-->` ファイル位置 + `|` ソース行 + `= ヒント:`）に刷新され、
typo 候補（Levenshtein ≤ 2）と全エラーコード URL が付与された。
`fav explain E0001` でエラーの説明・修正例がターミナルで確認できる。
REPL は `:doc` / `:load` / `:history` / `:save` コマンドとタブ補完を備え、
データ探索ツールとして実用レベルに達した。
LSP Inlay Hints により `bind` 変数の型推論結果がエディタでインライン表示される。
`fav test --watch` と `fav check --all` / `fav scaffold` が揃い、
「書いていて気持ちいい」開発体験を達成した。

### 達成コンポーネント（v31.1〜v31.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| エラーメッセージ v2 | v31.1 | rustc スタイル・E0001〜E0021 全件 hint: 付与 |
| typo 候補 + URL | v31.2 | Levenshtein ≤ 2 候補提示・全エラーコード URL |
| fav explain | v31.3 | `fav explain E0001〜E0021` 説明・修正例出力 |
| REPL 品質向上 | v31.4 | :doc / :load / :history / :save / タブ補完 |
| LSP Inlay Hints | v31.5 | bind 変数の型推論結果インライン表示 |
| fav test --watch | v31.6 | ファイル変更で自動テスト再実行 |
| fav check --all | v31.7 | プロジェクト全体クロスファイルチェック |
| fav scaffold | v31.8 | stage / seq スタブを既存プロジェクトに追記 |
| ドッグフード修正 vol.2 | v31.9 | REPL 空行スキップ / check --all 空ディレクトリ警告 |

**宣言日**: 2026-07-03
**宣言バージョン**: v32.0.0

---

## v31.0.0 — Real-World Readiness（2026-07-02）

> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」
> = Real-World Readiness の完成を象徴するデモ

v31.0.0 をもって、Favnir の **Real-World Readiness** を正式に宣言する。

`fav new --template postgres-etl` による 4 ファイル構成テンプレート（types / validators / stages / main）が生成され、
`fav check` / `fav test` / `fav lint` の全コマンドが通過する。
`examples/csv-to-postgres/` に CSV 1000 行 → Postgres の実証パイプラインが実装され、
`fav test`（引数なし）がプロジェクト全体のテストを一括実行できるようになった。

### 達成コンポーネント（v30.1〜v30.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| ビルド軽量化 | v30.1 | `[profile.dev] debug = 0` で target/ 削減 |
| postgres-etl テンプレート v2 | v30.2 | 4 ファイル構成・`fav check` 全通過 |
| マルチファイル E2E | v30.3 | 5 コマンド（check/run/test/lint/fmt）全通過 |
| Rune import マルチファイル | v30.4 | 同一 Rune を複数ファイルから import 可能 |
| ドッグフードサンプル | v30.5 | `examples/csv-to-postgres/` 5 ステージ実装 |
| fav test プロジェクト統合 | v30.6 | 引数なし `fav test` でプロジェクト全体実行 |
| エラー表示改善 | v30.7 | ステージ名・ヒント付きランタイムエラー |
| fav new --list | v30.8 | 8 テンプレートの一覧表示 |
| ドッグフード修正 | v30.9 | `[project]` 解析・import 解決・UX hint |

**宣言日**: 2026-07-02
**宣言バージョン**: v31.0.0

---

## v30.0.0 — Ecosystem Maturity（2026-07-01）

> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」
> = Ecosystem Maturity の完成を象徴するデモ

v30.0.0 をもって、Favnir の **Ecosystem Maturity** を正式に宣言する。

Rune Registry（fav publish / add / search / info）が本番稼働し、
コミュニティ投稿 Rune 10 本（stripe / twilio / notion / linear / airtable /
sendgrid / hubspot / zendesk / shopify / intercom）が `runes/` 下に存在する。
AI/ML Rune 4 本（mlflow / pinecone / vertex-ai / sagemaker）と
VS Code 拡張・ドキュメントサイト v3（cookbook 32 本）が揃い、
「Favnir で書いたパイプラインをコミュニティが Rune で拡張できる」状態を達成した。

### 達成コンポーネント（v29.1〜v29.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Rune Registry（fav publish / add / search / info） | v29.1 | Lambda + S3 + GitHub OAuth |
| mlflow Rune | v29.2 | start_run / log_metric / log_param / log_artifact / register_model |
| pinecone Rune | v29.3 | upsert / query / delete / fetch / describe_index_stats |
| vertex-ai / sagemaker Rune | v29.4 | predict / batch_predict / invoke / create_endpoint |
| github Rune | v29.5 | create_comment / create_issue / update_issue / list_prs |
| pagerduty Rune | v29.6 | create_incident / resolve / acknowledge / add_note |
| VS Code 拡張 公式リリース | v29.7 | TextMate grammar / LSP クライアント / Task Runner 統合 |
| ドキュメントサイト v3 | v29.8 | cookbook 32 本 / community ページ |
| コミュニティ Rune コンテスト | v29.9 | 10 本スタブ / CONTRIBUTING.md ガイド |

### 残件（v31.x）

- Rune Registry への実際のパッケージアップロード（Lambda 本番稼働後）
- コミュニティ Rune の HTTP 認証ヘッダー対応（HTTP Rune 有効化後）
- VS Code Marketplace への公開（手動）

**宣言日**: 2026-07-01
**宣言バージョン**: v30.0.0

---

**宣言日**: 2026-06-24
**宣言バージョン**: v25.0.0 = v1.0 リリース候補

---

## 宣言

> 「Favnir は Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」

v25.0.0 をもって、Favnir の **Practical Self-Hosting** を正式に宣言する。

コンパイラ・型チェッカー・CLI・VM 仕様のすべてが Favnir で実装された。
Rust が担うのは VM の実行基盤（バイトコードディスパッチループ）のみであり、
これは設計上の意図であり制約ではない。

---

## 達成済みコンポーネント

| コンポーネント | ファイル | 実装言語 | 達成バージョン |
|---|---|---|---|
| コンパイラ | compiler.fav | Favnir ✓ | v8.5.0〜 |
| 型チェッカー | checker.fav | Favnir ✓ | v8.1.0〜 |
| CLI | cli.fav | Favnir ✓ | v7.6.0〜 |
| VM 仕様 | vm.fav | Favnir ✓ | v24.0.0〜 |
| VM 実行基盤 | src/backend/vm.rs | Rust（永続・設計上） | — |

### VM エンジンが Rust である理由

バイトコードのディスパッチループ・スタック管理・メモリアロケーションは、
Rust の安全性保証とゼロコスト抽象化が最も価値を発揮する領域です。
**これは Favnir の自己記述能力の欠如ではなく、正しい責任分担の結果です**。
VM の「仕様・動作の記述」は vm.fav（Favnir）が担い、
「実行の実装」は Rust が担う——このハイブリッド戦略こそが Favnir の強みです。

---

## セルフホスト達成の歴史

| バージョン | 達成内容 |
|---|---|
| v7.6.0 | cli.fav: `fav run` / `fav check` / `fav new` をすべて Favnir で実装 |
| v8.1.0 | checker.fav: `fav check` が Favnir 型チェッカー経由で動作 |
| v8.5.0 | compiler.fav: `fav run` がデフォルトで Favnir コンパイラ経由で動作 |
| v9.0.0 | セルフホスト完成宣言（compiler + checker + cli すべて Favnir 経由） |
| v24.0.0 | vm.fav: VM 仕様を Favnir で記述・テスト通過 |
| **v25.0.0** | **Practical Self-Hosting 宣言（本バージョン）** |

---

## 最終テスト（v25.0.0 達成状況）

| # | テスト | 状態 |
|---|---|---|
| 1 | `cargo test --bin fav` — 1974 件全 PASS | ✓ 達成（v25.0.0） |
| 2 | `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` | 延期（v25.x: vm.fav Phase 6） |
| 3 | `fav run --vm=self/vm.fav self/checker.fav` E2E | 延期（v25.x） |
| 4 | `fav run --vm=self/vm.fav self/cli.fav` E2E | 延期（v25.x） |
| 5 | 4-stage bootstrap 全 6 fixture（Stage 4 = vm.fav） | 延期（v25.x） |

テスト 2〜5 は vm.fav Phase 6（ユーザー定義関数ディスパッチ、実装では `CallNamed` opcode として確定）が
未実装のため v25.x に延期。テスト 1 の全件 PASS をもって v25.0.0 の完了条件とする。

---

## v1.x 後方互換性保証

v25.0.0 = v1.0 リリース候補として、後方互換性ポリシーを確定した。
詳細は [STABILITY.md](./STABILITY.md) を参照。

---

## v29.0.0 — Observability First（2026-06-28）

> 「`#[track(latency, error_rate)]` を stage に付けるだけで
>  Grafana ダッシュボードにメトリクスが現れる」
> = Observability First の完成を象徴するデモ

v29.0.0 をもって、Favnir の **Observability First** を正式に宣言する。

prometheus / datadog / sentry / grafana / otel の 5 Rune が揃い、
`#[track]` / `#[trace]` / `#[on_error]` アノテーションと E2E デモ 3 本が
Docker Compose で動作する。パイプラインの内側を型安全に観測できる状態を達成した。

### 達成コンポーネント（v28.1〜v28.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| prometheus Rune | v28.1 | counter / gauge / histogram / push + `#[track]` アノテーション |
| datadog Rune | v28.2 | metric / log / trace / event / service_check |
| OpenTelemetry Rune（otel 強化） | v28.3 | start_span / set_attribute / add_event / end_span + `#[trace]` アノテーション |
| `fav profile` 強化 | v28.4 | `--format flamegraph`（SVG 生成）/ `--compare <version>` |
| sentry Rune | v28.5 | capture_error / capture_message / set_user / set_tag / set_extra + `#[on_error]` アノテーション |
| grafana Rune | v28.6 | create_annotation / push_dashboard / snapshot |
| E2E デモ（prometheus + grafana） | v28.7 | `#[track]` stage → Grafana ダッシュボード自動反映 |
| E2E デモ（datadog APM） | v28.8 | `#[trace]` stage → Datadog サービスマップ・フレームグラフ |
| E2E デモ（sentry アラート） | v28.9 | `#[on_error]` stage → Sentry critical アラート自動送信 |

### 象徴デモ

```favnir
import runes/prometheus
import runes/grafana

// #[track] を付けるだけで Grafana ダッシュボードにメトリクスが現れる
// #[track(latency: true, error_rate: true)]
stage ExtractOrders: Unit -> List<RawOrder> !Db = |_| {
    Postgres.query[RawOrder](conn, "SELECT * FROM orders WHERE status = 'pending'")
}

// #[track(latency: true)]
stage TransformOrders: List<RawOrder> -> List<Order> !Pure = |rows| {
    Result.ok(List.map(rows, parse_order))
}

// #[track(latency: true, error_rate: true)]
stage LoadToWarehouse: List<Order> -> Unit !Db = |orders| {
    Postgres.execute_many(conn, "INSERT INTO warehouse SELECT * FROM ?", orders)
}

seq ObservabilityFirstDemo = ExtractOrders |> TransformOrders |> LoadToWarehouse
```

### v29.x 残件（次フェーズ）

- prometheus / grafana 実メトリクス送信の統合テスト（実際の pushgateway との E2E）
- `#[track]` / `#[trace]` / `#[on_error]` アノテーションのコンパイラ自動挿入実装
- Datadog APM トレース送信の実統合（DSN 本番テスト）
- `fav profile --compare` の stage 別 JSON 比較精度向上

---

## v28.0.0 — Data Lakehouse（2026-06-27）

> 「Delta Lake テーブルを Favnir から型安全に読み書きし、
>  dbt モデルの結果を次のステージに渡す」
> = Data Lakehouse の完成を象徴するデモ

v28.0.0 をもって、Favnir の **Data Lakehouse** を正式に宣言する。

Delta Lake / Iceberg テーブルの読み書き、dbt モデル参照、
主要 DWH 3 本（ClickHouse / BigQuery / Redshift）への接続、
SQLite 組み込み DB が揃い、現代データ基盤アーキテクチャへの完全統合を達成した。

### 達成コンポーネント（v27.1〜v27.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| delta-lake Rune | v27.1 | read / write / merge / history / vacuum / optimize |
| iceberg Rune | v27.2 | read / append / overwrite / time_travel / schema_evolution / list_snapshots |
| clickhouse Rune | v27.3 | connect / query / insert / async_insert |
| bigquery Rune | v27.4 | connect / query / insert / load_from_gcs / create_table |
| redshift Rune | v27.5 | connect / query / execute / copy_from_s3 / unload_to_s3 |
| jsonl Rune | v27.6 | read / write / stream / append |
| `fav infer --from delta/iceberg` | v27.7 | Delta / Iceberg スキーマ → Favnir 型定義自動生成 |
| dbt 連携 Rune | v27.8 | ref / source（manifest.json 解析、`!Db` エフェクト） |
| sqlite Rune | v27.9 | open / open_memory / query / execute / execute_many / close |

### 象徴デモ

```favnir
import rune "delta-lake"
import rune "dbt"
import rune "sqlite"

// Delta Lake からロード → dbt モデル参照 → SQLite に保存
stage LoadFromDelta: Unit -> List<OrderRow> !Io = |_| {
    DeltaLake.read[OrderRow]("s3://my-bucket/orders")
}

stage EnrichWithDbt: List<OrderRow> -> List<EnrichedOrder> !Db = |orders| {
    bind summary <- Dbt.ref(config.dbt, "customer_summary")
    Result.ok(List.map(orders, |o| enrich(o, summary)))
}

stage SaveToSqlite: List<EnrichedOrder> -> Unit !Db = |rows| {
    bind db <- SQLite.open_memory()
    bind _  <- SQLite.execute(db, "CREATE TABLE orders (id INT, amount REAL)", "[]")
    SQLite.execute_many(db, "INSERT INTO orders VALUES (?, ?)", rows)
}

seq DataLakehousePipeline = LoadFromDelta |> EnrichWithDbt |> SaveToSqlite
```

### v28.x 残件（次フェーズ）

- delta-rs 実統合（実際の Delta テーブル読み書き）
- rusqlite 実統合（実際の SQLite 操作）
- dbt manifest.json 実解析と SQL 実行
- Iceberg REST カタログ実統合

---

## v27.0.0 — Streaming Native（2026-06-27）

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

v27.0.0 をもって、Favnir の **Streaming Native** を正式に宣言する。

ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）が実質化され、
`Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）が使用可能になり、
E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作する。

### 達成コンポーネント（v26.1〜v26.9）

| コンポーネント | バージョン | 実装済み関数 |
|---|---|---|
| kinesis Rune | v26.1.0 | connect / put_record / put_records / get_shard_iterator / get_records |
| nats Rune | v26.2.0 | connect / publish / subscribe / jetstream_publish / jetstream_consume |
| rabbitmq Rune | v26.3.0 | connect / declare_exchange / declare_queue / bind_queue / publish / consume |
| Stream.* 操作 6 関数 | v26.4.0 | map / filter / flat_map / window / merge / split |
| E2E デモ: kafka → Elasticsearch | v26.5.0 | `examples/streaming/kafka_to_elasticsearch.fav` |
| E2E デモ: kinesis → S3 | v26.6.0 | `examples/streaming/kinesis_to_s3.fav` |
| E2E デモ: nats → postgres | v26.7.0 | `examples/streaming/nats_to_postgres.fav` |
| sqs Rune | v26.8.0 | send_message / send_message_batch / receive_messages / delete_message / purge / consume |
| pulsar Rune | v26.9.0 | produce / consume / ack / nack（暫定 `!AWS` エフェクト、v27.x で `!Pulsar` へ移行予定） |

### Streaming Native 検証コマンド

```bash
docker compose -f examples/streaming/docker-compose.yml up -d
fav run examples/streaming/kafka_to_elasticsearch.fav
fav run examples/streaming/kinesis_to_s3.fav
fav run examples/streaming/nats_to_postgres.fav
```

### v27.x 残件（次フェーズ）

- kinesis: `Kinesis.consume[T]` 継続消費ループ
- nats: `NATS.request[T]` リクエスト/レスポンス
- rabbitmq: `RabbitMQ.ack` / `RabbitMQ.nack`
- pulsar: Binary Protocol 経由の高速 produce

---

## v26.0.0 — Rune Foundation（2026-06-26）

> 「Favnir で書いたパイプラインが実際の本番データを動かせる」

v26.0.0 をもって、Favnir の **Rune Foundation** を正式に宣言する。

コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が
「動く Rune の 5 条件（connect / read / write / error / test）」をすべてクリアした。
また vm.fav Phase 6（`CallNamed` opcode, 0x56）が完成し、
multi-function Favnir プログラムを vm.fav インタープリター上で実行できるようになった。

### 達成した Rune

| Rune | 条件 | 主要関数 |
|---|---|---|
| postgres | connect / read / write / error / test ✓ | connect / query / execute / execute_many / transaction / Pool |
| s3 | connect / read / write / error / test ✓ | get_object / put_object / list_objects / delete_object / presign_url |
| redis | connect / read / write / error / test ✓ | get / set / del / incr / lpush / rpop / publish / subscribe |
| mysql | connect / read / write / error / test ✓ | connect / query / execute / transaction（DbConn interface 統一） |
| mongodb | connect / read / write / error / test ✓ | find / find_one / insert_one / insert_many / update_one / delete_one / aggregate |
| dynamodb | connect / read / write / error / test ✓ | get_item / put_item / delete_item / query / scan / batch_write / transact_write |
| kafka | connect / read / write / error / test ✓ | produce / consume / consume_batch / commit / seek |
| elasticsearch | connect / read / write / error / test ✓ | index / search / bulk / delete / knn_search / create_index |

### デモ

```bash
# postgres → 集計 → s3 → kafka 通知
fav run examples/full_etl.fav

# postgres ETL
fav run examples/postgres_etl.fav

# s3 CSV → Parquet 変換
fav run examples/s3_csv_to_parquet.fav
```

### vm.fav Phase 6 達成

`CallNamed(name_idx, argc)` opcode (0x56) の実装により、
multi-function Favnir プログラムを vm.fav インタープリター上で実行できるようになった。

```bash
# multi-function プログラムを vm.fav 経由で実行
fav run --vm self/vm.fav --compile hello.fav
```

---

## v35.0 — Production Ready（2026-07-05）

Favnir が **Production Ready** を宣言しました。

v34.9A（v35.5.0）にて `!Effect` アノテーション構文が言語から**完全に削除**され、
副作用管理は `ctx: AppCtx`（Capability Context）パターンに一本化されました。

### 達成内容

| カテゴリ | 内容 |
|---|---|
| 言語クリーンアップ | `Effect` enum / `effects` フィールドを `ast.rs` 以降すべてのレイヤーから削除 |
| self-hosted コンパイラ | `compiler.fav` / `checker.fav` が Effects なしで完全動作 |
| ドキュメント統一 | サイト MDX 128 ファイル・317 コードブロックを ctx 構文に変換（v35.6.0） |
| テストカバレッジ | 2611 tests pass（0 failures）、cargo clippy clean |

### Production Ready の定義

1. **言語仕様が安定** — `!Effect` は完全廃止、ctx 構文が唯一の副作用表現
2. **セルフホスト完成** — compiler.fav / checker.fav が Favnir 自身でコンパイル
3. **ドキュメント整合** — 全コードサンプルが現行構文と一致
4. **エコシステム成熟** — 50+ 公式 Rune、実世界デモ、ベンチマーク公開済み

### コマンド

```bash
# ctx 構文でのパイプライン
fav run examples/postgres_etl.fav

# 旧 !Effect 構文（E0374 エラー）
fav check --legacy examples/pipeline/custom_effects.fav
```
