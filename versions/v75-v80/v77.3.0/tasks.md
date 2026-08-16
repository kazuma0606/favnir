# v77.3.0 タスクリスト — 集約系不変条件

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.2.0` であることを確認
- [x] `cargo test` が全 pass（3740 tests）であることを確認（v77.3.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.3.0: 集約系不変条件 ---` コメントを追加する
- [x] `AggregateProperty` enum を追加する（NonNegative / NonPositive / Bounded { min: f64, max: f64 } / NonNull、Debug / Clone / PartialEq 付き）
- [x] `AggregateInvariant` 構造体を追加する（column: String, property: AggregateProperty、Debug / Clone 付き）
- [x] `check_aggregate_invariant(values: &[f64], inv: &AggregateInvariant) -> Result<(), InvariantViolation>` を追加する
  - `NonNegative`: 全値 >= 0.0 → Ok、違反値あり → Err（expected: `"NonNegative (>= 0.0)"`）
  - `NonPositive`: 全値 <= 0.0 → Ok、違反値あり → Err（expected: `"NonPositive (<= 0.0)"`）
  - `Bounded { min, max }`: 全値が [min, max] 内 → Ok、違反値あり → Err（expected: `"[{:.4}, {:.4}]"`）
  - `NonNull`: values が非空 → Ok、空 → Err（expected: `"NonNull (non-empty)"`, actual: `"empty"`）
  - `invariant_name` は常に `inv.column.clone()`
- [x] `cargo test` で既存 3740 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.3.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v773000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `aggregate_invariant_non_negative_passes` テストを実装する
  - `[1.0, 2.0, 3.0]` + NonNegative → Ok
  - `[-1.0, -2.0, 0.0]` + NonPositive → Ok
  - `[42.0]` + NonNull → Ok
  - `[0.0, 50.0, 100.0]` + Bounded { min: 0.0, max: 100.0 } → Ok
- [x] `aggregate_invariant_bounded_violated` テストを実装する
  - `[0.0, 50.0, 150.0]` + Bounded { min: 0.0, max: 100.0 } → Err
  - `violation.invariant_name == "score"` を検証
  - `violation.expected.contains("100")` を検証
  - `violation.actual == "150.0000"` を検証
  - `[]` + NonNull → Err、`actual == "empty"` を検証
- [x] `cargo test v773000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.2.0"` → `"77.3.0"` に変更する
- [x] `driver.rs` 内の `77.2.0` バージョン文字列アサーションを `77.3.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.3.0 に更新する
- [x] 「次に切る版」を v77.4.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3742 tests）
- [x] `cargo test v773000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.3.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.3.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.3.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `aggregate_invariant_non_negative_passes` が pass
- [x] `aggregate_invariant_bounded_violated` が pass
- [x] テスト総数: 3742（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_3_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.3.0]` であること）で代替する
