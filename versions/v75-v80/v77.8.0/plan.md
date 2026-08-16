# v77.8.0 実装計画 — Probabilistic contracts

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `77.7.0` であることを確認
- `cargo test` が 3750 tests all pass であることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 2: driver.rs — 型・関数追加
`fav/src/driver.rs` の末尾（`// --- v77.7.0` ブロックの後）に追加：

1. セクションコメント `// --- v77.8.0: 確率的契約 ---`
2. `ProbabilisticContract` 構造体（`#[derive(Debug, Clone, PartialEq)]`、`Eq` は derive しない）
3. `check_probabilistic_invariant` 関数（空スライス → Err、平均計算 → 範囲チェック）

### Step 3: CHANGELOG.md 更新
先頭に v77.8.0 エントリを追加（テスト追加より先）。

### Step 4: driver.rs — テストモジュール追加
`v778000_tests` モジュールを追加（`use super::*`）：
- `probabilistic_contract_passes`
- `probabilistic_contract_low_confidence_fails`

### Step 5: Cargo.toml バージョン更新
- `77.7.0` → `77.8.0` に変更
- driver.rs 内の `77.7.0` バージョン文字列アサーションを一括更新
- grep で `// --- v77.7.0: 反例自動生成 ---` が維持されていることを確認（上書きされていたら戻す）
- generate_counter_example_values の DESIGN コメント内 v77.x 記述も確認・維持

### Step 6: versions/current.md 更新
- 「進行中バージョン」を v77.8.0 に更新
- 「次に切る版」を v77.9.0 に更新

### Step 7: 最終確認
- `cargo test` が 3752 tests all pass であることを確認
- `cargo test v778000` で 2 件が pass することを確認
