# Tasks: v54.6.0 — README / CONTRIBUTING 最終整備

Status: COMPLETE
Date: 2026-07-23

---

## T0 — 事前確認

- [x] `cargo test` 3195 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54600_tests` が**存在しない**ことを確認
- [x] `driver.rs` に `v54500_tests` が存在することを確認（挿入位置の確認）
- [x] `README.md` に `"Production 3.0"` が未存在であることを確認
- [x] `CONTRIBUTING.md` に `"fav doctor"` が未存在であることを確認
- [x] `Cargo.toml` の現在バージョンが `54.5.0` であることを確認

---

## T1 — `README.md` 更新

- [x] v54.0 Integration Sprint 宣言文の直後（v53.0 より前）に v54.1〜v54.5 サマリーを追記:
  - [x] `"Production 3.0"` という文字列を含む
  - [x] `"v54.1"` という文字列を含む（v54.6.0 追加行の特定のため）
  - [x] v54.1〜v54.5 各バージョンの機能を列挙（explain --error / watch-diff / CI 統合 / dq-report / doctor）
  - [x] v54.0 宣言の直後・v53.0 より前に配置（時系列整合）

---

## T2 — `CONTRIBUTING.md` 更新

- [x] テスト手順セクションの直前に「環境診断」セクションを追加:
  - [x] `fav doctor` コマンド例とサンプル出力を記載
- [x] テスト手順セクションの直後に「ベンチマーク・パフォーマンス確認」セクションを追加:
  - [x] `cargo test bench_ -- --nocapture` コマンドを記載
  - [x] `fav bench --compare ../benchmarks/baseline.json --fail-on-regression` を記載
  - [x] `--all` フラグを含めない（no-op のため）

---

## T3 — `driver.rs` — `v54600_tests` 追加

- [x] `v54500_tests` の直前に `v54600_tests` を追加（2 テスト）:
  - [x] `use super::*` を追加（他テストモジュールとの慣習統一）
  - [x] `readme_has_production3_mention`:
    - [x] `include_str!("../../README.md").contains("Production 3.0")` を確認
    - [x] `include_str!("../../README.md").contains("v54.1")` を確認（追加行の特定）
  - [x] `contributing_has_doctor_step`:
    - [x] `include_str!("../../CONTRIBUTING.md").contains("fav doctor")` を確認

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.5.0"` → `version = "54.6.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3197 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md`: v54.6.0 エントリ追加（v54.5.0 の直上）
- [x] `versions/current.md` を v54.6.0（3197 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.6.0 実績欄を更新（COMPLETE・3197 tests・2026-07-23）

---

## T6 — コードレビュー対応

- [x] [MED] `v54600_tests` に `use super::*` 欠落 → 追加（慣習統一）
- [x] [MED] `fav bench --all` が no-op でドキュメントが誤解を招く → `--all` を削除
- [x] [MED] README.md の v54.1〜v54.5 ブロックが v54.0 宣言より上に挿入 → v54.0 直後に移動
- [x] [LOW] `readme_has_production3_mention` が v54.6.0 追加行を特定しない → `"v54.1"` アサーション追加

---

## T7 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T7 全 `[x]`）
