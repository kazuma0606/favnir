# Tasks — v55.5.0 — Stateful stage（累積状態）

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.5.0 セクションを確認
- [x] ベーステスト数 3213（v55.4.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.4.0` であることを確認（更新前）
- [x] `fav/src/backend/vm.rs` の `STATE_STORE` / `STATE_BACKEND` thread-local ブロック（L1422〜L1428）を確認（`STATE_VALUE_STORE` の挿入位置）
- [x] `fav/src/backend/vm.rs` の `State.delete_raw` アーム末尾（L20612 付近）を確認（`State.get` / `State.set` / `State.get_or_default` の挿入位置）
- [x] `fav/src/backend/vm.rs` が `vm_call_builtin` 関数（L10013 付近）を持ち、エラー型が `String` であることを確認（`call_builtin` との混同を防ぐ）
- [x] `fav/src/error_catalog.rs` に E0420 エントリが存在することを確認（E0421 の挿入位置の前提）
- [x] `fav/src/error_catalog.rs` に E0421 が存在しないことを確認（新規追加）
- [x] `fav/src/error_catalog.rs` の E0420 エントリ直後（`// ── E05xx ──` コメントの直前）を確認（E0421 の挿入位置）
- [x] `fav/src/middle/checker.rs` の `("State", "get")` エントリ（L6446）を確認（`get_or_default` の挿入位置）
- [x] `fav/src/middle/checker.rs` に `("State", "get_or_default")` が存在しないことを確認（新規追加）
- [x] `fav/src/driver.rs` の `v55400_tests` モジュール位置を確認（直前に `v55500_tests` を挿入）
- [x] `v55400_tests` に `cargo_toml_version_is_55_4_0` テストが存在しないことを確認（削除タスク不要）

---

## 事前作業

- [x] T0a: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.5.0 ベース値を 3213 + 2 = 3215 に訂正（現状 3214 + 2 = 3216 と記載）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.5.0` に更新
- [x] T2: `fav/src/backend/vm.rs` に `STATE_VALUE_STORE` thread-local を追加（`STATE_STORE` ブロック直後）
  - [x] `static STATE_VALUE_STORE: RefCell<HashMap<String, VMValue>>` を定義
- [x] T3: `vm.rs` の `vm_call_builtin` に `State.get` を追加（`State.delete_raw` 直後）
  - [x] String キー引数チェック
  - [x] `STATE_VALUE_STORE` からクローン取得
  - [x] `some(v)` / `none` Variant を返す（ok_vm なし — bind は let 束縛のため）
- [x] T4: `vm.rs` の `vm_call_builtin` に `State.set` を追加（`State.get` の直後）
  - [x] String キー + VMValue 引数チェック
  - [x] `STATE_VALUE_STORE` への挿入
  - [x] `VMValue::Unit` を返す（ok_vm なし）
- [x] T5: `vm.rs` の `vm_call_builtin` に `State.get_or_default` を追加（`State.set` の直後）
  - [x] String キー + デフォルト値 引数チェック
  - [x] `STATE_VALUE_STORE` から取得し、なければデフォルト値を使用
  - [x] `val` を直接返す（ok_vm なし）
- [x] T6: `fav/src/error_catalog.rs` に E0421 stub エントリを追加（E0420 の直後）
- [x] T7: `fav/src/middle/checker.rs` に `("State", "get_or_default") => Some(Type::Unknown)` を追加（`("State", "get")` エントリの直後）
- [x] T8: `fav/src/driver.rs` に `v55500_tests` モジュールを追加（`v55400_tests` の直前）
  - [x] `stateful_stage_accumulates`（Int 値の set/get_or_default 検証）
  - [x] `stateful_stage_persists`（Bool 値の set/get_or_default 検証）

---

## テスト・検証

- [x] T9: `cargo build` でコンパイルエラーがないことを確認（`STATE_VALUE_STORE` `'static` 制約を含む）
- [x] T10: `cargo test` 全通過（3215 tests passed, 0 failed）
- [x] T11: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T12: `CHANGELOG.md` に v55.5.0 エントリ追加
- [x] T13: `versions/current.md` を v55.5.0 / 3215 tests に更新
- [x] T14: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.5.0 実績を COMPLETE に更新
- [x] T15: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.5.0 実績欄も COMPLETE に更新
- [x] ドキュメント MDX: v55.8 でまとめて追加するため本バージョンはスキップ

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [HIGH] `checker.rs` の `State.get` 戻り型を `Type::Option(String)` → `Type::Option(Unknown)` に修正（型付き VMValue 対応）
  - [MED] `State.get_or_default` に `require_state_effect` 呼び出しを追加
  - [MED] `STATE_VALUE_STORE` クリアヘルパー `clear_state_value_store()` を vm.rs に追加し、v55500_tests の各テスト冒頭で呼び出し（thread-local 汚染防止）
  - [LOW] `compiler.rs` のコメントバージョン番号を `v22.3.0` → `v55.5.0` に修正

---

## 完了確認

- [x] `stateful_stage_accumulates` pass
- [x] `stateful_stage_persists` pass
- [x] 3215 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `vm.rs` に `STATE_VALUE_STORE` が追加されている
- [x] `vm.rs` に `State.get` / `State.set` / `State.get_or_default` primitive が追加されている
- [x] `error_catalog.rs` に E0421 エントリが追加されている
- [x] `checker.rs` に `("State", "get_or_default")` が登録されている
- [x] `CHANGELOG.md` に v55.5.0 エントリが追加されている
- [x] `versions/current.md` が v55.5.0 / 3215 tests を反映
- [x] T14 / T15 のロードマップ更新が完了している

## 実装メモ

**重要な発見（compiler.rs namespace 登録の欠落）:**
- `"State"` が `compiler.rs` の namespace 登録リスト（L163〜254）に欠けていた
- 欠落により `compile_expr(Ident("State"))` が `IRExpr::Global(u16::MAX, ...)` を返し、VM が "global index out of bounds" エラー
- 修正: `"State"` を namespace リストに追加（`// v22.3.0 State` コメント付き）
- `is_known_builtin_namespace` には v22.3.0 から `"State"` が含まれていたが、compiler.rs 側が未登録だった

**bind の意味論（ok_vm 不使用の根拠）:**
- Favnir の `bind val <- expr` は compiler.rs では `Pattern::Bind(name)` として `IRStmt::Bind(slot, expr_ir)` にコンパイルされる
- これは単純な let 束縛であり、モナディックアンラップは行わない
- よって `State.set` / `State.get_or_default` は `ok_vm()` でラップせず生の値を返す必要がある
- `State.get_raw` / `State.set_raw` が `ok_vm()` なしで実装されているのと同様のパターン
