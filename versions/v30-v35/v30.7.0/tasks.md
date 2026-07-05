# v30.7.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.6.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2409 passed` を含むこと
- [x] `driver.rs` に `mod v307000_tests` が存在しないこと
- [x] `driver.rs` に `hint_for_runtime_error` が存在しないこと
- [x] v30.6.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.6.0` → `30.7.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_6_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `hint_for_runtime_error`（`pub(crate)`）を追加（3 パターン、具体 → 汎用の順）
- [x] **T4** `fav/src/driver.rs` — `format_runtime_error` を改善（プレフィックス・stage ラベル・fn_name 保持・ヒント付加）
- [x] **T5** `fav/src/driver.rs` — `v307000_tests`（3 件）を追加
- [x] **T6** `CHANGELOG.md` — `[v30.7.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v30.7.0.json` — 新規作成
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v30.7.0 に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v307000 2>&1 | tail -10` — 3/3 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2412 passed、0 failures）

---

## 完了処理

- [x] **T11** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"30.7.0"`
- [x] `hint_for_runtime_error` が `pub(crate)` で実装されている（3 パターン、具体 → 汎用の順）
- [x] `format_runtime_error` が `"runtime error:"` プレフィックスを使用する
- [x] `format_runtime_error` がステージ名を `"in stage X"` 形式で表示する
- [x] `format_runtime_error` がヒントを `"  = ヒント: ..."` で末尾に付加する
- [x] 空スタックトレース時も `fn_name` が保持される（`<none>` 以外の場合）
- [x] `cargo test v307000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2412 passed）
- [x] `CHANGELOG.md` に `[v30.7.0]` セクション
- [x] `benchmarks/v30.7.0.json` 存在
- [x] `versions/current.md` を v30.7.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] `hint_for_runtime_error` が `format_runtime_error` の直前に定義されていること
- [x] `hint_for_runtime_error` のパターン順序が「global index → index → type error」（具体 → 汎用）であること
- [x] `hint_for_runtime_error` のパターンに論理的な重複や死んだ条件がないこと
- [x] `format_runtime_error` でスタックトレース空の場合も `fn_name` が表示されること（`<none>` 以外）
- [x] ステージ検出ロジックが `frame.fn_name.chars().next().map(|c| c.is_uppercase())` であること
- [x] `v307000_tests` に `use super::hint_for_runtime_error` があること（`pub(crate)` 呼び出し）
- [x] `v307000_tests::hint_for_runtime_error_works` が `global` と `index` の返り値が異なることを `assert_ne!` で確認していること
- [x] `v307000_tests` に `benchmark_v30_7_0_exists` テストがあること

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘 8 件（実装前）をすべて spec/plan/tasks に反映:
- [HIGH] hint パターン順序: 具体（global index）→ 汎用（index）→ type error の順に修正
- [HIGH] テスト 2 を `pub(crate)` + 実呼び出し方式に変更（`include_str!` テキスト確認では動作保証不十分）
- [HIGH] ロードマップ目標乖離: spec に「ロードマップ目標との差異」セクションを追加し OUT OF SCOPE 理由を明記
- [MED] ステージ誤検知リスク: spec に「誤検知リスク」セクションを追加
- [MED] plan 挿入位置を grep アンカー方式に変更
- [MED] 空スタックトレース時の fn_name 保持: `fn_name != "<none>"` の条件付きで保持
- [LOW] current.md フォーマット要件を tasks T8 に明記
- [LOW] tasks チェックリストに `assert_ne!` 確認項目を追加
