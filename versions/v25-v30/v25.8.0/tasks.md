# v25.8.0 タスクリスト — elasticsearch Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.8.0"` に bump（ureq v2 は既存のため追加 crate 不要） | [x] |
| T1 | `fav/src/ast.rs` 更新（`Effect::Elasticsearch` 追加 — `DynamoDB,` の直後） | [x] |
| T2 | `fav/src/error_catalog.rs` 更新（E0324 `UndeclaredElasticsearchEffect` 追加 — **E0323 エントリの閉じ `},` の直後、E0365 エントリの前**） | [x] |
| T3 | `fav/src/fmt.rs` / `fav/src/emit_python.rs` / `fav/src/lint.rs` / `fav/src/middle/reachability.rs` / `fav/src/middle/ast_lower_checker.rs` / `fav/src/lineage.rs` 更新（`Effect::Elasticsearch` 対応・6 ファイル。`lineage.rs` は `format_effects`（DynamoDB の直後）と `classify_capability_kind`（DynamoDB の直後）**両方**に追加） | [x] |
| T4 | `fav/src/middle/checker.rs` 更新（`require_elasticsearch_effect` / `ns_to_inferred_effect` / ES builtin fns） | [x] |
| T5 | `fav/src/frontend/parser.rs` 更新（`"Elasticsearch" => Effect::Elasticsearch` アーム追加） | [x] |
| T6 | `fav/src/driver.rs` 更新（`format_effects` / `effect_json_name` に Elasticsearch アーム追加） | [x] |
| T7 | `cargo build` で exhaustive match エラーなし確認 | [x] |
| T8 | `fav/src/backend/vm.rs` 更新（`get_es_url` / `es_http` / `es_http_ndjson` ヘルパー + `ES.*_raw` 8 件 primitives） | [x] |
| T9 | `runes/elasticsearch/elasticsearch.fav` 新規作成（`type ESConn` + 8 関数、ディレクトリも新規作成） | [x] |
| T10 | `examples/elasticsearch_logs_etl.fav` 新規作成（`import rune "elasticsearch"` + IndexLog / SearchLogs / LogsETL） | [x] |
| T11 | `site/content/docs/runes/elasticsearch.mdx` 新規作成（全 API 記載、Docker セットアップ手順含む） | [x] |
| T12 | `CHANGELOG.md` 更新（`[v25.8.0]` エントリ追加） | [x] |
| T13 | `benchmarks/v25.8.0.json` 新規作成（test_count: 2028） | [x] |
| T14 | `fav/src/driver.rs` 更新（`v258000_tests` 7 件追加） | [x] |
| T15 | `cargo test v258000` — 7 件 PASS 確認 | [x] |
| T16 | `cargo test` 総テスト数 ≥ 2028 件 確認 | [x] |
| T17 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x]`ES.connect` が `runes/elasticsearch/elasticsearch.fav` に存在する
- [x]`ES.search` / `ES.knn_search` が実装済み（read 系）
- [x]`ES.index` / `ES.index_with_id` / `ES.bulk` / `ES.create_index` / `ES.delete` が実装済み（write 系）
- [x]`ES.*_raw` 8 件すべてが `fav/src/backend/vm.rs` に存在する
- [x]`Effect::Elasticsearch` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match エラーなし確認済み）
- [x]E0324 が `fav/src/error_catalog.rs` に存在する
- [x]`examples/elasticsearch_logs_etl.fav` が存在し `import rune "elasticsearch"` / `index` / `search` を含む
- [x]`CHANGELOG.md` に `[v25.8.0]` エントリが存在する
- [x]`site/content/docs/runes/elasticsearch.mdx` が存在し全 API を記載している
- [x]`v258000_tests` 7 件すべて PASS（`cargo test v258000` 実行済み）
- [x]総テスト数 ≥ 2028 件

---

## メモ

- `ureq = "2"` は既存（native-only deps）→ 追加 crate 不要
- `Effect::Elasticsearch` は完全新規（v25.8.0 で初追加）→ ast.rs + 6 ファイル exhaustive match 更新が必要
- E0324 の挿入位置: E0323（DynamoDB）エントリの**直後**
- `ESConn(String)`: checker は `Result<String, String>` として扱う（DynamoConn / KafkaConn と同パターン）
- `es_http` ヘルパー: `ureq::request(method, url)` でメソッドを文字列指定（vm.rs 既存パターン）。base64 は `base64::engine::general_purpose::STANDARD.encode(...)` インスタンス呼び出し（`use base64::Engine as _` が必要）
- `es_http_ndjson`: bulk 専用（Content-Type: application/x-ndjson が必須）
- `bulk_raw`: docs_json（JSON 配列）→ NDJSON 変換してから `POST /_bulk` に送信
- `search_raw` / `knn_search_raw`: レスポンスの `hits.hits[]._source` を抽出して JSON 配列文字列で返す
- `create_index_raw`: `mapping_json` が `""` のとき `{}` ボディで `PUT /{index}` を呼ぶ
- `lineage.rs` の ES 分類: `classify_capability_kind` に `ast::Effect::Elasticsearch => ("io", "Search")` 追加
- `cfg(not(target_arch = "wasm32"))` ガードを全 ES primitive に付与
- 目標テスト数 2028 件（v25.7.0 終了時 2021 件 + 7 件）
- **ロードマップ差分**: ロードマップの `search[T]` / `knn_search[T]` 型付きジェネリクスは未実装。JSON 配列文字列で「read 条件」を達成し、型付き変換は v26.x で対応予定
- ローカル: `docker run -p 9200:9200 -e "discovery.type=single-node" -e "xpack.security.enabled=false" elasticsearch:8.11.0`

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `examples/elasticsearch_logs_etl.fav` で `ES.connect/index/search` を使用（Kafka 例の `Kafka.connect` パターンと不一致） | `Elasticsearch.connect/index/search` に修正 |
| [LOW] `bulk_raw` — index 名のエスケープなし | ES 命名規則上 `"` 不可のため v26.x 対応予定として記録 |
| [LOW] `delete_raw` — 404 を Err で返す | 他 Rune との一貫性を優先、変更なし |
