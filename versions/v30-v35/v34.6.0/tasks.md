# v34.6.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.5.0` であること
- [x] `benchmarks/v34.5.0.json` の `tests_passed` が 2561 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2561 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v346000_tests` が存在しないこと
- [x] v34.5.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_5_0` が v345000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_5_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v345000` が 5/5 PASS であること
- [x] `runes/ctx/db.fav` が存在しないこと（新規作成対象）
- [x] `runes/ctx/http.fav` が存在しないこと（新規作成対象）
- [x] `runes/ctx/stream.fav` が存在しないこと（新規作成対象）
- [x] `site/content/docs/runes/ctx-migration-status.mdx` が存在しないこと（新規作成対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.5.0` → `34.6.0` に更新
- [x] **T2** `runes/ctx/db.fav` — `DbCtx` interface を新規作成（3 メソッド: execute / query_raw / transaction）
- [x] **T3** `runes/ctx/http.fav` — `HttpClient` interface を新規作成（4 メソッド: get / post / put / delete）
- [x] **T4** `runes/ctx/stream.fav` — `StreamClient` interface を新規作成（3 メソッド: produce / consume / commit）
- [x] **T5** `site/content/docs/runes/ctx-migration-status.mdx` — ctx 移行ステータスページを新規作成
- [x] **T6** `fav/src/driver.rs` — `cargo_toml_version_is_34_5_0` をスタブ化
- [x] **T7** `fav/src/driver.rs` — `v346000_tests`（5 件）を追加
        挿入位置: `v345000_tests` 直後・`// ── v31.7.0 tests` の前
- [x] **T8** `CHANGELOG.md` — `[v34.6.0]` セクションを先頭に追記
- [x] **T9** `benchmarks/v34.6.0.json` — 新規作成（`tests_passed`: 2566）
- [x] **T10** `versions/current.md` — 「最新安定版」欄を v34.6.0 に更新

---

## テスト確認

- [x] **T11** `cargo test --bin fav v346000 2>&1 | tail -8` — 5/5 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2566 passed、0 failures）

---

## 完了処理

- [x] **T13** `benchmarks/v34.6.0.json` の `tests_passed` を実測値で更新（実測値 2566 = 想定値と一致）
- [x] **T14** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.6.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.6.0"`
- [x] `cargo_toml_version_is_34_5_0` が空スタブになっていること
- [x] `cargo test --bin fav v346000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2566 件想定 = 2561 + 5、0 failures）
- [x] `runes/ctx/db.fav` が存在し `"DbCtx"` を含むこと
- [x] `runes/ctx/http.fav` が存在し `"HttpClient"` を含むこと
- [x] `runes/ctx/stream.fav` が存在し `"StreamClient"` を含むこと
- [x] `site/content/docs/runes/ctx-migration-status.mdx` が存在し `"DbCtx"` を含むこと
- [x] `CHANGELOG.md` に `[v34.6.0]` セクション
- [x] `benchmarks/v34.6.0.json` 存在かつ `tests_passed` が実測値（2566）
- [x] `versions/current.md` が v34.6.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v346000_tests` に `use super::*` が**ない**こと（`include_str!` のみ使用）
- [x] `db_ctx_rune_exists` / `http_ctx_rune_exists` / `stream_ctx_rune_exists` / `ctx_migration_status_page_exists` は `include_str!` のみ使用
- [x] `cargo_toml_version_is_34_5_0` が空スタブになっていること（コメント付き）
- [x] 挿入位置が `v345000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.6.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.6.0 に更新されていること
- [x] `runes/ctx/db.fav` が `DbCtx` interface の 3 メソッド（execute / query_raw / transaction）を含むこと
- [x] `runes/ctx/http.fav` が `HttpClient` interface の 4 メソッド（get / post / put / delete）を含むこと
- [x] `runes/ctx/stream.fav` が `StreamClient` interface の 3 メソッド（produce / consume / commit）を含むこと
- [x] ctx-migration-status.mdx に追加済みインターフェース一覧（IoCtx / DbCtx / HttpClient / StreamClient）が含まれていること
- [x] ctx-migration-status.mdx に `fav migrate --from-effects` の使い方への参照があること
