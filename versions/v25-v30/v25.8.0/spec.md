# v25.8.0 仕様書 — elasticsearch Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.8.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | elasticsearch Rune の「動く Rune」5 条件達成 |
| 依存関係 | なし（ureq は既存 Cargo.toml に存在） |
| 目標テスト数 | 2028 件（v25.7.0: 2021 件 + 7 件） |

---

## 背景と目的

v25.7.0 で kafka Rune を実質化した。次はコア 8 Rune の最後である Elasticsearch を実質化する。

Elasticsearch / OpenSearch は全文検索・ログ分析・ベクトル近傍検索（kNN）のハブであり、
後フェーズ（v29.1 の Rune Registry 検索基盤）でも再利用される。

Elasticsearch の REST API は HTTP + JSON であり、`ureq`（既存）で全操作を実装できる。
追加 crate は不要。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `ELASTICSEARCH_URL` 環境変数（例: `http://localhost:9200`）または Docker ES 8.x 経由で接続確立 |
| 2 | read | `ES.search` / `ES.knn_search` — hits を JSON 配列文字列で返す |
| 3 | write | `ES.index` / `ES.index_with_id` / `ES.bulk` / `ES.create_index` / `ES.delete` |
| 4 | error | `Result<T, String>` 統一、エラーメッセージにインデックス名を含む |
| 5 | test | `v258000_tests` 7 件 PASS + `examples/elasticsearch_logs_etl.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/elasticsearch/` | **存在しない** | v25.8.0 で新規作成 |
| `Effect::Elasticsearch` | **存在しない** | v25.8.0 で追加 |
| E0324 | **存在しない** | v25.8.0 で追加 |
| `ES.*_raw` | **存在しない** | v25.8.0 で 8 件追加 |
| `ureq = "2"` | 既存 Cargo.toml | native-only deps に存在 |

---

## 機能仕様

### 型定義

```favnir
// Elasticsearch ベース URL ラッパー型
// "" → ELASTICSEARCH_URL 環境変数 → "http://localhost:9200"
type ESConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `ES.connect` | `(url: String) -> Result<ESConn, String> !Elasticsearch` | GET / ping で接続確認 |
| `ES.index` | `(conn: ESConn, index: String, doc_json: String) -> Result<String, String> !Elasticsearch` | POST /{index}/_doc、`_id` を返す |
| `ES.index_with_id` | `(conn: ESConn, index: String, id: String, doc_json: String) -> Result<Unit, String> !Elasticsearch` | PUT /{index}/_doc/{id} |
| `ES.search` | `(conn: ESConn, index: String, query_json: String) -> Result<String, String> !Elasticsearch` | POST /{index}/_search、hits を JSON 配列文字列で返す |
| `ES.bulk` | `(conn: ESConn, index: String, docs_json: String) -> Result<Int, String> !Elasticsearch` | POST /_bulk、インデックス件数を返す（最大 1000 件推奨） |
| `ES.delete` | `(conn: ESConn, index: String, id: String) -> Result<Unit, String> !Elasticsearch` | DELETE /{index}/_doc/{id} |
| `ES.knn_search` | `(conn: ESConn, index: String, knn_json: String) -> Result<String, String> !Elasticsearch` | POST /{index}/_search（knn ボディ）、hits を JSON 配列文字列で返す |
| `ES.create_index` | `(conn: ESConn, index: String, mapping_json: String) -> Result<Unit, String> !Elasticsearch` | PUT /{index}、`""` で空マッピング |

> **戻り値**:
> - `search` / `knn_search` は `hits.hits[]._source` の JSON 配列文字列（例: `[{"title": "doc1"}, ...]`）
> - `index` は生成された `_id` 文字列
> - `bulk` は `docs_json` の JSON 配列長（インデックス件数）を返す
> - `connect` が失敗した場合: `Result.err("ES.connect_raw: ping failed: ...")`

---

## エラーコード追加仕様（E0324）

| コード | 名前 | 説明 |
|---|---|---|
| E0324 | UndeclaredElasticsearchEffect | `!Elasticsearch` エフェクトなしで ES 系 Rune を呼び出した場合 |

---

## Elasticsearch クライアント実装方針

- `ureq = "2"` を再利用（追加 crate なし）
- URL 解決ロジック:
  1. `ESConn` の文字列が空 → `ELASTICSEARCH_URL` 環境変数 → `"http://localhost:9200"`
  2. それ以外 → `ESConn` の文字列をそのまま使用
- 認証（優先順位）:
  1. `ELASTICSEARCH_API_KEY` 環境変数 → `Authorization: ApiKey <key>` ヘッダ
  2. `ELASTICSEARCH_USERNAME` / `ELASTICSEARCH_PASSWORD` → HTTP Basic 認証
  3. なし → 認証ヘッダなし（開発時の単純起動 ES に対応）
- 全 primitive は `cfg(not(target_arch = "wasm32"))` ガード付き
- 共通ヘルパー `fn es_http(method: &str, url: &str, body_opt: Option<&str>) -> Result<String, String>` で HTTP 呼び出しを統一

### REST API マッピング

| 操作 | HTTP | パス |
|---|---|---|
| 接続確認 | `GET` | `/` |
| ドキュメント追加（ID 自動） | `POST` | `/{index}/_doc` |
| ドキュメント追加（ID 指定） | `PUT` | `/{index}/_doc/{id}` |
| 検索 | `POST` | `/{index}/_search` |
| バルク | `POST` | `/_bulk`（NDJSON 形式） |
| 削除 | `DELETE` | `/{index}/_doc/{id}` |
| ベクトル検索 | `POST` | `/{index}/_search`（knn ボディ） |
| インデックス作成 | `PUT` | `/{index}` |

### VM primitives 一覧（新規 8 件）

| primitive 名 | 引数 | 戻り値 |
|---|---|---|
| `ES.connect_raw` | `url: String` | `Result<String, String>`（ESConn ラッパー） |
| `ES.index_raw` | `url: String, index: String, doc_json: String` | `Result<String, String>`（`_id`） |
| `ES.index_with_id_raw` | `url: String, index: String, id: String, doc_json: String` | `Result<Unit, String>` |
| `ES.search_raw` | `url: String, index: String, query_json: String` | `Result<String, String>`（JSON 配列） |
| `ES.bulk_raw` | `url: String, index: String, docs_json: String` | `Result<Int, String>` |
| `ES.delete_raw` | `url: String, index: String, id: String` | `Result<Unit, String>` |
| `ES.knn_search_raw` | `url: String, index: String, knn_json: String` | `Result<String, String>`（JSON 配列） |
| `ES.create_index_raw` | `url: String, index: String, mapping_json: String` | `Result<Unit, String>` |

> **connect_raw の戻り型**（checker レベル）: `Result<String, String>`。
> `ESConn(String)` は名目型ラッパーであり checker は String として扱う（DynamoConn / KafkaConn と同パターン）。

---

## `examples/elasticsearch_logs_etl.fav`

```favnir
import rune "elasticsearch"

// ── Elasticsearch を使ったログ ETL デモ (v25.8.0) ─────────────────────────────
// 前提: docker run -p 9200:9200 -e "discovery.type=single-node" \
//           -e "xpack.security.enabled=false" elasticsearch:8.11.0
// 実行: fav run examples/elasticsearch_logs_etl.fav

stage IndexLog: String -> Result<String, String> !Elasticsearch = |log_json| {
    bind conn <- ES.connect("http://localhost:9200")
    bind _    <- ES.create_index(conn, "logs", "")
    ES.index(conn, "logs", log_json)
}

stage SearchLogs: String -> Result<String, String> !Elasticsearch = |keyword| {
    bind conn <- ES.connect("http://localhost:9200")
    ES.search(conn, "logs", "{\"query\": {\"match\": {\"message\": \"" + keyword + "\"}}}")
}

seq LogsETL = IndexLog |> SearchLogs
```

---

## やらないこと（スコープ外）

- スクロール API（`search_after` / `scroll`）によるページネーション
- インデックスエイリアス管理（`_aliases`）
- スナップショット・リストア
- Index Template / Component Template 管理
- Watcher / Alerting
- 認証トークンの自動更新（API キー / JWT）

> **ロードマップとの差分**: ロードマップに `search[T]` / `knn_search[T]` のような型付きジェネリクスが記載されているが、checker は現在非ジェネリックシグネチャのみ対応。`search` / `knn_search` は JSON 配列文字列を返す形で「read 条件」を達成する。型付き変換は v26.x で対応予定。

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `ES.connect` が `runes/elasticsearch/elasticsearch.fav` に実装済み |
| 2 | `ES.search` / `ES.knn_search` が実装済み（read 系） |
| 3 | `ES.index` / `ES.index_with_id` / `ES.bulk` / `ES.create_index` / `ES.delete` が実装済み（write 系） |
| 4 | `ES.*_raw` 8 件すべてが `fav/src/backend/vm.rs` に存在する |
| 5 | `Effect::Elasticsearch` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match エラーなし） |
| 6 | E0324 が `fav/src/error_catalog.rs` に存在する |
| 7 | `examples/elasticsearch_logs_etl.fav` が存在し `import rune "elasticsearch"` + `index` + `search` を含む |
| 8 | `CHANGELOG.md` に `[v25.8.0]` エントリが存在する |
| 9 | `site/content/docs/runes/elasticsearch.mdx` に全 API が記載済み |
| 10 | `cargo test v258000` で 7 件すべて PASS |
| 11 | 総テスト数 ≥ 2028 件 |

---

## 設計判断

### ESConn(String) の checker 互換性

`connect_raw` は `Result<String, String>` を返す。Rune の `connect` は `Result<ESConn, String>` を返すが、
`ESConn(String)` は名目型ラッパーであり checker 内で `String` として扱われるため `fav check` はエラーにならない
（DynamoConn / KafkaConn と同パターン）。

### bulk の NDJSON 変換

`bulk_raw` は `docs_json`（JSON 配列文字列）を受け取り、内部で NDJSON 形式（`{"index": {}}\n{...}\n` の繰り返し）に変換して `POST /_bulk` に送信する。
`Content-Type: application/x-ndjson` ヘッダを設定する。

### search / knn_search の戻り値

ES の `_search` レスポンスの `hits.hits[]._source` を抽出して JSON 配列文字列として返す。
ユーザーが `_id` / `_score` も必要な場合は `query_json` に `"_source": true` を指定する（将来の改善余地）。

### create_index の空マッピング

`mapping_json` が `""` の場合、`PUT /{index}` に空ボディ（`{}`）を送信する（ES 8.x ではデフォルトマッピングが使用される）。

### connect ごとの GET / ping

`ES.connect` は呼び出しのたびに `GET /` を送信する。コネクションプールは v26.x で対応予定（vm.rs に TODO コメント追記）。

---

## 検証コマンド

```bash
cd fav && cargo test v258000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
