# Favnir v6.5.0 Tasks

Date: 2026-05-27

## Goal

実装済みだがドキュメントが存在しない機能の docs を追加する。
コード変更なし。ドキュメント 3 ページ新規作成 + 1 ページ更新。

## Phase A — language/pipeline.mdx 新規作成

- [x] A-1: `site/content/docs/language/pipeline.mdx` を作成
  - [x] A-1a: フロントマター (title/order: 6/category: 言語仕様)
  - [x] A-1b: `stage` 構文（型契約・エフェクト・実装）
  - [x] A-1c: `seq` 構文（パイプライン定義・エフェクト合成）
  - [x] A-1d: `|>` 演算子（型チェック・エフェクト union）
  - [x] A-1e: `abstract stage` / `abstract seq`（依存注入）
  - [x] A-1f: `fav explain` 出力例

## Phase B — language/schema.mdx 新規作成

- [x] B-1: `site/content/docs/language/schema.mdx` を作成
  - [x] B-1a: フロントマター (title/order: 7/category: 言語仕様)
  - [x] B-1b: スキーマとは（用途・配置場所）
  - [x] B-1c: YAML 構文の基本例（`schemas/orders.yaml`）
  - [x] B-1d: 制約一覧テーブル（positive / non_negative / max_length / min_length / pattern / nullable / one_of）
  - [x] B-1e: `fav build --schema` の使い方（DDL 生成）
  - [x] B-1f: `T.validate` のプレビュー記載（v6.6.0 で完全実装予定と明示）

## Phase C — stdlib/infer.mdx 新規作成

- [x] C-1: `site/content/docs/stdlib/infer.mdx` を作成
  - [x] C-1a: フロントマター (title/order: 6/category: 標準ライブラリ)
  - [x] C-1b: `fav infer --csv` の使い方と出力例
  - [x] C-1c: `fav infer --db` の使い方（PostgreSQL / SQLite）
  - [x] C-1d: `fav infer --proto` の使い方
  - [x] C-1e: `--out` でスキーマファイルに出力する方法
  - [x] C-1f: 生成型をコードで使う例

## Phase D — rune-cli.mdx 更新

- [x] D-1: `site/content/docs/rune-cli.mdx` 末尾に `fav deploy` セクションを追加
  - [x] D-1a: `fav deploy --target lambda` の基本使用例
  - [x] D-1b: `fav.toml [deploy]` の設定スキーマ
  - [x] D-1c: ECS 対応は v6.7.0 予定と注記
- [x] D-2: `rune-cli.mdx` 末尾に `fav build --schema` セクションを追加
  - [x] D-2a: DDL 生成の使用例（postgres / sqlite dialect）
  - [x] D-2b: ディレクトリ指定と `--out` オプション

## Phase E — 検証

- [x] E-1: 全 4 ファイルのコードブロックが有効な Favnir 構文であることを確認
- [x] E-2: order/category が既存ページと整合していることを確認
- [x] E-3: このファイルを完了状態に更新

## Recommended execution order

A → B → C → D → E

## 完了条件まとめ

- `language/pipeline.mdx`（stage/seq/|>/abstract）が存在する ✓
- `language/schema.mdx`（制約一覧・YAML 構文）が存在する ✓
- `stdlib/infer.mdx`（CSV/DB/Proto 推論）が存在する ✓
- `rune-cli.mdx` に `fav deploy` と `fav build --schema` が追記されている ✓
- すべてのコード例が有効な Favnir 構文 ✓
