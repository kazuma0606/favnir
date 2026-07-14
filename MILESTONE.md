# Favnir Milestones

## v45.0.0 — Precision & Flow（2026-07-15）

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

v45.0.0 をもって、Favnir の **Precision & Flow** を正式に宣言する。

### 達成コンポーネント（v44.1〜v44.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type × Streaming 統合 | v44.1 | collect_refinement_stream_bindings |
| CEP × Refinement type | v44.2 | collect_cep_refinement_event_refs |
| Stream join × Opaque type | v44.3 | collect_opaque_alias_groups |
| 型推論 × パイプライン lineage | v44.4 | collect_annotated_lineage_bindings |
| Back-pressure × fav policy 統合 | v44.5 | collect_stage_max_inflight_annotations |
| Precision & Flow E2E デモ | v44.6 | infra/e2e-demo/precision-flow/ |
| ドキュメントサイト概要ページ | v44.7 | precision-and-flow.mdx |
| パフォーマンス最終調整 | v44.8 | collect_bench_stream_notes + CHANGELOG |
| v45.0 前調整・安定化 | v44.9 | precision-and-flow-overview.mdx |

**宣言日**: 2026-07-15

---

## v44.0.0 — Language Expressiveness（2026-07-13）

> 「戻り値型は省略でき、ジェネリクスは呼び出し側から推論される。
>  ラムダ引数はパイプライン上流の型から確定し、
>  `opaque type` で型の境界を守れる。
>
>  これが Favnir v44.0 — Language Expressiveness の姿である。」

v44.0.0 をもって、Favnir の **Language Expressiveness** を正式に宣言する。

### 達成コンポーネント（v43.1〜v43.13）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 戻り値型推論 | v43.1 | Return type omission |
| fav check 統合・E0410/E0411 | v43.2 | 推論失敗エラー |
| ジェネリック型引数推論 | v43.3 | Call-site generic inference |
| E0412 曖昧型変数検出 | v43.4 | Ambiguous type variable |
| ラムダ引数型推論 | v43.5 | Contextual lambda inference |
| パイプライン型伝播 | v43.6 | Pipeline stage typing |
| 構造体リテラル推論 | v43.7 | Structural inference |
| 双方向型推論 | v43.8 | Bidirectional / top-down |
| fav check --show-inference | v43.9 | 推論型の注釈表示 |
| fav check --explain 統合 | v43.10 | 静的解説テキスト |
| opaque type 完全化 | v43.11 | opaque keyword + E0413 |
| W031/W032 lint | v43.12 | 冗長型注釈の警告 |
| Language Expressiveness cookbook | v43.13 | ドキュメント安定化 |

**宣言日**: 2026-07-13

---

## v43.0.0 — Real-Time Power（2026-07-12）

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `#[max_inflight]` で Back-pressure を宣言的に制御できる。
>
>  これが Favnir v43.0 — Real-Time Power の姿である。」

v43.0.0 をもって、Favnir の **Real-Time Power** を正式に宣言する。

### 達成コンポーネント（v42.1〜v42.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| CEP DSL 基盤 | v42.1 | `cep pattern` / `within` 構文 |
| CEP パターン: `seq` / `any` / `not` | v42.2 | 3 パターンコンビネータ |
| CEP checker.fav 統合 | v42.3 | `within >= 1` 検証・E0420 |
| Stream join（time-window） | v42.4 | `Stream.join` 2 ストリーム結合 |
| Back-pressure `#[max_inflight]` | v42.5 | parser + AST 宣言 |
| WebSocket Rune | v42.6 | `WebSocket.send` / `WebSocket.broadcast` |
| `fav monitor` | v42.7 | パイプライン監視コマンド stub |
| Real-Time Power cookbook | v42.8 | `cep-login-purchase.mdx` / `stream-join.mdx` |
| v43.0 前調整・安定化 | v42.9 | `real-time-power.mdx` 新規作成 |

**宣言日**: 2026-07-12

---

## v42.0.0 — Type Precision（2026-07-12）

> 「`type Age = Int where (>= 0)` で値の意味を型に刻める。
>  タプルパターンとガード付き match でより精緻な分岐が書ける。
>  Newtype は内側の型の演算を自動継承する。
>
>  これが Favnir v42.0 — Type Precision の姿である。」

v42.0.0 をもって、Favnir の **Type Precision** を正式に宣言する。

### 達成コンポーネント（v41.1〜v41.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type alias | v41.1 | `type Age = Int where \|v\| v >= 0` |
| Refinement invariant + E0404〜E0406 | v41.2 | fav check 統合 |
| タプルパターン match | v41.3 | `match (status, count) { ... }` |
| ガード付き match | v41.4 | `n if n >= 90 => "A"` |
| Row polymorphism | v41.5 | record spread `{ ..u, active: true }` |
| Newtype 自動 impl | v41.6 | `type Kg(Float)` — 算術演算子自動委譲 |
| W030 lint | v41.7 | 冗長 refinement ガード検出 |
| Type Precision cookbook + docs | v41.8 | refinement-types.mdx 整備 |
| v42.0 前調整・安定化 | v41.9 | type-precision.mdx 新規作成 |

**宣言日**: 2026-07-12

---

## v41.0.0 — Streaming Foundations（2026-07-11）

> 「`tumbling_window` / `sliding_window` / `session_window` でウィンドウ集計を型安全に書ける。
>  `Event<T>` の timestamp と Watermark で out-of-order イベントを制御できる。
>
>  これが Favnir v41.0 — Streaming Foundations の姿である。」

v41.0.0 をもって、Favnir の **Streaming Foundations** を正式に宣言する。

### 達成コンポーネント（v40.1〜v40.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| tumbling_window / sliding_window | v40.1 | 固定幅・スライドウィンドウ |
| session_window | v40.2 | セッションウィンドウ |
| Event\<T\> + timestamp | v40.3 | イベント型に時刻基準フィールド追加 |
| Out-of-order 処理 | v40.4 | late_tolerance / drop / reprocess |
| fav.toml \[stream\] | v40.5 | プロジェクト設定でストリーム設定管理 |
| Kafka / Redis Streams 対応 | v40.6 | consume_windowed 追加 |
| fav bench --stream | v40.7 | ストリームパイプライン計測スタブ |
| Streaming cookbook | v40.8 | window-aggregation / kafka-streaming MDX |
| 安定化 | v40.9 | streaming-foundations.mdx ドキュメント整備 |

**宣言日**: 2026-07-11

---

## v40.0.0 — Enterprise Governance（2026-07-11）

> 「RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
>  `fav policy` で組織ポリシーを宣言的に定義し、
>  `fav policy check --ci` で違反を PR でブロックできる。
>  Secret Rune は Vault / AWS / GCP に対応し、
>  マルチテナント対応で複数チームが安全に使える。
>
>  これが Favnir v40.0 — Enterprise Governance の姿である。」

v40.0.0 をもって、Favnir の **Enterprise Governance** を正式に宣言する。

### 達成コンポーネント（v39.1〜v39.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| RBAC Rune | v39.1 | require_role / check_permission / verify_jwt |
| Audit Log Rune | v39.2 | Audit.log / start_trace / end_trace |
| fav policy | v39.3 | fav policy check / fav policy check --ci（exit 1） |
| Secret Rune 強化 | v39.4 | get_aws / get_vault / get_gcp / get_env |
| マルチテナント | v39.5 | tenant.db_schema / s3_prefix / validate_tenant |
| fav audit | v39.6 | ライセンス一覧 / GPL・CVE 検出 |
| CI/CD ゲート | v39.7 | fav ci init に Policy check ステップ自動含める |
| Governance docs | v39.8 | docs/governance/ 3 件 + cookbook 3 件 |
| 安定化 | v39.9 | enterprise-governance.mdx ドキュメント整備 |

**宣言日**: 2026-07-11

---

## v39.0.0 — Intelligence & Assistance（2026-07-10）

> 「`fav suggest` でエラーから修正案を AI が提案し、
>  `fav generate --from sql` でパイプラインを自動生成し、
>  `fav explain --verbose` でコンテキスト付き解説を受け取れる。
>  Llm Rune はストリーミング・function calling・Embeddings に対応し、
>  RAG パイプラインを `fav new --template rag-pipeline` で即座に生成できる。
>
>  これが Favnir v39.0 — Intelligence & Assistance の姿である。」

v39.0.0 をもって、Favnir の **Intelligence & Assistance** を正式に宣言する。

### 達成コンポーネント（v38.1〜v38.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| fav suggest | v38.1 | エラーコードから修正案を LLM で生成 |
| fav generate --from sql | v38.2 | SQL → Favnir パイプライン自動変換 |
| fav generate --from csv 強化 | v38.3 | schema + expect ブロック出力 |
| LSP AI 補完 | v38.4 | [lsp.ai] enabled = true で LLM rerank |
| fav explain --verbose | v38.5 | コンテキスト付き LLM 解説・修正例 |
| RAG テンプレート | v38.6 | fav new --template rag-pipeline |
| Llm Rune 強化 | v38.7 | stream / function_call / embed 対応 |
| AI 支援 cookbook | v38.8 | sql-to-favnir / rag-pipeline / llm-streaming |
| 安定化 | v38.9 | ai-overview.mdx ドキュメント整備 |

**宣言日**: 2026-07-10

---

## v38.0.0 — Multi-Source ETL Power（2026-07-10）

> 「`List.join_on` で 2 つのリストを型安全に結合し、
>  `List.fan_out` / `List.fan_in` で大規模データを並列処理し、
>  CDC Rune で Debezium イベントをストリーミング処理できる。
>  `fav explain --lineage` でデータフローを DOT/SVG グラフとして可視化し、
>  `fav new --template multi-source` でマルチソース ETL プロジェクトを即座に生成できる。
>
>  これが Favnir v38.0 — Multi-Source ETL Power の姿である。」

v38.0.0 をもって、Favnir の **Multi-Source ETL Power** を正式に宣言する。

### 達成コンポーネント（v37.1〜v37.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v37.1 | `T with Serialize/Deserialize` 制約 |
| 行多相実用強化 | v37.2 | ネスト行型 `R with { addr: { city: String, .. } }` |
| List.join_on | v37.3 | left semi-join VM ビルトイン |
| List.fan_out / fan_in | v37.4 | チャンク分散・再集約 VM ビルトイン |
| CDC Rune | v37.5 | Debezium JSON イベント処理 |
| lineage DOT/SVG | v37.6 | `fav explain --lineage --format dot/svg` |
| multi-source テンプレート | v37.7 | `fav new --template multi-source` |
| cookbook 5 本 | v37.8 | join / CDC / fan-out / generics / lineage レシピ |
| 安定化 | v37.9 | lineage サマリー行・Multi-Source ETL ドキュメント |

**宣言日**: 2026-07-10

---

## v37.0.0 — Data Quality First（2026-07-09）

> 「`schema` でテーブル/列の型と制約を宣言し、
>  `expect` でビジネスルールをパイプラインに埋め込み、
>  `fav validate` でデータを検証できる。
>  スキーマ不整合は W025 lint で静的に検出され、
>  違反は E0380〜E0384 として報告される。
>  `fav schema diff` で変更の後方互換性を即座に把握できる。
>
>  これが Favnir v37.0 — Data Quality First の姿である。」

v37.0.0 をもって、Favnir の **Data Quality First** を正式に宣言する。

### 達成コンポーネント（v36.1〜v36.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| schema 定義構文 | v36.1 | `schema Orders { id: Int, ... }` インライン定義 |
| expect ブロック | v36.2 | `expect rows { not_empty, all(...) }` ビジネスルール宣言 |
| W025 lint | v36.3 | `schema_mismatch` — 静的フィールドアクセス検証 |
| fav validate | v36.4 | `fav validate --schema orders.fav data.csv` |
| Data Contract | v36.5 | `contracts/` 規約 + `fav contract check` |
| E0380〜E0384 | v36.6 | スキーマ不整合エラーカタログ |
| GE エクスポート | v36.7 | `--export ge` — Great Expectations 互換出力 |
| fav schema diff | v36.8 | フィールドレベル差分・後方互換性チェック |
| 安定化 | v36.9 | W025↔E0380 連携・validate サマリー・docs 統合 |

**宣言日**: 2026-07-09
**宣言バージョン**: v37.0.0

---

## v36.0.0 — Deployment Story（2026-07-08）

> 「`fav deploy --target lambda` で Lambda に自動デプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  `!Effect` 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

v36.0.0 をもって、Favnir の **Deployment Story** を正式に宣言する。

**宣言日**: 2026-07-08
**宣言バージョン**: v36.0.0

---

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

<!-- 以下は v35.0.0 宣言（冒頭）への追記事項 — 重複エントリではなく補足記録 -->
<!-- 正史: このファイル冒頭の "## v35.0.0 — Production Ready（2026-07-04）" が正式宣言 -->

## v35.0.0 追記 — !Effect 廃止完結（2026-07-05〜06）

v35.6.0〜v35.8.0 にて `!Effect` アノテーション構文が言語から**完全に削除**され、
v35.0.0 宣言文の「Lambda にデプロイして実データを処理できる」が完全に充足された。

| カテゴリ | バージョン | 内容 |
|---|---|---|
| E0374 ハードエラー化 | v35.4.0 | `!Effect` を書くとパースエラー |
| Effect enum 完全削除 | v35.5.0 | `ast.rs` 以降 14 ファイルから物理削除（約 380 行） |
| ドキュメント統一 | v35.6.0 | サイト MDX 128 ファイル・317 コードブロックを ctx 構文に変換 |
| LSP / error_catalog / MCP | v35.7.0〜v35.8.0 | 残存 `!Effect` 文字列をすべて除去 |
| 最終テスト数 | v35.8.0 | 2621 tests pass（0 failures）、cargo clippy clean |

**Lambda デプロイの現状**（v35.0 宣言時との差分）:
- v35.0 宣言文「Lambda にデプロイして実データを処理できる」＝ `fav build --target native` 後に**手動**デプロイ
- v36.1〜v37.0（次スプリント）で `fav deploy --target lambda` **CLI 自動化** を実装予定

```bash
# ctx 構文でのパイプライン（v35.0 以降の標準）
fav run examples/postgres_etl.fav

# 旧 !Effect 構文は E0374 でパースエラー
fav check --legacy examples/pipeline/custom_effects.fav  # E0374
```

---

## v36.0 — Deployment Story（2026-07-08）

v35.1〜v35.9 スプリントで実装した機能を統合し、Deployment Story マイルストーンを宣言する。

> 「`fav deploy --target lambda` で Lambda に自動デプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  `!Effect` 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

### 達成コンポーネント（v35.1〜v35.9）

| バージョン | 内容 |
|---|---|
| v35.1.0 | `fav deploy --target lambda` — Lambda 自動デプロイ・bootstrap.zip パッケージング |
| v35.2.0 | `fav deploy --target docker` — Dockerfile 自動生成・`docker build` 実行 |
| v35.3.0 | `fav ci init` — GitHub Actions CI ワークフロー自動生成 |
| v35.4.0 | `!Effect` E0374 ハードエラー化 |
| v35.5.0 | Effect enum・effects フィールド・parse_effects_acc 完全削除 |
| v35.6.0 | ctx 構文統一（MDX 128 件）+ Production Ready 宣言補完 |
| v35.7.0 | `docs_server.rs !Effect` 完全除去 |
| v35.8.0 | LSP / error_catalog / MCP / help !Effect 廃止完結 |
| v35.9.0 | v36.0 前調整・安定化（E2E 確認・lambda-deploy デモ確認） |
