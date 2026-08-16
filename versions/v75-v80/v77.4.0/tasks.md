# v77.4.0 タスクリスト — Join 系不変条件

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.3.0` であることを確認
- [x] `cargo test` が全 pass（3742 tests）であることを確認（v77.4.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.4.0: Join 系不変条件 ---` コメントを追加する
- [x] `JoinType` enum を追加する（Inner / Left / Right / Full、Debug / Clone / PartialEq 付き）
- [x] `JoinNullPolicy` enum を追加する（Fail / Warn / Allow、Debug / Clone / PartialEq 付き）
- [x] `JoinInvariant` 構造体を追加する（join_type: JoinType, null_policy: JoinNullPolicy、Debug / Clone 付き）
- [x] `check_join_invariant(left_count: usize, result_count: usize, null_count: usize, inv: &JoinInvariant) -> Result<(), InvariantViolation>` を追加する
  - **Step 1 — JoinType チェック:**
    - Left / Full: `result_count < left_count` → Err（invariant_name: `"join_row_count"`, expected: `">= N (left_count)"`, actual: result_count）
    - Inner / Right: チェックなし
  - **Step 2 — NullPolicy チェック:**
    - Fail: `null_count > 0` → Err（invariant_name: `"join_null_count"`, expected: `"0 nulls (Fail policy)"`, actual: null_count）
    - Warn / Allow: チェックなし
- [x] `cargo test` で既存 3742 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.4.0 エントリを追加する
- [x] Added セクション（enum 2 件・struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v774000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `join_invariant_inner_no_nulls` テストを実装する
  - Inner + Fail + null_count=0 → Ok
  - Inner + Fail + null_count=5 → Err（invariant_name: `"join_null_count"`, actual: `"5"`）
- [x] `join_invariant_left_preserves_rows` テストを実装する
  - Left + Allow + left=100, result=120 → Ok
  - Left + Allow + left=100, result=80 → Err（invariant_name: `"join_row_count"`, expected に "100" を含む, actual: `"80"`）
- [x] `cargo test v774000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.3.0"` → `"77.4.0"` に変更する
- [x] `driver.rs` 内の `77.3.0` バージョン文字列アサーションを `77.4.0` に一括更新（`replace_all: true` で全件置換）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.4.0 に更新する
- [x] 「次に切る版」を v77.5.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3744 tests）
- [x] `cargo test v774000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.4.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.4.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.4.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `join_invariant_inner_no_nulls` が pass
- [x] `join_invariant_left_preserves_rows` が pass
- [x] テスト総数: 3744（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_4_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.4.0]` であること）で代替する
