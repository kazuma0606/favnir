# v38.8.0 spec — AI 支援 cookbook 3 本

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.8.0 |
| テーマ | AI 支援機能（v38.2.0 / v38.6.0 / v38.7.0）の cookbook 3 本追加 |
| 前提 | v38.7.0 COMPLETE — Llm Rune 強化（stream / function_call / embed）実装済み |
| 完了条件 | `v38800_tests` 全テスト pass・`cargo test` 0 failures（≥ 2777 件） |

## 背景と目的

v38.1〜v38.7 で実装した AI 支援機能（`fav generate --from sql` / `fav new --template rag-pipeline` / `llm.stream`）の
実用例を cookbook として公開する。ユーザーがコピー＆ペーストで試せるレベルの具体的なコード例を提供する。

| cookbook ファイル | 対応機能 |
|---|---|
| `sql-to-favnir.mdx` | v38.2.0 — `fav generate --from sql` による SQL → Favnir 変換 |
| `rag-pipeline.mdx` | v38.6.0 + v38.7.0 — `llm.embed` を使った RAG パイプライン |
| `llm-streaming.mdx` | v38.7.0 — `llm.stream` によるストリーミング LLM 応答処理 |

## 実装スコープ

### 1. `site/content/cookbook/sql-to-favnir.mdx` — 新規作成

```mdx
---
title: "SQL → Favnir パイプライン自動変換"
description: "`fav generate --from sql` で PostgreSQL クエリを型安全な Favnir パイプラインに自動変換する"
---

# SQL → Favnir パイプライン自動変換

`fav generate --from sql` コマンドを使って、既存の SQL クエリを型安全な Favnir パイプラインに変換します。

## 使い方

```bash
# SELECT 文を Favnir パイプラインに変換
fav generate --from sql "SELECT id, name, amount FROM orders WHERE status = 'shipped'"

# JOIN を含む SQL を変換
fav generate --from sql \
  "SELECT o.id, c.name, o.amount FROM orders o JOIN customers c ON o.customer_id = c.id"
```

## 生成されるコード例（SELECT）

```favnir
import runes/postgres

stage LoadOrders -> List<{ id: Int  name: String  amount: Float }> {
    let sql = "SELECT id, name, amount FROM orders WHERE status = 'shipped'"
    postgres.query(ctx, sql)
}

pipeline sql_to_favnir {
    LoadOrders
}
```

## 生成されるコード例（JOIN）

```favnir
import runes/postgres

stage LoadOrders -> List<{ id: Int  customer_id: Int  amount: Float }> {
    postgres.query(ctx, "SELECT id, customer_id, amount FROM orders")
}

stage LoadCustomers -> List<{ id: Int  name: String }> {
    postgres.query(ctx, "SELECT id, name FROM customers")
}

stage JoinData(
    orders: List<{ id: Int  customer_id: Int  amount: Float }>,
    customers: List<{ id: Int  name: String }>
) -> List<{ id: Int  name: String  amount: Float }> {
    List.join_on(orders, customers, fn(o, c) { o.customer_id == c.id },
        fn(o, c) { { id: o.id  name: c.name  amount: o.amount } })
}

pipeline sql_to_favnir {
    [LoadOrders, LoadCustomers] |> JoinData
}
```

## ポイント

- `fav generate --from sql` は SELECT / JOIN / WHERE / ORDER BY をサポートします。
- 生成されたパイプラインを `fav run src/main.fav` でそのまま実行できます。
- スキーマの型は SQL カラム名から自動推定されます。実際の型は `fav.toml` の `[schema]` セクションで上書き可能です。
```

### 2. `site/content/cookbook/rag-pipeline.mdx` — 新規作成

```mdx
---
title: "RAG パイプライン"
description: "Retrieval-Augmented Generation パイプラインを Favnir で構築する — llm.embed でベクトル化し、関連チャンクを取得して LLM に渡す"
---

# RAG パイプライン

Favnir の `llm.embed` と `llm.complete` を使って RAG（Retrieval-Augmented Generation）パイプラインを構築します。

## クイックスタート

```bash
# テンプレートから RAG プロジェクトを作成
fav new my-rag --template rag-pipeline
cd my-rag
ANTHROPIC_API_KEY=sk-... LLM_PROVIDER=openai OPENAI_API_KEY=sk-... fav run src/main.fav
```

## コード例

```favnir
import llm
import csv

stage Ingest -> List<String> {
    csv.read_file("data/documents.csv")
}

stage Embed(docs: List<String>) -> List<String> {
    docs |> List.map(fn(doc) {
        match llm.embed(doc) {
            Ok(vec) => vec
            Err(_)  => "[]"
        }
    })
}

stage Retrieve(embeddings: List<String>) -> List<String> {
    // TODO: ベクトル DB から関連チャンクを取得
    // 例: embeddings |> VectorDB.search(ctx, top_k: 5)
    embeddings
}

stage Generate(context: List<String>) -> String {
    let prompt = String.join(context, "\n")
    match llm.complete(prompt) {
        Ok(answer) => answer
        Err(e)     => String.concat("Error: ", e)
    }
}

pipeline my_rag {
    Ingest |> Embed |> Retrieve |> Generate
}
```

## 環境変数

| 変数 | 説明 |
|---|---|
| `LLM_PROVIDER` | `openai`（embed 必須）または `anthropic`（generate のみ） |
| `OPENAI_API_KEY` | OpenAI API キー（`llm.embed` に必要） |
| `ANTHROPIC_API_KEY` | Anthropic API キー（`llm.complete` に必要） |
| `LLM_EMBED_MODEL` | Embedding モデル（デフォルト: `text-embedding-3-small`） |

## ポイント

- `fav new my-rag --template rag-pipeline` でスターターテンプレートを生成できます。
- `llm.embed` は `LLM_PROVIDER=openai` 設定が必要です（Anthropic は embedding API を提供しません）。
- `llm.embed` のシグネチャは v38.7.0 の新 API `(text: String) -> Result<String, String>` です（`pinecone-rag.mdx` の旧 2 引数 API `LLM.embed(config, text)` とは異なります）。
- `Result<String, String>` の `Ok` 値は JSON 配列文字列（例: `"[0.12, -0.34, ...]"`）です。
- Retrieve ステージは Pinecone / Weaviate / pgvector など任意のベクトル DB Rune に置き換え可能です。
```

### 3. `site/content/cookbook/llm-streaming.mdx` — 新規作成

```mdx
---
title: "LLM ストリーミングレスポンス"
description: "`llm.stream` でストリーミング LLM 応答を処理し、リアルタイム出力パイプラインを構築する"
---

# LLM ストリーミングレスポンス

`llm.stream` を使って LLM のレスポンスをストリーミングで処理するパイプラインを構築します。

## コード例

```favnir
import llm

stage BuildPrompt -> String {
    "Favnir を一文で説明してください。"
}

stage AskLlm(prompt: String) -> String {
    match llm.stream(prompt) {
        Ok(response) => response
        Err(e)       => String.concat("Error: ", e)
    }
}

stage FormatOutput(text: String) -> String {
    String.concat("[LLM] ", text)
}

pipeline llm_streaming {
    BuildPrompt |> AskLlm |> FormatOutput
}
```

## 実行例

```bash
ANTHROPIC_API_KEY=sk-... fav run src/main.fav
# [LLM] Favnir is a type-safe data pipeline language...
```

## v38.7.0 の制約

v38.7.0 では `llm.stream` は **collect-all 実装**です。
レスポンス全体を受信してから文字列として返します（真の SSE ストリーミングは v39.x で実装予定）。

パイプライン内でチャンクごとに処理する場合は、現時点では `List.map` + `llm.complete` の分割呼び出しを使用してください。

## ポイント

- `llm.stream` のシグネチャは `(prompt: String) -> Result<String, String>` です。
- `ANTHROPIC_API_KEY` または `OPENAI_API_KEY` を設定して使用します（`LLM_PROVIDER` 環境変数で切り替え）。
- 長大なレスポンスを処理する場合は `String.split(response, "\n")` でチャンク分割できます。
```

### 4. `fav/src/driver.rs` — テストモジュール

#### `v38700_tests::cargo_toml_version_is_38_7_0` のスタブ化

```rust
// Stubbed: version bumped to 38.8.0 — assertion intentionally removed
```

#### `v38800_tests` モジュール新規追加（3 テスト）

```rust
// ── v38800_tests (v38.8.0) — AI 支援 cookbook 3 本 ──────────────────────────
#[cfg(test)]
mod v38800_tests {
    #[test]
    fn cargo_toml_version_is_38_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.8.0"), "Cargo.toml must contain version 38.8.0");
    }

    #[test]
    fn changelog_has_v38_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.8.0]"), "CHANGELOG.md must contain [v38.8.0]");
    }

    #[test]
    fn ai_cookbook_files_exist() {
        let sql    = include_str!("../../site/content/cookbook/sql-to-favnir.mdx");
        let rag    = include_str!("../../site/content/cookbook/rag-pipeline.mdx");
        let stream = include_str!("../../site/content/cookbook/llm-streaming.mdx");
        assert!(sql.contains("fav generate"),  "sql-to-favnir.mdx must contain 'fav generate'");
        assert!(rag.contains("llm.embed"),     "rag-pipeline.mdx must contain 'llm.embed'");
        assert!(stream.contains("llm.stream"), "llm-streaming.mdx must contain 'llm.stream'");
    }
}
```

**注意**:
- `include_str!` パスは `fav/src/driver.rs` 起点: `../../site/content/cookbook/<file>.mdx`
- `use super::*;` は不要（`include_str!` のみ使用）

### 5. `CHANGELOG.md` — `[v38.8.0]` エントリ追加

```
## [v38.8.0] — 2026-07-10

### Added
- `site/content/cookbook/sql-to-favnir.mdx` — SQL → Favnir 変換 cookbook
- `site/content/cookbook/rag-pipeline.mdx` — RAG パイプライン cookbook
- `site/content/cookbook/llm-streaming.mdx` — LLM ストリーミング cookbook
- `v38800_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 6. その他ドキュメント更新

- `fav/Cargo.toml`: `38.7.0` → `38.8.0`
- `versions/current.md`: 最新安定版 → v38.8.0、次バージョン → v38.9.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.8.0 を ✅ 完了済みにマーク・テスト件数を 3 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.7.0 | 2774 |
| v38.8.0 追加分（Rust） | +3 |
| v38.8.0 期待値 | 2777 |

## 注意事項

### `include_str!` パスの確認

`driver.rs`（`fav/src/driver.rs`）から `site/content/cookbook/` への相対パスは `../../site/content/cookbook/` となる。
`../../CHANGELOG.md` と同様のパス構造（`fav/src/` → `fav/` → root）。

実際に MDX ファイルを作成してから `cargo build` でパスを確認すること（compile-time マクロのためパス誤りはコンパイルエラー）。

### MDX アサーションのキーワード

| ファイル | アサーション対象キーワード | 理由 |
|---|---|---|
| `sql-to-favnir.mdx` | `"fav generate"` | v38.2.0 の中核コマンド名 |
| `rag-pipeline.mdx` | `"llm.embed"` | v38.7.0 の新関数、このファイル固有 |
| `llm-streaming.mdx` | `"llm.stream"` | v38.7.0 の新関数、このファイル固有 |

### `rag-pipeline.mdx` と既存 `pinecone-rag.mdx` の区別

`site/content/cookbook/pinecone-rag.mdx` はすでに存在するが、`rag-pipeline.mdx` は別ファイル。
`rag-pipeline.mdx` は Favnir テンプレートシステム（`fav new --template rag-pipeline`）の説明に焦点を当てる。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | 3 つの MDX ファイルが作成され適切なコード例を含む | `ai_cookbook_files_exist` テスト |
| 2 | `CHANGELOG.md` に `[v38.8.0]` が含まれる | `changelog_has_v38_8_0` テスト |
| 3 | `Cargo.toml` バージョンが `38.8.0` | `cargo_toml_version_is_38_8_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2777） | `cargo test` 実行結果 |
| 5 | `roadmap-v38.1-v39.0.md` の v38.8.0 が ✅ かつテスト件数が 3 件 | T9 後に目視確認 |
| 6 | `versions/current.md` が v38.8.0（最新安定版）に更新されている | T9 後に目視確認 |
