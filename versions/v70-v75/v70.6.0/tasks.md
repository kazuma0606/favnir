# v70.6.0 タスクリスト — `bind` 分割束縛拡張 / Named Destructuring

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.5.0` であることを確認
- [x] driver.rs の `cargo_toml_version_is_70_5_0` テストが存在することを確認
- [x] `cargo test` が全 pass（3570 tests）であることを確認
- [x] parser.rs の `parse_bind_stmt` が `parse_pattern()` を呼ぶことを確認（line 2683）
- [x] parser.rs の `parse_pattern` が `TokenKind::LBrace` → `Pattern::Record` を処理することを確認（line 2807）
- [x] parser.rs の `parse_pattern` が `TokenKind::LBracket` → `Pattern::List`（DotDot スプレッド）を処理することを確認（line 2941）
- [x] checker.rs が `BindStmt` の `Pattern::Record` / `Pattern::List` を型チェックできることを確認（`check_bind_stmt` または `check_stmt` で `PatternKind::Record` を処理しているか）
- [x] compiler.fav の `TkBind` ハンドラが `TkIdent` のみ処理し `TkLBrace` / `TkLBracket` 未対応であることを確認（line 1497）
- [x] compiler.fav に `EAccess(Expr, String)` が定義されていることを確認（line 572）

---

## T1: compiler.fav に `parse_destr_fields` / `make_destr_binds` を追加

- [x] `fav/self/compiler.fav` の `parse_arms` 関数の近傍（line 1400 付近）に以下を追加する:
  - `type FieldsParse = { fields: List<String>  rest: List<Token> }` — 型定義
  - `fn parse_destr_fields(toks: List<Token>, acc: List<String>) -> Result<FieldsParse, String>` — `{field1, field2}` をパース
  - `fn make_destr_binds(tmp: String, fields: List<String>, cont: Expr) -> Expr` — `EBind` チェーンを生成
- [x] `cargo test` で既存テスト（3570 件）が全 pass することを確認

---

## T2: compiler.fav の `TkBind` ハンドラに `TkLBrace` 分岐を追加

- [x] `TkBind` ハンドラ（line 1494〜1519）の `Some(TkIdent(vname))` アームの直前に `Some(TkLBrace)` アームを追加する:
  - `parse_destr_fields(rest1, [])` でフィールドリストを取得
  - `TkBackArrow` を消費
  - `parse_expr` で RHS をパース
  - `parse_block_inner` で継続をパース
  - `make_destr_binds("$_d", fields, cont)` でデシュガーした `EBind` チェーンを生成
  - 全体を `EBind("$_d", rhs, inner)` でラップ
- [x] `cargo test` で既存テスト（3570 件）が全 pass することを確認

---

## T3: `v706000_tests` モジュールを driver.rs 末尾に追加

- [x] `v705000_tests` の直後（driver.rs 末尾）に `v706000_tests` モジュールを追加する
- [x] `bind_destructure_record` テストを実装する:
  - `type User = { name: String score: Int }` を定義
  - `bind {name, score} <- u` の Record 分割束縛
  - `Parser::parse_str` → parse 成功を assert
  - `Checker::check_program` → errors が空であることを assert
  - `build_artifact` → パニックなし
- [x] `bind_destructure_list_spread` テストを実装する:
  - `List<Int>` に対して `bind [head, ..tail] <- items`
  - `Parser::parse_str` → parse 成功を assert
  - `Checker::check_program` → errors が空であることを assert
  - `build_artifact` → パニックなし
- [x] `cargo test v706000` で 2 件 pass することを確認

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.5.0"` → `"70.6.0"` に変更する
- [x] driver.rs 内の `"70.5.0"` 文字列を `replace_all: true` で `"70.6.0"` に一括更新
  - 対象: `cargo_toml_version_is_70_5_0` テスト関数内の `"70.5.0"` 文字列
  - 注: テスト関数名 `cargo_toml_version_is_70_5_0` 自体はリネームしない

---

## T5: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.5.0 エントリの直前）に v70.6.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `v706000_tests` 2 件（3570 → 3572 tests）
  - Fixed: `compiler.fav` `TkBind` ハンドラに `TkLBrace` 対応追加
  - Verified: Rust パイプラインの Record/List 分割束縛 E2E 確認

---

## T6: versions/current.md 更新

- [x] `versions/current.md` を開く
- [x] 「進行中バージョン」を `v70.6.0`（bind 分割束縛拡張）に更新する
- [x] 「次に切る版」を `v70.7.0` に更新する

---

## T7: 最終確認

- [x] `cargo test v706000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3572 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.6.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### 実装時判明（spec-reviewer 指摘対応）
- `make_destr_binds` の `[fname, ..rest]` パターン: compiler.fav 自身の `parse_pat` が list パターン非対応 → `List.first` / `List.drop` パターンに変更
- `[]` 空リストリテラル: Rust パーサーの `LBracket` は list comprehension のみ対応（空リスト非対応） → `List.empty()` に変更
- `bind inner <- Result.ok(make_destr_binds(...))`: `bind` は単純代入（モナドアンラップなし）なので `Result.ok()` ラップ不要 → `bind inner <- make_destr_binds(...)` に修正
- `parse_destr_fields(rest1, [])` → `parse_destr_fields(rest1, List.empty())` に修正

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `bind_destructure_record` が pass
- [x] `bind_destructure_list_spread` が pass
- [x] テスト総数: 3572（+2）
