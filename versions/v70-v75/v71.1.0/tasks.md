# v71.1.0 タスクリスト — 依存型の基礎 `Vec<T>[N]`

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.0.0` であることを確認
- [x] `cargo test` が全 pass（3584 tests）であることを確認
- [x] `Vec<Float>[1536]` が現行パーサーで通るか確認（`Parser::parse_str` を試す）
  - → 現行パーサーは `[N]` 未対応 → プランA（パーサー拡張）を実施
- [x] 現行チェッカーに E0420 が未定義であることを確認
  - → E0420 は既に CEP 用に使用済み → E0421 を使用

---

## T1: パーサー対応

- [x] `src/frontend/parser.rs` の `parse_base_type` に `Vec<T>[N]` サフィックスを追加する
  - `Vec<T>` パース後に `[` が続く場合、整数または識別子を次元として読む
  - 次元エンコード: `Vec<Float>[1536]` → `TypeExpr::Named("Vec#1536", [Float], span)`
  - 型変数次元: `Vec<Float>[N]` → `TypeExpr::Named("Vec#?N", [Float], span)`
- [x] `cargo test` で既存テスト（3584 件）が全 pass することを確認

---

## T2: チェッカーに E0421 を追加

- [x] `src/middle/checker.rs` に `is_dim_annotated_name_mismatch` ヘルパーを追加する
  - `Named("Vec#1536", _)` と `Named("Vec#768", _)` の名前プレフィクスが同一で次元が異なる場合 true
- [x] `Expr::Apply` の unify ループで E0421 を検出・発行する
  - `is_dim_annotated_name_mismatch` が true の場合 E0421、それ以外は従来通り E0218/E0219
- [x] `cargo test` で既存テスト（3584 件）が全 pass することを確認

---

## T3: driver.rs に `v711000_tests` を追加

- [x] driver.rs 末尾（`v71000_tests` の直後）に `v711000_tests` モジュールを追加する
- [x] `dependent_type_vec_dim_param` テストを実装する:
  - `fn process(v: Vec<Float>[1536]) -> Int { 1536 }` が parse + typecheck で通ることを確認
  - E9999 等の予期しないエラーが出ないことを assert
- [x] `dependent_type_dim_mismatch_error` テストを実装する:
  - `fn dot(a: Vec<Float>[1536], b: Vec<Float>[1536]) -> Float { 0.0 }` に `Vec<Float>[768]` を渡す
  - E0421 が発生することを assert
- [x] `cargo test v711000` で 2 件 pass することを確認

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.0.0"` → `"71.1.0"` に変更する
- [x] driver.rs 内の `"71.0.0"` 文字列リテラルを `"71.1.0"` に一括更新する（replace_all）

---

## T5: CHANGELOG.md 更新

- [x] `## [v71.1.0] — 2026-08-09 — 依存型の基礎 Vec<T>[N]` エントリを先頭に追加する
- [x] エントリに以下を含める:
  - Added: `v711000_tests` 2 件（3584 → 3586 tests）
  - Added: チェッカー E0421（依存型次元不一致）
  - Added: パーサー `Vec<T>[N]` 次元注釈サポート

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.1.0`（依存型の基礎 `Vec<T>[N]`）に更新する
- [x] 「次に切る版」を `v71.2.0` に更新する
- [x] v71.0 — Language Complete 1.0 を「完了」に更新する

---

## T7: 最終確認

- [x] `cargo test v711000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3586 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.1.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### 第 1 回レビュー指摘対応

#### [HIGH] `fav fmt` round-trip 破損 — formatter が `Vec#1536<Float>` を出力する
- **対応**: `fav/src/fmt.rs` の `type_expr` で `TypeExpr::Named` の name に `#` が含まれる場合、`Vec<Float>[1536]` 形式に逆変換して出力するよう修正

#### [HIGH] `display()` が内部エンコード `Vec#1536<Float>` をユーザー向けエラーメッセージに露出する
- **対応**: `fav/src/middle/checker.rs` の `Type::display` で `Named` 型の name に `#` が含まれる場合、`Vec<Float>[1536]` 形式に逆変換して出力するよう修正

### 第 2 回レビュー指摘対応

#### [HIGH] `#` エンコードが未デコードのまま残る表示パスが 4 箇所ある
- **対応**: `fav/src/ast.rs::TypeExpr::display()` を修正（`#` 含む name をデコード）
- **対応**: `fav/src/driver.rs` に `decode_dim_name(name: &str) -> (String, String)` ヘルパーを追加
- **対応**: `favnir_type_display()` / `format_type_expr()` / `type_expr_kind()` の 3 関数で `decode_dim_name` を使うよう修正

#### [LOW] `dependent_type_vec_dim_param` テストの assertion が弱い（`!E9999` のみ）
- **対応**: `errors.is_empty()` に変更し、あらゆる型エラーが出ないことを確認するよう強化

#### [LOW] `cargo_toml_version_is_*` テストの失敗メッセージが stale（`70.0.0` のまま）
- **対応**: `replace_all` で `"Cargo.toml version should be 70.0.0"` → `"Cargo.toml version should be 71.1.0"` に一括更新

### [MEDIUM/LOW] その他の指摘（最小実装スコープ外として許容）
- `Vec[1536]`（型引数なし）のサイレント受け入れ: テスト対象外。v71.4.0（Const Eval）時に対応予定
- `is_dim_annotated_name_mismatch` が annotated vs unannotated を処理しない: テスト対象外
- 識別子次元 `#?var` の型多相性なし: v71.1.0 の最小実装スコープ外
- 負の次元: 実際には lexer が `-` を別トークンとして処理するため問題なし

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `dependent_type_vec_dim_param` が pass
- [x] `dependent_type_dim_mismatch_error` が pass
- [x] テスト総数: 3586（+2）
- [x] `Vec<Float>[N]` 構文がパーサーに追加されていることを確認
- [x] E0421 がチェッカーに追加されていることを確認
- [x] 実装方針: プランA（パーサー `[N]` サフィックス + チェッカー E0421）
  - 次元を型名にエンコード（`Vec#1536`）し、既存 unify 機構で検出
