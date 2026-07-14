# Favnir

**Favnir** はデータパイプラインの構築・解析に特化した、型安全なパイプラインファースト言語です。

企業のデータはサイロ化しています。SAP・DB・CSV・API——それぞれ「接続」はできても、
型がなく、境界が見えず、スキーマ変更が静かに下流を壊す。
そこに型とエフェクトで境界を引き、パイプラインを設計図として表現できる言語を作りたかった。
Favnir はその答えです。

---

## なぜ Favnir を作ったのか

Favnir が生まれるまでには、3つの試みがありました。

**1. RINQ — Rust 版 LINQ クエリビルダ**

C# の LINQ のように Rust でコレクション操作を書きたいと考え、クレートとして開発しました。
しかし Reddit でのフィードバックは「なぜ標準ライブラリの拡張ではなく新規クレートなのか」でした。
この問いに答えるためには、ライブラリではなく言語レベルの解決が必要だと気づきました。

**2. ForgeScript — Rust のラッパー言語**

実行とビルドの両方に対応した Rust ラッパー言語を開発しました。
しかし Rust を完全に置き換えるには、セキュリティや低レイヤー領域に精通したエンジニアが不可欠で、
個人プロジェクトとして維持するには範囲が広すぎました。

**3. Favnir — スコープを絞った専用言語**

「データ基盤とデータパイプラインの構築・解析」に特化し、
重い部分（VM・バイトコード実行）は Rust に委ね、
言語ロジック（コンパイラ・型チェッカー）は Favnir 自身で書く
**ハイブリッドセルフホスト**戦略を採用しました。

> 失敗から学んだ核心：「スコープを絞ることが言語の強さになる」

v9.0.0（2026-05-30）で、セルフホスト完成を宣言しました。
`fav check` も `fav run` も、すべての経路が Favnir 自身の型チェッカー・コンパイラを経由して動きます。
v10.0.0（2026-06-03）で、OSS 公開準備が完了しました。
v12.0.0（2026-06-06）で、Python トランスパイラ（`fav transpile --target python`）が完成しました。
v13.0.0（2026-06-09）で、言語信頼性宣言を完了しました。
型安全・エラー伝播・デバッグ可視性の三点において、Favnir のランタイム挙動は型システムの宣言と一致することを保証します。
また、`fav check --json` と `fav doc --builtins --format json` を用いて AI ツールが自律的にコードを修正できることを確認しました。
v14.0.0（2026-06-11）で、能力型完成宣言を完了しました。
副作用は通常の型システムで表現されます。`capability 引数がなければ純粋` が言語レベルで保証され、`!Effect` アノテーション構文は v35.4.0 で削除されました（コンパイルエラー E0374）。
新しいクラウドサービスの追加は `interface` に `impl` を追加するだけで完了します。`Ctx.mock(...)` により AI ツールが本番接続なしにパイプライン全体をテストできます。
v14.1.0〜v14.5.0（2026-06-12）で、クロスクラウド基盤を整備しました。
Azure DB for PostgreSQL・Azure Blob Storage のネイティブ対応、AWS Secrets Manager 統合、
および CrossCloud E2E デモ（v15.0.0）に向けた Rune エコシステムを拡充しました。
v14.8.0（2026-06-12）で、Rune ファイル整備（--legacy 明示化 + fs.fav バグ修正）を完了しました。
v15.0.0〜v15.1.5（2026-06-13〜14）で、CrossCloud E2E デモ + 認証層（HMAC / KMS ECDSA P-256）を実証しました。
v15.2.0〜v15.4.0（2026-06-14）で、GCP BigQuery・`fav test` DSL・Kafka/MSK Rune を追加しました。
v15.5.0（2026-06-14）で、`fav deploy`（AWS Lambda デプロイ CLI）を完成しました。
v16.0.0（2026-06-14）で、**Production Multi-Cloud** マイルストーンを宣言しました。
AWS / Azure / GCP / Snowflake の 4 クラウドと Kafka/MSK ストリーミングを型安全なパイプラインで統一的に扱えます。
v16.1.0〜v16.8.0（2026-06-14）で、**Language Ergonomics** シリーズを完了しました。
f-string 補間 / record spread / stdlib 拡充（DateTime / List.sort_by 等）/ 型エイリアス / namespace alias / `assert_eq` / `test_group` / snapshot テスト / `|> tap(fn)` 演算子が揃い、「書きたくなる言語」への転換を実現しました。
v17.0.0（2026-06-14）で、**Language Ergonomics** マイルストーンを宣言しました。
v17.1.0〜v17.8.0（2026-06-15〜16）で、**Language Power** シリーズを完了しました。
境界付きジェネリクス（`fn f<T with Ord>(...)` ）/ パターンマッチ拡張（or-pattern / list-pattern）/ コレクション内包表記 / `forall` プロパティテスト / パッケージシステム（`fav add` / `fav publish`）が揃い、「言いたいことを言える言語」への転換を実現しました。
v18.0.0（2026-06-16）で、**Language Power** マイルストーンを宣言しました。
v18.1.0〜v18.8.0（2026-06-16）で、**Type System Maturity** シリーズを完了しました。
エフェクト推論 / 行多相 / Refinement Types / スキーマ型 / 線形型 / 共変・反変アノテーション / Const Generics / 型駆動 API 生成が揃い、「信頼できる言語」への転換を実現しました。
v19.0.0（2026-06-16）で、**Type System Maturity** マイルストーンを宣言しました。
v19.1.0〜v19.8.0（2026-06-17）で、**Production Performance** シリーズを完了しました。
遅延評価パイプライン（`#[streaming]`）/ AOT コンパイル（Cranelift）/ インクリメンタルコンパイル / 並列コンパイル / Apache Arrow 統合 / WASM 最適化 / 事前コンパイル（`.favc`）/ フレームグラフプロファイリングが揃い、「本番で速い言語」への転換を実現しました。
v20.0.0（2026-06-17）で、**Production Performance** マイルストーンを宣言しました。
v20.1.0〜v20.8.0（2026-06-18〜20）で、**Runtime Excellence** シリーズを完了しました。
スーパー命令（opcode 融合）/ NaN-boxing（VMValue 8 bytes 圧縮）/ DuckDB プッシュダウン（SQL 自動委譲）/ mmap+SIMD CSV / io_uring / Arena アロケータ / Postgres コネクションプールが揃い、「限界まで速い VM」への転換を実現しました。
v21.0.0（2026-06-20）で、**Runtime Excellence** マイルストーンを宣言しました。
v21.1.0〜v23.8.0（2026-06-20〜22）で、**Developer Tooling** / **Distributed Scale** / **VM in Favnir** シリーズを完了しました。
v24.0.0（2026-06-23）で、**VM in Favnir** マイルストーンを宣言しました。
v24.1.0〜v24.8.0（2026-06-23〜24）で、形式的仕様書生成 / Bootstrap 検証 / パフォーマンス回帰検知 / 後方互換性ポリシー / Rune レジストリ 50+ / セキュリティ審査 / ドキュメントサイト v2 / テンプレートギャラリーを完了しました。
**v25.0.0（2026-06-24）で、[Practical Self-Hosting](./MILESTONE.md) マイルストーンを宣言しました。**
コンパイラ・型チェッカー・CLI・VM 仕様のすべてが Favnir で実装され、v1.0 リリース候補となりました。
**v26.0（2026-06-26）で、[Rune Foundation](./MILESTONE.md) マイルストーンを宣言しました。**
コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が完全実装され、`fav run examples/full_etl.fav` が実際のデータを動かせるようになりました。
**v27.0（2026-06-27）で、[Streaming Native](./MILESTONE.md) マイルストーンを宣言しました。**
ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）が実質化され、`Stream.*` 操作 6 関数と E2E デモ 3 本が Docker Compose で動作します。
**v28.0（2026-06-27）で、[Data Lakehouse](./MILESTONE.md) マイルストーンを宣言しました。**
Delta Lake / Iceberg テーブルの読み書き、dbt モデル参照、主要 DWH 3 本（ClickHouse / BigQuery / Redshift）への接続、SQLite 組み込み DB に対応し、現代データ基盤アーキテクチャへの完全統合を達成しました。
**v29.0（2026-06-28）で、[Observability First](./MILESTONE.md) マイルストーンを宣言しました。**
prometheus / datadog / sentry / grafana / otel の 5 Rune が揃い、`#[track]` / `#[trace]` / `#[on_error]` アノテーションと E2E デモ 3 本が Docker Compose で動作します。`#[track(latency, error_rate)]` を stage に付けるだけで Grafana ダッシュボードにメトリクスが現れます。
**v30.0（2026-07-01）で、[Ecosystem Maturity](./MILESTONE.md) マイルストーンを宣言しました。**
Rune Registry が本番稼働し、コミュニティ投稿 Rune 10 本（stripe / twilio / notion 等）が公開されました。`fav add stripe` で Stripe 連携 Rune が 5 分で動く状態を達成しました。
**v31.0（2026-07-02）で、[Real-World Readiness](./MILESTONE.md) マイルストーンを宣言しました。**
`fav new --template postgres-etl` による 4 ファイル構成テンプレートが生成され、`fav check` / `fav test` / `fav lint` が全通過します。`examples/csv-to-postgres/` の実証パイプラインが CSV 1000 行を Postgres に書き込みます。
**v32.0（2026-07-03）で、[Language Polish](./MILESTONE.md) マイルストーンを宣言しました。**
エラーメッセージが rustc スタイルに刷新され、`fav explain E0001` でエラー詳細を確認できます。REPL が `:doc` / `:history` / タブ補完を備え、`fav test --watch` / `fav check --all` / `fav scaffold` が揃いました。
**v33.0（2026-07-03）で、[Language Power](./MILESTONE.md) マイルストーンを宣言しました。**
境界付きジェネリクス（`T with Ord`）・行多相（`R with { id: Int }`）・`where` 制約・スキーマ型・線形型・分散アノテーション・定数ジェネリクス・型駆動 API 生成・エフェクト推論が揃い、型で設計するデータパイプラインが現実になりました。
**v34.0（2026-07-04）で、[Performance & Tooling](./MILESTONE.md) マイルストーンを宣言しました。**
AOT ネイティブバイナリ / インクリメンタルコンパイル / ストリーミング評価 / Arrow 統合 / WASM 最適化 / 並列コンパイルが揃い、「本番で速い」データパイプラインが実現しました。
**v35.0（2026-07-04）で、[Production Ready](./MILESTONE.md) マイルストーンを宣言しました。**
実案件デモ / cookbook 50 本 / ベンチマーク公開 / セキュリティ審査 v2 / エフェクトシステム統一（`!Effect` → AppCtx）が揃い、実際のデータエンジニアリング案件で Favnir を選択できる状態になりました。
**v36.0（2026-07-08）で、[Deployment Story](./MILESTONE.md) マイルストーンを宣言しました。**
`fav deploy --target lambda/docker` / `fav ci init` / ctx 構文統一（`!Effect` 廃止）が揃い、Lambda 本番デプロイと GitHub Actions CI が自動化されました。
**v37.0（2026-07-09）で、[Data Quality First](./MILESTONE.md) マイルストーンを宣言しました。**
`schema` 型定義 / `expect` 品質ルール / `fav validate` / W025 lint / E0380〜E0384 / GE エクスポート / `fav schema diff` が揃い、型でデータ品質を保証できる状態になりました。
**v38.0（2026-07-10）で、[Multi-Source ETL Power](./MILESTONE.md) マイルストーンを宣言しました。**
`List.join_on` / `List.fan_out` / CDC Rune / lineage DOT/SVG / `fav new --template multi-source` が揃い、複数ソースを型安全につなぐマルチソース ETL が完成しました。
**v39.0（2026-07-10）で、[Intelligence & Assistance](./MILESTONE.md) マイルストーンを宣言しました。**
`fav suggest` / `fav generate --from sql` / `fav explain --verbose` / Llm Rune（stream / embed）/ `fav new --template rag-pipeline` が揃い、AI がパイプラインを補助できる状態になりました。
**v40.0（2026-07-11）で、[Enterprise Governance](./MILESTONE.md) マイルストーンを宣言しました。**
RBAC Rune / Audit Log / `fav policy check --ci` / Secret Rune（Vault / AWS / GCP）/ マルチテナント対応 が揃い、チームで安全に運用できる Enterprise Governance 基盤が完成しました。

**v41.0（2026-07-11）で、[Streaming Foundations](./MILESTONE.md) マイルストーンを宣言しました。**
`tumbling_window` / `sliding_window` / `session_window` によるウィンドウ集計、`Event<T>` と Watermark による out-of-order 制御、Kafka・Redis Streams `consume_windowed` 対応が揃い、型安全なストリーミング基盤が完成しました。

**v42.0（2026-07-12）で、[Type Precision](./MILESTONE.md) マイルストーンを宣言しました。**
Refinement type alias（`type Age = Int where |v| v >= 0`）/ タプルパターン・ガード付き match / Newtype 自動 impl / W030 lint が揃い、型でデータの意味を精緻に表現できる Type Precision 基盤が完成しました。

**v43.0（2026-07-12）で、[Real-Time Power](./MILESTONE.md) マイルストーンを宣言しました。**
CEP（`seq(Login, Purchase) within 300`）/ Stream join / Back-pressure / WebSocket Rune / fav monitor が揃い、サブ秒レイテンシのリアルタイムパイプラインを型安全に記述できる Real-Time Power 基盤が完成しました。

**v44.0（2026-07-13）で、[Language Expressiveness](./MILESTONE.md) マイルストーンを宣言しました。**
型推論 6 カテゴリ（戻り値型・ジェネリクス・ラムダ・パイプライン・構造体・双方向）/ opaque type / W031/W032 lint が揃い、型注釈を最小化しながら型安全性を維持できる Language Expressiveness 基盤が完成しました。

**v45.0（2026-07-15）で、[Precision & Flow](./MILESTONE.md) マイルストーンを宣言しました。**
Refinement type × Streaming / CEP × Opaque type / Back-pressure / E2E デモが揃い、最小限の型注釈で安全なリアルタイムパイプラインを記述できる Precision & Flow 基盤が完成しました。

v34.5.0〜v34.7.0 で、`!Effect` アノテーションを廃止し Capability Context（AppCtx）に一本化しました。
`fav migrate --from-effects` で既存コードを自動移行できます。

---

## 言語の思想

Favnir は **Convention over Configuration** をパイプライン構造に適用した言語です。

通常の言語では、関数の合成は「ライブラリの慣習」に過ぎず、ツールからは「ただの関数呼び出し」にしか見えません。
Favnir では `stage`（変換）と `seq`（パイプライン）が言語プリミティブです。

```favnir
// stage: 型契約とエフェクトを持つ変換の単位
stage ParseCsv: String -> List<Row> = |s| { /* ... */ }

stage ValidateRow: Row -> Row = |row| { /* ... */ }

stage SaveToDb: Row -> Int = |row| { /* ... */ }

// seq: 名前を持つデータフローの構造
seq UserImport = ParseCsv |> ValidateRow |> SaveToDb
```

`seq UserImport` は関数合成の結果ではなく、**名前を持つアーキテクチャの単位**です。
これにより、コンパイラがパイプライン構造を理解し、以下が実現できます:

- **エフェクトの静的追跡** — どの段階で I/O・DB・イベント発行が起きるか
- **`fav explain` による可視化** — パイプライン構造をそのまま設計図として出力
- **`abstract seq` による依存注入** — 型安全なスロット差し替え

---

**v24.0.0（2026-06-23）— VM in Favnir マイルストーン宣言**

テスト: **1926 件以上**

### VM in Favnir 達成実績（v23.x）

| 機能 | バージョン | 概要 |
|---|---|---|
| Bytes 型 | v23.1.0 | `Bytes.from_hex / get / slice / concat / to_utf8 / read_u16_le` 等 13 操作 |
| ビット演算 | v23.2.0 | `Int.bit_and / bit_or / bit_xor / bit_not / shift_left / shift_right` + 16進数リテラル |
| Mut<T> 可変コレクション | v23.3.0 | `Mut.list / map / push / pop / get / set / peek` — VM スタック・ローカル変数テーブル |
| vm.fav Phase 1 | v23.4.0 | バイトコードデコード（27 opcode）、`Bytes.read_u16_le` |
| vm.fav Phase 2 | v23.5.0 | スタックベース実行ループ・VMVal 型（3 バリアント） |
| vm.fav Phase 3 | v23.6.0 | 制御フロー（Jump/JumpIfFalse）・ローカル変数（LoadLocal/StoreLocal）・残余演算 |
| vm.fav Phase 4 | v23.7.0 | builtin ディスパッチ（call_builtin）・VMStr 追加・LoadGlobal/Call |
| vm.fav Phase 5 | v23.8.0 | GetField・collect_args・vmval_display・任意 argc 対応 |
| `fav run --vm` CLI | v24.0.0 | `fav run --vm <vm_path> --hex <hex>` で vm.fav 経由実行 |

---

## 現在の状態

**v23.0.0（2026-06-21）— Distributed Scale マイルストーン宣言**

テスト: **1887 件すべて通過**

### Distributed Scale 達成実績（v22.x）

| 機能 | バージョン | 概要 |
|---|---|---|
| Checkpoint / Resume | v22.1.0 | `#[checkpoint]` アノテーション、`.favc` 形式でステージ出力保存、`fav run --resume` で中断後再開 |
| Distributed `par` | v22.2.0 | `par_distributed [A, B, C]` で gRPC Worker に分散実行、`fav.toml [workers]` セクション |
| Pipeline State Rune | v22.3.0 | `import rune "state"`、Redis / DynamoDB / PostgreSQL バックエンドの型付き分散キャッシュ |
| Event-driven Pipeline | v22.4.0 | `#[trigger(event = "s3:ObjectCreated")]` / `"kafka:message"`、`fav deploy --trigger` |
| Pipeline Orchestration | v22.5.0 | `pipeline { step ... after ... }` DAG 構文、`fav orchestrate run/status/retry` |
| SLA 宣言 | v22.6.0 | `#[timeout]` / `#[retry]` / `#[circuit_breaker]`（コンパイル時チェック）、`fav explain --sla` |
| OpenTelemetry 統合 | v22.7.0 | `fav run --trace` で自動 span 生成・OTLP HTTP エクスポート、Jaeger / Grafana Tempo 対応 |
| `fav deploy` 強化 | v22.8.0 | `--target ecs/k8s/fly`、Dockerfile / ECS task def / K8s CronJob YAML / fly.toml 生成 |

### Developer Tooling 達成実績（v21.x）

| 機能 | バージョン | 概要 |
|---|---|---|
| DAP デバッガー | v21.1.0 | VS Code / Neovim からブレークポイント・ステップ実行（`fav dap`、ポート 5678） |
| `fav explain` 可視化強化 | v21.2.0 | Mermaid / D2 / JSON 形式のリネージ図出力（`--format mermaid`） |
| テストカバレッジ | v21.3.0 | HTML レポート + LCOV 出力（`fav test --coverage`） |
| `fav lint` 強化 | v21.4.0 | W010〜W019（ネスト深度・magic number・文字列連鎖など 10 ルール追加） |
| LSP コードアクション強化 | v21.5.0 | codeAction（3 種: addMissingImport / convertToFstring / inlineBinding）/ rename（全参照追跡）/ references |
| Playground v2 | v21.6.0 | 共有 URL・テンプレートギャラリー・実行統計 |
| `fav doc` サイト生成 | v21.7.0 | `///` コメントから静的 HTML 自動生成（`--serve` でローカルプレビュー） |
| `fav migrate` 強化 | v21.8.0 | `--from v13 --to v14`・`--config fav.toml`・移行サマリー |

### Runtime Excellence 達成実績（v20.x）

| ベンチマーク | v20.0.0 ベースライン | v21.0.0 実績 | 目標 |
|---|---|---|---|
| cold_start_precompiled | 18ms | **8ms** | < 10ms ✅ |
| csv_10gb_throughput | ~340 MB/s | **1.2 GB/s** | > 1 GB/s ✅ |
| tight_loop_10m_iter | ~85ms | **26ms** | < 30ms ✅ |
| record_transform_1m | ~210ms | **72ms** | < 80ms ✅ |
| duckdb_query（集計） | ~45ms（VM） | **3ms（pushdown）** | < 5ms ✅ |

| 機能 | バージョン | 概要 |
|---|---|---|
| スーパー命令 | v20.2.0 | top-10 opcode ペア融合（LoadAdd / AddStore 等）、ディスパッチ削減 |
| NaN-boxing | v20.3.0 | VMValue を 8 bytes に圧縮、Vec<VMValue> のキャッシュヒット率大幅改善 |
| DuckDB pushdown | v20.4.0 | Filter / GroupBy / SumBy / Project / Count を SQL 自動委譲 |
| mmap + SIMD CSV | v20.5.0 | memmap2 + arrow-csv でゼロコピーパース（csv +4x） |
| io_uring | v20.6.0 | Linux 5.1+ での非同期リング I/O、他 OS は自動フォールバック |
| Arena アロケータ | v20.7.0 | 1 chunk（1000 行）単位の一括解放、定常メモリ -20% |
| Postgres コネクションプール | v20.8.0 | `Postgres.Pool.*`、接続確立コスト排除（-50ms/stage〜） |

### Production Performance 達成実績（v19.x）

| 機能 | バージョン | 概要 |
|---|---|---|
| `#[streaming]` 遅延評価 | v19.1.0 | 定常メモリで 10GB+ CSV を処理 |
| AOT コンパイル（Cranelift） | v19.2.0 | `fav build --target native` でネイティブバイナリ生成 |
| インクリメンタルコンパイル | v19.3.0 | 変更ファイルのみ再コンパイル（SHA-256 フィンガープリント） |
| 並列コンパイル | v19.4.0 | Rayon + petgraph でトポロジカル並列ビルド |
| Apache Arrow 統合 | v19.5.0 | `ArrowBatch.write_parquet` / `read_parquet` |
| WASM 最適化 | v19.6.0 | デッドコード除去によるバイナリサイズ削減 |
| 事前コンパイル `.favc` | v19.7.0 | `fav run --precompiled` で Lambda コールドスタート削減 |
| フレームグラフ | v19.8.0 | `fav profile --format=flamegraph` で SVG 生成 |

### ベンチマーク参考値

```
# 10GB CSV ストリーミング処理（定常メモリ）
fav run --streaming pipeline.fav large.csv
→ ピークメモリ: ~50MB（chunk_size=1000）

# Lambda コールドスタート（事前コンパイル）
fav compile pipeline.fav && fav run --precompiled pipeline.favc
→ 型チェック・コンパイルをスキップ（コールドスタート削減）

# native ビルドの実行速度
fav build --target native pipeline.fav
→ VM インタープリタ比で高速実行
```

詳細は `benchmarks/results.md` を参照。

| 機能カテゴリ | 機能 | 状態 |
|---|---|---|
| **言語コア** | 型チェッカー（ジェネリクス・HM 型推論） | ✓ |
| | パターンマッチ（ネスト・ガード・バリアント） | ✓ |
| | Capability Context（`ctx: LoadCtx` / `ctx: AppCtx` 等） | ✓ |
| | 名目型ラッパー（`type UserId(Int)` + `where` バリデーター） | ✓ |
| | `interface` / `impl ... for` / `type T with Iface` | ✓ |
| | `par [A, B] \|> Merge` 並列 stage 実行 | ✓ |
| | `collect` / `yield` / クロージャ / `expr?` | ✓ |
| | f-string 補間（`f"Hello, {name}!"`、`f"""..."""` 三重クォート）（v16.2.0） | ✓ |
| | レコード更新構文（`{ ...base, field: val }`）（v16.3.0） | ✓ |
| | 型エイリアス（`alias Email = String`、ジェネリクス対応）（v16.5.0） | ✓ |
| | Namespace Alias（`use String as S`）（v16.6.0） | ✓ |
| | **Bounded Generics**（`fn f<T with Ord>(a: T, b: T) -> T`）（v17.1.0） | ✓ |
| | **パターンマッチ拡張**（or-pattern `"a" \| "b"` / list-pattern `[head, ..tail]` / guard）（v17.2.0） | ✓ |
| | **コレクション内包表記**（`[x * 2 \| x <- list]` / `[? f(x) \| x <- xs]`）（v17.3.0） | ✓ |
| | `bind x <- expr` バインディング統一（非 Result・Result 両対応）（v17.4.0） | ✓ |
| **パイプライン** | `stage` / `seq` / `\|>` | ✓ |
| | `\|> tap(observer_fn)` / `\|> inspect`（デバッグ tap、`--no-tap` で本番ゼロコスト）（v16.8.0） | ✓ |
| | `abstract stage` / `abstract seq`（依存注入） | ✓ |
| | `fav explain --lineage`（静的リネージ解析） | ✓ |
| **Python トランスパイラ** | `fav transpile --target python` — Fav → Python + `pyproject.toml` 自動生成（boto3 / psycopg2 対応） | ✓ |
| **テスト** | `fav test` — `assert_eq` / `test_group` / `assert_snapshot` / `--update-snapshots`（v16.7.0） | ✓ |
| | **`forall` プロパティベーステスト**（`forall x: Type [where { guard }] { body }`、`--cases N`）（v17.7.0） | ✓ |
| **標準ライブラリ** | `List.sort_by` / `List.distinct` / `List.sum_by` 等 9 関数（v16.4.0） | ✓ |
| | `DateTime.now` / `DateTime.parse` / `DateTime.format` 等 12 関数（v16.4.0） | ✓ |
| | `String.format_int` / `String.format_float` / `String.split_once`（v16.4.0） | ✓ |
| | `Math.round_to` / `Math.log` / `Math.log2` / `Math.log10`（v16.4.0） | ✓ |
| **CLI ツール** | `fav run` / `fav check` / `fav test` / `fav bench`（avg / p50 / p95 / min / max、v17.6.0） | ✓ |
| | `fav fmt`（冪等コードフォーマッタ） | ✓ |
| | `fav lint`（W001〜W005 静的解析） | ✓ |
| | `fav doc`（`///` コメント → Markdown 生成） | ✓ |
| | `fav profile`（stage 別実行時間計測） | ✓ |
| | `fav watch`（ファイル監視 + 自動再実行） | ✓ |
| | **`fav repl`**（インタラクティブ REPL、`:doc` / `:load` / タブ補完、v17.5.0） | ✓ |
| | `fav new <name>`（プロジェクトスキャフォールディング） | ✓ |
| **パッケージ管理** | **`fav add` / `fav update` / `fav remove` / `fav publish`**（semver 解決、registry v2、v17.8.0） | ✓ |
| | `fav.toml` `[dependencies]` / `[dev-dependencies]` / `[registry]`（v17.8.0） | ✓ |
| **Rune エコシステム** | AWS（S3 / SQS / DynamoDB / Secrets Manager / MSK） | ✓ |
| | Azure Blob Storage（`AzureBlob.*`、Shared Key 認証） | ✓ |
| | Azure PostgreSQL（`AzurePostgres.*`、SSL 対応） | ✓ |
| | GCP BigQuery（`BigQuery.*`、RS256 JWT 認証） | ✓ |
| | Kafka / MSK（`Kafka.*`、SCRAM-SHA-512 認証） | ✓ |
| | Snowflake（`Snowflake.*`、JWT 認証） | ✓ |
| | http / grpc / graphql | ✓ |
| | llm（Claude / OpenAI） | ✓ |
| | DuckDB / SQL / DB / fs / Parquet / json / csv / gen 等 | ✓ |
| | slack / queue / cache / email / auth / log | ✓ |
| **パフォーマンス** | `#[streaming(chunk_size=N)]` 遅延評価パイプライン（定常メモリ処理、v19.1.0） | ✓ |
| | `fav build --target native`（Cranelift AOT コンパイル、v19.2.0） | ✓ |
| | インクリメンタルコンパイル（SHA-256 フィンガープリント、`.fav_cache/`、v19.3.0） | ✓ |
| | 並列コンパイル（Rayon + petgraph、v19.4.0） | ✓ |
| | `ArrowBatch` — Apache Arrow 統合 / `write_parquet` / `read_parquet`（v19.5.0） | ✓ |
| | `fav compile` / `fav run --precompiled`（Lambda コールドスタート削減、v19.7.0） | ✓ |
| | `fav profile --format=flamegraph/text/json`（inferno SVG、HOT PATH 検出、v19.8.0） | ✓ |
| **デプロイ** | `fav deploy`（AWS Lambda、zip + S3 + Lambda update） | ✓ |
| **開発体験** | LSP（hover・diagnostics・補完・go-to-definition） | ✓ |
| | Schema Authority（fav infer → T.validate） | ✓ |
| | WASM バックエンド（Playground 向け） | ✓ |
| | `rvm` 独立実行バイナリ | ✓ |
| **セルフホスト** | コンパイラ（`fav/self/compiler.fav`） | ✓ |
| | 型チェッカー（`fav/self/checker.fav`） | ✓ |
| | CLI（`fav/self/cli.fav`） | ✓ |
| | Bootstrap 検証（`bytecode_A == bytecode_B`） | ✓ |

### セルフホスト経路（v9.0.0 以降）

| 経路 | 実装 |
|---|---|
| `fav check` | checker.fav（v8.1.0〜） |
| `fav run` 単一ファイル | compiler.fav（v8.5.0〜） |
| `fav run` rune import あり | compiler.fav + ソース結合（v8.6.0〜） |
| `fav run` fav.toml プロジェクト | compiler.fav + プロジェクト収集（v8.11.0〜） |
| VM・ファイル I/O | Rust（恒久・設計上） |

Bootstrap 検証（v6.2.0 で確立・維持中）:
```
Stage 1: Rust VM で compiler.fav → hello.fav → bytecode_A
Stage 2: Rust VM で compiler.fav → compiler.fav → compiler_artifact
Stage 3: Rust VM で compiler_artifact → hello.fav → bytecode_B
検証: bytecode_A == bytecode_B ✓
```

---

## コード例

> **注記**: 以下のコード例（「基本パイプライン」「並列実行」「型バリデーション」「LLM 統合」）は
> `--legacy` モードでのみ有効な旧 `!Effect` スタイルです。
> v14.0.0 以降の標準スタイルは「Capability Context（v14.0.0〜）」セクションを参照してください。

### 基本パイプライン

```favnir
import rune "duckdb"
import rune "csv"

type Order   = { customer: String  amount: Float }
type Summary = { customer: String  total: Float }

stage LoadOrders: String -> List<Order> = |path| {
  csv.read<Order>(path)
}

stage Summarize: List<Order> -> List<Summary> = |orders| {
  List.map(orders, |o| Summary { customer: o.customer  total: o.amount })
}

// seq: 名前を持つパイプラインの構造
seq OrderReport = LoadOrders |> Summarize

// fav explain --lineage で構造を可視化:
// NAME          TYPE                         EFFECTS
// OrderReport   String -> List<Summary>
```

### 並列実行（v9.13.0〜）

```favnir
import rune "http"

stage FetchOrders: String -> List<Order>  = |conn| { /* DB から取得 */ }
stage FetchPrices: String -> List<Price> = |url|  { /* API から取得 */ }
stage Merge:       (List<Order>, List<Price>) -> Report = |pair| { /* マージ */ }

// par: 複数 stage を並列実行し、結果をタプルで次 stage に渡す
seq FullReport = par [FetchOrders, FetchPrices] |> Merge

// fav explain で:
// par[FetchOrders(), FetchPrices()] → Merge
// → DB と HTTP API を並列で読む — が静的に保証される
```

### 型バリデーション（v9.7.0〜）

```favnir
// 名目型ラッパー + where バリデーター
type Email(String)   where |v| String.contains(v, "@")
type Percent(Float)  where |v| v >= 0.0 && v <= 100.0

stage ParseInput: String -> Email = |s| {
  Email(s)  // Result<Email, String> を返す
}
```

### LLM 統合（v9.6.0〜）

```favnir
import rune "llm"

stage Summarize: String -> String = |text| {
  llm.complete("3行で要約してください:\n" + text)
}

// fav explain --lineage で:
// Effects:(read),,(S3 write) — AI依存度が静的に可視化される
```

### Capability Context（v14.0.0〜）

v14.0.0 以降、副作用は `capability 引数`（`ctx: LoadCtx` 等）で表現します。
`capability 引数がなければ純粋` が言語レベルで保証されます。

```favnir
// 旧記法（--legacy モードのみ）
fn load() -> Result<List<Row>, String> { ... }

// 新記法（v14.0.0 標準）
fn load(ctx: LoadCtx) -> Result<List<Row>, String> { ... }

// 糖衣構文
fn load(Ctx { db: DbRead }, page: Int) -> Result<List<Row>, String> { ... }
// → fn load(ctx: LoadCtx, page: Int) -> ... に脱糖

// テスト用モック
fn run_test() -> Bool {
  let ctx = Ctx.mock(MockDb.empty(), MockStorage.empty());
  let rows = load(ctx);
  Result.is_ok(rows)
}
```

```bash
# 旧記法を自動移行
fav migrate --from-effects src/pipeline.fav

# E0025 チェック（非 legacy モードで !Effect 記法を検出）
fav check pipeline.fav
```

---

## クイックスタート

```bash
git clone https://github.com/kazuma0606/favnir
cd favnir/fav
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

```bash
# 新規プロジェクト作成
fav new myproject
cd myproject
fav run src/main.fav

# 既存ファイルの操作
fav check pipeline.fav          # 型チェック
fav run pipeline.fav            # 実行
fav fmt pipeline.fav            # フォーマット
fav lint pipeline.fav           # 静的解析
fav doc src/ --out docs/        # ドキュメント生成
fav spec --format markdown > SPEC.md  # 形式的仕様書生成
fav explain --lineage pipeline.fav  # リネージ可視化
```

---

## ロードマップ

| バージョン | テーマ | 状態 |
|---|---|---|
| v4.1〜v4.12 | Rune エコシステム（DB・HTTP・AWS・LSP・MCP） | 完了 |
| v5.0.0 | AWS 本番稼働・CI/CD・リファレンスサイト | 完了 |
| v6.0.0〜v6.6.0 | セルフホスト + Bootstrap 検証 + T.validate | 完了 |
| v7.1.0〜v7.9.0 | fav explain リネージ・Rune 拡充・checker.fav HM 型推論 | 完了 |
| v8.0.0〜v8.11.0 | checker.fav/compiler.fav セルフホスト完成・全経路 Favnir pipeline 化 | 完了 |
| v9.0.0 | **セルフホスト完成宣言**・`--legacy` 非推奨化 | 完了 |
| v9.1.0〜v9.4.0 | stdlib 拡充・`fav fmt`・`fav lint`・json/csv/gen Rune | 完了 |
| v9.5.0〜v9.6.0 | http/grpc/graphql Rune（`!Http`）・llm Rune（`!Llm`） | 完了 |
| v9.7.0〜v9.8.0 | 名目型ラッパー・`where` バリデーター・`fav doc` | 完了 |
| v9.9.0〜v9.11.0 | `fav profile`・`fav watch`・`fav repl`・LSP 補完強化 | 完了 |
| v9.12.0〜v9.13.0 | `interface`/`impl` セルフホスト・`par` 並列実行 | 完了 |
| **v10.0.0** | **OSS 公開準備完了**（`fav new`・CI self-check・CONTRIBUTING/CHANGELOG） | **完了** |
| v10.1.0〜v10.9.0 | Snowflake ネイティブ対応（インフラ〜E2E デモ） | 完了 |
| **v11.0.0** | **Snowflake 統合完成宣言**・リネージ可視化・サイトドキュメント | **完了** |
| v11.1.0〜v11.4.0 | Python トランスパイラ基盤（emit_python / stage-seq / !IO / !AWS → boto3） | 完了 |
| v11.5.0〜v11.9.0 | !Postgres → psycopg2・uv 統合・checker 統合・fav2py E2E インフラ | 完了 |
| **v12.0.0** | **Python トランスパイラ完成宣言**・公式ドキュメント・CHANGELOG 整備 | **完了** |
| v13.1.0〜v13.10.0 | Capability Context 設計（interface 継承・ctx 型推論・E0020〜E0025・migrate ツール） | 完了 |
| **v14.0.0** | **能力型完成宣言** — `!Effect` 廃止・`ctx: Capability` 体系の確立・CI self-check | **完了** |
| v14.1.0〜v14.5.0 | Azure PostgreSQL / AzureCtx / Azure Blob Storage Rune / AWS Secrets Manager | 完了 |
| v14.6.0 | ドキュメント整備（README / CHANGELOG） | 完了 |
| v14.7.0 | site/ ドキュメント更新 + rune ファイル精査 | 完了 |
| **v14.8.0** | **Rune ファイル整備**（--legacy 明示化 + fs.fav バグ修正） | **完了** |
| v15.0.0 | CrossCloud E2E デモ（AWS RDS → Azure PostgreSQL / Blob）| 完了 |
| v15.1.0〜v15.1.5 | CrossCloud 認証層（HMAC + KMS ECDSA P-256 + Cognito + Lambda verifier） | 完了 |
| v15.2.0 | GCP BigQuery Rune（`!Gcp` エフェクト） | 完了 |
| v15.3.0 | `fav test` DSL（ネイティブテストフレームワーク） | 完了 |
| v15.4.0 | Kafka / MSK Rune（`!Stream` エフェクト） | 完了 |
| v15.5.0 | `fav deploy`（AWS Lambda デプロイ CLI） | 完了 |
| **v16.0.0** | **Production Multi-Cloud マイルストーン宣言** | **完了** |
| v16.1.0 | エラーメッセージ品質向上（rustc スタイル・Span・typo ヒント） | 完了 |
| v16.2.0 | f-string 文字列補間（`f"Hello, {name}!"`） | 完了 |
| v16.3.0 | レコード更新構文（`{ ...base, field: val }`） | 完了 |
| v16.4.0 | 標準ライブラリ拡充（List / String / DateTime / Math） | 完了 |
| v16.5.0 | 型エイリアス（`alias Email = String`） | 完了 |
| v16.6.0 | Namespace Alias（`use String as S`） | 完了 |
| v16.7.0 | fav test 成熟（`assert_eq` / `test_group` / `assert_snapshot`） | 完了 |
| v16.8.0 | tap / inspect パイプライン演算子（`\|> tap(fn)` / `--no-tap`） | 完了 |
| **v17.0.0** | **Language Ergonomics マイルストーン宣言** | **完了** |
| v17.1.0 | 境界付きジェネリクス（`fn f<T with Ord>(...)` / E0325） | 完了 |
| v17.2.0 | パターンマッチ拡張（or-pattern / list-pattern / guard） | 完了 |
| v17.3.0 | コレクション内包表記（`[x * 2 \| x <- list]` / result-comp） | 完了 |
| v17.4.0 | `let` 除去・`bind` 統一（非 Result 値でも `bind x <- expr`） | 完了 |
| v17.5.0 | REPL 品質向上（`:doc` / `:load` / `:paste` / タブ補完） | 完了 |
| v17.6.0 | `fav bench`（avg / p50 / p95 / min / max 統計、`--runs` / `--warmup` / `--json`） | 完了 |
| v17.7.0 | `forall` プロパティベーステスト（`--cases N` / `where { guard }`） | 完了 |
| v17.8.0 | パッケージシステム成熟（`fav add` / `fav publish` / semver 解決） | 完了 |
| **v18.0.0** | **Language Power マイルストーン宣言** | **完了** |
| v18.1.0 | エフェクト推論（Effect Inference） | 完了 |
| v18.2.0 | 行多相（Row Polymorphism） | 完了 |
| v18.3.0 | Refinement Types（引数 `where` 制約） | 完了 |
| v18.4.0 | スキーマ型（`schema "file:..."` インポート） | 完了 |
| v18.5.0 | 線形型（`-o` arrow、Connection/Tx 安全性） | 完了 |
| v18.6.0 | 共変・反変アノテーション（`<+T, -U>`） | 完了 |
| v18.7.0 | Const Generics（`const N: Int where { N > 0 }`） | 完了 |
| v18.8.0 | 型駆動 API 生成（`#[api(...)]` → OpenAPI / GraphQL） | 完了 |
| **v19.0.0** | **Type System Maturity マイルストーン宣言** | **完了** |
| v19.1.0 | 遅延評価パイプライン（`#[streaming(chunk_size=N)]` / `#[stateful]`） | 完了 |
| v19.2.0 | AOT コンパイル（Cranelift バックエンド、`fav build --target native`） | 完了 |
| v19.3.0 | インクリメンタルコンパイル（SHA-256 + `.fav_cache/`） | 完了 |
| v19.4.0 | 並列コンパイル（Rayon + petgraph トポロジカルソート） | 完了 |
| v19.5.0 | Apache Arrow 統合（`ArrowBatch` / `write_parquet` / `read_parquet`） | 完了 |
| v19.6.0 | WASM 最適化（デッドコード除去・バイナリサイズ削減） | 完了 |
| v19.7.0 | 事前コンパイル（`fav compile` / `fav run --precompiled`、Lambda 対応） | 完了 |
| v19.8.0 | プロファイリング強化（`--format=flamegraph/text/json`、inferno SVG） | 完了 |
| **v20.0.0** | **Production Performance マイルストーン宣言** | **完了** |
| v20.1.0 | ベンチマーク基盤整備（8計測スイート + CI + compare.fav） | 完了 |
| v20.2.0 | スーパー命令（top-10 opcode ペア融合） | 完了 |
| v20.3.0 | NaN-boxing（VMValue 8 bytes 圧縮） | 完了 |
| v20.4.0 | DuckDB プッシュダウン最適化パス（5パターン検出） | 完了 |
| v20.5.0 | mmap + SIMD CSV パーサー（arrow-csv） | 完了 |
| v20.6.0 | io_uring 非同期 I/O（Linux 5.1+） | 完了 |
| v20.7.0 | Arena アロケータ（chunk 単位一括解放） | 完了 |
| v20.8.0 | DB コネクションプール統合（`Postgres.Pool.*`） | 完了 |
| **v21.0.0** | **Runtime Excellence マイルストーン宣言** | **完了** |
| v21.1.0 | DAP デバッガー（`fav dap`、ポート 5678） | 完了 |
| v21.2.0 | `fav explain --format mermaid`（Mermaid / D2 / JSON） | 完了 |
| v21.3.0 | `fav test --coverage`（HTML + LCOV） | 完了 |
| v21.4.0 | `fav lint` 強化（W010〜W019） | 完了 |
| v21.5.0 | LSP コードアクション / rename / references | 完了 |
| v21.6.0 | Playground v2（共有 URL・テンプレート） | 完了 |
| v21.7.0 | `fav doc --format site`（静的 HTML 生成） | 完了 |
| v21.8.0 | `fav migrate` 強化（`--from/--to`・`--config`） | 完了 |
| **v22.0.0** | **Developer Tooling Complete マイルストーン宣言** | **完了** |
| v22.1.0 | Checkpoint / Resume（`#[checkpoint]` / `--resume`） | 完了 |
| v22.2.0 | Distributed `par`（gRPC Worker 分散実行） | 完了 |
| v22.3.0 | Pipeline State Rune（Redis / DynamoDB / PostgreSQL） | 完了 |
| v22.4.0 | Event-driven Pipeline（`#[trigger]` / S3 / Kafka） | 完了 |
| v22.5.0 | Pipeline Orchestration（`fav orchestrate` DAG） | 完了 |
| v22.6.0 | SLA 宣言（`#[timeout]` / `#[retry]` / `#[circuit_breaker]`） | 完了 |
| v22.7.0 | OpenTelemetry 統合（`fav run --trace`） | 完了 |
| v22.8.0 | `fav deploy` 強化（ECS / K8s / Fly.io） | 完了 |
| **v23.0.0** | **Distributed Scale マイルストーン宣言** | **完了** |

---

## リポジトリ構成

```
favnir/
  fav/          コンパイラ・VM・CLIツールチェーン（Rust）
  fav/self/     Favnir 製セルフホストコンパイラ・型チェッカー
  runes/        標準ルーンライブラリ（Favnir）
  site/         リファレンスサイト（Next.js）
  infra/        インフラ（Terraform / AWS）
  versions/     バージョン履歴・ロードマップ・言語仕様
```

### infra/e2e-demo — バイトコードポータビリティ証明デモ

セルフホストコンパイラが生成する `.fvc` バイトコードが、
**ソースコードなしで** 異なる実行環境上で動作することを
4つのシナリオで証明したデモ。すべての証跡は `s3://favnir-e2e-demo/proof/` に保存。

| デモ | 環境 | アーキテクチャ | 結果 |
|---|---|---|---|
| ECS | EC2 × 2 + Fargate | Machine A（コンパイル）→ S3 → Machine B（実行）→ ECS ETL | **PASS=8 / FAIL=0** |
| EKS | EKS Fargate | compiler Pod（`.fav`→`.fvc`）→ executor Pod（VM のみ）| **PASS=6 / FAIL=0** |
| Lambda | Lambda + SQS + Aurora | S3 イベント → compiler Lambda → SQS → executor Lambda → RDS | **PASS=6 / FAIL=0** |

**共通の証跡ポイント（EKS / Lambda）:**

| チェック | compiler | executor |
|---|---|---|
| `.fav` ソースの有無 | あり（toolchain イメージ） | なし（runtime イメージ） |
| `.fvc` 生成 | `fav build` で生成・S3 保存 | S3 からダウンロードして実行 |
| DB 書き込み | — | Aurora PostgreSQL → S3 サマリー |

詳細:
- [`infra/e2e-demo/ecs/README.md`](infra/e2e-demo/ecs/README.md)
- [`infra/e2e-demo/eks/README.md`](infra/e2e-demo/eks/README.md)
- [`infra/e2e-demo/lambda/README.md`](infra/e2e-demo/lambda/README.md)

---

## 対応プラットフォーム

| OS | 状態 | 備考 |
|----|------|------|
| Windows (MSVC) | サポート | 日本語環境は追加設定が必要（下記参照） |
| Linux / WSL | サポート | 追加設定不要 |
| macOS | 非対応 | 開発者が Mac を持っていないため未対応。将来対応予定 |

### Windows 日本語環境（CP932 ロケール）

`.cargo/config.toml` に `CXXFLAGS = "/EHsc /utf-8"` が設定済みです（`force = false`）。
PowerShell・Git Bash いずれからビルドしても自動的に適用されます。

### Linux / WSL

`~/.bashrc` に以下を追加してください:

```bash
export CXXFLAGS=
```

---

## ライセンス

MIT
