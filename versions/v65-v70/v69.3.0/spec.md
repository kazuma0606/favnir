# v69.3.0 仕様書 — ドキュメントサイト「Intelligent ETL ガイド」

Status: DRAFT
Version: 69.3.0
Date: 2026-08-07

---

## 概要

v65〜v69 で実装した全機能をまとめた公式ガイド「Intelligent ETL ガイド」を作成する。
「AI データパイプラインをゼロから構築する」チュートリアル形式でまとめ、
採用検討者・新規ユーザーが Favnir の全機能を体系的に理解できる状態にする。

---

## スコープ

### IN（本バージョンで実施）

`site/content/docs/intelligent-etl/` 以下に 8 ファイルを新規作成:

| ファイル | 内容 |
|---|---|
| `overview.mdx` | Intelligent ETL とは何か（ビジョン・設計思想） |
| `quickstart.mdx` | 15 分で動かすチュートリアル |
| `math-foundation.mdx` | Math Rune 群の使い方（linalg / stats / autodiff） |
| `ai-stages.mdx` | AI ステージの構築（embed / llm / vectordb） |
| `debugging.mdx` | `fav debug` / `fav viz` / `fav suggest` の使い方 |
| `distributed.mdx` | クラスタ実行・チェックポイント・K8s |
| `reference/math-runes.mdx` | Math Rune API リファレンス |
| `reference/ai-runes.mdx` | AI Rune API リファレンス |

各ファイルの要件:
- MDX として valid（先頭に ESM import 行なし）
- コードブロックはフェンス記法（` ```favnir ``` ` / ` ```sh ``` `）

### OUT（本バージョンでは実施しない）

- Rust テストの追加: ドキュメント確認テストは v69.9.0 で実施（ロードマップ方針）
- Cargo.toml / CHANGELOG.md の変更: v70.0.0 宣言時に一括更新（sub-version ポリシー）
- サイトナビゲーション（sidebars.ts 等）の更新: 将来フェーズ
- 翻訳対応: 将来フェーズ

---

## 各ドキュメントの概要仕様

### `overview.mdx`
- Intelligent ETL の定義（型安全 × AI × 分散実行）
- v65〜v69 の機能マップ（Math / AI / Debug / Distributed）
- 「なぜ Favnir か」のビジョン説明

### `quickstart.mdx`
- 15 分チュートリアル（前提: fav CLI インストール済み）
- `infra/e2e-demo/ai-etl/` のコードをチュートリアル説明のために引用する（`include_str!` 等は行わない。コードは MDX 内に直接記述）
- コマンド例: `fav run src/pipeline.fav --env dev`

### `math-foundation.mdx`
- `Rune.linalg`（行列演算・ノルム・内積）の使い方
- `Rune.stats`（記述統計・分布）の使い方
- `Rune.autodiff`（自動微分）の使い方
- コードサンプルあり

### `ai-stages.mdx`
- `Rune.embed`（埋め込みベクトル生成）の使い方
- `Rune.llm`（LLM 型安全抽出）の使い方
- `Rune.pinecone` / `Rune.qdrant`（VectorDB）の使い方
- bind 構文（`bind x <- expr`）の説明（`bind x = expr` は不可。コードサンプルは必ず `<-` 記法を使うこと）
- **型注記**: `Vec<Float>[N]` は Favnir の未定義型。コードサンプルでは `List<Float>` を使うこと

### `debugging.mdx`
- `fav debug`（ステップ実行）の使い方
- `fav viz`（DAG 可視化）の使い方
- `fav suggest`（AI 最適化アドバイザー）の使い方

### `distributed.mdx`
- `--cluster workers.yaml` による分散実行
- `--checkpoint` による耐障害性・再開
- `fav deploy --target kubernetes` による K8s 対応
- `--distributed-cache` / `--otel-endpoint` の設定

### `reference/math-runes.mdx`
- Math Rune 全関数の API リファレンス（linalg / stats / autodiff / optim / numeric / timeseries / ml）

### `reference/ai-runes.mdx`
- AI Rune 全関数の API リファレンス（embed / llm / pinecone / qdrant / weaviate / pgvector）

---

## テスト仕様

**テスト追加なし**（ロードマップ方針）

ドキュメント確認テスト（MDX ファイルの存在・内容確認）は v69.9.0 で実施する。
本バージョンでは `cargo test` のテスト数は 3545 のまま変化しない。

---

## 完了条件

- `site/content/docs/intelligent-etl/` 以下の 8 ファイルが全て作成されていること
- `cargo test -j 8 -- --test-threads=8` で **3545 tests passed, 0 failed**（テスト数変化なし）
- `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.3.0 行の状態が「完了」になっていること
- `versions/current.md` の「進行中バージョン」が `v69.2.0` から `v69.3.0` に更新されていること（T3 で対応）
