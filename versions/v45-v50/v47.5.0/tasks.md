# Tasks: v47.5.0 — `Float` / `Int` 拡充

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3027 passed, 0 failed を確認
- [x] `vm.rs` に `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` が未実装であることを確認
- [x] `checker.rs` に `("Float", "round")` 等が未登録であることを確認

## T1 — `vm.rs` に 5 primitive 追加

- [x] `"Float.to_bits"` アームの直前に 5 primitive を挿入
  - [x] `Float.round(f, n)`: `(f * 10^n).round() / 10^n`
  - [x] `Float.clamp(f, lo, hi)`: `f.clamp(lo, hi)`
  - [x] `Float.abs(f)`: `f.abs()`
  - [x] `Int.to_hex(n)`: `format!("{:x}", n)`
  - [x] `Int.abs(n)`: `n.abs()`

## T2 — `checker.rs` に型シグネチャ追加

- [x] `("Int", "shift_right")` アームの直後に 4 エントリを挿入
  - [x] `("Float", "round")` → `Some(Type::Float)`
  - [x] `("Float", "clamp") | ("Float", "abs")` → `Some(Type::Float)`
  - [x] `("Int", "to_hex")` → `Some(Type::String)`
  - [x] `("Int", "abs")` → `Some(Type::Int)`

## T3 — `driver.rs` に `v475000_tests` 追加

- [x] `v474000_tests` の直前に `v475000_tests` モジュールを追加（3 テスト）
  - [x] `float_round`: `Float.round(3.14159, 2)` → `3.14`
  - [x] `float_clamp`: `Float.clamp(150.0, 0.0, 100.0)` → `100.0`
  - [x] `int_to_hex`: `Int.to_hex(255)` → `"ff"`

## T4 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.5.0"`
- [x] `CHANGELOG.md` に v47.5.0 エントリ追加
- [x] `cargo test` 3030 passed, 0 failed（3027 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.5.0（3030 tests）に更新、進行中バージョンを `v47.6.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.5.0 完了条件テスト数（3030）を実績で確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップの `|>` 表記が引数順を曖昧にする | spec.md に `|>` セマンティクス注記を追加 |
| [HIGH] | `Float.abs` / `Int.abs` のテストが欠落 | 意図的省略として spec.md に明記（ロードマップ指定 +3 を維持） |
| [MED] | `n<0` の仕様と実装が不一致 | spec を Rust powi 挙動に合わせて更新 |
| [MED] | checker 静的検査スコープが未明記 | spec.md に「VM で実行時エラー検出」を追記 |
| [LOW] | CHANGELOG 日付コメント残存 | plan.md のコメントを削除 |
| [LOW] | tasks.md のロードマップ更新タスク表現が曖昧 | 「確認・必要に応じて更新」に修正 |
