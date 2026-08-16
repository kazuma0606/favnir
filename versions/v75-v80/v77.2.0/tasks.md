# v77.2.0 タスクリスト — フィルター系不変条件

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.1.0` であることを確認
- [x] `cargo test` が全 pass（3738 tests）であることを確認（v77.2.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.2.0: フィルター系不変条件 ---` コメントを追加する
- [x] `FilterInvariant` 構造体を追加する（expected_ratio_min: f64, expected_ratio_max: f64）
- [x] `check_filter_invariant(input_count: usize, output_count: usize, inv: &FilterInvariant) -> Result<(), InvariantViolation>` を追加する
  - `input_count == 0` → `Ok(())`（ゼロ除算回避）
  - `ratio in [min, max]` → `Ok(())`
  - それ以外 → `Err(InvariantViolation { invariant_name: "filter_ratio", expected: "[min, max]", actual: ratio })`
- [x] `format_filter_invariant_report(inv: &FilterInvariant, result: &Result<(), InvariantViolation>) -> String` を追加する
  - `Ok` → `"filter_ratio OK: ratio in [{:.4}, {:.4}]"` 形式（min・max を埋め込む）
  - `Err` → `"filter_ratio VIOLATED: expected {}, actual {}"` 形式（violation フィールドを埋め込む）
- [x] `cargo test` で既存 3738 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.2.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v772000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `filter_invariant_ratio_valid` テストを実装する
  - input=100, output=50 → ratio=0.5, bounds=[0.01, 1.0] → Ok
  - `format_filter_invariant_report` が "OK" を含むことを検証
  - input_count=0 → Ok（ゼロ除算なし）
- [x] `filter_invariant_ratio_violated` テストを実装する
  - input=100, output=0 → ratio=0.0, min=0.01 → Err
  - `violation.invariant_name == "filter_ratio"` を検証
  - `violation.expected.contains("0.01")` を検証
  - `violation.actual == "0.0000"` を検証
  - `format_filter_invariant_report` が "VIOLATED" を含むことを検証
- [x] `cargo test v772000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.1.0"` → `"77.2.0"` に変更する
- [x] `driver.rs` 内の `77.1.0` バージョン文字列アサーションを `77.2.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.2.0 に更新する
- [x] 「次に切る版」を v77.3.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3740 tests）
- [x] `cargo test v772000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.2.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.2.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.2.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `filter_invariant_ratio_valid` が pass
- [x] `filter_invariant_ratio_violated` が pass
- [x] テスト総数: 3740（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_2_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.2.0]` であること）で代替する
