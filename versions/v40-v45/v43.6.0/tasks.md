# v43.6.0 タスク — パイプライン型伝播（Pipeline stage typing）

## ステータス: COMPLETE（2026-07-12）— 2920 tests

---

## T0 — 事前確認

- [x] `cargo test` 2917 / 0 確認
- [x] `Cargo.toml` version = `43.5.0` 確認
- [x] `v43600_tests` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — driver.rs — v43600_tests 追加

- [x] `v43500_tests` モジュールの直前に `v43600_tests` を挿入
- [x] `cargo_toml_version_is_43_6_0` テスト追加（`Cargo.toml` に `"43.6.0"` を含む）
- [x] `pipeline_two_step_bind_infers_types` テスト追加
  - `fn process(xs: List<Int>) -> List<Int> { bind doubled <- List.map(xs, |x| x*2); List.filter(doubled, |x| x > 0) }` → `Ok`
- [x] `pipeline_three_step_bind_infers_types` テスト追加
  - 3段 bind チェーン（map → filter → map）→ `Ok`

---

## T2 — Cargo.toml + v43500_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.5.0` → `43.6.0` に更新
- [x] `v43500_tests::cargo_toml_version_is_43_5_0` の assert を削除してスタブ化

---

## T3 — CHANGELOG.md

- [x] v43.6.0 エントリ追加
  - Added: `v43600_tests` 3 件
  - Changed: `cargo_toml_version_is_43_5_0` スタブ化
  - Notes: checker.fav 変更なし（既存機能で動作）

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2920 passed; 0 failed 確認
- [x] `v43600_tests` 3 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.6.0 最新安定版（2920 tests）、次版 v43.7.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.6.0 を `✅ COMPLETE（2026-07-12）`、推定 2920 → 実績 2920 に修正
- [x] `versions/v40-v45/v43.6.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- checker.fav の変更は不要。`infer_hm_let` が `EBind` を処理して `vr.ty` を `env_insert` で次式の環境に追加する仕組みが既に存在しており、v43.5.0 の `infer_list_lambda_call` と組み合わせることで多段パイプラインが機能する
- `bind x <- non_result_expr` は Favnir で let バインドとして機能（v43.5.0 の知見を再確認）
- `EAccess`（フィールドアクセス）は常に `"Unknown"` を返す — フィールド型追跡は将来バージョンの課題

## 既知制限の記録

ロードマップ掲載例（`Csv.read("data.csv")` / `r.value` フィールドアクセス経由のパイプライン）は v43.6.0 では非対応:
- `builtin_ret_ty("Csv", "read")` → `"Unknown"`（Csv.read の戻り型が未定義）
- `EAccess`（`r.value` 等のフィールドアクセス）→ 常に `"Unknown"`（v41.5.0 コメント: "フィールド追跡は v42.0+ 以降"）

これらは v43.6.0 の既知制限であり、`EAccess` 型追跡が実装される将来バージョンで対応する。
型付きパラメータ経由（`xs: List<Int>`）のパイプラインは本バージョンで正常動作することを確認済み。
