# v69.1.0 タスクリスト

Status: COMPLETE
Version: 69.1.0
Note: E2E デモ（CSV → Embed → VectorDB → Semantic Search）— infra/e2e-demo/ai-etl/ 作成 + 2 テスト追加
Base tests: 3541
Target tests: 3543

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3541 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `driver.rs` に `v69000_tests` が存在することを確認（`v69100_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v69100_tests` を `v69000_tests` の直前に挿入する
- [x] `driver.rs` に `v69100_tests` が存在しないことを確認（新規追加）
- [x] `versions/current.md` の「進行中バージョン」が `v69.0.0` であることを確認（v69.0.0 の T8 完了後、v69.0.0 のまま。本バージョン T5 で v69.1.0 に更新する）
- [x] `cargo test --bin fav v69000_tests` で 4 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `cargo_toml_version_is_69_0_0`, `changelog_has_v69_0_0`, `milestone_has_distributed`, `readme_mentions_distributed`
- [x] `infra/e2e-demo/ai-etl/` ディレクトリが存在しないことを確認（新規作成）

---

## T1: `infra/e2e-demo/ai-etl/src/pipeline.fav` 作成

- [x] `infra/e2e-demo/ai-etl/src/` ディレクトリを作成
- [x] `pipeline.fav` を新規作成
  - [x] `"IndexPipeline"` キーワードを含む（`ai_etl_e2e_demo_structure` テスト要件）
  - [x] `"LoadArticles"` を含む（Stage 1）
  - [x] `"EmbedAndSummarize"` を含む（Stage 2）
  - [x] `"StoreToVectorDB"` を含む（Stage 3）
  - [x] `"SemanticSearch"` を含む（Stage 4、スタンドアロンステージ）
  - [x] `par` キーワードで EmbedAndSummarize を並列実行する定義を含む
  - [x] ロードマップのサンプルコードに準拠（schema Article / schema IndexedArticle）
  - [x] `fav.toml [ai]` セクションの設定例をコメントとして記載（ファイル作成は OUT スコープ）
- [x] `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` でアクセスできることを確認

---

## T2: サポートファイル作成

- [x] `infra/e2e-demo/ai-etl/data/articles.csv` 作成
  - [x] ヘッダ: `id,title,body,tags`
  - [x] 10 行以上のサンプル記事データ
- [x] `infra/e2e-demo/ai-etl/workers.yaml` 作成
  - [x] 4 ワーカー定義（localhost ポート 9001〜9004）
- [x] `infra/e2e-demo/ai-etl/README.md` 作成
  - [x] セットアップ手順（fav run コマンド例を含む）
- [x] `infra/e2e-demo/ai-etl/scripts/run.sh` 作成
  - [x] `fav run` コマンドを含む実行スクリプト

---

## T3: `driver.rs` — `v69100_tests` 追加

- [x] `// -- v69000_tests (v69.0.0) -- Distributed Favnir 宣言 --` の直前に挿入
  - [x] `ai_etl_e2e_demo_structure`:
    - [x] `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` でファイルを読み込む
    - [x] `"IndexPipeline"` を個別 `assert!` で検証
  - [x] `ai_etl_demo_has_all_stages`:
    - [x] `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` でファイルを読み込む
    - [x] `"LoadArticles"` を個別 `assert!` で検証
    - [x] `"EmbedAndSummarize"` を個別 `assert!` で検証
    - [x] `"StoreToVectorDB"` を個別 `assert!` で検証
    - [x] `"SemanticSearch"` を個別 `assert!` で検証
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v69100_tests` で 2 件 PASS
  - [x] `ai_etl_e2e_demo_structure` PASS
  - [x] `ai_etl_demo_has_all_stages` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3543 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.1.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v69.1.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。v70.0.0 宣言時に一括更新する。

---

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [HIGH] | `pipeline.fav` 全5箇所 | `bind x = expr` は Favnir 構文違反（正しくは `bind x <- expr`） | 全5箇所を `<-` に修正 |
| [MED] | `pipeline.fav` 26行目 | `Vec<Float>[1536]` は未定義型（現行型システム非対応） | `List<Float>` + 次元数コメントに変更 |

---

## 設計上の意図的省略

- 実際の API 呼び出し（Pinecone / OpenAI / Claude）: 将来フェーズ（スタブのまま）
- `fav.toml [ai]` セクションファイル作成: pipeline.fav 内コメントとして言及のみ
- CI 自動実行: 将来フェーズ
- WASM Playground 更新: v69.2.0
- ドキュメントサイト更新: v69.3.0
