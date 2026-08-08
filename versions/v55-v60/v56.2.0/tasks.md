# Tasks — v56.2.0 — 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）

## ステータス: COMPLETE（2026-07-25）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.2.0 セクションを確認
- [x] ベーステスト数 3229（v56.1.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.0.0` であることを確認（更新前 — v56.1.0 で未反映）
- [x] `error_catalog.rs` に E0422 が存在し、E0423 が存在しないことを確認
- [x] `checker.rs` の `InterfaceImplEntry` に `is_auto: bool` が存在することを確認
- [x] `checker.rs` に `is_explicitly_implemented` が存在しないことを確認（新規追加対象）
- [x] `driver.rs` に `v56200_tests` が存在しないことを確認（新規追加対象）
- [x] `v56000_tests` に `cargo_toml_version_is_56_0_0` が存在することを確認（削除対象）
- [x] 複数 `with` 制約（`T with A with B`）が parser でサポート済みであることを確認
- [x] `check_interface_impl_decl` に coherence check がないことを確認（変更対象）
- [x] Favnir の impl 構文: `method_name = expr`（等号）であることを確認
- [x] Favnir の interface メソッド宣言構文: `method_name: TypeExpr`（コロン）であることを確認

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.2.0` に更新（56.0.0 から直接変更）
- [x] T2: `fav/src/error_catalog.rs` に E0423 エントリを追加（E0422 直後）
  - [x] `code: "E0423"`
  - [x] `title: "duplicate impl: coherence violation"`
  - [x] `category: "types"`
  - [x] `description`: coherence 違反の説明（Favnir enforces coherence）
  - [x] `example`: 正しい Favnir 構文（`hello: Self -> String` / `hello = |s| "hello"`）
  - [x] `fix`: 重複 impl を削除するか merge する案内
  - [x] `suggestion: Some(...)`
- [x] T3: `fav/src/middle/checker.rs` — coherence check 追加
  - [x] `InterfaceRegistry` に `is_explicitly_implemented` メソッドを追加（`is_implemented` の直後）
  - [x] `check_interface_impl_decl` に coherence check（E0423）を挿入（`register_impl` の直前）
  - [x] built-in impl（is_auto=true）は対象外とするコメントを追加
  - [x] `continue` に `// skip registration — duplicate impl rejected` コメントを追加
- [x] T4: `fav/src/driver.rs` — 既存テスト更新
  - [x] `v56000_tests::cargo_toml_version_is_56_0_0` を削除
- [x] T5: `fav/src/driver.rs` — `v56200_tests` モジュールを `v56100_tests` の直前に追加
  - [x] `check_errors` 定義（`Parser::parse_str` + `Checker::check_program` + `.code.to_string()`）
  - [x] `cargo_toml_version_is_56_2_0`（Cargo.toml に `56.2.0` が含まれる）
  - [x] `where_multiple_constraints`（`Int with Ord with Serialize` → `errors.is_empty()`）
  - [x] `impl_coherence_violation`（Greet for Foo を 2 回 impl → E0423 assert）

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認（`Finished` を確認）
- [x] T7: `cargo test` 全通過（**3231 tests passed, 0 failed**）
  - `v56200_tests::cargo_toml_version_is_56_2_0` ok
  - `v56200_tests::where_multiple_constraints` ok
  - `v56200_tests::impl_coherence_violation` ok
  - 既存 3229 件全通過（built-in impl が E0423 を発行しないことを確認）
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v56.2.0 エントリを追加（version: `56.1.0 → 56.2.0`）
- [x] T10: `versions/current.md` を v56.2.0 / 3231 tests に更新
- [x] T11: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.2.0 実績を COMPLETE に更新（ベース 3229、削除 1 件 + 追加 3 件 = net +2 → 3231）
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.2.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応（[MED]×2、[LOW]×2）
  - [MED] CHANGELOG.md の version 記述 `56.0.0 → 56.2.0` → `56.1.0 → 56.2.0` に修正
  - [MED] `checker.rs` の `continue` に `// skip registration — duplicate impl rejected` コメント追加
  - [LOW] E0245 と E0423 の description 差別化不足 → E0245 は `invariant` 型チェック用（既存課題）のため対応保留、備考に記録
  - [LOW] `where_multiple_constraints` 異常系テスト不足 → v56.2.0 スコープ外、次版へ持ち越し

---

## 完了確認

- [x] `cargo_toml_version_is_56_2_0` pass
- [x] `where_multiple_constraints` pass（`errors.is_empty()`）
- [x] `impl_coherence_violation` pass（E0423 assert）
- [x] **3231 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `error_catalog.rs` に E0423 エントリが含まれる（`hello: Self -> String` / `hello = |s|` の正しい構文）
- [x] `checker.rs` に `is_explicitly_implemented` が追加されている
- [x] `checker.rs` の coherence check が E0423 を emit する（built-in impl は対象外）
- [x] `checker.rs` の `continue` に `// skip registration — duplicate impl rejected` コメントあり
- [x] `v56000_tests` から `cargo_toml_version_is_56_0_0` が削除されている
- [x] `CHANGELOG.md` に v56.2.0 エントリが追加されている（version: `56.1.0 → 56.2.0`）
- [x] `versions/current.md` が v56.2.0 / 3231 tests を反映
- [x] T11 / T12 のロードマップ更新が完了している

---

## 実装メモ

- **Favnir 構文の落とし穴**（実装中に発見）:
  - interface メソッド宣言: `method_name: TypeExpr`（コロン構文、`fn` キーワード不使用）
  - impl メソッド定義: `method_name = expr`（等号構文、`fn` キーワード不使用）
  - 誤って `fn method(self: Self) -> Type` や `method: |s| body` と書くとパースエラー
- **built-in impl の coherence 誤検知対策**:
  - `is_explicitly_implemented`（`!is_auto`）を追加し、stdlib 組み込み impl を対象外に
  - この修正により既存 3229 テストが全通過することを確認
- **Cargo.toml の段飛び**:
  - v56.1.0 で Cargo.toml を 56.1.0 に更新する予定だったが未反映のまま残存
  - 本バージョンで 56.2.0 へ直接更新（v56000_tests の version テストも合わせて削除）
