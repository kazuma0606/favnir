# v24.5.0 実装計画 — Rune レジストリ成熟（公式パッケージ 50+）

## 前提確認

```bash
grep -n "version = " fav/Cargo.toml
# → "24.4.0" であること

grep -n "mod v245000_tests" fav/src/driver.rs | head -3
# → 0 件であること

grep -n "cmd_search\|OFFICIAL_CATALOG" fav/src/driver.rs | head -5
# → 全 0 件であること

ls runes/ | wc -l
# → 35 ディレクトリであること（rune_loader 含む）
```

---

## T0: `fav/src/driver.rs` — `OFFICIAL_CATALOG` + `cmd_search` 追加

### T0-1: `OFFICIAL_CATALOG` 定数を追加

`driver.rs` の末尾（v244000_tests の直前）に以下を追加する:

```rust
// ── v24.5.0: 公式 Rune カタログ ──────────────────────────────────────────────
/// 公式 Favnir Rune カタログ（name, latest_version, description）。
/// `fav search <query>` の検索対象。
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
```

### T0-2: `pub fn cmd_search` を追加

`OFFICIAL_CATALOG` 定数の直後に追加する:

```rust
/// `fav search [<query>]` — 公式 Rune カタログを検索する。
/// 大文字小文字を区別しない部分一致（name または description）。
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

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T1: `fav/src/main.rs` — `Some("search")` アーム追加

### T1-1: `cmd_search` を imports に追加

`main.rs` 行 79–92 の `use driver::{...}` ブロック内、アルファベット順で `cmd_run` の前（`cmd_publish` の後）に `cmd_search,` を追加する:

```rust
// main.rs の use driver::{...} ブロック内（既存の cmd_publish, cmd_registry の近く）
cmd_publish, cmd_registry, cmd_repl, cmd_run, ...,
// ↓ v24.5.0: cmd_search を追加（cmd_run の前）
cmd_publish, cmd_registry, cmd_repl, cmd_run, cmd_search, ...,
```

実際の修正: `cmd_publish, cmd_registry, cmd_repl, cmd_run,` を含む行に `cmd_search,` を追加する。

### T1-2: `Some("search")` アームを追加

`Some("registry")` アームの直後に配置する（アルファベット順: `"registry"` → `"search"`）:

```rust
Some("search") => {
    let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
    cmd_search(query);
}
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T2: 15 新規 Rune スタブ作成

`runes/` 配下に以下のディレクトリを追加する。各ディレクトリに `rune.toml` と `<name>.fav` を作成。

### 作成対象一覧

| ディレクトリ | description |
|---|---|
| `runes/avro/` | Apache Avro シリアライズ / デシリアライズ |
| `runes/orc/` | Apache ORC カラムナーフォーマット |
| `runes/excel/` | Excel (.xlsx) 読み書き |
| `runes/xml/` | XML パース / シリアライズ（XPath 対応）|
| `runes/huggingface/` | HuggingFace API 統合（テキスト / 画像 / 埋め込み）|
| `runes/scikit/` | scikit-learn ML モデル統合（Python ブリッジ）|
| `runes/gcs/` | Google Cloud Storage 読み書き |
| `runes/pubsub/` | Google Cloud Pub/Sub パブリッシュ / サブスクライブ |
| `runes/redis/` | Redis キャッシュ・メッセージブローカー |
| `runes/mysql/` | MySQL / MariaDB 接続 |
| `runes/mongodb/` | MongoDB ドキュメントストア |
| `runes/s3/` | AWS S3 オブジェクトストレージ |
| `runes/sqs/` | AWS SQS メッセージキュー |
| `runes/dynamodb/` | AWS DynamoDB NoSQL |
| `runes/azure-servicebus/` | Azure Service Bus メッセージング |

### 各ファイルのテンプレート

**`rune.toml`**:
```toml
[rune]
name        = "<name>"
version     = "1.0.0"
description = "<description>"
```

**`<name>.fav`**:
```
// runes/<name>/<name>.fav — <Title> Rune (v24.5.0)
//
// 使い方:
//   import rune "<name>"
//
// NOTE: このスタブは v24.5.0 で追加。完全実装は v25.x 以降で対応予定。
```

- [ ] 作成後確認: `ls runes/ | grep -v README | grep -v fav.toml | wc -l` → `50` 以上

---

## T3: `fav/src/driver.rs` — v245000_tests 追加

### T3-1: `v244000_tests::version_is_24_4_0` を削除

`v244000_tests` モジュール内の `version_is_24_4_0` **関数のみ**を削除する。
モジュール自体（`mod v244000_tests { ... }`）および残りの5件のテスト（`deprecated_fn_annotation_parsed`、`deprecated_call_emits_w020`、`deprecated_fn_self_call_no_w020`、`stability_md_has_policy`、`changelog_has_v24_4_0`）はそのまま保持すること。

```rust
// ↓ この関数のみ削除（モジュールと他5件は残す）
#[test]
fn version_is_24_4_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(
        cargo.contains("version = \"24.4.0\""),
        "Cargo.toml should have version 24.4.0"
    );
}
```

### T3-2: `v245000_tests` モジュールを `v244000_tests` の直後に追加

```rust
// ── v245000_tests (v24.5.0) — Rune レジストリ成熟 ───────────────────────────
#[cfg(test)]
mod v245000_tests {
    use super::*;

    #[test]
    fn version_is_24_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.5.0\""),
            "Cargo.toml should have version 24.5.0"
        );
    }

    #[test]
    fn fav_search_command_exists() {
        let main_src = include_str!("../main.rs");
        assert!(
            main_src.contains("Some(\"search\")"),
            "main.rs should have Some(\"search\") arm"
        );
        let drv_src = include_str!("../driver.rs");
        assert!(
            drv_src.contains("pub fn cmd_search"),
            "driver.rs should have pub fn cmd_search"
        );
    }

    #[test]
    fn official_catalog_50_plus() {
        // use super::* により OFFICIAL_CATALOG は直接アクセス可能
        assert!(
            OFFICIAL_CATALOG.len() >= 50,
            "OFFICIAL_CATALOG should have 50+ entries, got {}",
            OFFICIAL_CATALOG.len()
        );
    }

    #[test]
    fn catalog_covers_cloud_formats_ml() {
        // use super::* により OFFICIAL_CATALOG は直接アクセス可能
        let names: Vec<&str> = OFFICIAL_CATALOG
            .iter()
            .map(|(n, _, _)| *n)
            .collect();
        let required = [
            "avro", "orc", "excel", "xml",
            "huggingface", "scikit",
            "gcs", "redis",
        ];
        for name in required {
            assert!(
                names.contains(&name),
                "OFFICIAL_CATALOG should contain '{}' but does not; catalog: {:?}",
                name, names
            );
        }
    }

    #[test]
    fn changelog_has_v24_5_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("[v24.5.0]"),
            "CHANGELOG.md should have [v24.5.0] entry"
        );
    }
}
```

- [ ] `cargo test v245000 --bin fav` — 5/5 PASS を確認
- [ ] `cargo test --bin fav` — リグレッションなし（1953 件合格）を確認

> **件数計算**: 現在 1949 件 → `version_is_24_4_0` 削除 (-1) → v245000_tests 追加 (+5) → 1953 件

---

## T4: ドキュメントサイト更新

`site/content/docs/runes/` に `catalog.mdx` を新規作成して、公式 Rune カタログを一覧表示する。

```mdx
---
title: "Rune カタログ"
description: "Favnir 公式 Rune パッケージ一覧（v24.5.0、50+ パッケージ）"
---

# Rune カタログ

`fav search` で検索できる公式 Rune パッケージの一覧です。

## データフォーマット

| Rune | 説明 | バージョン |
|---|---|---|
| `avro` | Apache Avro シリアライズ / デシリアライズ | 1.0.0 |
| `orc` | Apache ORC カラムナーフォーマット | 1.0.0 |
| `excel` | Excel (.xlsx) 読み書き | 1.0.0 |
| `xml` | XML パース / シリアライズ（XPath 対応）| 1.0.0 |

## ML / AI

| Rune | 説明 | バージョン |
|---|---|---|
| `llm` | LLM 統合（Claude / OpenAI）| 1.0.0 |
| `huggingface` | HuggingFace API 統合 | 1.0.0 |
| `scikit` | scikit-learn モデル統合 | 1.0.0 |

...（全カテゴリを記載）
```

---

## T5: Cargo.toml + CHANGELOG + benchmarks

> **注意**: T3-1 の `version_is_24_4_0` 削除完了後に Cargo.toml を更新すること。

### T5-1: `fav/Cargo.toml` バージョン更新

```
version = "24.4.0" → "24.5.0"
```

### T5-2: `CHANGELOG.md` 先頭に v24.5.0 エントリ追加

```markdown
## [v24.5.0] — 2026-06-23 — Rune レジストリ成熟（公式パッケージ 50+）

### Added
- `fav search <query>` — 公式 Rune カタログを検索するトップレベルコマンド
- `OFFICIAL_CATALOG` — 50 パッケージを収録した組み込み公式カタログ（driver.rs）
- 15 新規 Rune スタブ（avro / orc / excel / xml / huggingface / scikit /
  gcs / pubsub / redis / mysql / mongodb / s3 / sqs / dynamodb / azure-servicebus）

### Notes
- 新規 Rune は v24.5.0 時点ではスタブ（rune.toml + .fav ヘッダー）
  完全実装（VM primitive / checker 対応）は v25.x 以降で個別に対応
- `fav search` は OFFICIAL_CATALOG（組み込み）を検索。
  ローカルインストール済み Rune の検索は `fav registry search <q>` を使用
```

### T5-3: `benchmarks/v24.5.0.json` 作成

```json
{
  "version": "24.5.0",
  "date": "2026-06-23",
  "test_count": 1953,
  "feature": "Rune レジストリ成熟（公式パッケージ 50+）",
  "metrics": {
    "test_count": 1953,
    "duration_ms": 17000
  }
}
```

> **注意**: `duration_ms` は推定値。実装完了後に実測値で更新すること。

---

## 実装順序

```
T0（driver.rs: OFFICIAL_CATALOG + cmd_search 追加）
cargo check → エラー 0 確認
T1（main.rs: Some("registry") 直後に Some("search") + use ブロックに cmd_search 追加）
cargo check → エラー 0 確認
T2（runes/: 15 新規スタブ作成）
T3-1（version_is_24_4_0 関数のみ削除、モジュール・他5件は保持）← T5-1 より前に必須
T3-2（v245000_tests 追加）
cargo test v245000 → 5/5 PASS 確認
T4（site/content/docs/runes/catalog.mdx 作成）
T5-1（version 更新）← T3-1 完了後
T5-2〜3（CHANGELOG / v24.5.0.json）
cargo test --bin fav → リグレッションなし確認（1953 件）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `cmd_search` の import 行に追加し忘れる | `cargo check` でコンパイルエラー | main.rs の import 行に `cmd_search,` を追加 |
| `Some("search")` アームが既存 `Some("registry")` アームと干渉 | `cargo check` の unreachable パターン警告 | `Some("search")` を `Some("registry")` より前（または後）に置く |
| `OFFICIAL_CATALOG.len()` が 50 未満 | `official_catalog_50_plus` テスト失敗 | カタログ定数のエントリ数を数え直す |
| 新規 Rune ディレクトリの `rune.toml` フォーマットが既存と不一致 | `fav install <name>` 実行時パースエラー | `runes/slack/rune.toml` のフォーマットを参照（`[rune]` セクション）|
| `crate::driver::OFFICIAL_CATALOG` が pub でないためテストからアクセス不可 | `cargo test` コンパイルエラー | `pub const OFFICIAL_CATALOG` を確認 |
