# v29.3.0 Spec — pinecone Rune 追加

**バージョン**: 29.3.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 3)
**前バージョン**: v29.2.0 (mlflow Rune 追加)

---

## 概要

ベクトル DB として業界標準の Pinecone を Favnir から使えるようにする。
LLM Rune（v9.6.0）と組み合わせることで、「Favnir で RAG パイプラインを書く」ことを実現する。
ドキュメントのベクトル化・保存から近傍検索・コンテキスト取得まで `stage` として表現できる。

> **ポジショニング**: MLflow（実験管理）の次のステップとして、
> LLM × ベクトル DB を Favnir pipeline に統合する最初の Rune。
> `fav run rag.fav` でドキュメント検索付き LLM パイプラインが動く。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `runes/pinecone/pinecone.fav` | Pinecone Rune 実装（5 関数）|
| `runes/pinecone/rune.toml` | Rune メタデータ |
| `fav/src/driver.rs` | `v293000_tests` 6 件追加 |
| `fav/Cargo.toml` | version 29.2.0 → 29.3.0 |
| `CHANGELOG.md` | `[v29.3.0]` セクション追加 |
| `benchmarks/v29.3.0.json` | ベンチマーク記録 |
| `site/content/docs/runes/pinecone.mdx` | Pinecone ドキュメント |

---

## Pinecone Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Pinecone.upsert` | `(index: String, vectors: List<PineconeVector>) -> Result<Unit, String> !Http` | ベクトル追加・更新（バッチ対応）|
| `Pinecone.query` | `(index: String, vector: List<Float>, k: Int, filter: String) -> Result<List<String>, String> !Http` | 近傍検索（メタデータフィルタ付き）|
| `Pinecone.delete` | `(index: String, ids: List<String>) -> Result<Unit, String> !Http` | ID 指定でベクトル削除 |
| `Pinecone.fetch` | `(index: String, ids: List<String>) -> Result<List<String>, String> !Http` | ID 指定でベクトル取得 |
| `Pinecone.describe_index_stats` | `(index: String) -> Result<String, String> !Http` | インデックス統計取得 |

### ヘルパー型

```favnir
// ベクトルエントリ
type PineconeVector = {
  id: String,
  values: List<Float>,
  metadata: String
}
```

### 設定

Pinecone Rune は以下の環境変数で接続先を指定する。

| 環境変数 | 説明 |
|---|---|
| `PINECONE_API_KEY` | Pinecone API キー（必須）|
| `PINECONE_ENV` | 環境名（例: `us-east1-gcp`）|
| `PINECONE_BASE_URL` | エンドポイント URL（デフォルト: `https://{index}-{env}.svc.pinecone.io`）|

### 使用例

```favnir
import runes/pinecone
import runes/llm

// ドキュメントをベクトル化して Pinecone に保存
stage IndexDocuments: List<String> -> Unit !Http = |docs| {
  bind vectors <- docs
    |> List.map(|doc|
      bind embedding <- LLM.embed(config.openai, doc)
      Result.ok(PineconeVector { id: Gen.uuid(), values: embedding, metadata: doc })
    )
    |> Result.all
  Pinecone.upsert(config.pinecone_index, vectors)
}

// クエリに関連するドキュメントを取得（RAG）
stage SearchDocuments: String -> List<String> !Http = |query| {
  bind embedding <- LLM.embed(config.openai, query)
  Pinecone.query(config.pinecone_index, embedding, 5, "")
}
```

---

## テスト戦略

### v293000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `pinecone_rune_file_exists` | `runes/pinecone/pinecone.fav` が存在し `upsert` を含む |
| `pinecone_query_fn_exists` | `pinecone.fav` に `query` が存在する |
| `pinecone_delete_and_fetch_fn_exists` | `pinecone.fav` に `delete` と `fetch` が存在する |
| `pinecone_describe_index_stats_fn_exists` | `pinecone.fav` に `describe_index_stats` が存在する |
| `pinecone_rune_toml_exists` | `runes/pinecone/rune.toml` が存在し `pinecone` を含む |
| `changelog_has_v29_3_0` | `CHANGELOG.md` に `[v29.3.0]` が存在する |

検証関数カバレッジ: `upsert`, `query`, `delete`, `fetch`, `describe_index_stats`（5/5 関数 = 100%）

テスト数: 2324 → **2330**（+6）

---

## 完了条件

- [ ] `runes/pinecone/pinecone.fav` に 5 関数が実装されている
- [ ] `runes/pinecone/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `cargo test --bin fav v293000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2330 tests PASS
- [ ] `CHANGELOG.md` に `[v29.3.0]` セクションあり
- [ ] `benchmarks/v29.3.0.json` 存在（test_count: 2330）
- [ ] `site/content/docs/runes/pinecone.mdx` 存在

---

## スコープ外

- Pinecone API への実際の HTTP 接続 — インフラ稼働後に有効化
- `PineconeVector` 型のジェネリクス対応（`T` 型パラメータ）— Favnir の型システム拡張が必要
- Pinecone Serverless API（v29.3.x+ で対応）
