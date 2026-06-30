# v27.7.0 タスクリスト — `fav infer --from delta` / `--from iceberg`

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T11 | spec-reviewer レビュー実施（実装前・T0 開始前に完了） | [x] |
| T0 | 事前確認: `Cargo.toml` が `27.6.0`、テスト数 2186 件、`vm.rs` に `DeltaLake.infer_schema_raw` がないことを確認。`grep "DeltaLake\|Iceberg" fav/self/checker.fav` で ns_to_effect の登録状況を確認（未登録だが v27.7.0 は CLI のみのため更新不要を確認）。`cargo test infer_delta --bin fav` のベースライン件数（0 件）を記録 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.7.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に `DeltaLake.infer_schema_raw` / `Iceberg.infer_schema_raw` 追加（各ブロック末尾・`#[cfg]` ガード付き） | [x] |
| T3 | `fav/src/driver.rs` に `delta_type_to_favnir` 型マッピング関数追加 | [x] |
| T4 | `fav/src/driver.rs` に `cmd_infer_delta` / `cmd_infer_iceberg` 追加 | [x] |
| T5 | `fav/src/main.rs` に `--path` / `--catalog` フラグと `--from delta` / `--from iceberg` dispatch 追加。`use driver::{..., cmd_infer_delta, cmd_infer_iceberg}` のインポートも追加。`--table` は既存実装済みのため追加不要 | [x] |
| T6 | `site/content/docs/` に `fav infer --from delta/iceberg` のドキュメント追加 | [x] |
| T7 | `CHANGELOG.md` 更新: 先頭に `[v27.7.0]` エントリ追加 | [x] |
| T8 | `benchmarks/v27.7.0.json` 新規作成（test_count: 2195） | [x] |
| T9 | `fav/src/driver.rs` 更新: `v277000_tests`（9 件）を `v276000_tests` の直後に追加 | [x] |
| T9.5 | `cargo test v277000 --bin fav` — 8/8 PASS 確認 | [x] |
| T9.6 | `cargo test infer_delta --bin fav` — PASS 確認 | [x] |
| T10 | `cargo test --bin fav` — 2195 件 PASS 確認（リグレッションなし） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.7.0"` であること
- [x] `fav/src/backend/vm.rs` に `DeltaLake.infer_schema_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.infer_schema_raw` が含まれること
- [x] `fav/src/driver.rs` に `pub fn cmd_infer_delta` が含まれること
- [x] `fav/src/driver.rs` に `pub fn cmd_infer_iceberg` が含まれること
- [x] `fav/src/driver.rs` に `delta_type_to_favnir` 型マッピング関数が含まれること
- [x] `fav/src/main.rs` に `--from delta` ディスパッチが含まれること
- [x] `fav/src/main.rs` に `--from iceberg` ディスパッチが含まれること
- [x] `fav/src/main.rs` に `--path` フラグのパースが含まれること
- [x] `fav/src/main.rs` に `--catalog` フラグのパースが含まれること
- [x] `site/content/docs/` に `fav infer --from delta/iceberg` のドキュメントが存在すること
- [x] `CHANGELOG.md` に `[v27.7.0]` エントリが存在すること
- [x] `benchmarks/v27.7.0.json` が存在すること（test_count: 2195）
- [x] `v277000_tests` 9 件すべて PASS
- [x] 総テスト数 ≥ 2195 件

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] trailing slash バグ: `delta_path_to_type_name` で `rsplit('/').next()` が空文字列を返す | `rsplit('/').find(|s| !s.is_empty())` に修正 |
| [MED] newline インジェクション: `path`/`catalog`/`table` を直接 `format!` に埋め込んでいた | `safe_path = path.replace('\n', "").replace('\r', "")` 等でサニタイズしてから使用 |
