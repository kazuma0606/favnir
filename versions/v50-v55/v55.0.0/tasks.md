# Tasks — v55.0.0 — Production 3.0 宣言

## ステータス: 未着手

---

## 事前確認（T0）

- [ ] `versions/roadmap/roadmap-v54.1-v55.0.md` の v55.0.0 セクションを確認
- [ ] ベーステスト数 3203（v54.9.0 完了時点）を確認
- [ ] `fav/Cargo.toml` が現在 `54.9.0` であることを確認（更新前）
- [ ] `MILESTONE.md` に `Production 3.0` が記載済み（v54.8.0 で追加済み）を確認
- [ ] `README.md` に `Production 3.0` が記載済み（v54.6.0 で追加済み）を確認

---

## 実装タスク

- [ ] T1: `fav/src/driver.rs` の `v54900_tests` から `cargo_toml_version_is_54_9_0` 関数を削除
- [ ] T2: `fav/Cargo.toml` version を `55.0.0` に更新
- [ ] T3: `CHANGELOG.md` に v55.0.0 エントリ追加（最上部に挿入）
- [ ] T3.5: `MILESTONE.md` の `## v55.0.0（予定）` から `（予定）` を除去し宣言日（2026-07-23）を追記
- [ ] T4: `fav/src/driver.rs` に `v55000_tests` モジュールを追加（`v54900_tests` の直前）
  - [ ] `cargo_toml_version_is_55_0_0`
  - [ ] `changelog_has_v55_0_0`
  - [ ] `milestone_has_production3`
  - [ ] `readme_mentions_production3`

---

## テスト・検証

- [ ] T5: `cargo test` 全通過（3206 tests passed, 0 failed）
- [ ] T6: `cargo clippy -- -D warnings` クリーン

---

## ★クリーンアップ

- [ ] T7: `cargo clean` 実行
- [ ] T8: `cargo clean` 後の `cargo test` 全通過確認（3206 tests passed, 0 failed）

---

## ポスト処理

- [ ] T9: `versions/current.md` を v55.0.0 / 3206 tests に更新（最新安定版・前バージョン・「次に切る版」→ 未定 を反映）
- [ ] T10: `versions/roadmap/roadmap-v54.1-v55.0.md` の v55.0.0 実績を COMPLETE に更新

---

## コードレビュー

- [ ] コードレビュー実施（`/review code`）
- [ ] 指摘事項: （実施後に記入）

---

## 完了確認

- [ ] `cargo_toml_version_is_55_0_0` pass
- [ ] `changelog_has_v55_0_0` pass
- [ ] `milestone_has_production3` pass
- [ ] `readme_mentions_production3` pass
- [ ] 3206 tests passed, 0 failed（failures=0 かつ ≥ 3201）
- [ ] `cargo clean` 完了
- [ ] `versions/current.md` が v55.0.0 を反映
- [ ] `roadmap-v54.1-v55.0.md` の v55.0.0 実績: COMPLETE
