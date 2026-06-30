# v27.4.0 タスクリスト — bigquery Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `27.3.0`、テスト数 2152 件、`runes/bigquery/bigquery.fav` が v15.2.0 旧実装（`!Gcp` エフェクト）であること、`vm.rs` に既存 `BigQuery.query_raw`/`BigQuery.execute_raw` があり `BigQuery.connect_raw` がないことを確認。また `cargo test bigquery --bin fav` のベースライン件数（= `changelog_has_v27_4_0` を含まない既存ヒット数）を記録すること | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.4.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に新 BigQuery primitive 5 件追加（ClickHouse ブロック末尾・Azure Blob 直前。既存 `BigQuery.query_raw`/`BigQuery.execute_raw` は削除しない） | [x] |
| T3 | `runes/bigquery/bigquery.fav` を v27.4.0 新 API（`public fn`、`!Db` エフェクト、5 関数）に全置換 | [x] |
| T4 | `examples/bigquery_analytics.fav` 新規作成（CreateEventTable \|> LoadFromGcs \|> QueryStats） | [x] |
| T5 | `site/content/docs/runes/bigquery.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.4.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.4.0.json` 新規作成（test_count: 2164） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v274000_tests`（12 件）を `v273000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v274000 --bin fav` — 12/12 PASS 確認 | [x] |
| T8.6 | `cargo test bigquery --bin fav` — 11 件以上 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2164 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前・T1 開始前に完了） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.4.0"` であること
- [x] `runes/bigquery/bigquery.fav` に `public fn connect(` が含まれること
- [x] `runes/bigquery/bigquery.fav` に `public fn query(` が含まれること
- [x] `runes/bigquery/bigquery.fav` に `public fn insert(` が含まれること
- [x] `runes/bigquery/bigquery.fav` に `public fn load_from_gcs(` が含まれること
- [x] `runes/bigquery/bigquery.fav` に `public fn create_table(` が含まれること
- [x] `fav/src/backend/vm.rs` に `BigQuery.connect_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `BigQuery.conn_query_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `BigQuery.insert_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `BigQuery.load_from_gcs_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `BigQuery.create_table_raw` が含まれること
- [x] `examples/bigquery_analytics.fav` に `BigQueryAnalyticsPipeline` が含まれること
- [x] `site/content/docs/runes/bigquery.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.4.0]` エントリが存在すること
- [x] `benchmarks/v27.4.0.json` が存在すること（test_count: 2164）
- [x] `v274000_tests` 12 件すべて PASS
- [x] `cargo test bigquery --bin fav` で 11 件以上 PASS
- [x] 総テスト数 ≥ 2164 件

---

## メモ

### 既存 BigQuery primitive の扱い

v15.2.0 の `BigQuery.query_raw`（4 引数: project_id/dataset/sql/params）と `BigQuery.execute_raw` は vm.rs に残す。
新 5 primitives は別名を使用（`conn_query_raw` など）して名前衝突を回避。

### vm.rs 挿入位置

既存 BigQuery ブロック末尾（`"BigQuery.infer_table_raw"` の `}` 直後、約行 17225）に挿入。
Kafka/MSK ブロック（`// ── Kafka / MSK primitives (v15.4.0)`）の直前。
既存 `BigQuery.query_raw` / `BigQuery.execute_raw` / `BigQuery.infer_table_raw` と同一ブロック内にまとめることでコードの可読性を維持する。
wasm32 ガードは ClickHouse と同パターン（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アーム）で追加する。

### `connect_raw` の戻り値

接続ハンドルとして `"bigquery-stub-conn"` を返す。
v28.x で google-cloud-bigquery クレートを統合した際は `_config` を実接続に渡す予定（TODO コメント明記）。

### `runes/bigquery/bigquery.fav` 置換

旧実装（v15.2.0）は `!Gcp` エフェクト・非 `public` fn。
新実装（v27.4.0）は `!Db` エフェクト・`public fn` 5 関数に全置換。

### テスト数計算

2152（v27.3.0 完了後）+ 12（v274000_tests）= 2164

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/bigquery/bigquery.fav` | `favnir/runes/bigquery/bigquery.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/bigquery_analytics.fav` | `favnir/examples/bigquery_analytics.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [BUG] vm.rs 既存 `BigQuery.query_raw` / `BigQuery.execute_raw` / `BigQuery.infer_table_raw` に wasm32 ガードがなく、新規 5 件との非対称 | v28.x 移行時にまとめて対処する旨の TODO コメントを旧 BigQuery ブロック先頭に追記 |
| [STYLE] `connect_raw` TODO に config フォーマット仕様の記述なし | `"project:X,dataset:Y"` カンマ区切り形式と v28.x での `fav.toml [bigquery]` 統合予定を TODO に補足 |
