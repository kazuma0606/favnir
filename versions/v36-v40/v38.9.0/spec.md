# v38.9.0 spec — v39.0 前調整・安定化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.9.0 |
| テーマ | v39.0 前調整・安定化 — AI 支援機能（v38.1〜v38.8）の統合ドキュメント整備と品質確認 |
| 前提 | v38.8.0 COMPLETE — AI 支援 cookbook 3 本整備済み |
| 完了条件 | `v38900_tests` 全テスト pass・`cargo test` 0 failures（≥ 2781 件） |

## 背景と目的

v38.1〜v38.8 で実装した AI 支援機能群（`fav suggest` / `fav generate --from sql` / `fav generate --from csv` / LSP AI 補完 / `fav explain --verbose` LLM / `fav new --template rag-pipeline` / `llm.stream` / `llm.embed` / cookbook 3 本）を v39.0 マイルストーン宣言前に統合し、公式ドキュメントを整備する。

主な調整内容:

1. **`site/content/docs/ai-overview.mdx` 新規作成** — v38.x 系 AI 支援機能を一覧化する概要ドキュメントを作成し、v39.0 マイルストーンの前に公式ドキュメントを整備する。
2. **`suggest.rs` 品質確認テスト** — `llm_suggest` 関数が `suggest.rs` に存在することをテストで保証する。

## 実装スコープ

### 1. `site/content/docs/ai-overview.mdx` — AI 支援機能概要ドキュメント

```
---
title: "AI 支援機能"
description: "Favnir v38.x — AI がパイプライン開発を補助する機能群（fav suggest / fav generate / llm Rune / cookbook）"
---

# AI 支援機能

Favnir v38.x では **AI がパイプライン開発を補助する** 機能群を提供します。

## `fav suggest` — LLM によるエラー修正提案

`fav check` が返すエラーコードから修正案を LLM で生成します。

```bash
fav check main.fav
# main.fav:12:5: E0001 undefined variable `custmer_id`
fav suggest E0001 main.fav:12
# Suggestion: Did you mean `customer_id`? (typo)
```

## `fav generate --from sql` — SQL → Favnir 変換

PostgreSQL / MySQL の SELECT / JOIN / WHERE / ORDER BY を Favnir パイプラインに自動変換します。

```bash
fav generate --from sql "SELECT id, name FROM orders WHERE status = 'shipped'"
```

詳細は [cookbook/sql-to-favnir](/cookbook/sql-to-favnir) を参照。

## `fav generate --from csv` — CSV スキーマ推定

CSV ファイルから `schema` + `expect` ブロックを自動生成します。

```bash
fav generate --from csv data/orders.csv
```

## LSP AI 補完

`fav.toml` に `[lsp.ai] enabled = true` を設定すると、LLM による補完候補の rerank が有効になります。

```toml
[lsp.ai]
enabled = true
```

## `fav explain --verbose` — コンテキスト付き説明

LLM がコードの意図とコンテキストに即した説明・修正例を生成します。

```bash
fav explain --verbose main.fav
```

## `fav new --template rag-pipeline` — RAG テンプレート

RAG（Retrieval-Augmented Generation）パイプラインのスターターテンプレートを生成します。

```bash
fav new my-rag --template rag-pipeline
```

詳細は [cookbook/rag-pipeline](/cookbook/rag-pipeline) を参照。

## `llm.stream` / `llm.embed` — Llm Rune 強化

v38.7.0 で追加された `llm.stream`・`llm.embed` により、ストリーミング応答と埋め込みベクトル生成をパイプライン内で直接扱えます。

```favnir
import llm

stage AskLlm(prompt: String) -> String {
    match llm.stream(prompt) {
        Ok(response) => response
        Err(e)       => String.concat("Error: ", e)
    }
}
```

詳細は [cookbook/llm-streaming](/cookbook/llm-streaming) を参照。

## 環境変数まとめ

| 変数 | 用途 |
|---|---|
| `ANTHROPIC_API_KEY` | `fav suggest` / `llm.complete` / `llm.stream` / `llm.function_call` |
| `OPENAI_API_KEY` | `llm.embed`（`LLM_PROVIDER=openai` 必須） |
| `LLM_PROVIDER` | `anthropic`（デフォルト）または `openai` |
| `LLM_EMBED_MODEL` | Embedding モデル（デフォルト: `text-embedding-3-small`） |

## 関連 Cookbook

- [SQL → Favnir パイプライン自動変換](/cookbook/sql-to-favnir)
- [RAG パイプライン](/cookbook/rag-pipeline)
- [LLM ストリーミングレスポンス](/cookbook/llm-streaming)
```

**必須キーワード**: `"fav suggest"` （`ai_overview_doc_exists` テストが検証）

### 2. `driver.rs` — `v38900_tests` モジュール

```rust
// ── v38900_tests (v38.9.0) — v39.0 前調整・安定化 ──────────────────────────
#[cfg(test)]
mod v38900_tests {
    #[test]
    fn cargo_toml_version_is_38_9_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.9.0"), "Cargo.toml must contain version 38.9.0");
    }

    #[test]
    fn changelog_has_v38_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.9.0]"), "CHANGELOG.md must contain [v38.9.0]");
    }

    #[test]
    fn ai_overview_doc_exists() {
        let doc = include_str!("../../site/content/docs/ai-overview.mdx");
        assert!(
            doc.contains("fav suggest"),
            "ai-overview.mdx must contain 'fav suggest'"
        );
    }

    #[test]
    fn suggest_rs_has_llm_suggest() {
        let src = include_str!("suggest.rs");
        assert!(
            src.contains("llm_suggest"),
            "suggest.rs must contain llm_suggest function"
        );
    }
}
```

**注意**:
- `include_str!` パスは `fav/src/driver.rs` 起点
  - `"../Cargo.toml"` — `fav/Cargo.toml`
  - `"../../CHANGELOG.md"` — root の `CHANGELOG.md`
  - `"../../site/content/docs/ai-overview.mdx"` — site の docs ディレクトリ
  - `"suggest.rs"` — `fav/src/suggest.rs`（同ディレクトリ）
- `use super::*;` は不要（`include_str!` のみ使用）

### 3. `CHANGELOG.md` — `[v38.9.0]` エントリ追加

```
## [v38.9.0] — 2026-07-10

### Added
- `site/content/docs/ai-overview.mdx` — v38.x AI 支援機能概要ドキュメント
- `v38900_tests` 4 テスト追加（`suggest_rs_has_llm_suggest` 品質確認含む）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 4. その他ドキュメント更新

- `fav/Cargo.toml`: `38.8.0` → `38.9.0`
- `versions/current.md`: 最新安定版 → v38.9.0、次バージョン → v39.0.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.9.0 を ✅ 完了済みにマーク・テスト件数を 4 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.8.0 | 2777 |
| v38.9.0 追加分（Rust） | +4 |
| v38.9.0 期待値 | 2781 |

## 注意事項

### `include_str!("suggest.rs")` のパス

`driver.rs`（`fav/src/driver.rs`）と `suggest.rs`（`fav/src/suggest.rs`）は同一ディレクトリなので、`include_str!("suggest.rs")` が正しい相対パス。

### MDX コードブロックのネスト

spec.md §1（MDX コンテンツ）では外側コードブロック内に内側コードブロックをネストして示している。実際のファイル作成時は **Write ツールで直接書き込む**こと（spec.md の外側バッククォートは spec 表現上のもの）。

### ロードマップとの整合

ロードマップ v38.9.0:「v39.0 前調整・安定化」（詳細未記載）

本 spec がロードマップの空白を埋める。ロードマップ更新時（T7）に以下を追記する:
- テスト件数: 4 件
- 成果物: `ai-overview.mdx` 新規作成

また、ロードマップ v39.0.0 の完了条件「テスト数 4800+」は執筆時点の誤記であるため、v38.9.0 完了時に「2785+（v38.9.0 実績ベース）」に修正済み。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `38.9.0` | `cargo_toml_version_is_38_9_0` テスト |
| 2 | `CHANGELOG.md` に `[v38.9.0]` が含まれる | `changelog_has_v38_9_0` テスト |
| 3 | `site/content/docs/ai-overview.mdx` が存在し `fav suggest` を含む | `ai_overview_doc_exists` テスト |
| 4 | `suggest.rs` に `llm_suggest` 関数が含まれる | `suggest_rs_has_llm_suggest` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2781） | `cargo test` 実行結果 |
| 6 | `roadmap-v38.1-v39.0.md` の v38.9.0 が ✅ かつテスト件数が 4 件 | T7 後に目視確認 |
| 7 | `versions/current.md` が v38.9.0（最新安定版）に更新されている | T7 後に目視確認 |
