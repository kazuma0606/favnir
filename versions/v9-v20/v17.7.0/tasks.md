# v17.7.0 — `forall` プロパティベーステスト タスク

## ステータス: 完了

---

## タスク一覧

### T1: lexer — `TokenKind::Forall` 追加

- [x] `fav/src/frontend/lexer.rs` の `TokenKind` enum に `Forall,` を追加
- [x] キーワードマッチに `"forall" => TokenKind::Forall,` を追加

### T2: AST — `Stmt::Forall` / `ForallStmt` / `ForallVar` 追加

- [x] `fav/src/ast.rs` の `Stmt` enum に `Forall(ForallStmt),` を追加
- [x] `ForallStmt` 構造体を追加
  - フィールド: `vars: Vec<ForallVar>`, `guard: Option<Expr>`, `body: Block`, `span: Span`
- [x] `ForallVar` 構造体を追加
  - フィールド: `name: String`, `ty: TypeExpr`, `span: Span`
- [x] `Stmt::span()` メソッドに `Stmt::Forall(f) => &f.span,` を追加

### T3: parser — `parse_forall_stmt` 追加

- [x] `fav/src/frontend/parser.rs` の `parse_block_stmts` に `TokenKind::Forall` ブランチを追加
- [x] `parse_forall_stmt` 関数を実装
  - `forall` を消費
  - 変数名を `Ident` として取得
  - `:` を消費
  - `parse_type_expr()` で型を取得
  - オプションで `where` キーワードを検出し `{ expr }` でガード取得
  - `{` ... `}` でボディブロックを `parse_block()` で取得
  - `ForallStmt { vars, guard, body, span }` を返す

### T4: checker — `Stmt::Forall` 型検査 + E0327

- [x] `fav/src/middle/checker.rs` の `collect_helpers_in_stmt` に `Stmt::Forall(f)` を追加
- [x] `check_stmt` に `Stmt::Forall(f)` を追加
  - 型が `Int / Float / String / Bool` 以外なら E0327 を報告
  - ガードがあれば `check_expr` で型チェック
  - `self.env.define(var.name, ty)` でスコープに変数を追加し body をチェック
- [x] `scan_expr_for_pipeline_calls` に `Stmt::Forall` を追加

### T5: VM primitives — `__forall_gen_int/str/bool/float` 追加

- [x] `fav/src/backend/vm.rs` に `__forall_gen_int` を実装
  - 先頭: `0, 1, -1, i32::MAX as i64, i32::MIN as i64`
  - 残り: xorshift64 疑似乱数整数（シード固定: 12345）
- [x] `__forall_gen_str` を実装
  - 先頭: `"", " ", "a", "\n", "hello world"`
  - 残り: xorshift64 ベースの ASCII ランダム文字列（長さ 0〜20）
- [x] `__forall_gen_bool` を実装
  - `[true, false, true, false, ...]` の繰り返し（最大 n 件）
- [x] `__forall_gen_float` を実装
  - 先頭: `0.0, 1.0, -1.0, 0.5, -0.5`
  - 残り: xorshift64 ベースの浮動小数点（NaN/Inf 除外）
- [x] `fav/src/middle/compiler.rs` の builtin 名前テーブル（2 箇所）に 4 関数を追加

### T6: compiler — `Stmt::Forall` デシュガー

- [x] `fav/src/middle/compiler.rs` の `collect_free_vars_block` に `Stmt::Forall(f)` を追加
- [x] `compile_stmt_into` に `Stmt::Forall(f)` を追加し、以下のデシュガーを実装：

  **ガードなし:**
  ```
  bind __vals <- __forall_gen_{type}(CASES)
  for x in __vals { body }
  ```

  **ガードあり:**
  ```
  bind __vals_raw <- __forall_gen_{type}(CASES * 10)
  bind __vals <- [x | x <- __vals_raw, guard]
  bind __taken <- List.take(__vals, CASES)
  for x in __taken { body }
  ```

  - `CASES` は `std::env::var("FORALL_CASES").unwrap_or("100")` で取得
  - 型に応じて gen 関数を選択

### T7: exhaustive match — 全 Stmt::Forall ハンドラ追加

- [x] `fav/src/fmt.rs` の `fmt_stmt` に `Stmt::Forall(f)` を追加
- [x] `fav/src/emit_python.rs` に `Stmt::Forall(f)` を追加（コメント出力）
- [x] `fav/src/lineage.rs` の 4 関数に `Stmt::Forall(f)` を追加
- [x] `fav/src/lint.rs` の 7 関数に `Stmt::Forall(f)` を追加

### T8: `main.rs` — `--cases N` オプション追加

- [x] `fav/src/main.rs` の `Some("test")` ブランチに `--cases` オプション解析を追加
- [x] `FORALL_CASES` 環境変数を設定

### T9: `driver.rs` — `v177000_tests` 追加

- [x] `fav/src/driver.rs` の `v176000_tests` の `version_is_17_6_0` テストを削除
- [x] `v177000_tests` モジュールを追加

```rust
#[cfg(test)]
mod v177000_tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn version_is_17_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.7.0\""), "Cargo.toml should have version 17.7.0");
    }

    #[test]
    fn forall_int_parses() {
        let src = r#"
test "forall int parses" {
  forall n: Int {
    assert_true(Math.abs(n) >= 0 || Math.abs(n) < 0)
  }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let has_forall = prog.items.iter().any(|i| {
            if let crate::ast::Item::TestDef(t) = i {
                t.body.stmts.iter().any(|s| matches!(s, crate::ast::Stmt::Forall(_)))
            } else {
                false
            }
        });
        assert!(has_forall, "forall stmt should be parsed inside test block");
    }

    #[test]
    fn forall_string_idempotent() {
        let src = r#"
test "trim は冪等" {
  forall s: String {
    bind trimmed <- String.trim(s)
    assert_eq(trimmed, String.trim(trimmed))
  }
}
"#;
        let result = run_test_source(src, "test.fav");
        assert!(result.is_ok(), "forall string idempotent should pass: {:?}", result);
    }

    #[test]
    fn forall_finds_counterexample() {
        let src = r#"
test "すべての Int は正" {
  forall n: Int {
    assert_true(n > 0)
  }
}
"#;
        let result = run_test_source(src, "test.fav");
        assert!(result.is_err(), "forall should find counterexample for false property");
        let err = result.unwrap_err();
        assert!(err.contains("counterexample") || err.contains("assert"), "error should mention counterexample");
    }

    #[test]
    fn forall_with_guard() {
        let src = r#"
test "非ゼロ整数の符号は非ゼロ" {
  forall n: Int where { n != 0 } {
    bind abs_n <- Math.abs(n)
    assert_true(abs_n > 0)
  }
}
"#;
        let result = run_test_source(src, "test.fav");
        assert!(result.is_ok(), "forall with guard should pass: {:?}", result);
    }
}
```

### T10: バージョン更新

- [ ] `fav/Cargo.toml` のバージョンを `17.6.0` → `17.7.0` に更新
- [ ] `cargo build` で `Cargo.lock` 更新

### T11: ドキュメント

- [ ] `site/content/docs/tools/property-testing.mdx` を新規作成
  - `forall` 構文の説明
  - 対応型（Int / Float / String / Bool）
  - `where { guard }` ガード条件の使い方
  - `--cases N` オプション
  - 失敗時の反例出力形式
  - ユースケース例

---

## テスト（v177000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_7_0` | Cargo.toml に "17.7.0" が含まれる |
| `forall_int_parses` | `forall n: Int { ... }` が AST として解析される |
| `forall_string_idempotent` | `String.trim` の冪等性が forall で確認できる |
| `forall_finds_counterexample` | 偽のプロパティが失敗し反例が報告される |
| `forall_with_guard` | `where { n != 0 }` ガードで 0 を除外した Int のプロパティが成立する |

---

## 完了条件チェックリスト

- [ ] `TokenKind::Forall` が lexer に追加されている
- [ ] `Stmt::Forall(ForallStmt)` が AST に追加されている
- [ ] `parse_forall_stmt` がパースできる
- [ ] `check_stmt` が型・ガードを検査し E0327 を報告できる
- [ ] `__forall_gen_int/str/bool/float` VM primitive が動作する
- [ ] `compile_stmt_into` が ForIn ループにデシュガーする
- [ ] `--cases N` オプションが `FORALL_CASES` 環境変数に反映される
- [ ] `cargo test v177000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし

---

## 優先度

T1（lexer）
→ T2（AST）
→ T3（parser）
→ T4（checker）
→ T5（VM primitives）
→ T6（compiler desugar）
→ T7（exhaustive match 全ファイル）
→ T8（main.rs --cases）
→ T9（driver.rs v177000_tests）
→ T10（バージョン更新）
→ T11（ドキュメント）

T5 と T7 は T6 の前に完了させること（compile_stmt_into が builtin 名を参照するため）。
T9 のテストは T1〜T7 完了後に追加（コンパイル通過が前提）。

---

## 補足: xorshift64 実装

```rust
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}
```

シードは `12345u64` 固定（再現性確保）。
