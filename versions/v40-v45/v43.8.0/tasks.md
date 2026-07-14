# v43.8.0 タスク — 双方向型推論（Bidirectional / top-down）

## ステータス: COMPLETE（2026-07-13）— 2925 tests

---

## T0 — 事前確認

- [x] `cargo test` 2922 / 0 確認
- [x] `Cargo.toml` version = `43.7.0` 確認
- [x] `v43800_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `checker.fav` に `fn infer_list_lambda_call` が存在することを確認（現在 line 1849 付近）

---

## T1 — driver.rs — v43800_tests 追加

各テスト関数内で個別に `use crate::...` を宣言（`use super::*` は不要）。

- [x] `v43700_tests` モジュールの直前に `v43800_tests` を挿入
- [x] `cargo_toml_version_is_43_8_0` テスト追加（`Cargo.toml` に `"43.8.0"` を含む）
- [x] `bidirectional_filter_infers_elem_type` テスト追加
  - `fn filter_positive(xs: List<Int>) -> List<Int> { List.filter(xs, |x| x > 0) }` → `Ok`
- [x] `bidirectional_nested_map_filter_expression` テスト追加
  - `fn transform(xs: List<Int>) -> List<Int> { List.filter(List.map(xs, |x| x + 1), |y| y > 0) }` → `Ok`

---

## T2 — Cargo.toml + v43700_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.7.0` → `43.8.0` に更新
- [x] `v43700_tests::cargo_toml_version_is_43_7_0` の assert を削除してスタブ化

---

## T3 — CHANGELOG.md

- [x] v43.8.0 エントリ追加
  - Added: `v43800_tests` 3 件
  - Changed: `cargo_toml_version_is_43_7_0` スタブ化
  - Notes: checker.fav 変更なし（v43.5.0 の infer_list_lambda_call で双方向型推論が動作）

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2925 passed; 0 failed 確認
- [x] `v43800_tests` 3 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.8.0 最新安定版（2925 tests）、次版 v43.9.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.8.0 を `✅ COMPLETE（2026-07-13）`、推定 2925 → 実績 2925 に修正
- [x] `versions/v40-v45/v43.8.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 既知制限の記録

- **匿名レコードリテラル**（`{ name: "Alice", age: 30 }` 型名なし）の文脈推論は非対応（将来バージョン）
- **関数型引数**（`(Int -> Bool)` 型注釈からラムダ引数型を決定）の一般化は非対応（将来）
- **`ELambda`** は `"Fn"` 固定（具体的な `Fn<A,B>` 表現は将来）
- **`EAccess`**（フィールドアクセス `r.field`）は `"Unknown"` 固定（将来）
