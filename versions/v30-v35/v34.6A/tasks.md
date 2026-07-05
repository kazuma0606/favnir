# v34.6A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.1.0` であること
- [x] v34.5A が COMPLETE であること
- [x] `benchmarks/v35.1.0.json` の `tests_passed` を確認（2591）
- [x] `driver.rs` に `mod v35200_tests` が存在しないこと
- [x] `cargo_toml_version_is_35_1_0` が v35100_tests 内に存在すること（スタブ化対象）
- [x] `runes/postgres/client.fav` にまだ `!Postgres` が存在すること（移行対象確認）
- [x] `grep -rl "-> .* !\w" runes/ --include="*.fav" | grep -v "runes/ctx/" | wc -l` が 100 であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.1.0` → `35.2.0` に更新
- [x] **T2-A** `runes/` グループ A（DB系 25 ファイル）— !Effect → ctx 移行
- [x] **T2-B** `runes/` グループ B（Redis系 2 ファイル）— !Effect → ctx 移行
- [x] **T2-C** `runes/` グループ C（HTTP系 28 ファイル）— !Effect → ctx 移行
- [x] **T2-D** `runes/` グループ D（Stream系 9 ファイル）— !Effect → ctx 移行
- [x] **T2-E** `runes/` グループ E（IO/File系 13 ファイル）— !Effect → ctx 移行
- [x] **T2-F** `runes/` グループ F（LLM/AI系 6 ファイル）— !Effect → ctx 移行
- [x] **T2-G** `runes/` グループ G（監視/ログ系 7 ファイル）— !Effect → ctx 移行
- [x] **T2-H** `runes/` グループ H（汎用/その他 11 ファイル）— !Effect → ctx 移行
- [x] **T3** 移行後チェック — `!Effect` アノテーション残存なし（コメント行のみ）
- [x] **T4** `fav/src/driver.rs` — `cargo_toml_version_is_35_1_0` をスタブ化
- [x] **T5** `fav/src/driver.rs` — `v35200_tests`（5 件）を追加
- [x] **T6** `CHANGELOG.md` — `[v35.2.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v35.2.0.json` — 新規作成（`tests_passed`: 2596）
- [x] **T8** `versions/current.md` — 最新安定版を v35.2.0 に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v35200 2>&1 | tail -8` — 5/5 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2596 passed、0 failures）
- [x] **T11** コードレビュー指摘対応:
       - `compiler.rs` に `"Ctx"` / `"AppCtx"` namespace 登録追加（`Ctx.test_ctx_raw()` がテストブロックで動作するように）
       - `aws.test.fav` / `http.test.fav` / `auth.test.fav` / `env.test.fav` / `grpc.test.fav` / `incremental.test.fav` を ctx 構文に更新
       - `driver.rs` の `http_rune_put_returns_err_on_bad_host` を ctx 構文に更新

---

## 完了処理

- [x] **T12** `benchmarks/v35.2.0.json` の `tests_passed` を実測値（2596）で確定
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"35.2.0"`
- [x] `cargo_toml_version_is_35_1_0` が空スタブになっていること
- [x] `runes/` 配下 100 件に `!Effect` アノテーションが残存しないこと（コメントのみ）
- [x] `postgres_client_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Postgres` を含まない
- [x] `redis_rune_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Redis` を含まない
- [x] `kafka_rune_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Stream` を含まない
- [x] `http_client_rune_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Http` を含まない
- [x] `cargo test --bin fav v35200` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2596 passed、0 failures）
- [x] `CHANGELOG.md` に `[v35.2.0]` セクション
- [x] `benchmarks/v35.2.0.json` 存在かつ `tests_passed` が実測値（2596）
- [x] `benchmarks/v35.2.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.2.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v35200_tests` に `use super::*` が**ない**こと
- [x] 否定テスト（`!src.contains("!Effect")`）と肯定テスト（`src.contains("ctx: AppCtx")`）が両立していること
- [x] `runes/ctx/` の 4 ファイルが移行対象から**除外**されていること
- [x] 移行後の fn シグネチャが `fn f(ctx: AppCtx, ...)` の形式であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-05）
- [x] `benchmarks/v35.2.0.json` の `tests_failed` が `0` であること
