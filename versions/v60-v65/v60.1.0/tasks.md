# v60.1.0 Tasks — エラーメッセージ span 表示（ソース位置・アンダーライン）

Date: 2026-07-30
Status: COMPLETE

---

## T0: 事前確認

- [x]`cargo test` でベースラインが 3330 tests passed, 0 failed であることを確認
- [x]`fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x]`fav/src/driver.rs` に `v60100_tests` がまだ存在しないことを確認
  - `grep -c 'v60100_tests' fav/src/driver.rs` = 0 件
- [x]`fav/src/driver.rs` L47-95 に `format_diagnostic` が存在することを確認
  - `grep -n 'fn format_diagnostic' fav/src/driver.rs` がヒットすること
- [x]`cmd_check_span_output` がまだ存在しないことを確認
  - `grep -c 'cmd_check_span_output' fav/src/driver.rs` = 0 件

---

## T1: `cmd_check_span_output` 追加（`driver.rs`）

`format_warning` 関数の直前（`format_diagnostic` の直後、L96 付近）に追加する。

```rust
/// v60.1.0: テスト・デバッグ用 span 出力ヘルパー
/// `src` を "<test>" ファイルとして型チェックし、span 付きエラー文字列を返す。
/// エラーがない場合は空文字列を返す。
pub fn cmd_check_span_output(src: &str) -> String {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    let program = match Parser::new("<test>", src).parse_program() {
        Ok(p) => p,
        Err(e) => return format!("parse error: {}", e),
    };
    let mut checker = Checker::new();
    match checker.check_program(&program) {
        Ok(_) => String::new(),
        Err(errors) => errors
            .iter()
            .map(|e| format_diagnostic(src, e))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
```

- [x]関数を `format_diagnostic` の直後に追加した
- [x]事後確認（T1 完了時点）: `grep -c 'cmd_check_span_output' fav/src/driver.rs` = 1 件（定義のみ）
  - T2 完了後は 3 件になる（定義 1 + テスト内呼び出し 2）

---

## T2: `v60100_tests` モジュール追加（`driver.rs`）

`v60000_tests` の直前に挿入する。

```rust
// -- v60100_tests (v60.1.0) -- エラーメッセージ span 表示 --
#[cfg(test)]
mod v60100_tests {
    use super::*;

    #[test]
    fn error_span_display_e0001() {
        // 未定義変数を含むソース → E0001 が span 付きで出力される
        let out = cmd_check_span_output("bind x <- undefined_var_abc");
        assert!(
            out.contains("-->"),
            "span display should contain '-->' but got:\n{}", out
        );
    }

    #[test]
    fn error_span_underline_format() {
        // アンダーライン（'^'）が出力に含まれることを確認
        let out = cmd_check_span_output("bind x <- undefined_var_abc");
        assert!(
            out.contains('^'),
            "span display should contain '^' underline but got:\n{}", out
        );
    }
}
```

- [x]`v60100_tests` モジュールを `v60000_tests` の直前（上側）に追加した
  （driver.rs はテストモジュールを新しい順＝ファイル上部に追加する慣例）
- [x]`use super::*;` が含まれている
- [x]`error_span_display_e0001` テストが含まれている
- [x]`error_span_underline_format` テストが含まれている

---

## T3: テスト実行・確認

- [x]`cargo test -j 8 -- --test-threads=8` を実行
- [x]`v60100_tests::error_span_display_e0001` pass
- [x]`v60100_tests::error_span_underline_format` pass
- [x]総テスト数 **3332** tests passed, 0 failed を確認

---

## T4: 事後処理

- [x]`versions/current.md` を v60.1.0 / 3332 tests に更新
  - 「進行中バージョン」を `v60.1.0` に更新
  - 「最新安定版」はまだ v60.0.0 のまま（milestone ではないため）
- [x]`versions/roadmap/roadmap-v60.1-v61.0.md` の v60.1.0 実績欄を更新
  - `**実績**: — （未実施）` → `**実績**: 3332 tests passed, 0 failed（2026-07-30 完了）`
- [x]CHANGELOG.md: サブバージョンのため個別エントリは不要（v61.0 宣言時にまとめて記載）
- [x]このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

- テスト入力 `"bind x <- undefined_var_abc"` が parse error になった（`bind` はトップレベル不可）
  → `"fn foo() -> Int { undefined_var_abc }"` に修正して解決
- `Parser::new` の引数が `(src, file)` ではなく `Vec<Token>` だった
  → `Lexer::new(src, "<test>").tokenize()` → `Parser::new(tokens)` の正しい呼び出し順に修正
- `Checker::check_program` が `&mut self` メソッドではなく静的関数・戻り値が `Result` ではなくタプル `(Vec<TypeError>, Vec<FavWarning>)` だった
  → `Checker::check_program(&program)` としてタプルアンパック形式に修正

**code-reviewer 対応（実装後レビュー）:**
- [MED] テストコメント・関数名が `E0001` と誤記（実際は `E0102` が生成される）
  → コメントを `E0102` に修正。`assert!(out.contains("E0102"))` と `assert!(!out.is_empty())` を追加
- [MED] テストの検証が弱い（偽陽性リスク）
  → 上記アサート追加で対応
- [LOW] `collect::<Vec<String>>()` のターボフィッシュが冗長
  → `collect::<Vec<_>>()` に変更

---

Status: COMPLETE
