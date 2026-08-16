# v77.7.0 タスクリスト — 反例自動生成

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.6.0` であることを確認
- [x] `cargo test` が全 pass（3748 tests）であることを確認（v77.7.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.7.0: 反例自動生成 ---` コメントを追加する
- [x] `CounterExampleResult` 構造体を追加する（invariant_name: String, example: Vec<f64>, violates: bool、Debug / Clone 付き）
  - `f64` フィールドを含むため `PartialEq` / `Eq` は derive しない
- [x] `generate_counter_example_values(inv: &AggregateInvariant, seed: u64) -> CounterExampleResult` を追加する
  - `seed % 2 == 0`: adversarial 候補 `[0.0, -0.001, -1.0, 1.0]` を生成（NonNegative で違反を引き起こす）
  - `seed % 2 == 1`: 安全な正値候補 `[0.0, 0.001, 1.0, 100.0]` を生成（NonNegative で違反しない）
  - `check_aggregate_invariant(&candidates, inv)` を呼び出し `Err` なら `violates: true`
  - `CounterExampleResult { invariant_name: inv.column.clone(), example: candidates, violates }` を返す
- [x] `cargo test` で既存 3748 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.7.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v777000_tests` モジュールを追加する（`use super::*` 必須: `AggregateInvariant` / `AggregateProperty` / `generate_counter_example_values` 等が outer scope にあるため）
- [x] `counter_example_finds_violation` テストを実装する
  - `AggregateInvariant { column: "amount", property: AggregateProperty::NonNegative }` を用意
  - `generate_counter_example_values(&inv, 0)` → `result.violates == true` を検証
  - `result.invariant_name == "amount"` を検証
  - `!result.example.is_empty()` を検証
- [x] `counter_example_none_for_trivially_valid` テストを実装する
  - `AggregateInvariant { column: "score", property: AggregateProperty::NonNegative }` を用意
  - `generate_counter_example_values(&inv, 1)` → `result.violates == false` を検証
  - `result.invariant_name == "score"` を検証
  - `!result.example.is_empty()` を検証
- [x] `cargo test v777000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.6.0"` → `"77.7.0"` に変更する
- [x] `driver.rs` 内の `77.6.0` バージョン文字列アサーションを `77.7.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "v77.6.0" fav/src/driver.rs` を実行し、`// --- v77.6.0: 証明付き CI 統合 ---` が残っていることを確認する（`v77.7.0` に書き換わっていた場合は手動で `v77.6.0` に戻す）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.7.0 に更新する
- [x] 「次に切る版」を v77.8.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3750 tests）
- [x] `cargo test v777000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.7.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.7.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.7.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `counter_example_finds_violation` が pass
- [x] `counter_example_none_for_trivially_valid` が pass
- [x] テスト総数: 3750（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_7_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.7.0]` であること）で代替する
