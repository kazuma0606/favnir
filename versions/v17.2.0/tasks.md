# v17.2.0 Tasks — パターンマッチ拡張

Date: 2026-06-15
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"17.2.0"` に変更
- [ ] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — AST 拡張

- [ ] B-1: `fav/src/ast.rs` に `Pattern::Or(Vec<Pattern>, Span)` 追加
- [ ] B-2: `fav/src/ast.rs` に `Pattern::List { head: Vec<Pattern>, tail: Option<String>, span: Span }` 追加
- [ ] B-3: `fav/src/ast.rs` の `MatchArm` に `guard: Option<Expr>` フィールド追加
- [ ] B-4: `MatchArm` の既存構築箇所すべてに `guard: None` を追加（exhaustive 対応）
- [ ] B-5: `cargo build` → コンパイルエラー一覧確認（Phase F で修正）

---

## Phase C — パーサー拡張

- [ ] C-1: `fav/src/frontend/parser.rs` の `parse_match_arm` で or-pattern 解析追加
  - パターン解析後に `|` が続く場合、`Pattern::Or` として収集
  - 例: `"a" | "b" => body` → `Pattern::Or(vec![Lit("a"), Lit("b")])`
- [ ] C-2: `parse_pattern` に list-pattern 解析追加
  - `[` で始まるパターンを `Pattern::List` として解析
  - `[]` / `[x]` / `[head, ..tail]` / `[a, b, ..rest]` をすべてカバー
  - `..name` → `tail: Some("name")`
- [ ] C-3: `parse_match_arm` で guard 解析追加
  - パターン（+ or-pattern）解析後に `if` キーワードがあれば guard 式を解析
  - 結果を `MatchArm.guard: Some(expr)` に設定
- [ ] C-4: パース結果が正しい AST になることをスモークテストで確認

---

## Phase D — 型チェッカー拡張

- [ ] D-1: `fav/src/middle/checker.rs` の `check_pattern` に `Pattern::Or` ハンドラ追加
  - 各サブパターンが scrutinee 型と適合することを確認
  - Or 内の全パターンで同じバインディング変数セットを持つことを確認
- [ ] D-2: `check_pattern` に `Pattern::List` ハンドラ追加
  - scrutinee 型が `List<T>` であることを確認
  - head 要素の型が `T` にマッチすることを確認
  - tail がある場合、変数型を `List<T>` として環境に登録
- [ ] D-3: `check_match_arm` で `arm.guard` がある場合の型チェック追加
  - guard 式の型が `Bool` であることを確認
- [ ] D-4: 網羅性チェック（`check_match_exhaustiveness`）を Or / List パターンに対応
  - `Pattern::Or` → 各サブパターンを展開して判定
  - `Pattern::List { head: [], tail: None }` → 空リストカバー
  - `Pattern::List { tail: Some(_) }` → 1 要素以上カバー

---

## Phase E — コンパイラ拡張

- [ ] E-1: `fav/src/middle/compiler.rs` に or-pattern コンパイル追加
  - `Pattern::Or(pats)` → 各パターンを順次試行、いずれかマッチで成功
  - 失敗時は次アームへフォールスルー
- [ ] E-2: list-pattern コンパイル追加
  - `Pattern::List { head, tail }` → `GetLength` + 長さ確認 + `GetIndex` + `SliceFrom`
  - `tail: None` の場合は長さ厳密一致
  - `tail: Some(name)` の場合は長さ `>= head.len()` でマッチし残りを束縛
- [ ] E-3: guard コンパイル追加
  - `arm.guard: Some(expr)` → guard 評価、`false` なら次アームへジャンプ
- [ ] E-4: VM で `SliceFrom` / `GetLength` が未実装の場合、`List.drop` / `List.length` プリミティブを利用した実装に切り替え

---

## Phase F — exhaustive match 対応

- [ ] F-1: `fav/src/middle/checker.rs` の `match pattern { ... }` を `Pattern::Or` / `Pattern::List` に対応
- [ ] F-2: `fav/src/middle/compiler.rs` の `match pattern { ... }` を対応
- [ ] F-3: `fav/src/fmt.rs` の `fmt_pattern` に Or / List 追加
- [ ] F-4: `fav/src/middle/ast_lower_checker.rs` の `lower_pattern` に Or / List 追加
- [ ] F-5: その他 exhaustive match エラーを全件解消
- [ ] F-6: `cargo build` → コンパイルエラーなし確認

---

## Phase G — テスト追加（v172000_tests）

- [ ] G-1: `fav/src/driver.rs` に `v172000_tests` モジュール追加
- [ ] G-2: `version_is_17_2_0` — `Cargo.toml` に `"17.2.0"` が含まれる
- [ ] G-3: `or_pattern_string` — `"active" | "pending" => ...` が動作する
- [ ] G-4: `list_pattern_head_tail` — `[head, ..tail]` でリストを先頭と残りに分解できる
- [ ] G-5: `list_pattern_empty_single` — `[]` / `[x]` パターンが動作する
- [ ] G-6: `match_guard` — `{ amount: a } if a > 0.0 => ...` guard 条件が動作する
- [ ] G-7: `cargo test v172000` → 5/5 PASS 確認
  - `v171000_tests::version_is_17_1_0` に `#[ignore = "historical version check"]` を追加

---

## Phase H — サイトドキュメント作成

- [ ] H-1: `site/content/docs/language/patterns.mdx` を新規作成
  - or-pattern 構文説明（文字列・バリアント双方の例）
  - list-pattern 構文説明（`[]` / `[x]` / `[head, ..tail]` / `[a, b, ..rest]`）
  - guard 条件の構文説明
  - Before / After 比較（煩雑な if-else vs パターンマッチ）
  - 網羅性チェックとの連携説明

---

## Phase I — 最終確認 + コミット

- [ ] I-1: `cargo test v172000` → 5/5 PASS 最終確認
- [ ] I-2: `cargo test` → 全件 PASS（リグレッションなし）
  - 旧版 version_is_xxx テストは `#[ignore]` 済みのため除外
- [ ] I-3: コミット: `feat: v17.2.0 — パターンマッチ拡張（or-pattern / list-pattern / guard）`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "17.2.0"` | [ ] |
| `"a" \| "b" => ...` or-pattern が動作する | [ ] |
| `[head, ..tail]` list-pattern が動作する | [ ] |
| `[]` / `[x]` パターンが動作する | [ ] |
| `if guard` 条件が動作する | [ ] |
| or-pattern / list-pattern を含む match の網羅性チェックが正しく動作する | [ ] |
| `cargo test v172000` → 5/5 PASS | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 技術メモ

- **`|` の曖昧性**: `parse_match_arm` 内のパターン文脈でのみ `|` を OR として解釈。式文脈（`|x| ...` クロージャや bitwise OR）とは区別される。実装上は `parse_match_arm` で pattern パース後に `peek() == &TokenKind::Pipe` かつクロージャ外であれば Or と判定する。
- **`..tail` の字句解析**: `..` は既存 `DotDot` トークン、`..name` は `DotDot + Ident(name)` の 2 トークン。パターン内で `DotDot` の次が `Ident` の場合に rest binding として処理する。
- **List パターンのコンパイル**: `GetIndex` は `CallBuiltin("list_get_raw", [list, idx])` で実装可能。`SliceFrom(n)` は `CallBuiltin("list_drop_raw", [list, n])` で代用。VM に新 opcode は不要。
- **Guard のフォールスルー**: guard が `false` の場合、次のアームへ制御を移す。`compile_match` でアーム境界にラベルを設け、guard 失敗時は次ラベルへジャンプ。
- **MatchArm.guard の既存構築箇所**: parser.rs / checker.rs / compiler.rs の `MatchArm { pattern, body, span }` 構築箇所すべてに `guard: None` を追加する（Phase B-4）。
- **`include_str!` パス**: driver.rs → Cargo.toml は `"../Cargo.toml"`（`fav/src/` からの相対パス）。
