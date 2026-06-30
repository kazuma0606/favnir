# v27.5.0 タスクリスト — redshift Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `27.4.0`、テスト数 2164 件、`runes/redshift/` が存在しない、`vm.rs` に `Redshift.connect_raw` がないことを確認。また `cargo test redshift --bin fav` のベースライン件数（0 件であること）を記録 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.5.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に Redshift primitive 5 件追加（ClickHouse ブロック末尾・Azure Blob 直前。`#[cfg]` ガード付き） | [x] |
| T3 | `runes/redshift/redshift.fav` 新規作成（5 関数: connect / query / execute / copy_from_s3 / unload_to_s3） | [x] |
| T4 | `examples/redshift_analytics.fav` 新規作成（LoadFromS3 \|> QuerySummary \|> UnloadToS3） | [x] |
| T5 | `site/content/docs/runes/redshift.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.5.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.5.0.json` 新規作成（test_count: 2176） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v275000_tests`（12 件）を `v274000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v275000 --bin fav` — 12/12 PASS 確認 | [x] |
| T8.6 | `cargo test redshift --bin fav` — 11 件以上 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2176 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前・T1 開始前に完了） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.5.0"` であること
- [x] `runes/redshift/redshift.fav` に `public fn connect(` が含まれること
- [x] `runes/redshift/redshift.fav` に `public fn query(` が含まれること
- [x] `runes/redshift/redshift.fav` に `public fn execute(` が含まれること
- [x] `runes/redshift/redshift.fav` に `public fn copy_from_s3(` が含まれること
- [x] `runes/redshift/redshift.fav` に `public fn unload_to_s3(` が含まれること
- [x] `fav/src/backend/vm.rs` に `Redshift.connect_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Redshift.query_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Redshift.execute_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Redshift.copy_from_s3_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Redshift.unload_to_s3_raw` が含まれること
- [x] `examples/redshift_analytics.fav` に `RedshiftAnalyticsPipeline` が含まれること
- [x] `site/content/docs/runes/redshift.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.5.0]` エントリが存在すること
- [x] `benchmarks/v27.5.0.json` が存在すること（test_count: 2176）
- [x] `v275000_tests` 12 件すべて PASS
- [x] `cargo test redshift --bin fav` で 11 件 PASS
- [x] 総テスト数 ≥ 2176 件

---

## メモ

### vm.rs 挿入位置

ClickHouse ブロック末尾（`"ClickHouse.async_insert_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17878 付近）に挿入。
Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。
wasm32 ガードは ClickHouse と同パターン（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アーム）。

### `connect_raw` の戻り値

接続ハンドルとして `"redshift-stub-conn"` を返す。
v28.x で postgres クレートを統合した際は `_config`（DSN / 接続文字列）を実接続に渡す予定（TODO コメント明記）。

### `execute_raw` の戻り値型

`Result<Int, String>` → `ok_vm(VMValue::Int(0))` を返す（影響行数 0）。
他の stub と異なり `VMValue::Unit` ではなく `VMValue::Int(0)` であることに注意。

### example の型連鎖

```
LoadFromS3:    Unit   -> Result<Unit, String>   !Db
QuerySummary:  Unit   -> Result<String, String> !Db
UnloadToS3:    String -> Result<Unit, String>   !Db = |query| { ... }
```
`LoadFromS3` の成功値（Unit）→ `QuerySummary` の引数（Unit）: 接続 OK
`QuerySummary` の成功値（String）→ `UnloadToS3` の引数（String）: `|query|` で受け取り `unload_to_s3(conn, query, ...)` に渡す

**注意**: `UnloadToS3` は `|_|` ではなく `|query|` でラムダ引数を受け取ること（前ステージの SQL 結果文字列を UNLOAD に渡す）。

### テスト数計算

2164（v27.4.0 完了後）+ 12（v275000_tests）= 2176

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/redshift/redshift.fav` | `favnir/runes/redshift/redshift.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/redshift_analytics.fav` | `favnir/examples/redshift_analytics.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [BUG] checker.fav の `ns_to_effect` に `"Redshift"` / `"ClickHouse"` が未登録。セルフホストパスで `!Db` エフェクト検証がスキップされる | `checker.fav` `ns_to_effect` に `"ClickHouse" => "Db"` / `"Redshift" => "Db"` を追加。2176 件 PASS 確認 |
| [STYLE] `execute_raw` の戻り値 `VMValue::Int(0)` にコメント不足 | 「影響行数を返す（stub は 0 固定）」コメントは既存コメントで補完 |
