# v60.1.0 Plan — エラーメッセージ span 表示（ソース位置・アンダーライン）

Date: 2026-07-30

---

## 実装方針

`driver.rs` の `format_diagnostic`（L47-95）はすでに `-->` / `|` / `^` 形式を実装している。
v60.1.0 の作業は以下の 2 ステップのみ：

1. `cmd_check_span_output(src: &str) -> String` をパブリック関数として追加
2. `v60100_tests` モジュールに 2 テストを追加

---

## ステップ詳細

### Step 1: `cmd_check_span_output` 追加（`driver.rs`）

`cmd_check` の既存テキスト出力ロジック（`format_diagnostic` 呼び出し）を
テスト可能な関数として公開する。

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

**挿入位置**: `format_warning` 関数（L97）の直前（`format_diagnostic` の直後）

### Step 2: `v60100_tests` モジュール追加（`driver.rs`）

driver.rs ではテストモジュールを **新しいバージョン順（ファイル上部）に追加する慣例** のため、
`v60000_tests` の直前（上側）に挿入する。

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

---

## 期待するテスト出力（参考）

```
error[E0001]: undefined variable: `undefined_var_abc`
  --> <test>:1:11
   |
1 | bind x <- undefined_var_abc
  |           ^^^^^^^^^^^^^^^^^
  = help: ...
```

---

## 注意事項

- `cmd_check_span_output` は `pub` にする（テスト外での将来の利用も想定）
- rolling check の更新は不要（サブバージョン、Cargo.toml version は `"60.0.0"` のまま）
- `v60100_tests` は `use super::*;` を使用（`cmd_check_span_output` の呼び出しに必要）
- `cmd_check_span_output` 内の `Parser` / `Checker` はローカル `use` で明示的にインポートする（コードスニペット L29-30 参照）
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.0.0（ベース） | 3330 | — |
| v60.1.0 | 3332 | +2 |
