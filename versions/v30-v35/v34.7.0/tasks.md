# v34.7.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.6.0` であること
- [x] `benchmarks/v34.6.0.json` の `tests_passed` が 2566 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2566 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v347000_tests` が存在しないこと
- [x] v34.6.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_6_0` が v346000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_6_0" fav/src/driver.rs | head -5
  ```
- [x] `cargo test --bin fav v346000` が 5/5 PASS であること
- [x] `site/content/docs/ctx-syntax-guide.mdx` が存在しないこと（新規作成対象）
  ```bash
  ls site/content/docs/ctx-syntax-guide.mdx 2>/dev/null || echo "does not exist"
  ```
- [x] `site/content/learn/getting-started.mdx` に `"AppCtx"` が含まれないこと（追記対象）
  ```bash
  grep "AppCtx" site/content/learn/getting-started.mdx | wc -l
  # 0 であることを確認
  ```
- [x] `README.md` に `"v34.5"` が含まれないこと（追記対象）
  ```bash
  grep "v34.5" README.md | wc -l
  # 0 であることを確認
  ```

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.6.0` → `34.7.0` に更新
- [x] **T2** `site/content/docs/ctx-syntax-guide.mdx` — ctx 構文完全リファレンスガイドを新規作成
        （AppCtx 設計思想 / bind 分解構文 / フィールド一覧 / Before-After / mock テスト例 / 自動移行）
- [x] **T3** `site/content/learn/getting-started.mdx` — 末尾に「Capability Context を使う」セクションを追加
        （AppCtx パイプライン例 + `/docs/ctx-syntax-guide` へのリンク）
- [x] **T4** `README.md` — v34.0 宣言行の直後に v34.5〜v34.7 ctx 移行シリーズ追記
- [x] **T5** `fav/src/driver.rs` — `cargo_toml_version_is_34_6_0` をスタブ化
- [x] **T6** `fav/src/driver.rs` — `v347000_tests`（5 件）を追加
        挿入位置: `v346000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし（`include_str!` のみ使用）
- [x] **T7** `CHANGELOG.md` — `[v34.7.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v34.7.0.json` — 新規作成（`tests_passed`: 2571）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v34.7.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v347000 2>&1 | tail -8` — 5/5 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2571 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v34.7.0.json` の `tests_passed` を実測値で更新（実測値 2571 = 想定値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.7.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.7.0"`
- [x] `cargo_toml_version_is_34_6_0` が空スタブになっていること
- [x] `cargo test --bin fav v347000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2571 件想定 = 2566 + 5、0 failures）
- [x] `site/content/docs/ctx-syntax-guide.mdx` が存在し `"AppCtx"` と `"bind"` を含むこと
- [x] `site/content/learn/getting-started.mdx` が `"AppCtx"` を含むこと
- [x] `README.md` が `"v34.5"` を含むこと
- [x] `CHANGELOG.md` に `[v34.7.0]` セクション
- [x] `benchmarks/v34.7.0.json` 存在かつ `tests_passed` が実測値（2571）
- [x] `versions/current.md` が v34.7.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v347000_tests` に `use super::*` が**ない**こと（`include_str!` のみ使用）
- [x] `readme_has_ctx_migration_ref` のパスが `"../../README.md"` であること
- [x] `cargo_toml_version_is_34_6_0` が空スタブになっていること（コメント付き）
- [x] 挿入位置が `v346000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.7.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.7.0 に更新されていること
- [x] `ctx-syntax-guide.mdx` に AppCtx 設計思想 / bind 分解構文 / Before-After / Ctx.mock 例が含まれていること
- [x] `getting-started.mdx` の追加が additive（既存コードの変更なし）であること
- [x] `README.md` の追加が additive（既存行の変更なし）であること
