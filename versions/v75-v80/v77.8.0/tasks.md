# v77.8.0 タスクリスト — 確率的契約

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.7.0` であることを確認
- [x] `cargo test` が全 pass（3750 tests）であることを確認（v77.8.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.8.0: 確率的契約 ---` コメントを追加する
- [x] `ProbabilisticContract` 構造体を追加する（name: String, confidence: f64, sample_size: usize、`#[derive(Debug, Clone, PartialEq)]`、`Eq` は derive しない）
- [x] `check_probabilistic_invariant(samples: &[f64], target_min: f64, target_max: f64, contract: &ProbabilisticContract) -> Result<(), String>` を追加する
  - 空スライス → `Err(format!("probabilistic invariant '{}': samples is empty", contract.name))`
  - 平均計算 → `mean >= target_min && mean <= target_max` なら `Ok(())`
  - 範囲外 → `Err(format!("probabilistic invariant '{}' violated: avg={:.4} not in [{:.4}, {:.4}] (confidence={:.2}, sample_size={})", ...))`
- [x] `cargo test` で既存 3750 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.8.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v778000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `probabilistic_contract_passes` テストを実装する
  - `ProbabilisticContract { name: "score_distribution", confidence: 0.95, sample_size: 10_000 }` を用意
  - `samples = vec![40.0, 60.0, 50.0]`（mean=50.0 → [40.0,60.0] 内）
  - `check_probabilistic_invariant(&samples, 40.0, 60.0, &contract)` → `is_ok()` を検証
- [x] `probabilistic_contract_low_confidence_fails` テストを実装する
  - 同じ contract を用意
  - `samples = vec![10.0, 20.0, 15.0]`（mean=15.0 → [40.0,60.0] 外）
  - `is_err()` を検証
  - `msg.contains("score_distribution")` を検証
  - `msg.contains("violated")` を検証
- [x] `cargo test v778000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.7.0"` → `"77.8.0"` に変更する
- [x] `driver.rs` 内の `77.7.0` バージョン文字列アサーションを `77.8.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "v77.7.0" fav/src/driver.rs` を実行し、`// --- v77.7.0: 反例自動生成 ---` が残っていることを確認する（`v77.8.0` に書き換わっていた場合は手動で `v77.7.0` に戻す）
- [x] `grep "v77.7.0" fav/src/driver.rs` で `generate_counter_example_values` の DESIGN コメント内の `v77.7.0` 記述が維持されていることを確認する（`v77.8.0` に書き換わっていた場合は手動で `v77.7.0` に戻す）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.8.0 に更新する
- [x] 「次に切る版」を v77.9.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3752 tests）
- [x] `cargo test v778000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.8.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.8.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.8.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `probabilistic_contract_passes` が pass
- [x] `probabilistic_contract_low_confidence_fails` が pass
- [x] テスト総数: 3754（+4、code-reviewer 指摘で empty_samples / inverted_range テストを追加）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_8_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.8.0]` であること）で代替する
