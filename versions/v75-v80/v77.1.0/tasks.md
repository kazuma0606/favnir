# v77.1.0 タスクリスト — `PipelineInvariant` 型基盤

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.0.0` であることを確認
- [x] `cargo test` が全 pass（3736 tests）であることを確認（v77.1.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.1.0: PipelineInvariant 型基盤 ---` コメントを追加する
- [x] `InvariantCheckPoint` enum を追加する（Pre / Post / Both、PartialEq 付き）
- [x] `PipelineInvariant` 構造体を追加する（name: String, expression: String, check_point: InvariantCheckPoint）
- [x] `InvariantViolation` 構造体を追加する（invariant_name: String, expected: String, actual: String）
- [x] `check_count_invariant(expected_max: usize, actual: usize, name: &str) -> Result<(), InvariantViolation>` を追加する
  - `actual <= expected_max` → `Ok(())`
  - `actual > expected_max` → `Err(InvariantViolation { invariant_name, expected: "<= N", actual: actual.to_string() })`
- [x] `cargo test` で既存 3736 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.1.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 2 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v771000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `invariant_count_passes` テストを実装する
  - `actual <= expected_max` → Ok（80 <= 100、50 == 50 の境界値含む）
  - `PipelineInvariant` の構築と `check_point` の検証
- [x] `invariant_count_violated` テストを実装する
  - `actual > expected_max`（150 > 100）→ Err
  - `violation.invariant_name`・`violation.expected`（"100" を含む）・`violation.actual`（"150"）を検証
- [x] `cargo test v771000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.0.0"` → `"77.1.0"` に変更する
- [x] `driver.rs` 内の `77.0.0` バージョン文字列アサーションを `77.1.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.1.0 に更新する
- [x] 「次に切る版」を v77.2.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3738 tests）
- [x] `cargo test v771000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.1.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.1.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.1.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `invariant_count_passes` が pass
- [x] `invariant_count_violated` が pass
- [x] テスト総数: 3738（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_1_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.1.0]` であること）で代替する
