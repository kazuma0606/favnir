# Tasks — v55.7.0 — Checkpoint / Replay API

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x]`versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 セクションを確認
- [x]ベーステスト数 3217（v55.6.0 完了時点の実績値）を確認
- [x]`fav/Cargo.toml` が現在 `55.6.0` であることを確認（更新前）
- [x]`fav/src/backend/vm.rs` の `STATE_VALUE_STORE` thread-local ブロック末尾を確認（`RESUME_FROM_CHECKPOINT` の挿入位置）
- [x]`fav/src/backend/vm.rs` に `RESUME_FROM_CHECKPOINT` が存在しないことを確認（新規追加）
- [x]`fav/src/backend/vm.rs` に `checkpoint_save_direct` / `set_checkpoint_backend` が `pub fn` として存在することを確認（テストから使用）
- [x]`fav/src/driver.rs` の `checkpoint_list_string` 関数が存在することを確認（`super::` 経由でテストから使用）
- [x]`fav/src/driver.rs` の `v55600_tests` モジュール位置を確認（直前に `v55700_tests` を挿入）
- [x]`tempfile` crate が `[dev-dependencies]` に登録済みであることを確認（`fav/Cargo.toml`）
- [x]CI self-lint 対象（`self/compiler.fav` / `self/checker.fav`）に今回の変更が影響しないか確認（vm.rs / driver.rs の変更は Favnir ソースに非依存のため影響なし）

---

## 事前作業

- [x]T0a: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 完了条件テスト数を 3220 → 3219 に訂正（v55.6.0 実績 3217 + 2）

---

## 実装タスク

- [x]T1: `fav/Cargo.toml` version を `55.7.0` に更新
- [x]T2: `fav/src/backend/vm.rs` に `RESUME_FROM_CHECKPOINT` thread-local を追加（`STATE_VALUE_STORE` ブロック直後）
  - [x]`static RESUME_FROM_CHECKPOINT: RefCell<Option<String>>` を定義
- [x]T3: `fav/src/backend/vm.rs` に `set_resume_from_checkpoint(name: &str)` を追加
- [x]T4: `fav/src/backend/vm.rs` に `get_resume_from_checkpoint() -> Option<String>` を追加
- [x]T5: `fav/src/backend/vm.rs` に `clear_resume_from_checkpoint()` を追加
- [x]T6: `fav/src/driver.rs` に `v55700_tests` モジュールを追加（`v55600_tests` の直前）
  - [x]`cmd_checkpoint_list`（tempdir + save + list 出力検証）
  - [x]`cmd_resume_from_checkpoint`（set/get/clear round-trip 検証）

---

## テスト・検証

- [x]T7: `cargo build` でコンパイルエラーがないことを確認
- [x]T8: `cargo test` 全通過（3219 tests passed, 0 failed）
- [x]T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x]T10: `CHANGELOG.md` に v55.7.0 エントリ追加
- [x]T11: `versions/current.md` を v55.7.0 / 3219 tests に更新
- [x]T12: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 実績を COMPLETE に更新
- [x]T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.7.0 実績欄も COMPLETE に更新
- [x]ドキュメント MDX: v55.8 でまとめて追加するため本バージョンはスキップ

---

## コードレビュー

- [x]コードレビュー実施（`/review code`）
- [x]指摘事項対応

---

## 完了確認

- [x]T0a: ロードマップテスト数訂正（3220→3219）が `roadmap-v55.1-v56.0.md` に反映済みであること
- [x]`cmd_checkpoint_list` pass
- [x]`cmd_resume_from_checkpoint` pass
- [x]3219 tests passed, 0 failed
- [x]`cargo clippy --all-targets -- -D warnings` クリーン
- [x]`vm.rs` に `RESUME_FROM_CHECKPOINT` thread-local が追加されている
- [x]`vm.rs` に `set/get/clear_resume_from_checkpoint` が追加されている
- [x]`CHANGELOG.md` に v55.7.0 エントリが追加されている
- [x]`versions/current.md` が v55.7.0 / 3219 tests を反映
- [x]T12 / T13 のロードマップ更新が完了している
