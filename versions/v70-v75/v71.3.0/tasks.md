# v71.3.0 タスクリスト — Phantom Types（型タグによる誤使用防止）

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.2.0` であることを確認
- [x] `cargo test` が全 pass（3589 tests）であることを確認
- [x] `phantom` がレキサー/パーサーで特殊扱いされていないことを確認（識別子として認識される）
  - `grep -rn '"phantom"' fav/src/frontend/` → 結果なし（未定義）
- [x] `TypeDef.is_opaque` の初期化箇所一覧を確認（4箇所 + checker.rs 1箇所）

---

## T1: AST — `is_phantom: bool` フィールド追加

- [x] `fav/src/ast.rs` の `TypeDef` 構造体に `is_opaque` の直後に追加した:
  - `pub is_phantom: bool,  // v71.3.0: phantom type キーワード（デフォルト false）`
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T2: パーサー — 全 TypeDef 初期化に `is_phantom: false` 追加

- [x] `fav/src/frontend/parser.rs` の TypeDef を構築する全箇所（4箇所）に `is_phantom: false` を追加した:
  1. Wrapper body
  2. Record body
  3. Alias body（既存 + phantom ブランチの `is_phantom: true`）
  4. Sum body
- [x] `fav/src/middle/checker.rs` の TypeDef 構築箇所にも `is_phantom: false` を追加した
- [x] `cargo test` で既存テスト（3589 件）が全 pass することを確認

---

## T3: パーサー — `phantom` 文脈キーワードの解析追加

- [x] `parse_type_def` 内の `self.expect(&TokenKind::Eq)?` の直後、alias body 解析の前に `phantom` ハンドラを追加した
- [x] `Parser::parse_str("type UserId = phantom String", "test.fav")` が成功することを確認（テスト実行で確認）
- [x] `cargo test` で既存テスト（3589 件）が全 pass することを確認

---

## T4: チェッカー — pre-pass から phantom を除外

- [x] `register_item_signatures` 先頭の alias invariant pre-pass に `!td.is_phantom` 条件を追加した:
  - `if !td.is_opaque && !td.is_phantom && !td.invariants.is_empty()`

---

## T5: チェッカー — phantom コンストラクタ登録

- [x] `register_item_signatures` の `TypeBody::Alias` ブランチに `else if td.is_phantom` を追加した:
  - `resolve_type_expr(target)` → `env.define(name, Fn([inner_ty], Named(name)))` → `continue`
  - `type_aliases` には登録しない（透過解決させない）
- [x] `cargo test` で既存テスト（3589 件）が全 pass することを確認

---

## T6: fmt.rs — phantom 型のフォーマット

- [x] `fav/src/fmt.rs` の `TypeBody::Alias` ブランチに `is_phantom` / `is_opaque` 分岐を追加した
  - `phantom` → `type Name = phantom Inner`
  - `opaque` → `type Name = opaque Inner`（既存バグも同時修正）
  - それ以外 → `type Name = Inner`
- [x] `cargo test` で既存テスト（3589 件）が全 pass することを確認

---

## T7: driver.rs に `v713000_tests` を追加

- [x] driver.rs 末尾（`v712000_tests` の直後）に `v713000_tests` モジュールを追加した
- [x] `phantom_type_explicit_cast` テストを実装した
- [x] `phantom_type_prevents_id_confusion` テストを実装した
- [x] `cargo test v713000` で 2 件 pass することを確認

---

## T8: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.2.0"` → `"71.3.0"` に変更した
- [x] driver.rs 内の `"71.2.0"` 文字列リテラルを `"71.3.0"` に一括更新した（replace_all）

---

## T9: CHANGELOG.md 更新

- [x] `## [v71.3.0] — 2026-08-09 — Phantom Types（型タグによる誤使用防止）` エントリを先頭に追加した

---

## T10: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.3.0`（Phantom Types）に更新した
- [x] 「次に切る版」を `v71.4.0` に更新した

---

## T11: 最終確認

- [x] `cargo test v713000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3591 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.3.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### [HIGH] opaque + phantom 同時指定でフラグ競合
- `opaque type Foo = phantom Bar` により `is_opaque: true && is_phantom: true` になる可能性
- 対応: `register_item_signatures` の `TypeBody::Alias` ブランチ先頭に E0246 ガード追加
- `error_catalog.rs` に E0246（conflicting type modifiers）エントリ追加

### [LOW] fmt.rs round-trip テストがない
- 対応: `v713000_tests` に `fmt_phantom_and_opaque_types` テスト追加（3 件目）

### [LOW] `check_type_def` に `!td.is_phantom` ガードがない
- 対応: Alias ブランチの条件に `!td.is_phantom` を追加

### [LOW] `cmd_bench_all` の `.expect()` 呼び出し
- スコープ外（v70.3.0 既存コード）、対応なし

---

## 完了チェックリスト

- [x] 全タスク（T0〜T11）が完了している
- [x] `phantom_type_explicit_cast` が pass
- [x] `phantom_type_prevents_id_confusion` が pass
- [x] `fmt_phantom_and_opaque_types` が pass（コードレビュー対応で追加）
- [x] E0246（opaque + phantom 同時指定）が実装済み（コードレビュー対応）
- [x] テスト総数: 3592（+3）
- [x] `type UserId = phantom String` が parse できる
- [x] `UserId("u-123")` が typecheck で通る（コンストラクタ登録）
- [x] `OrderId` を `UserId` 引数に渡すとコンパイルエラーになる
