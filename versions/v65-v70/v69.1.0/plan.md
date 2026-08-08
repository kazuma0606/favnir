# v69.1.0 実装計画 — E2E デモ

Status: DRAFT
Version: 69.1.0

---

## 実装ステップ

### Step 1: `infra/e2e-demo/ai-etl/src/pipeline.fav` 作成

ロードマップのサンプルコードに準拠した 4 ステージパイプライン:

```favnir
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

public stage LoadArticles: String -> List<Article> = |csv_path| {
    Rune.csv.read(csv_path, schema: Article)
}

public stage EmbedAndSummarize: Article -> IndexedArticle = |article| {
    bind summary   = Rune.llm.extract(article.body, schema: String, model: "claude-haiku-4-5-20251001")
    bind embedding = Rune.embed.openai(article.title + " " + summary, model: "text-embedding-3-small")
    IndexedArticle {
        id:        article.id,
        title:     article.title,
        embedding: embedding,
        summary:   summary
    }
}

public stage StoreToVectorDB: List<IndexedArticle> -> Unit = |articles| {
    bind pairs = List.map(articles, |a| { (a.id, a.embedding) })
    Rune.pinecone.upsert(pairs, namespace: "articles", index: "demo-index")
}

public stage SemanticSearch: String -> List<IndexedArticle> = |query| {
    bind query_vec = Rune.embed.openai(query, model: "text-embedding-3-small")
    bind results   = Rune.pinecone.query(query_vec, top_k: 5)
    results
}

pipeline IndexPipeline {
    step "load"   = seq LoadArticles
    step "embed"  = par [EmbedAndSummarize, EmbedAndSummarize, EmbedAndSummarize, EmbedAndSummarize] after "load"
    step "store"  = seq StoreToVectorDB after "embed"
}
```

### Step 2: `infra/e2e-demo/ai-etl/data/articles.csv` 作成

ヘッダ: `id,title,body,tags`
10 行以上のサンプル記事データ

### Step 3: `infra/e2e-demo/ai-etl/workers.yaml` 作成

4 ワーカー定義（localhost ポート 9001〜9004）

### Step 4: `infra/e2e-demo/ai-etl/README.md` 作成

セットアップ手順（fav run コマンド例を含む）

### Step 5: `infra/e2e-demo/ai-etl/scripts/run.sh` 作成

実行スクリプト（`fav run src/pipeline.fav --cluster workers.yaml --checkpoint ./checkpoints/` を含む）

### Step 6: `driver.rs` — `v69100_tests` 追加

`v69000_tests` ブロックの直前に挿入（driver.rs は降順配置）。

テスト 2 件:
- `ai_etl_e2e_demo_structure`: `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` → `"IndexPipeline"` assert
- `ai_etl_demo_has_all_stages`: 同ファイルを読み込み → `"LoadArticles"` / `"EmbedAndSummarize"` / `"StoreToVectorDB"` / `"SemanticSearch"` を個別 assert

### Step 7: テスト実行

```bash
cargo test --bin fav v69100_tests  # 2 件 PASS
cargo test -j 8 -- --test-threads=8  # 3543 tests PASS
```

---

## ファイルパス参照（include_str! 基準）

`driver.rs` は `fav/src/driver.rs` のため:
- `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` → repo root の `infra/e2e-demo/ai-etl/src/pipeline.fav`

> 確認済みパターン: v13.x ECS デモで `include_str!("../../infra/e2e-demo/ecs/src/pipeline.fav")` を使用

---

## sub-version ポリシー

v69.x では `Cargo.toml` / `CHANGELOG.md` は変更しない。v70.0.0 宣言時に一括更新する。
