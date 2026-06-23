# v24.5.0 — Rune レジストリ成熟（公式パッケージ 50+）

Date: 2026-06-23

## 目標

OSS コミュニティが Rune を公開・検索・インストールできるエコシステムを整備する。
具体的には:
1. `fav search <query>` — 公式カタログを対象とするトップレベル検索コマンドの追加
2. `OFFICIAL_CATALOG` — 50+ 公式パッケージを記述した定数（driver.rs）
3. 15 個の新規 Rune スタブ追加（`runes/<name>/`）— 合計 50+ Rune 達成
4. ドキュメントサイト更新（新規 Rune の紹介ページ追記）

---

## ロードマップとの対応

| ロードマップ | v24.5.0 での対応 |
|---|---|
| `fav search "bigquery"` | `pub fn cmd_search(query: &str)` をトップレベルコマンドとして追加 ✓ |
| `fav install bigquery` | 既存実装（v13〜v14 で追加済み）。v24.5.0 スコープ外 |
| `fav publish my-rune` | 既存実装（`cmd_publish`、v13〜v14 で追加済み）。v24.5.0 スコープ外 |
| 公式パッケージ 50+（クラウド全カバー / データフォーマット / ML 統合） | 15 新規 Rune スタブ + `OFFICIAL_CATALOG` 定数（50 エントリ） ✓ |

> **注意**: `fav registry search <q>`（既存）はローカルレジストリ（`~/.fav/registry/`）を対象とする。
> 新設の `fav search <q>` は `OFFICIAL_CATALOG`（組み込み公式カタログ）を対象とするため、
> ローカルインストール済みでなくても検索できる点が異なる。

---

## スコープ

### 変更種別

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 新規関数 | `driver.rs` | `OFFICIAL_CATALOG` 定数（50 エントリ）+ `pub fn cmd_search(query: &str)` |
| CLI 追加 | `main.rs` | `Some("search")` アームを追加（`cmd_search` をディスパッチ） |
| 新規 Rune スタブ | `runes/` | 15 ディレクトリ（各 `rune.toml` + `<name>.fav`） |
| ドキュメント更新 | `site/content/docs/runes/` | 新規 Rune を紹介するカタログページ追加 |
| 新規作成 | `benchmarks/v24.5.0.json` | test_count: 1954 |
| 更新 | `CHANGELOG.md` | v24.5.0 エントリ追加 |

### スコープ外

- リモートレジストリサーバー（`https://registry.favnir.dev`）— v25.x で構築予定
- `fav search` のページング・フィルタ（`--tag`、`--author`）— v25.x で対応
- 新規 Rune の完全実装（VM primitive 登録・checker.fav 対応）— 各 Rune v25.x+ で対応
- `fav publish` のリモート送信（現状はローカルのみ）— v25.x で対応

---

## `fav search` コマンド仕様

### 使い方

```bash
fav search                 # 全公式パッケージを一覧表示
fav search "bigquery"      # 名前または説明に "bigquery" を含むものを検索
fav search "ml"            # "ml" にマッチするパッケージを検索
```

### 出力フォーマット

```
NAME                 VERSION  DESCRIPTION
bigquery             1.0.0    Google BigQuery read/write
huggingface          1.0.0    HuggingFace API integration (text/image/embedding)
scikit               1.0.0    scikit-learn ML model integration via Python bridge
```

### 実装方針

`OFFICIAL_CATALOG` 定数（`&[(&str, &str, &str)]`）を driver.rs に定義し、`cmd_search` はそれを線形検索する。
検索は大文字小文字を区別しない（`to_lowercase` で統一）。

```rust
/// v24.5.0: 公式 Favnir Rune カタログ（name, latest_version, description）。
pub const OFFICIAL_CATALOG: &[(&str, &str, &str)] = &[
    // ── 既存 Rune（35 件） ────────────────────────────────────────────────────
    ("auth",             "1.0.0", "JWT / OAuth2 / API Key 認証"),
    ("aws",              "1.0.0", "AWS SDK 統合（S3 / SQS / DynamoDB / Lambda）"),
    ("azure-blob",       "1.0.0", "Azure Blob Storage 読み書き"),
    ("azure-postgres",   "1.0.0", "Azure Database for PostgreSQL 接続"),
    ("bigquery",         "1.0.0", "Google BigQuery 読み書き"),
    ("cache",            "1.0.0", "インメモリ / TTL キャッシュ"),
    ("csv",              "1.0.0", "CSV 読み書き（ストリーミング対応）"),
    ("ctx",              "1.0.0", "Capability Context（AppCtx / MockCtx）"),
    ("db",               "1.0.0", "汎用 DB インターフェース（DbRead / DbWrite）"),
    ("duckdb",           "1.0.0", "DuckDB 組み込み分析エンジン"),
    ("email",            "1.0.0", "SMTP / SendGrid メール送信"),
    ("env",              "1.0.0", "環境変数読み取り"),
    ("fs",               "1.0.0", "ファイルシステム操作（read / write / watch）"),
    ("gen",              "1.0.0", "UUID / NanoID / ランダム値生成"),
    ("graphql",          "1.0.0", "GraphQL クライアント（型付きクエリ）"),
    ("grpc",             "1.0.0", "gRPC クライアント / サーバー"),
    ("http",             "1.0.0", "HTTP/1.1 + HTTP/2 クライアント"),
    ("incremental",      "1.0.0", "増分処理チェックポイント"),
    ("io",               "1.0.0", "標準 I/O プリミティブ"),
    ("json",             "1.0.0", "JSON エンコード / デコード"),
    ("kafka",            "1.0.0", "Apache Kafka / AWS MSK プロデューサー・コンシューマー"),
    ("llm",              "1.0.0", "LLM 統合（Claude / OpenAI）"),
    ("log",              "1.0.0", "構造化ログ（JSON / text）"),
    ("parquet",          "1.0.0", "Apache Parquet 読み書き"),
    ("postgres",         "1.0.0", "PostgreSQL 接続"),
    ("queue",            "1.0.0", "インメモリ Queue プリミティブ"),
    ("rune_loader",      "1.0.0", "Rune 動的ロードユーティリティ"),
    ("slack",            "1.0.0", "Slack Webhook 通知・Block Kit"),
    ("snowflake",        "1.0.0", "Snowflake クエリ・ロード"),
    ("sql",              "1.0.0", "汎用 SQL クエリビルダー"),
    ("stat",             "1.0.0", "基本統計量（mean / stddev / percentile）"),
    ("state",            "1.0.0", "ステートフル処理（!State エフェクト）"),
    ("stdlib",           "1.0.0", "Favnir 標準ライブラリ（List / Map / String）"),
    ("toml",             "1.0.0", "TOML 設定ファイルパーサー"),
    ("validate",         "1.0.0", "スキーマバリデーション（where 節統合）"),
    // ── v24.5.0 新規 Rune（15 件） ───────────────────────────────────────────
    ("avro",             "1.0.0", "Apache Avro シリアライズ / デシリアライズ"),
    ("orc",              "1.0.0", "Apache ORC カラムナーフォーマット"),
    ("excel",            "1.0.0", "Excel (.xlsx) 読み書き"),
    ("xml",              "1.0.0", "XML パース / シリアライズ（XPath 対応）"),
    ("huggingface",      "1.0.0", "HuggingFace API 統合（テキスト / 画像 / 埋め込み）"),
    ("scikit",           "1.0.0", "scikit-learn ML モデル統合（Python ブリッジ）"),
    ("gcs",              "1.0.0", "Google Cloud Storage 読み書き"),
    ("pubsub",           "1.0.0", "Google Cloud Pub/Sub パブリッシュ / サブスクライブ"),
    ("redis",            "1.0.0", "Redis キャッシュ・メッセージブローカー"),
    ("mysql",            "1.0.0", "MySQL / MariaDB 接続"),
    ("mongodb",          "1.0.0", "MongoDB ドキュメントストア"),
    ("s3",               "1.0.0", "AWS S3 オブジェクトストレージ"),
    ("sqs",              "1.0.0", "AWS SQS メッセージキュー"),
    ("dynamodb",         "1.0.0", "AWS DynamoDB NoSQL"),
    ("azure-servicebus", "1.0.0", "Azure Service Bus メッセージング"),
];

pub fn cmd_search(query: &str) {
    let q = query.to_lowercase();
    let results: Vec<_> = OFFICIAL_CATALOG
        .iter()
        .filter(|(name, _, desc)| {
            q.is_empty() || name.contains(&q) || desc.to_lowercase().contains(&q)
        })
        .collect();
    if results.is_empty() {
        println!("(no packages matching \"{}\")", query);
        return;
    }
    println!("{:<20} {:<8} {}", "NAME", "VERSION", "DESCRIPTION");
    for (name, version, desc) in results {
        println!("{:<20} {:<8} {}", name, version, desc);
    }
}
```

---

## `OFFICIAL_CATALOG` エントリ数

| カテゴリ | Rune 一覧 | 件数 |
|---|---|---|
| 認証 | auth | 1 |
| クラウド（AWS） | aws, s3, sqs, dynamodb | 4 |
| クラウド（Azure） | azure-blob, azure-postgres, azure-servicebus | 3 |
| クラウド（GCP） | bigquery, gcs, pubsub | 3 |
| クラウド（Snowflake） | snowflake | 1 |
| データフォーマット | csv, json, parquet, avro, orc, excel, xml, toml | 8 |
| データベース | db, duckdb, postgres, sql, mysql, mongodb | 6 |
| メッセージング | kafka, queue, slack, redis | 4 |
| ML / AI | llm, huggingface, scikit | 3 |
| HTTP / API | http, graphql, grpc | 3 |
| ユーティリティ | cache, ctx, email, env, fs, gen, incremental, io, log, rune_loader, stat, state, stdlib, validate | 14 |
| **合計** | | **50** |

---

## 新規 Rune スタブ仕様

各 Rune は最小構成（`rune.toml` + `<name>.fav` のコメントヘッダー）のスタブとして追加する。
完全実装（VM primitive 登録・checker 対応）は v25.x 以降で個別に対応。

### rune.toml テンプレート

```toml
[rune]
name        = "<name>"
version     = "1.0.0"
description = "<説明>"
```

> **バージョンについて**: 新規スタブは `version = "1.0.0"` とする。既存 Rune（slack 等）は `0.1.0` を使用しているが、新規スタブは OFFICIAL_CATALOG と揃えて `1.0.0` とする（意図的な不一致）。

### `<name>.fav` テンプレート

```
// runes/<name>/<name>.fav — <Title> Rune (v24.5.0)
//
// 使い方:
//   import rune "<name>"
//
// NOTE: このスタブは v24.5.0 で追加。完全実装は v25.x 以降で対応予定。
```

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_5_0` | Cargo.toml に `version = "24.5.0"` | — |
| `fav_search_command_exists` | `main.rs` に `Some("search")` アーム、`driver.rs` に `pub fn cmd_search` | — |
| `official_catalog_50_plus` | `OFFICIAL_CATALOG.len() >= 50` | `>= 50` |
| `catalog_covers_cloud_formats_ml` | avro / orc / excel / xml / huggingface / scikit / gcs / redis が OFFICIAL_CATALOG に存在 | 全件 `true` |
| `changelog_has_v24_5_0` | `CHANGELOG.md` に `[v24.5.0]` | — |

---

## 完了条件

- [ ] `OFFICIAL_CATALOG` 定数（50 エントリ）追加済み（`driver.rs`）
- [ ] `pub fn cmd_search(query: &str)` 実装済み（`driver.rs`）
- [ ] `Some("search")` アーム追加済み（`main.rs`）
- [ ] 15 新規 Rune スタブ作成済み（`runes/<name>/rune.toml` + `runes/<name>/<name>.fav`）
- [ ] `cargo test v245000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1953 件合格）
- [ ] `CHANGELOG.md` に v24.5.0 エントリ
- [ ] `benchmarks/v24.5.0.json` 作成済み（test_count: 1953）
- [ ] `site/content/docs/runes/catalog.mdx` 作成済み（全 50 Rune 一覧）
