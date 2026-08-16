# v77.9.0 実装計画 — 安定化・コードフリーズ

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `77.8.0` であることを確認
- `cargo test` が 3754 tests all pass であることを確認（v77.9.0 テスト追加前の状態）
- `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

### Step 2: CHANGELOG.md 更新（テスト追加より先）
先頭に v77.9.0 エントリを追加。

### Step 3: driver.rs — テストモジュール追加
`fav/src/driver.rs` の末尾（`// --- v77.8.0` ブロックの後）に `v779000_tests` モジュールを追加（`use super::*`）：

1. `verifiable_full_sprint_all_stable`: v77.1〜v77.8 の全主要型を instantiate して基本動作確認
2. `verifiable_e2e_pipeline_verified`: aggregate → filter → probabilistic → verify → ci の E2E 合成テスト

### Step 4: Cargo.toml バージョン更新
- `77.8.0` → `77.9.0` に変更
- driver.rs 内の `77.8.0` バージョン文字列アサーションを一括更新（`replace_all: true`）
- grep で `// --- v77.8.0: 確率的契約 ---` が維持されていることを確認（上書きされていたら戻す）
- `check_probabilistic_invariant` の doc コメント内 v77.8.0 記述も確認・維持

### Step 5: versions/current.md 更新
- `## 進行中バージョン` 欄を `**v77.9.0**（安定化・コードフリーズ）` に更新
- `## 次に切る版` 欄を `**v78.0.0**（Verifiable Pipelines 宣言 ★クリーンアップ）` に更新

### Step 6: 最終確認
- `cargo test` が 3756 tests all pass であることを確認
- `cargo test v779000` で 2 件が pass することを確認
