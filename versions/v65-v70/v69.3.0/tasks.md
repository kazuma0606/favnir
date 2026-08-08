# v69.3.0 タスクリスト

Status: COMPLETE
Version: 69.3.0
Note: ドキュメントサイト「Intelligent ETL ガイド」— site/content/docs/intelligent-etl/ 以下 8 ファイル作成（テスト追加なし）
Base tests: 3545
Target tests: 3545（変化なし）

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3545 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.2.0` であることを確認（本バージョン T3 で v69.3.0 に更新する）
- [x] `site/content/docs/intelligent-etl/` ディレクトリが存在しないことを確認（新規作成）
- [x] `infra/e2e-demo/ai-etl/src/pipeline.fav` が存在することを確認（v69.1.0 成果物。`quickstart.mdx` で参照する）

---

## T1: `site/content/docs/intelligent-etl/` MDX ファイル作成

- [x] `site/content/docs/intelligent-etl/overview.mdx` 作成
  - [x] Intelligent ETL の定義・ビジョンを含む
  - [x] v65〜v69 の機能マップを含む
  - [x] MDX として valid（先頭に ESM import 行なし）

- [x] `site/content/docs/intelligent-etl/quickstart.mdx` 作成
  - [x] 15 分チュートリアル形式
  - [x] `fav run` コマンド例を含む
  - [x] コードサンプルに `Vec<Float>` が使われていないことを確認（`List<Float>` を使う）
  - [x] コードサンプルに `bind x = expr` が使われていないことを確認（`bind x <- expr` のみ）
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/math-foundation.mdx` 作成
  - [x] `Rune.linalg` の使い方・コードサンプルを含む
  - [x] `Rune.stats` の使い方・コードサンプルを含む
  - [x] `Rune.autodiff` の使い方を含む
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/ai-stages.mdx` 作成
  - [x] `Rune.embed` の使い方を含む
  - [x] `Rune.llm` の使い方・`bind x <- expr` 構文説明を含む
  - [x] VectorDB Rune（pinecone / qdrant）の使い方を含む
  - [x] コードサンプルに `bind x = expr` が使われていないことを確認（`bind x <- expr` のみ）
  - [x] コードサンプルに `Vec<Float>` が使われていないことを確認（`List<Float>` を使う）
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/debugging.mdx` 作成
  - [x] `fav debug` の使い方を含む
  - [x] `fav viz` の使い方を含む
  - [x] `fav suggest` の使い方を含む
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/distributed.mdx` 作成
  - [x] `--cluster workers.yaml` の使い方を含む
  - [x] `--checkpoint` / `--resume` の使い方を含む
  - [x] `fav deploy --target kubernetes` の使い方を含む
  - [x] `--distributed-cache` / `--otel-endpoint` の設定を含む
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/reference/math-runes.mdx` 作成
  - [x] Math Rune 群（linalg / stats / autodiff / optim / numeric / timeseries / ml）の API を含む
  - [x] コードサンプルに `Vec<Float>` が使われていないことを確認（`List<Float>` を使う）
  - [x] MDX として valid

- [x] `site/content/docs/intelligent-etl/reference/ai-runes.mdx` 作成
  - [x] AI Rune 群（embed / llm / pinecone / qdrant / weaviate / pgvector）の API を含む
  - [x] コードサンプルに `Vec<Float>` が使われていないことを確認（`List<Float>` を使う）
  - [x] MDX として valid

---

## T2: テスト確認（変化なし）

- [x] `cargo test -j 8 -- --test-threads=8` で **3545 tests passed, 0 failed** を確認（テスト数変化なし）

> 注: ドキュメント確認テスト（MDX の存在・内容確認）は v69.9.0 で実施（ロードマップ方針）

---

## T3: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.3.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を `v69.2.0` から `v69.3.0` に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。v70.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- Rust テストの追加: v69.9.0 で実施（ロードマップ明示的方針）
- サイトナビゲーション（sidebars.ts 等）の更新: 将来フェーズ
- 翻訳対応: 将来フェーズ
