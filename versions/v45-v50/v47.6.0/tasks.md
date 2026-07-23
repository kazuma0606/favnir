# Tasks: v47.6.0 — `Option` 拡充

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3030 passed, 0 failed を確認
- [x] `vm.rs` に `Option.map`（4194）・`Option.unwrap_or`（4259）・`Option.and_then`（4221）が存在することを確認
- [x] `checker.rs` に `("Option", "map")` / `("Option", "unwrap_or")` / `("Option", "and_then")` が存在することを確認（line 6108/6115/6119）

## T1 — `driver.rs` に `v476000_tests` 追加

- [x] `v475000_tests` の直前に `v476000_tests` モジュールを追加（3 テスト）
  - [x] `option_map`: `Option.map(some(5), |n| n*2)` → `unwrap_or(0)` == `10`
  - [x] `option_unwrap_or`: `Option.unwrap_or(none(), "default")` == `"default"`
  - [x] `option_and_then`: `Option.and_then(some(5), |n| some(n+1))` → `unwrap_or(0)` == `6`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.6.0"`
- [x] `CHANGELOG.md` に v47.6.0 エントリ追加
- [x] `cargo test` 3033 passed, 0 failed（3030 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.6.0（3033 tests）に更新、進行中バージョンを `v47.7.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.6.0 完了条件テスト数（3033）を実績で確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップ「VM primitive として追加」と実態（実装済み）の齟齬 | spec.md 冒頭にロードマップ表現の注記を追加 |
| [MED] | plan.md の VM 表現が `VMValue::Tagged`（存在しない型）→ 実際は `VMValue::Variant` | plan.md の注意事項テーブルを修正 |
| [LOW] | spec.md の注意事項に `unwrap_or` が直接値を返す挙動の補足なし | spec.md に一行追記 |
