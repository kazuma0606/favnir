# v71.5.0 タスクリスト — Generic Constraints（`impl Trait` 風の境界）

Date: 2026-08-10
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.4.0` であることを確認
- [x] `cargo test` が全 pass（3594 tests）であることを確認
- [x] `parse_type_bounds` 関数を確認（`parser.rs` ~line 1709）
- [x] `parse_type_bounds` の `with` ブランチが `TypeConstraint::Interface(name)` を push することを確認
- [x] `TokenKind::Colon` が定義されているかを確認（定義済み）
- [x] `&` の TokenKind 名称を確認 → `TokenKind::Amp`（`TokenKind::AmpAmp` は `&&`）
- [x] `impl` の TokenKind 名称を確認 → `TokenKind::Impl`（キーワードとして独立）
- [x] E0422 が既存チェッカーで境界違反に使用されていることを確認（checker.rs line 5202）
- [x] E0423 が `error_catalog.rs` で「duplicate impl」として使用中であることを確認 → 新規追加なし
- [x] `fmt.rs` が `GenericParam.bounds` を `with` キーワードで出力していることを確認

---

## T1: パーサー — `parse_type_bounds` に `:` 記法追加

- [x] `parse_type_bounds` の `while self.peek() == &TokenKind::With...` ループ直後に `:` 記法ブランチを追加した
  - `TokenKind::Colon` を検出 → advance
  - `TokenKind::Impl` を検出 → advance（糖衣構文スキップ）
  - インターフェース名を `expect_ident()` で取得 → `TypeConstraint::Interface(name)` を push
  - `TokenKind::Amp` を検出 → advance してループ継続
  - それ以外 → break
- [x] エラーガード追加: `<T:>` → 明確なエラーメッセージ
- [x] エラーガード追加: `<T: impl>` → 明確なエラーメッセージ
- [x] エラーガード追加: `&&` 使用 → `"use '&' not '&&' to separate type bounds"`
- [x] 混在記法サポート: コロン後の `with` ループを追加（`<T: A with B>` → `[A, B]`）
- [x] `cargo build` でエラーがないことを確認

---

## T2: 既存テスト通過確認

- [x] `cargo test` で既存テスト（3594 件）が全 pass することを確認

---

## T3: `v715000_tests` 追加（`driver.rs`）

- [x] `v715000_tests` モジュールを `v714000_tests` の直後に追加した
- [x] `generic_constraint_multi_interface` — `<T: Serializable & Comparable>` の型チェック確認
- [x] `generic_constraint_impl_trait` — `<T: impl Printable>` の型チェック確認
- [x] `generic_constraint_bounds_content` — AST 内の bounds が `[Interface("Serializable"), Interface("Comparable")]` であることを検証
- [x] `generic_constraint_colon_then_with` — `<T: A with B>` が bounds を union することを検証
- [x] `generic_constraint_variance_colon` — `interface Container<+T: Ord>` で variance + coロン記法が動作することを確認
- [x] `cargo test v715000` で 5 件 pass することを確認

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.4.0"` → `"71.5.0"` に変更した
- [x] `driver.rs` 内の cargo_toml_version テストを `"71.5.0"` に更新した

---

## T5: CHANGELOG.md 更新

- [x] `## [v71.5.0]` エントリを先頭に追加した（5 テスト・エラー強化・混在記法サポートを記録）

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.5.0`（Generic Constraints）に更新した
- [x] 「次に切る版」を `v71.6.0` に更新した

---

## T7: 最終確認

- [x] `cargo test v715000` で 5 件 pass することを確認
- [x] `cargo test` 全体で 3599 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.5.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `peek_ident_text("impl")` が `TokenKind::Impl` に一致しない | `self.peek() == &TokenKind::Impl` に修正 |
| [HIGH] | `<T: impl>` — 不明瞭なエラーメッセージ | `impl` スキップ後に次トークンが Ident か確認、専用メッセージ追加 |
| [HIGH] | `<T:>` — 不明瞭なエラーメッセージ | `:` 消費後に Ident/Impl 以外なら専用メッセージ |
| [MED] | `&&` を使用した場合の不明瞭なエラー | `TokenKind::AmpAmp` 検出 → `"use '&' not '&&'"` メッセージ |
| [MED] | `<T: A with B>` — with 境界が解析されない | コロンブランチ後に `while with` ループを追加 |
| [LOW] | バリアンス + コロン記法の未テスト | `generic_constraint_variance_colon` テスト追加 |
| [LOW] | AST 内容（bounds）の未検証 | `generic_constraint_bounds_content` テスト追加 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `generic_constraint_multi_interface` が pass
- [x] `generic_constraint_impl_trait` が pass
- [x] `generic_constraint_bounds_content` が pass
- [x] `generic_constraint_colon_then_with` が pass
- [x] `generic_constraint_variance_colon` が pass
- [x] テスト総数: 3599（+5、実績ベース: 3594 + 5）
- [x] `<T: A & B>` がパースされ `TypeConstraint::Interface` が 2 つ生成される
- [x] `<T: impl A>` がパースされ `impl` キーワードがスキップされ `TypeConstraint::Interface("A")` になる
- [x] `<T: A with B>` がパースされ 2 つの bounds が生成される（混在記法）
- [x] 既存 `<T with Ord>` が引き続き pass（後方互換性）
- [x] コードレビュー全 7 件の指摘に対応済み
