# Tasks: v54.1.0 — 全エラーコード fav explain --error 対応完備

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3185 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54100_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v54100_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v54000_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v54000_tests" fav/src/driver.rs` → 行番号を特定（47644）
- [x] `fav/tmp/hello.fav` が正しい内容であることを確認:
  - [x] `fn add(a: Int, b: Int) -> Int { a + b }` が含まれる
  - [x] `fn main() -> Bool { add(1, 2) == 3 }` が含まれる
- [x] `Cargo.toml` の現在バージョンが `54.0.0` であることを確認
- [x] `error_catalog.rs` のエントリ数が 92 件であることを確認:
  - [x] `grep -c "    code:" fav/src/error_catalog.rs` → 92

---

## T1 — `driver.rs` — `v54100_tests` 追加 + `cargo_toml_version_is_54_0_0` 空化

- [x] `v54000_tests` の直前に `v54100_tests` を追加（2 テスト）:
  - [x] `explain_error_all_codes_have_collect_text`: `list_all()` で全コード走査 → `cmd_explain_error_collect` が `Some` かつ非空を assert
  - [x] `explain_error_e0419_exists`: E0419 のテキストが `"E0419"` と `"assert_schema"` を含むことを assert
- [x] `v54000_tests::cargo_toml_version_is_54_0_0` を空化:
  - [x] 関数ボディを削除し `// v54.1.0 にバンプしたためアサートを空化。` コメントのみにする
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.0.0"` → `version = "54.1.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3187 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T3 — 後処理

- [x] `CHANGELOG.md`: v54.1.0 エントリ追加（v54.0.0 の直上）
  - [x] `## [v54.1.0] — 2026-07-22 — 全エラーコード fav explain --error 対応完備` 形式
  - [x] テスト数 3187 と 2 テスト名を記載
  - [x] エラーコード数は実数 92 件を記載
- [x] `versions/current.md` を v54.1.0（3187 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.1.0 実績欄を更新（COMPLETE・3187 tests・2026-07-22）
  - [x] 完了条件のテスト推定値「3181」→「ベース 3185 + 2 = 3187」に修正

---

## T4 — コードレビュー対応

- [x] [MED] テスト名重複: `explain_error_all_codes_have_text` → `explain_error_all_codes_have_collect_text` に改名（v503000_tests との衝突回避）
- [x] [LOW] CHANGELOG エラーコード数誤記修正: 「93」→「92」
- [x] [LOW] roadmap 完了条件の推定値修正: 「3181」→「ベース 3185 + 2 = 3187」、テスト名も更新

---

## T5 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
