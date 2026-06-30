# v27.3.0 タスクリスト — clickhouse Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `27.2.0`、テスト数 2142 件、`runes/clickhouse/` が存在しない、`vm.rs` に `ClickHouse.connect_raw` がない、`examples/clickhouse_analytics.fav` が存在しないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.3.0"` に bump | [x] |
| T2 | `runes/clickhouse/clickhouse.fav` を新規作成（4 関数: connect / query / insert / async_insert） | [x] |
| T3 | `fav/src/backend/vm.rs` に `ClickHouse.*_raw` primitive 4 件追加（Iceberg ブロック直後・Azure Blob 直前、`#[cfg]` ガード付き、stub のため rune_loader 登録なし） | [x] |
| T4 | `examples/clickhouse_analytics.fav` 新規作成（LoadEvents \|> InsertProcessed） | [x] |
| T5 | `site/content/docs/runes/clickhouse.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.3.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.3.0.json` 新規作成（test_count: 2152） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v273000_tests`（10 件）を `v272000_tests` の直後に追加（v28.x で `query[T]` に変更する際は `clickhouse_rune_has_query_fn` のアサーション文字列も更新すること） | [x] |
| T8.5 | `cargo test v273000 --bin fav` — 10/10 PASS 確認 | [x] |
| T8.6 | `cargo test clickhouse --bin fav` — 9 件以上 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2152 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前・T1 開始前に完了） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.3.0"` であること
- [x] `runes/clickhouse/clickhouse.fav` に `fn connect(` が含まれること
- [x] `runes/clickhouse/clickhouse.fav` に `fn query(` が含まれること
- [x] `runes/clickhouse/clickhouse.fav` に `fn insert(` が含まれること
- [x] `runes/clickhouse/clickhouse.fav` に `fn async_insert(` が含まれること
- [x] `fav/src/backend/vm.rs` に `ClickHouse.connect_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `ClickHouse.query_raw` が含まれること（v28.x で `query[T]` に変更予定）
- [x] `fav/src/backend/vm.rs` に `ClickHouse.insert_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `ClickHouse.async_insert_raw` が含まれること
- [x] `examples/clickhouse_analytics.fav` に `ClickHouseAnalyticsPipeline` が含まれること
- [x] `site/content/docs/runes/clickhouse.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.3.0]` エントリが存在すること
- [x] `benchmarks/v27.3.0.json` が存在すること（test_count: 2152）
- [x] `v273000_tests` 10 件すべて PASS
- [x] `cargo test clickhouse --bin fav` で 9 件以上 PASS
- [x] 総テスト数 ≥ 2150 件

---

## メモ

### v273000_tests が 8 → 10 件になった理由

spec.md では 8 件を想定（`vm_has_connect_raw` と `vm_has_insert_raw` の 2 件のみ）だったが、
v27.1.0（`optimize_fn`/`read_with_filter_fn` 欠落）・v27.2.0（`list_snapshots_fn`/`list_snapshots_raw` 欠落）の
コードレビュー教訓を踏まえ、全 4 VM primitive（`connect_raw`/`query_raw`/`insert_raw`/`async_insert_raw`）の
確認テストを追加した。結果 test_count: 2152（2142 + 10）。

### vm.rs 挿入位置

Iceberg ブロック末尾（`"Iceberg.list_snapshots_raw" => Ok(err_vm(...))` の wasm32 アーム直後）に挿入。
Azure Blob ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。

### `connect_raw` の戻り値

接続ハンドルとして `"clickhouse-stub-conn"` を返す。
v28.x で clickhouse-rs を統合した際は実際の接続識別子に置き換える。

### テスト数計算

2142（v27.2.0 完了後）+ 10（v273000_tests）= 2152

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/clickhouse/clickhouse.fav` | `favnir/runes/clickhouse/clickhouse.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/clickhouse_analytics.fav` | `favnir/examples/clickhouse_analytics.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] vm.rs `connect_raw`: v28.x 移行 TODO が不具体 | `_config` を clickhouse-rs の Client 初期化に渡す旨を TODO コメントに明記 |
| [LOW] `InsertProcessed` の戻り値型が `Result<String, String>` で不正確 | `Result<Unit, String>` に変更し `ClickHouse.insert` の結果を直接返すよう修正（example・docs 両方） |
