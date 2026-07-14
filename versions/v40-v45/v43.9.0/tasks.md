# v43.9.0 タスク — `fav check --show-inference`

## ステータス: COMPLETE（2026-07-13）— 2927 tests

---

## T0 — 事前確認

- [x] `cargo test` 2925 / 0 確認
- [x] `Cargo.toml` version = `43.8.0` 確認
- [x] `v43900_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `driver.rs` に `--show-inference で対応予定` TODO コメントが存在することを確認（line 3948 付近）

---

## T1 — driver.rs — collect_inference_annotations + cmd_check 更新

T1・T2 は cmd_check シグネチャ変更を含むため**必ず同時適用**すること（T2 と同一コミット）。

- [x] `collect_inference_annotations(src: &str, filename: &str) -> Vec<String>` を追加
  - 単一パースで FnDef シグネチャを収集（二重パース TODO を解消）
  - `run_checker_fav` 通過時のみ注釈を返す
  - `display_ty_inline` をローカル関数としてインライン定義
- [x] `cmd_check` シグネチャ末尾に `show_inference: bool` を追加
- [x] `show_inference` が `true` の場合に `collect_inference_annotations` を呼び出す出力ブロックを追加
- [x] driver.rs line 3948–3949 の二重パース TODO コメントを削除

---

## T2 — main.rs — --show-inference フラグ追加

- [x] `let mut show_inference = false;` を追加
- [x] `"--show-inference" => { show_inference = true; i += 1; }` を追加
- [x] `cmd_check` 呼び出しに `show_inference` を末尾に追加

---

## T3 — driver.rs — v43900_tests 追加 / Cargo.toml / スタブ化

各テスト関数内で個別に呼び出し（`use super::*` は不要。`collect_inference_annotations` は `super::` 経由で参照）。

- [x] `v43800_tests` モジュールの直前に `v43900_tests` を挿入
- [x] `cargo_toml_version_is_43_9_0` テスト追加（`Cargo.toml` に `"43.9.0"` を含む）
- [x] `show_inference_collects_fn_annotations` テスト追加
  - `fn add(a: Int, b: Int) -> Int { a + b }` + `fn identity(x: Int) -> Int { x }` → annotations に "add" と "identity" を含む
- [x] `v43800_tests::cargo_toml_version_is_43_8_0` をスタブ化
- [x] `fav/Cargo.toml` version を `43.8.0` → `43.9.0` に更新

---

## T4 — CHANGELOG.md

- [x] v43.9.0 エントリ追加
  - Added: `collect_inference_annotations` / `fav check --show-inference` / `v43900_tests` 2 件
  - Changed: `cmd_check` シグネチャ更新 / `cargo_toml_version_is_43_8_0` スタブ化 / TODO 解消

---

## T5 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2927 passed; 0 failed 確認
- [x] `v43900_tests` 2 件 pass 確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.9.0 最新安定版（2927 tests）、次版 v43.10.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.9.0 を `✅ COMPLETE（2026-07-13）`、推定 2927 → 実績 2927 に修正
- [x] `versions/v40-v45/v43.9.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- `Param.ty` は `TypeExpr`（非 Option）のため、`p.ty.as_ref()` は不可。`display_ty_inline(&p.ty)` で直接参照する
- `FnDef.return_ty` は `Option<TypeExpr>` のため `as_ref().map(...)` が正しい
- `v43900_tests` 内から `collect_inference_annotations` を呼ぶには `super::collect_inference_annotations(...)` が必要（同一ファイル内の親モジュールへの参照）

---

## 既知制限の記録

- **`display_ty_inline`** は `Named` / `Named<args>` のみ対応。`Arrow`・`Optional`・`RecordType` 等は `"?"` にフォールバック（将来拡張可）
- **`cmd_check_all`** は `show_inference` 非対応（単一ファイルモードのみ）
- **式レベルの型注釈**（全 ECall・EBind への型表示）は非対応（checker.fav の型返却機構が必要）
