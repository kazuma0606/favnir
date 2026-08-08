# Roadmap v69.1.0 〜 v70.0.0 — Intelligent ETL 1.0 宣言

Date: 2026-08-04
Status: 未着手（v69.0.0 完了後に開始）

マスターロードマップ: [roadmap-v65.1-v70.0.md](roadmap-v65.1-v70.0.md)

---

## 前提

- 直前完了: v69.0.0「Distributed Favnir」（tests = 3541）
- 本スプリントは Phase 5「Intelligent ETL 1.0 宣言」の詳細計画
- 目標: v70.0.0「Intelligent ETL 1.0 宣言」（tests ≥ 3545）

### スプリントの性格

Phase 5 は「統合・実証・宣言」のスプリントである。
v65.1〜v69.0 で積み上げた全機能を、実際のユースケースで E2E 検証する。

- **E2E デモ**: AI データパイプラインの完全なエンドツーエンドデモを作成
- **Playground**: ブラウザで AI パイプラインを動かせる WASM Playground
- **ドキュメント**: 「Intelligent ETL ガイド」としてまとめた公式ドキュメント
- **マイグレーション**: 旧 ETL パイプラインを AI ETL に変換するアシスト
- **安定化**: v70.0 宣言に向けた最終調整

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v69.1.0 | E2E デモ（CSV → Embed → VectorDB → Semantic Search） | 3541 + 2 = 3543 | 完了 |
| v69.2.0 | Playground WASM 版 AI パイプライン | 3543 + 2 = 3545 | 完了 |
| v69.3.0 | ドキュメントサイト「Intelligent ETL ガイド」 | 3545（変化なし） | 完了 |
| v69.4.0 | `fav migrate --ai`（旧 ETL → AI ETL 自動変換） | 3545（変化なし） | 完了 |
| v69.5.0 | E2E デモ動作確認 | 3545 + 2 = 3547 | 完了 |
| v69.6.0 | Playground サンプル追加 | 3549 | 完了 |
| v69.7.0 | ドキュメント校正・内部リンク確認 | 3551 | 完了 |
| v69.8.0 | パフォーマンス回帰テスト | 3553 | 完了 ✓ |
| v69.9.0 | コードフリーズ・最終チェック | — | 完了 ✓ |
| v70.0.0 | Intelligent ETL 1.0 宣言 ★クリーンアップ | 3559 | 完了 ✓ |

> **注意**: v69.3.0 以降のテスト数は着手時に確定する。ドキュメント・移行ツール中心のため
> テスト増加は最小限に抑え、品質は E2E テストと手動検証で担保する。

---

## v69.1.0 — E2E デモ（CSV → Embed → VectorDB → Semantic Search）

**概要**: Favnir v65〜v69 の全機能を使った完全な AI ETL デモパイプライン。
「実際に動く AI パイプライン」として公開し、採用検討者が即試せる形にする。

```favnir
// infra/e2e-demo/ai-etl/src/pipeline.fav

schema Article {
    id:      String,
    title:   String,
    body:    String,
    tags:    List<String>
}

schema IndexedArticle {
    id:        String,
    title:     String,
    embedding: Vec<Float>[1536],
    summary:   String
}

// Stage 1: CSV → Article レコード
public stage LoadArticles: String -> List<Article> = |csv_path| {
    Rune.csv.read(csv_path, schema: Article)
}

// Stage 2: 記事を要約 + 埋め込みベクトル生成
public stage EmbedAndSummarize: Article -> IndexedArticle = |article| {
    bind summary   <- Rune.llm.extract(article.body, schema: String, model: "claude-haiku-4-5-20251001")
    bind embedding <- Rune.embed.openai(article.title + " " + summary, model: "text-embedding-3-small")
    IndexedArticle {
        id:        article.id,
        title:     article.title,
        embedding: embedding,
        summary:   summary
    }
}

// Stage 3: ベクトル DB に保存
public stage StoreToVectorDB: List<IndexedArticle> -> Unit = |articles| {
    bind pairs <- List.map(articles, |a| { (a.id, a.embedding) })
    Rune.pinecone.upsert(pairs, namespace: "articles", index: "demo-index")
}

// Stage 4: セマンティック検索
public stage SemanticSearch: String -> List<IndexedArticle> = |query| {
    bind query_vec <- Rune.embed.openai(query, model: "text-embedding-3-small")
    bind results   <- Rune.pinecone.query(query_vec, top_k: 5)
    results
}

// パイプライン定義
pipeline IndexPipeline {
    step "load"   = seq LoadArticles
    step "embed"  = par [EmbedAndSummarize, EmbedAndSummarize, EmbedAndSummarize, EmbedAndSummarize] after "load"
    step "store"  = seq StoreToVectorDB after "embed"
}
```

```bash
$ cd infra/e2e-demo/ai-etl
$ fav run src/pipeline.fav --cluster workers.yaml --checkpoint ./checkpoints/
[step load]  1000 articles loaded (2ms)
[step embed] 4 workers × 250 articles each (avg 1240ms)
[step store] 1000 embeddings → Pinecone (45ms)
[done] Pipeline completed. Cost: $0.14

$ fav run src/pipeline.fav --stage SemanticSearch <<< "machine learning pipelines"
Result 1: "Introduction to ML Pipelines" (score: 0.94)
Result 2: "Data Engineering with AI" (score: 0.91)
Result 3: "Favnir: Type-Safe AI ETL" (score: 0.89)
```

**実装内容**:

- `infra/e2e-demo/ai-etl/` ディレクトリ作成
  - `src/pipeline.fav` — 4 ステージのデモパイプライン
  - `data/articles.csv` — サンプルデータ（100 記事）
  - `workers.yaml` — ローカル 4 ワーカー設定
  - `README.md` — セットアップ手順
  - `scripts/run.sh` — 実行スクリプト
- `fav.toml` の `[ai]` セクションを使ってプロバイダー設定

**完了条件**: Rust テスト 2 件（3541 + 2 = **3543**）

```rust
// driver.rs mod v69100_tests
fn ai_etl_e2e_demo_structure()   // e2e-demo/ai-etl/src/pipeline.fav が存在し "IndexPipeline" を含む
fn ai_etl_demo_has_all_stages()  // LoadArticles / EmbedAndSummarize / StoreToVectorDB / SemanticSearch を含む
```

---

## v69.2.0 — Playground WASM 版 AI パイプライン

**概要**: ブラウザ上で AI パイプラインを試せる Playground。
Math Rune（linalg / stats / autodiff）と AI Rune（LLM 抽出・埋め込みモック）をブラウザで動作させる。

```
https://favnir.dev/playground

[ コードエディタ ]                    [ 出力 ]
public stage Normalize: ...          Vec<Float>[128]:
  ...                                [0.021, -0.134, 0.082, ...]
                                     norm = 1.000

[ Run ]  [ Examples ▼ ]              [ Cost estimate: $0.003 ]
  > Linear Algebra Demo
  > Statistics Pipeline
  > LLM Extraction (mock)
  > Semantic Search (mock)
```

**実装内容**:

- `@favnir/wasm` の更新: Math Rune + AI Rune モックの WASM 組み込み
- AI Rune モック: ブラウザ用にモック実装（実際の API 呼び出しなし）
  - `Rune.embed.mock(text)` → ランダム正規化ベクトルを返す
  - `Rune.llm.mock(text, schema)` → schema のデフォルト値を返す
  - `Rune.stats.describe(data)` → 実際に計算（pure 関数なので WASM で動く）
- サンプルコード: 4 種類のプリセット（Linear Algebra / Stats / LLM Extract / Semantic Search）
- コスト見積もり表示: 本番 API 使用時の推定コストをリアルタイム表示

**サイトファイル**:
- `site/content/playground/ai-examples.mdx` — AI サンプル例の説明

**完了条件**: Rust テスト 2 件（3543 + 2 = **3545**）

```rust
// driver.rs mod v69200_tests
fn playground_ai_wasm_examples()  // playground/ai-examples.mdx が存在し "mock" を含む
fn playground_math_rune_wasm()    // "Rune.stats" / "Rune.linalg" / "WASM" キーワードを含む
```

---

## v69.3.0 — ドキュメントサイト「Intelligent ETL ガイド」

**概要**: v65〜v69 で実装した全機能をまとめた公式ガイド。
「AI データパイプラインをゼロから構築する」チュートリアル形式でまとめる。

**作成するドキュメント**:

```
site/content/docs/intelligent-etl/
├── overview.mdx            — Intelligent ETL とは何か（ビジョン）
├── quickstart.mdx          — 15 分で動かすチュートリアル
├── math-foundation.mdx     — Math Rune 群の使い方（linalg/stats/autodiff）
├── ai-stages.mdx           — AI ステージの構築（embed/llm/vectordb）
├── debugging.mdx           — fav debug / fav viz / fav suggest の使い方
├── distributed.mdx         — クラスタ実行・チェックポイント・K8s
└── reference/
    ├── math-runes.mdx      — Math Rune API リファレンス
    └── ai-runes.mdx        — AI Rune API リファレンス
```

**完了条件**: テスト追加なし（ドキュメント確認は v69.9.0 で実施）

---

## v69.4.0 — `fav migrate --ai`（旧 ETL → AI ETL 自動変換）

**概要**: 既存の Favnir ETL パイプラインを AI ETL パターンに変換するアシスタント。
手動書き換えの手間を最小化し、既存ユーザーの移行を支援する。

```bash
$ fav migrate --ai src/old-pipeline.fav --output src/ai-pipeline.fav

Analyzing old-pipeline.fav...

Suggestions:
[1] LoadCsv → LoadCsv（変更なし）
[2] Transform → EmbedAndTransform
    + Rune.embed.openai を追加（text フィールドから埋め込み生成）
[3] InsertDB → InsertDB + StoreToVectorDB
    + ベクトルを Pinecone に並行保存
[4] SendReport → SemanticEnrich（LLM で要約を追加）

Apply all? [y/N/select]: y
Generated: src/ai-pipeline.fav
```

**実装内容**:

- `cmd_migrate_ai(src, output)` — AI パターンへの変換提案
- LLM 連携: 既存パイプラインを Claude API が分析して変換提案を生成
- `--dry-run` フラグ — 変換結果のプレビュー（ファイル書き出しなし）
- `--interactive` フラグ — 各変換を個別に確認・承認
- 変換後の型チェック: 変換結果が型エラーなくコンパイルできることを確認

**完了条件**: テスト追加なし（提案生成の品質は手動検証）

---

## v69.5.0〜v69.9.0 — 安定化・細部調整

**概要**: v70.0.0 宣言に向けた最終安定化スプリント。
各バージョンで発見された問題を修正し、コードフリーズに向けて品質を高める。

**候補タスク**（着手時に詳細化）:

- v69.5: E2E デモの動作確認（ローカル + CI 環境）
- v69.6: Playground の UI 改善・サンプル追加
- v69.7: ドキュメントのレビュー・校正・内部リンク確認
- v69.8: パフォーマンス回帰テスト（v65.0 ベースラインとの比較）
- v69.9: コードフリーズ・最終 lint / チェック

**完了条件**: 各バージョン 2 件（詳細は着手時に確定）

---

## v70.0.0 — Intelligent ETL 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

### v70.0 達成のエビデンス

| 機能 | 実装バージョン | 状態 |
|---|---|---|
| 型付き行列・ベクトル演算（linalg） | v65.1 | 予定 |
| 統計解析 Rune（stats） | v65.2 | 予定 |
| 自動微分（autodiff） | v65.3 | 予定 |
| 最適化 Rune（optim） | v65.4 | 予定 |
| 数値解析（numeric） | v65.5 | 予定 |
| 時系列解析（timeseries） | v65.6 | 予定 |
| 古典 ML（ml） | v65.7 | 予定 |
| Math lint rules（W050〜W054） | v65.8 | 予定 |
| 型付きベクトルステージ | v66.1 | 予定 |
| LLM 型安全抽出 | v66.2 | 予定 |
| 埋め込み Rune | v66.3 | 予定 |
| VectorDB Rune 群 | v66.4 | 予定 |
| ストリーミング推論 | v66.5 | 予定 |
| モデルサービング | v66.6 | 予定 |
| フィーチャーストア | v66.7 | 予定 |
| AI lint rules（W055〜W059） | v66.8 | 予定 |
| ステップ実行デバッガ | v67.1 | 予定 |
| タイムトラベルデバッグ | v67.2 | 予定 |
| DAG 可視化 | v67.3 | 予定 |
| AI 最適化アドバイザー | v67.4 | 予定 |
| 合成データテスト | v67.5 | 予定 |
| プロパティテスト | v67.6 | 予定 |
| インタラクティブプロファイル | v67.7 | 予定 |
| 数式ドキュメント生成 | v67.8 | 予定 |
| マルチノード par | v68.1 | 予定 |
| チェックポイント | v68.2 | 予定 |
| Kubernetes オーケストレーション | v68.3 | 予定 |
| リトライポリシー | v68.4 | 予定 |
| 分散キャッシュ | v68.5 | 予定 |
| コスト見積もり | v68.6 | 予定 |
| マルチクラウドルーティング | v68.7 | 予定 |
| 分散オブザーバビリティ | v68.8 | 予定 |
| E2E AI ETL デモ | v69.1 | 予定 |
| WASM Playground（AI Rune） | v69.2 | 予定 |
| Intelligent ETL ガイド | v69.3 | 予定 |
| ETL → AI ETL 移行ツール | v69.4 | 予定 |

**タスク**:

- [ ] `fav/Cargo.toml` version を `"70.0.0"` に更新
- [ ] `MILESTONE.md` 先頭に v70.0.0「Intelligent ETL 1.0」エントリを追加
- [ ] `README.md` に v70.0.0 宣言文を追加
- [ ] `CHANGELOG.md` 先頭に v70.0.0 エントリを追加
- [ ] `v70000_tests` 4 件を `driver.rs` に追加
- [ ] `cargo clean` 実行（★クリーンアップ）
- [ ] `cargo test -j 8 -- --test-threads=8` で ≥3559 tests passed を確認

**完了条件**: `v70000_tests` 4 件（v69.9.0 ベース 3555 + 4 = **3559**）
（v69.5〜v69.9 各バージョンで +2 追加 = +10。v69.5 時点 3547 + 8 = 3555 + 4 = 3559）

```rust
// driver.rs mod v70000_tests
fn cargo_toml_version_is_70_0_0()      // Cargo.toml に "version = \"70.0.0\"" を含む
fn changelog_has_v70_0_0()             // CHANGELOG.md に "v70.0.0" を含む
fn milestone_has_intelligent_etl()     // MILESTONE.md に "Intelligent ETL" を含む
fn readme_mentions_intelligent_etl()   // README.md に "Intelligent ETL" または "v70.0" を含む
```

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v69.0.0（ベース） | 3541 | — | Distributed Favnir 宣言 |
| v69.1.0 | 3543 | +2 | E2E デモ |
| v69.2.0 | 3545 | +2 | Playground WASM |
| v69.3.0 | 3545 | ±0 | ドキュメントサイト |
| v69.4.0 | 3545 | ±0 | fav migrate --ai |
| v69.5.0 | 3547 | +2 | E2E デモ動作確認 |
| v69.6.0 | 3549 | +2 | Playground サンプル追加 ✓ |
| v69.7.0 | 3551 | +2 | ドキュメント校正・内部リンク確認 ✓ |
| v69.8.0 | 3553 | +2 | パフォーマンス回帰テスト ✓ |
| v69.9.0 | 3555 | +2 | コードフリーズ ✓ |
| v70.0.0 | 3559 | +4 | Intelligent ETL 1.0 宣言 ✓ |
