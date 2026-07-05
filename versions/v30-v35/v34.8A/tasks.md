# v34.8A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.3.0` であること
- [x] v34.7A が COMPLETE であること
- [x] `benchmarks/v35.3.0.json` の `tests_passed` を確認（2601）
- [x] `driver.rs` に `mod v35400_tests` が存在しないこと
- [x] `cargo_toml_version_is_35_3_0` が v35300_tests 内に存在すること（スタブ化対象）
- [x] `grep -n "parse_effects_acc" fav/src/frontend/parser.rs` で変更対象行を確認
- [x] `grep -n "check_w022" fav/src/lint.rs` で削除対象行を確認

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.3.0` → `35.4.0` に更新
- [x] **T2** `fav/src/error_catalog.rs` — E0374 エントリを追加（E0373 直後）
- [x] **T3** `fav/src/frontend/parser.rs` — `parse_effect_ann` が `!Effect` を見た瞬間に E0374 を返すよう修正（13 テスト更新）
- [x] **T4** `fav/src/lint.rs` — `check_w022_deprecated_effect_annotation` 関数を削除、`run_lint` からの呼び出し行を削除
- [x] **T5** `fav/src/driver.rs` — `pipeline_state_effect_parsed` テストをスタブ化
- [x] **T6** `fav/src/driver.rs` — `w022_deprecated_effect_annotation_fires` を E0374 アサーションに更新
- [x] **T7** `fav/src/driver.rs` — `cargo_toml_version_is_35_3_0` をスタブ化
- [x] **T8** `fav/src/driver.rs` — `v35400_tests`（5 件）を追加（`v35300_tests` 直後に挿入）
- [x] **T9** `CHANGELOG.md` — `[v35.4.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v35.4.0.json` — 新規作成
- [x] **T11** `versions/current.md` — 最新安定版を v35.4.0 に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v35400` — 5/5 PASS
- [x] **T13** `cargo test` — 全件 PASS（0 failures）— lib 727 passed; 0 failed
- [x] **T14** コンパイル警告なし（関係ファイル）

---

## 完了処理

- [x] **T15** `benchmarks/v35.4.0.json` の `tests_passed` を実測値で確定（2606）
- [x] **T16** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = `"35.4.0"`
- [x] `fn f(x: Int) -> Int !Http { x }` を parse すると E0374 エラー
- [x] `fn f(ctx: AppCtx, x: Int) -> Int { x }` は正常コンパイル
- [x] W022 が lint 出力に現れない（関数削除済み）
- [x] `cargo_toml_version_is_35_3_0` が空スタブになっていること
- [x] `cargo test --bin fav v35400` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v35.4.0]` セクション
- [x] `benchmarks/v35.4.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.4.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] E0374 のエラーメッセージに「ctx: AppCtx を使え」という具体的な移行ガイドが含まれていること
- [x] `!` トークンが fn/stage のエフェクト注釈文脈でのみエラーになること（否定演算子は影響なし）
- [x] W022 の削除が `run_lint` 関数・`check_w022` 関数・関連 W022 テストの 3 箇所すべてを対象にしていること
- [x] `v35400_tests` に `use super::*` が**ない**こと（Parser を直接 use）
- [x] `benchmarks/v35.4.0.json` の `tests_failed` が `0` であること

---

## コードレビュー指摘・対応記録

| 優先 | 内容 | 対応 |
|---|---|---|
| — | 初回実装で全テスト通過 | — |
