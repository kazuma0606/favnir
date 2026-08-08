# Tasks — v56.6.0 — パターンエイリアス（as-patterns `@`）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.6.0 セクションを確認
- [x] ベーステスト数 3238（v56.5.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.5.0` であることを確認（更新前）
- [x] `Pattern::As` が `ast.rs` に存在しないことを確認（新規追加対象）
- [x] `IRPattern::As` が `ir.rs` に存在しないことを確認（新規追加対象）
- [x] `TokenKind::At` が `lexer.rs` に存在しないことを確認（新規追加対象）
- [x] `test_unexpected_char` が `lexer.rs` に存在し `@` を invalid として検証していることを確認（更新対象）
- [x] `emit_python.rs` の `arm_condition` が `Pattern::List` で終了し catch-all なしであることを確認（`Pattern::As` 追加必須）
- [x] `v56600_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.5.0"` を期待していることを確認（更新対象 — モジュール名・関数名は変更しない慣例）
- [x] `pattern_is_catch_all` が `matches!` マクロを使用していることを確認（`match` 式への変更対象）
- [x] `collect_pattern_variants` に `_ => {}` catch-all が存在することを確認（`Pattern::As` が catch されること）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.6.0` に更新（56.5.0 から変更）
- [x] T2: `fav/src/frontend/lexer.rs` — `@` トークン追加
  - [x] `TokenKind` 列挙体に `At,` を追加（Symbols セクション末尾 `DotDot` / `LinearArrow` 付近）
  - [x] lex ディスパッチに `'@' => { self.advance(); TokenKind::At }` を追加（`self.advance()` 必須！）
  - [x] `test_unexpected_char` を `"foo @ bar"` → `"foo $ bar"` に変更（`@` は有効トークンになる）
- [x] T3: `fav/src/ast.rs` — `Pattern::As` 追加
  - [x] `Pattern` 列挙体に `As(String, Box<Pattern>, Span),` を追加（`List { ... }` の直後）
  - [x] `Pattern::span()` に `Pattern::As(_, _, s) => s,` を追加
- [x] T4: `fav/src/frontend/parser.rs` — `@` パース
  - [x] `parse_pattern()` の `TokenKind::Ident` アーム（小文字 Bind 分岐）に `@` チェックを追加
  - [x] `TokenKind::At` の場合は `Pattern::As(name, Box::new(sub_pattern), span)` を返す
- [x] T5: `fav/src/middle/ir.rs` — `IRPattern::As` 追加
  - [x] `IRPattern` 列挙体に `As(u16, Box<IRPattern>),` を追加（`List { ... }` の直後）
- [x] T6: `fav/src/middle/compiler.rs` — `Pattern::As` 対応
  - [x] `pattern_binds`: `Pattern::As(name, inner, _)` → `out.insert(name)` + `pattern_binds(inner, out)`
  - [x] `compile_pattern`: `Pattern::As(name, inner, _)` → `IRPattern::As(slot, inner_compiled)`
- [x] T7: `fav/src/backend/codegen.rs` — `IRPattern::As` 対応
  - [x] `emit_pattern_test`: `IRPattern::As(slot, inner)` → `Dup + StoreLocal slot + emit_pattern_test(inner, depth)`
  - [x] 末尾に `;` を付けないこと（`emit_pattern_test` の戻り値 `usize` をそのまま返す）
- [x] T8: `fav/src/middle/checker.rs` — `Pattern::As` 対応
  - [x] `check_pattern_bindings`: `Pattern::As(name, inner, _)` → `env.define(name, ty)` + recurse into inner
  - [x] `collect_pattern_variants`: `Pattern::As(_, inner, _)` → `collect_pattern_variants(inner, ...)`
- [x] T9: `fav/src/middle/ast_lower_checker.rs` — `Pattern::As` 対応
  - [x] `lower_pat`: `ast::Pattern::As(_, inner, _) => lower_pat(inner),`
- [x] T10: `fav/src/fmt.rs` — `Pattern::As` フォーマット
  - [x] `fmt_pattern`: `Pattern::As(name, inner, _) => format!("{} @ {}", name, fmt_pattern(inner))`
- [x] T11: `fav/src/emit_python.rs` — `Pattern::As` 対応
  - [x] `arm_condition`: `Pattern::As(name, inner, _)` を追加（inner の条件 + `name = var` bind）
- [x] T12: `fav/src/lint.rs` — `pattern_is_catch_all` 更新
  - [x] `matches!` マクロを `match` 式に変更
  - [x] `Pattern::As(_, inner, _) => pattern_is_catch_all(inner),` を追加
  - [x] `_ => false,` を末尾に追加
- [x] T13: `fav/src/driver.rs` — `v56600_tests` モジュールを `v56500_tests` の直前に追加
  - [x] `pattern_alias_binds_whole`: `v @ Ok(_) => true` が Checker エラーなし
  - [x] `pattern_alias_with_destructure`: `n @ 1 => "one"` が Checker エラーなし
- [x] T14: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.5.0"` → `"56.6.0"` に更新
  - [x] モジュール名 `v56300_tests` / 関数名 `cargo_toml_version_is_56_3_0` は変更しない（慣例）

---

## テスト・検証

- [x] T15: `cargo build` でコンパイルエラーがないことを確認
- [x] T16: `cargo test` 全通過（**3240 tests passed, 0 failed**）
  - [x] `v56600_tests::pattern_alias_binds_whole` ok
  - [x] `v56600_tests::pattern_alias_with_destructure` ok
  - [x] 既存 3238 件全通過
- [x] T17: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T18: `CHANGELOG.md` に v56.6.0 エントリを追加
- [x] T19: `versions/current.md` を v56.6.0 / 3240 tests に更新
- [x] T20: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.6.0 実績を COMPLETE に更新
- [x] T21: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.6.0 実績欄も COMPLETE に更新

---

## 完了確認

- [x] `pattern_alias_binds_whole` pass
- [x] `pattern_alias_with_destructure` pass
- [x] **3240 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `TokenKind::At` が `lexer.rs` に追加されている
- [x] `test_unexpected_char` が `$` を使用するように更新されている（`@` ではなく）
- [x] `Pattern::As(String, Box<Pattern>, Span)` が `ast.rs` に追加されている
- [x] `Pattern::span()` に `Pattern::As(_, _, s) => s` が追加されている
- [x] `IRPattern::As(u16, Box<IRPattern>)` が `ir.rs` に追加されている
- [x] `pattern_binds` が `Pattern::As` を処理している（name + inner 両方）
- [x] `compile_pattern` が `Pattern::As` → `IRPattern::As` を生成している
- [x] `emit_pattern_test` が `IRPattern::As` を処理している（Dup + StoreLocal + inner test）
- [x] `check_pattern_bindings` が `Pattern::As` の name を env に定義している
- [x] `collect_pattern_variants` が `Pattern::As` を inner に委譲している
- [x] `lower_pat` が `Pattern::As` を inner に委譲している
- [x] `fmt_pattern` が `Pattern::As` を `"name @ sub"` としてフォーマットしている
- [x] `arm_condition` が `Pattern::As` を処理している（exhaustive match の欠損なし）
- [x] `pattern_is_catch_all` が `Pattern::As(_, _)` を inner の catch-all に委譲している
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.6.0"` になっている
- [x] `CHANGELOG.md` に v56.6.0 エントリが追加されている
- [x] `versions/current.md` が v56.6.0 / 3240 tests を反映
- [x] T20 / T21 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ（コードレビュー・トラブルシュート）

### 重大バグ修正: lexer で `self.advance()` を呼ばないと無限ループ
- `'@' => TokenKind::At` と書くと `@` が消費されず lexer が同じ文字を無限に返す
- 正しくは `'@' => { self.advance(); TokenKind::At }` — 全 char dispatch アームは必ず `self.advance()` を先頭で呼ぶ

### IRPattern::As の追加漏れ場所
- `codegen.rs` だけでなく `driver.rs` の `remap_ir_pattern` にも `IRPattern::As` アームが必要
- cargo build エラーで `src\driver.rs:13821` と表示される

### Pattern::As の各ファイル対応
- `emit_python.rs::arm_condition` は catch-all なし → 明示的に `Pattern::As` アームが必須
- `lint.rs::pattern_is_catch_all` は `matches!` → `match` 式に変換して `Pattern::As` を追加
- `collect_pattern_variants` は `_ => {}` catch-all があるため変更不要（ただし明示的に追加推奨）
