# Tasks: v47.4.0 — `String` 拡充

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3024 passed, 0 failed を確認
- [x] `vm.rs` に `String.pad_left`（10672）・`String.repeat`（10883）・`String.trim_start`（10927）が存在することを確認
- [x] `checker.rs` に `("String", "pad_left")` / `("String", "repeat")` / `("String", "trim_start")` が存在することを確認（line 6062/6079/6085）

## T1 — `driver.rs` に `v474000_tests` 追加

- [x] `v473000_tests` の直後に `v474000_tests` モジュールを追加（3 テスト）
  - [x] `string_pad_left`: `String.pad_left("42", 6, "0")` → `"000042"`
  - [x] `string_trim_start`: `String.trim_start("  hello  ")` → `"hello  "`
  - [x] `string_repeat`: `String.repeat("ab", 3)` → `"ababab"`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.4.0"`
- [x] `CHANGELOG.md` に v47.4.0 エントリ追加
- [x] `cargo test` 3027 passed, 0 failed（3024 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.4.0（3027 tests）に更新、進行中バージョンを `v47.5.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.5.0 以降の推定テスト数を実績ベースで確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

spec-reviewer 4 件の指摘を実装前に修正済み（spec.md/plan.md/tasks.md 更新）。
実装後は clippy クリーン・3027 tests passed を確認。ロードマップ推定値（3027/3030）は実績と一致。
