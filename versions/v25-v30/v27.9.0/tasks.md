# v27.9.0 タスクリスト — sqlite Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T11 | spec-reviewer レビュー実施（実装前・T0 開始前に完了） | [x] |
| T0 | 事前確認: `Cargo.toml` が `27.8.0`、テスト数 2204 件、`vm.rs` に `SQLite.open_raw` がないこと、`runes/sqlite/` が存在しないことを確認。`grep "SQLite" fav/self/checker.fav` で ns_to_effect に未登録を確認。`cargo test sqlite --bin fav` のベースライン件数（0 件）を記録 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.9.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に SQLite primitive 6 件追加（Dbt 末尾・行 18040 直後、Azure Blob 直前。`#[cfg]` ガード付き） | [x] |
| T3 | `runes/sqlite/` ディレクトリ作成 + `sqlite.fav` 新規作成（6 関数: open / open_memory / query / execute / execute_many / close、`!Db` エフェクト） | [x] |
| T4 | `examples/sqlite_etl.fav` 新規作成（`seq SqliteEtlPipeline = CreateTable` パイプライン定義） | [x] |
| T5 | `site/content/docs/runes/sqlite.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.9.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.9.0.json` 新規作成（test_count: 2220） | [x] |
| T10 | `fav/self/checker.fav` 更新: `ns_to_effect` に `"SQLite" => "Db"` を追加（`"Dbt" => "Db"` の直後。**T9 より前に完了すること**。= plan.md Phase 9a） | [x] |
| T9 | `fav/src/driver.rs` 更新: `v279000_tests`（16 件）を `v278000_tests` の直前に追加（= plan.md Phase 9b） | [x] |
| T9.5 | `cargo test v279000 --bin fav` — 16/16 PASS 確認 | [x] |
| T9.6 | `cargo test sqlite --bin fav` — 15 件 PASS 確認 | [x] |
| T9.7 | `cargo test --bin fav` — 2220 件 PASS 確認（リグレッションなし） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.9.0"` であること
- [x] `fav/src/backend/vm.rs` に `SQLite.open_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQLite.open_memory_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQLite.query_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQLite.execute_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQLite.execute_many_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQLite.close_raw` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn open(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn open_memory(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn query(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn execute(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn execute_many(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `public fn close(` が含まれること
- [x] `runes/sqlite/sqlite.fav` に `!Db` エフェクトが使われていること
- [x] `examples/sqlite_etl.fav` に `SqliteEtlPipeline` が含まれること
- [x] `site/content/docs/runes/sqlite.mdx` が存在すること
- [x] `fav/self/checker.fav` の `ns_to_effect` に `"SQLite"` が登録されていること
- [x] `CHANGELOG.md` に `[v27.9.0]` エントリが存在すること
- [x] `benchmarks/v27.9.0.json` が存在すること（test_count: 2220）
- [x] `v279000_tests` 16 件すべて PASS
- [x] 総テスト数 ≥ 2220 件

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `checker_has_sqlite_effect` テストの assert が弱い（`src.contains("\"SQLite\"")` のみ） | `src.contains("ns == \"SQLite\"")` に強化 |
| [MED] `sqlite.fav` に関数間の空行があり dbt.fav パターンと不一致 | 空行を削除して関数間を詰める形式に統一 |
