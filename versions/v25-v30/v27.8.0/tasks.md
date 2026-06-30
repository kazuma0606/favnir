# v27.8.0 タスクリスト — dbt 連携 Rune

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T11 | spec-reviewer レビュー実施（実装前・T0 開始前に完了） | [x] |
| T0 | 事前確認: `Cargo.toml` が `27.7.0`、テスト数 2195 件、`vm.rs` に `Dbt.ref_raw` がないこと、`runes/dbt/` が存在しないことを確認。`grep "Dbt" fav/self/checker.fav` で ns_to_effect に未登録を確認。`cargo test dbt --bin fav` のベースライン件数（0 件）を記録 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.8.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に `Dbt.ref_raw` / `Dbt.source_raw` 追加（JSONL 末尾・行 18013 直後、Azure Blob 直前。`#[cfg]` ガード付き） | [x] |
| T3 | `runes/dbt/dbt.fav` 新規作成（2 関数: ref / source、`!Db` エフェクト） | [x] |
| T4 | `examples/dbt_pipeline.fav` 新規作成（`seq DbtRefPipeline = LoadCustomerSummary` パイプライン定義） | [x] |
| T5 | `fav/tests/fixtures/dbt_manifest.json` 新規作成（"nodes" / "sources" キー含む最小フィクスチャ） | [x] |
| T6 | `site/content/docs/runes/dbt.mdx` 新規作成 | [x] |
| T7 | `CHANGELOG.md` 更新: 先頭に `[v27.8.0]` エントリ追加 | [x] |
| T8 | `benchmarks/v27.8.0.json` 新規作成（test_count: 2204） | [x] |
| T10 | `fav/self/checker.fav` 更新: `ns_to_effect` に `"Dbt" => "Db"` を追加（`"JSONL" => "IO"` の直後。**T9 より前に完了すること**。= plan.md Phase 9a） | [x] |
| T9 | `fav/src/driver.rs` 更新: `v278000_tests`（9 件）を `v277000_tests` の直前に追加（= plan.md Phase 9b） | [x] |
| T9.5 | `cargo test v278000 --bin fav` — 9/9 PASS 確認 | [x] |
| T9.6 | `cargo test dbt --bin fav` — 7 件以上 PASS 確認 | [x] |
| T9.7 | `cargo test --bin fav` — 2204 件 PASS 確認（リグレッションなし） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.8.0"` であること
- [x] `fav/src/backend/vm.rs` に `Dbt.ref_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Dbt.source_raw` が含まれること
- [x] `runes/dbt/dbt.fav` に `public fn ref(` が含まれること
- [x] `runes/dbt/dbt.fav` に `public fn source(` が含まれること
- [x] `runes/dbt/dbt.fav` に `!Db` エフェクトが使われていること
- [x] `examples/dbt_pipeline.fav` に `DbtRefPipeline` が含まれること
- [x] `fav/tests/fixtures/dbt_manifest.json` に `"nodes"` が含まれること
- [x] `site/content/docs/runes/dbt.mdx` が存在すること
- [x] `fav/self/checker.fav` の `ns_to_effect` に `"Dbt"` が登録されていること
- [x] `CHANGELOG.md` に `[v27.8.0]` エントリが存在すること
- [x] `benchmarks/v27.8.0.json` が存在すること（test_count: 2204）
- [x] `v278000_tests` 9 件すべて PASS
- [x] 総テスト数 ≥ 2204 件

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] vm.rs section header が `// ── dbt primitives`（小文字）で他と不統一 | `// ── Dbt primitives` に修正 |
| [MED] テスト件数が当初 8 件、checker.fav 未テスト | `checker_has_dbt_effect` を 9 件目として追加、test_count 2203 → 2204 に修正 |
