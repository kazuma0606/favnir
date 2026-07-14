# v44.6.0 タスク — Precision & Flow E2E デモ

## ステータス: COMPLETE（2026-07-15）— 2956 tests

---

## T0 — 事前確認

- [x] `cargo test` 2955 / 0 確認
- [x] `Cargo.toml` version = `44.5.0` 確認
- [x] `v44600_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `infra/e2e-demo/precision-flow/` が存在しないことを確認

---

## T1 — デモファイル作成

- [x] `infra/e2e-demo/precision-flow/src/demo.fav` 作成
  - Refinement type（`type HighValue = Float where |v| v > 1000.0`）
  - Opaque type（`opaque type OrderId = String`）
  - CEP pattern（`cep pattern HighValueDetected { HighValue within 300 }`）
  - `#[max_inflight(50)]` 付きステージ
  - 型注釈付き `bind` 束縛
- [x] `infra/e2e-demo/precision-flow/README.md` 作成
  - パイプライン概要・機能一覧・実行方法（将来版）

---

## T2 — driver.rs: `v44600_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44500_tests` の直前に `v44600_tests` を挿入（1 件）
  - `precision_flow_e2e_demo_structure`
- [x] スタブ化: `v44500_tests::cargo_toml_version_is_44_5_0` の `assert!` を削除し `// Stubbed: version bumped to 44.6.0 in v44.6.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.5.0` → `44.6.0` に更新

---

## T3 — CHANGELOG.md に v44.6.0 エントリ追加

- [x] v44.6.0 エントリを CHANGELOG.md の先頭に追加（`[v44.6.0]` を含む）
  - Precision & Flow E2E デモの説明
  - `infra/e2e-demo/precision-flow/` 成果物

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2956 passed; 0 failed 確認
- [x] `v44600_tests` 1 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.6.0 最新安定版（2956 tests）、次版 v44.7.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.6.0 を `✅ COMPLETE（2026-07-15）`、推定テスト数を実績に修正
- [x] `versions/v40-v45/v44.6.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
