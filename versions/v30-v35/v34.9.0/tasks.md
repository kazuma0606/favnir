# v34.9.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.8.0` であること
- [x] `benchmarks/v34.8.0.json` の `tests_passed` が 2576 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2576 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v349000_tests` が存在しないこと
- [x] v34.8.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_8_0` が v348000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v348000` が 5/5 PASS であること
- [x] `site/content/docs/tools/upgrade-guide.mdx` が存在しないこと（新規作成対象）
- [x] `fav/tests/fixtures/ctx_migration/` が存在しないこと（新規作成対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.8.0` → `34.9.0` に更新
- [x] **T2** `site/content/docs/tools/upgrade-guide.mdx` — fav upgrade コマンド公式ドキュメントを新規作成
        （概要 / --dry-run・--in-place フラグ / fav migrate との使い分け / ワークフロー / トラブルシューティング）
- [x] **T3** `fav/tests/fixtures/ctx_migration/before.fav` — !Http エフェクト使用の移行前フィクスチャを作成
- [x] **T4** `fav/tests/fixtures/ctx_migration/after.fav` — AppCtx 使用の移行後フィクスチャを作成
- [x] **T5** `fav/src/driver.rs` — `cargo_toml_version_is_34_8_0` をスタブ化
- [x] **T6** `fav/src/driver.rs` — `v349000_tests`（5 件）を追加
        挿入位置: `v348000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし（`include_str!` のみ使用）
- [x] **T7** `CHANGELOG.md` — `[v34.9.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v34.9.0.json` — 新規作成（`tests_passed`: 2581）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v34.9.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v349000 2>&1 | tail -8` — 5/5 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2581 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v34.9.0.json` の `tests_passed` を実測値（2581）で更新
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.9.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.9.0"`
- [x] `cargo_toml_version_is_34_8_0` が空スタブになっていること
- [x] `cargo test --bin fav v349000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2581 件、0 failures）
- [x] `site/content/docs/tools/upgrade-guide.mdx` が存在し `"fav upgrade"` と `"--from-effects"` を含むこと
- [x] `fav/tests/fixtures/ctx_migration/before.fav` が存在し `"!Http"` を含むこと
- [x] `fav/tests/fixtures/ctx_migration/after.fav` が存在し `"AppCtx"` を含むこと
- [x] `CHANGELOG.md` に `[v34.9.0]` セクション
- [x] `benchmarks/v34.9.0.json` 存在かつ `tests_passed` が実測値（2581）
- [x] `versions/current.md` が v34.9.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v349000_tests` に `use super::*` が**ない**こと（`include_str!` のみ使用）
- [x] `upgrade_guide_exists` の `include_str!` パスが `"../../site/content/docs/tools/upgrade-guide.mdx"` であること
- [x] フィクスチャの `include_str!` パスが `"../tests/fixtures/ctx_migration/..."` であること
- [x] `cargo_toml_version_is_34_8_0` が空スタブになっていること（コメント付き）
- [x] 挿入位置が `v348000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.9.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` の「次に切る版」が `v35.0.0` になっていること
- [x] `upgrade-guide.mdx` に fav upgrade / --dry-run / --in-place / fav migrate との使い分けが含まれていること
- [x] フィクスチャが 2 ファイルとも正しい内容（before: `!Http`、after: `AppCtx`）であること
