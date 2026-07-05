# Roadmap v34.1.0 〜 v35.0.0 — Production Ready

Date: 2026-07-01
Status: 骨格確定 + v34.0 完了後更新（2026-07-04）

---

## 目標

v34.0「Performance & Tooling」で「本番で速い」を実現した。
このフェーズは **「Production Ready の宣言」** だ。

> **Production Ready の定義（本プロジェクト固有）**
> 「実際のデータエンジニアリング案件で Favnir を選択できる状態。
>  500 行以上の実データパイプラインが end-to-end で動き、
>  エラーが起きたときに原因を特定して修正できる。
>  ドキュメントを読めば新しいエンジニアが 1 日で Favnir を使い始められる」

v35.0 は「新機能を入れる」マイルストーンではない。
v30.1〜v34.9 で積み上げた成果を **「実案件で使える」レベルに確認・安定化する** マイルストーンだ。

---

## v34.0 完了時に判明した実装変更

v34.0.0（Performance & Tooling 宣言、2026-07-04）完了時点での判明事項:

### エフェクトシステム統一（コンテキスト構文移行）

v33.7.0 でエフェクトシステム移行ツール（`migrate_effects_in_source` / `resolve_use_effects`）を確認済み。
v34.x 系で `!Effect` アノテーションを廃止し、Capability Context（ctx パラメータ）に一本化する。

設計は `versions/roadmap/roadmap-v33.1-v34.0.md` のエフェクトシステム統一方針セクションで確定済み：

```favnir
// 移行前（廃止予定）
fn fetch(url: String) -> Result<String, String> !Http {
    HTTP.get(url)
}

// 移行後（v34.x）
fn fetch(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

**`!Effect` → ctx 対応表**:

| 廃止する `!Effect` | 代替 ctx フィールド | 型 |
|---|---|---|
| `!Io` | `ctx.io` | `IoCtx` |
| `!DbRead` / `!DbWrite` | `ctx.db` | `DbRead` / `DbWrite` |
| `!Http` | `ctx.http` | `HttpClient` |
| `!Postgres` / `!MySQL` | `ctx.db` | `PgConn` / `MySqlConn` |
| `!Redis` | `ctx.redis` | `RedisClient` |
| `!Llm` | `ctx.llm` | `LlmClient` |
| `!Stream` | `ctx.stream` | `StreamClient` |
| `!Trace` | `ctx.tracer` | `Tracer` |
| `!Snowflake` | `ctx.warehouse` | `SnowflakeConn` |
| `!Emit<T>` | `ctx.emitter` | `Emitter<T>` |

**v34.5〜v34.7 で実装予定**（後述）。

### v33.x 確認シリーズの残件

v33.x は「確認・記録」パターンのためコア機能の実装はすべて v19.x 済み。
v34.x では確認済み機能を **本番品質** に引き上げることが主目的。

---

## ⚠️ 更新済み（v34.0 完了時）

v34.0 完了時点での判断:
1. エフェクトシステム統一が v34.x の最大実装変更 → v34.5〜v34.7 に割り当て
2. ドキュメントサイト v4: v34.2 で実装（計画通り）
3. ベンチマーク比較対象: Python pandas / Apache Spark / dbt（変更なし）
4. v34.5〜v34.9 の具体的内容は後述セクションで確定

---

## 設計決定事項（暫定）

| 項目 | 暫定決定 | 確定時期 |
|---|---|---|
| 実案件デモの規模 | 500 行以上・複数 Rune・複数ファイル構成 | v34.0 後 |
| ベンチマーク比較対象 | Python pandas / Apache Spark / dbt | v34.0 後 |
| ドキュメントサイト v4 | 既存 Next.js 16 を継続・コンテンツ大幅増強 | v34.0 後 |
| セキュリティ審査 v2 の範囲 | エフェクトシステム形式検証 + OSS ライセンス | v34.0 後 |
| 破壊的変更 | `!Effect` 廃止（v34.5、`fav migrate --effects` で自動移行） | v34.5 実施 |

---

## バージョン計画（骨格）

### v34.1 — 実案件デモ実装

**テーマ**: 複数 Rune・複数ファイルを使った実規模パイプラインを `examples/` に追加する。

**対象デモ（暫定）**:

```
examples/real-world-etl/
├── fav.toml
├── src/
│   ├── types.fav           注文データの型定義
│   ├── validators.fav      ビジネスルールのバリデーション
│   ├── stages.fav          ETL ステージ群
│   ├── notifications.fav   Slack / Email 通知
│   └── main.fav            エントリポイント
├── data/
│   └── orders_sample.csv   サンプルデータ 10,000 行
└── README.md               30 分で動かす手順
```

処理フロー:
```
S3 から CSV ダウンロード
    |> バリデーション（欠損値・範囲チェック・重複除去）
    |> Postgres に書き込み
    |> BigQuery に同期
    |> 処理結果を Slack に通知
    |> OTel でトレース記録
```

完了条件:
- `examples/real-world-etl/` が完全な状態で存在する
- README に「30 分で動かす」手順が書かれている
- `fav check` / `fav test` が通る
- Rust テスト 1 件（examples の存在確認）

---

### v34.2 — ドキュメントサイト v4

**テーマ**: 新しいエンジニアが 1 日で Favnir を使い始められるドキュメントを整備する。

**構成（暫定）**:

```
favnir.dev/
├── /                      ランディング（30 秒で何ができるかわかる）
├── /learn/                チュートリアル
│   ├── getting-started    10 分チュートリアル
│   ├── first-pipeline     最初の ETL パイプライン
│   └── rune-guide         Rune の使い方
├── /cookbook/             実用レシピ 50 本以上
│   ├── postgres-etl
│   ├── s3-to-parquet
│   ├── kafka-consumer
│   ├── rag-pipeline
│   └── ...（50 本）
├── /errors/               エラーコードリファレンス（E0001〜）
│   ├── E0001              undefined variable
│   └── ...
├── /runes/                全 Rune ドキュメント（自動生成）
├── /playground/           ブラウザ内実行（WASM）
├── /bench/                ベンチマーク比較グラフ
└── /spec/                 形式的仕様書
```

新規追加コンテンツ:
- `/errors/` — `fav explain` コマンドと同内容を Web で閲覧できる
- cookbook を 30 本 → 50 本に増強
- ベンチマーク比較グラフ（Python / Spark との実測比較）

---

### v34.3 — ベンチマーク公開

**テーマ**: 実測ベンチマークを `bench/` ページで公開する。

**比較対象**（暫定）:
- Python pandas（CSV 読み込み・変換・Postgres 書き込み）
- Apache Spark（同上、大規模データ）
- dbt（SQL 変換パイプライン）

**計測項目**:
- 処理速度（行数/秒）
- メモリ使用量（ピーク）
- Lambda コールドスタート時間
- コンパイル時間

```
benchmarks/real-world/
├── python_pandas.json
├── apache_spark.json
└── favnir.json
```

---

### v34.4 — セキュリティ審査 v2

**テーマ**: v24.6.0（セキュリティ審査 v1）を更新し、Production Ready を確認する。

**審査対象**:
1. エフェクトシステムの形式的検証（`pure_fn_calls_effectful` W021 lint が機能しているか）
2. OSS 依存ライセンス確認（Cargo.toml の全依存が MIT / Apache-2.0 互換か）
3. Rune の認証情報の扱い（環境変数経由のみか、コードに埋め込めないか）
4. `fav run` の実行サンドボックス確認

---

### v34.5 — `!Effect` 廃止・コンテキスト構文統一

**テーマ**: `!Effect` アノテーションを廃止し、Capability Context（ctx パラメータ）に一本化する。

これは **破壊的変更** だが、v33.7.0 実装済みの `migrate_effects_in_source` / `fav migrate --effects` で自動移行できる。

**実装内容**:
1. `W0XX deprecated_effect_annotation` lint ルール追加（`!Effect` 使用箇所に警告）
2. `IoCtx` interface 定義（`io.println` / `io.read_file` / `io.env`）を追加
3. `AppCtx` に `io: IoCtx` フィールドを追加
4. `fav/src/ast.rs` — `Effect` enum を deprecated 化（lint 警告のみ、削除は v35.x）
5. `fav/src/middle/checker.rs` — ctx 型チェックを優先、`!Effect` は deprecated 警告
6. `fav/self/compiler.fav` / `checker.fav` — `!Effect` 宣言を ctx 引数に書き換え

**完了条件**: `fav migrate --effects sample.fav` で `!Http { ... }` → `fn f(ctx: AppCtx, ...)` に自動変換される

---

### v34.6 — Rune ファイル ctx 移行

**テーマ**: `runes/` 配下の全 Rune を ctx ベース構文に移行する。

対象（優先度順）:
- `runes/postgres/client.fav` — `!Postgres` → `ctx.db: PgConn`
- `runes/redis/redis.fav` — `!Redis` → `ctx.redis: RedisClient`
- `runes/kafka/kafka.fav` — `!Kafka` → `ctx.stream: StreamClient`
- その他全 Rune（50+ ファイル）— `fav migrate --effects --dir runes/` で一括移行

---

### v34.7 — ドキュメント・examples ctx 移行

**テーマ**: サイト / examples / README のコードサンプルをすべて ctx 構文に更新する。

対象:
- `site/content/docs/` 配下の MDX（`!Effect` コードサンプルを ctx 構文に更新）
- `site/content/learn/` 入門記事（チュートリアルの `!Effect` 例を ctx 構文に書き換え）
- `examples/` 配下の全 `.fav` ファイル
- `README.md` — エフェクトシステム統一の設計思想セクション追記

---

### v34.8A — `!Effect` 構文のパースエラー化（v35.4.0）

`!Effect` アノテーションを書いた場合に E0374 ハードエラーとして返す。
W022 lint（非推奨警告）を削除し、「書けばエラー」に格上げ。
`parser.rs` の `parse_effects_acc` をエラー生成コードに置き換える。

### v34.9A — `Effect` enum / `effects` フィールドの完全削除（v35.5.0）

`ast.rs` / `parser.rs` / `checker.rs` / `lineage.rs` / `fmt.rs` / `wasm_codegen.rs` 等
14 ファイルから `Effect` に関するすべてのコードを物理削除する。
削除行数は約 380 行。

### v35.0A — ドキュメント ctx 構文統一 + Production Ready 宣言（v35.6.0）

サイト MDX 125 件のコードサンプルを ctx 構文に一括変換。
`ctx-syntax-guide.mdx` を公式ガイドとして完成させ、
「副作用のある処理は ctx: AppCtx を渡す」を唯一のパターンとして明示する。
v35.0 Production Ready マイルストーン宣言。

---

## v35.0 — Production Ready マイルストーン宣言

**暫定完了条件（v34.0 完了後に確定）:**

| コンポーネント | 暫定完了基準 |
|---|---|
| 実案件デモ | `examples/real-world-etl/` が end-to-end で動作する |
| ドキュメント | `/errors/` ページ・cookbook 50 本・ベンチマーク比較が公開済み |
| ベンチマーク | Python pandas との比較で速度優位が示されている |
| セキュリティ | v2 審査で問題なし |
| 安定性 | テスト数 3000+、既知バグゼロ |
| 後方互換性 | v30.0.0 時点の .fav コードが v35.0.0 でも動作する |

**最終宣言文（暫定）:**

> 「`fav new --template postgres-etl my-pipeline` で始め、
>  `fav check` で型安全性を確認し、
>  `fav build --target native` でネイティブバイナリを生成し、
>  Lambda にデプロイして実データを処理できる。
>  エラーが起きれば `fav explain` で原因がわかり、
>  `fav test --watch` でリグレッションを防げる。
>
>  これが Favnir v35.0 — Production Ready の姿である。」

**★ クリーンアップ実施（v35.0 リリース時 — 最終クリーンアップ）:**

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings
./target/debug/fav lint --deny-warnings --allow W017 --allow W018 --allow W019 self/compiler.fav
./target/debug/fav lint --deny-warnings --allow W012 --allow W017 --allow W018 --allow W019 self/checker.fav
du -sh target/
echo "=== v35.0.0 Production Ready クリーンアップ完了 ==="
```

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v33.1-v34.0.md`
- 達成宣言: `MILESTONE.md`
