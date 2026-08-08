# Tasks — v56.1.0 — 境界付きジェネリクス本番品質化（where T: Interface 拡張）

## ステータス: COMPLETE（2026-07-25）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.1.0 セクションを確認
- [x] ベーステスト数 3227（v56.0.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.0.0` であることを確認（更新前）
- [x] `error_catalog.rs` に E0421 が存在し、E0422 が存在しないことを確認
- [x] `checker.rs` の `TypeConstraint::Interface` ブランチが `"E0325"` を emit していることを確認（変更対象）
- [x] `driver.rs` に `v56100_tests` が存在しないことを確認（新規追加）
- [x] `v55.5.0` 完了（E0421 追加済み）を確認 — E0422 は E0421 の直後のコード
- [x] `TypeConstraint::HasField`（E0337）が変更対象外であることを確認
- [x] `fav/self/checker.fav` に `E0325` 文字列参照がないことを確認（参照なし — セルフホスト側への影響なし）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.1.0` に更新（※実装時は 56.0.0 から変更）
- [x] T2: `fav/src/error_catalog.rs` に E0422 エントリを追加（E0421 直後）
  - [x] `code: "E0422"`
  - [x] `title: "where clause interface constraint not satisfied"`
  - [x] `category: "types"`
  - [x] `description`: Interface 制約違反の説明を含む
  - [x] `example`: `Bool with Ord` 違反のコード例
  - [x] `fix`: impl 追加または正しい型の使用を案内
  - [x] `suggestion: Some(...)`
- [x] T3: `fav/src/middle/checker.rs` — E0325 → E0422 変更
  - [x] `TypeConstraint::Interface` ブランチのエラーコード文字列を `"E0422"` に変更
  - [x] call-site 検証コメントを `E0422 for Interface` に更新
  - [x] `type_implements_bound` の doc コメントを `E0422 for Interface` に更新
- [x] T4: `fav/src/driver.rs` — 既存テスト更新
  - [x] `v171000_tests::bounded_generic_violation_e0325` → `bounded_generic_violation_e0422`（関数名 + assertion E0422 + コメント）
  - [x] `v321000_tests::bounded_generics_hash_violation_e0325` → `bounded_generics_hash_violation_e0422`（同上）
  - [x] `v321000_tests::bounded_generics_display_and_hash_bounds` のコメント `E0325 なし` → `E0422 なし`（2箇所）
- [x] T5: `fav/src/driver.rs` — `v56100_tests` モジュールを `v56000_tests` の直前に追加
  - [x] `check_errors` 定義（`Parser::parse_str` + `Checker::check_program` + `.code.to_string()`）
  - [x] `where_clause_e0422_emitted`（Bool with Ord 違反 → E0422 assert）
  - [x] `where_clause_stdlib_fn`（Int with Ord → `errors.is_empty()` assert）

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認（`Finished` を確認）
- [x] T7: `cargo test` 全通過（**3229 tests passed, 0 failed**）
  - `v56100_tests::where_clause_e0422_emitted` ok
  - `v56100_tests::where_clause_stdlib_fn` ok
  - `v171000_tests::bounded_generic_violation_e0422` ok
  - `v321000_tests::bounded_generics_hash_violation_e0422` ok
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v56.1.0 エントリを追加
- [x] T10: `versions/current.md` を v56.1.0 / 3229 tests に更新
- [x] T11: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.1.0 実績を COMPLETE に更新（ベース 3227 + 2 = 3229）
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.1.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応（[LOW]×3）
  - [LOW] テスト関数名に `_e0325` が残存 → `_e0422` に改名
  - [LOW] `where_clause_stdlib_fn` の assert が `E0422` 不在のみ → `errors.is_empty()` に強化
  - [LOW] コメント内の `E0325 なし` が残存 → `E0422 なし` に更新（2箇所）

---

## 完了確認

- [x] `where_clause_e0422_emitted` pass
- [x] `where_clause_stdlib_fn` pass
- [x] **3229 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `error_catalog.rs` に E0422 エントリが含まれる
- [x] `checker.rs` の Interface 境界違反が E0422 を emit する（E0325 は checker コード内に残っていない）
- [x] 既存テスト `bounded_generic_violation_e0422` / `bounded_generics_hash_violation_e0422` pass
- [x] `CHANGELOG.md` に v56.1.0 エントリが追加されている
- [x] `versions/current.md` が v56.1.0 / 3229 tests を反映
- [x] T11 / T12 のロードマップ更新が完了している
