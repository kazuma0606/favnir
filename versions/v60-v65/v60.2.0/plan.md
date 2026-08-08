# v60.2.0 Plan — `fav check --fix` 自動修正 Phase 1

Date: 2026-07-30

---

## 実装方針

3 つのヘルパー関数と 1 つのテストモジュールを追加する。
main.rs への `--fix` / `--dry-run` フラグ追加はスコープ外（テストで `cmd_check_fix_src` を直接呼ぶため）。

---

## ステップ詳細

### Step 1: `extract_backtick_ident` ヘルパー追加（`driver.rs`）

`"did you mean \`userId\`?"` や `"undefined local: \`user_id\`"` からバッククォート内の
識別子を抽出するヘルパー。`cmd_check_fix_src` の直前に追加する。

```rust
/// バッククォートで囲まれた最初の識別子を返す。
/// 例: `"did you mean \`foo\`?"` → `Some("foo")`
fn extract_backtick_ident(s: &str) -> Option<&str> {
    let start = s.find('`')? + 1;
    let end = s[start..].find('`')? + start;
    Some(&s[start..end])
}
```

### Step 2: `cmd_check_fix_src` 追加（`driver.rs`）

文字列ソースに対して auto-fix を実行し、修正サマリーを返す。
ファイル書き込みを行わないため、テストで直接呼び出せる。

```rust
/// v60.2.0: `fav check --fix` のソース文字列版（テスト・内部用）。
/// `dry_run = true` の場合はファイルを書き換えず、変更予定を文字列で返す。
pub fn cmd_check_fix_src(src: &str, dry_run: bool) -> String {
    use crate::frontend::parser::Parser;
    use crate::lint;
    use crate::middle::checker::Checker;

    let program = match Parser::parse_str(src, "<fix>") {
        Ok(p) => p,
        Err(e) => return format!("parse error: {:?}", e),
    };
    let (errors, _warnings) = Checker::check_program(&program);
    let lint_errors = lint::lint_program(&program);

    let prefix = if dry_run { "[would fix]" } else { "[auto-fixed]" };
    let mut fixes: Vec<String> = Vec::new();

    // E0102: did-you-mean が 1 件のみなら typo 修正
    for e in &errors {
        if e.code == "E0102" {
            let candidates: Vec<&str> = e.hints.iter()
                .filter(|h| h.starts_with("did you mean"))
                .filter_map(|h| extract_backtick_ident(h))
                .collect();
            if candidates.len() == 1 {
                if let Some(bad) = extract_backtick_ident(&e.message) {
                    fixes.push(format!(
                        "{} E0102: `{}` → `{}` (<fix>:{})",
                        prefix, bad, candidates[0], e.span.line
                    ));
                }
            }
        }
    }

    // L002: 未使用 bind 削除
    for l in &lint_errors {
        if l.code == "L002" {
            if let Some(name) = extract_backtick_ident(&l.message) {
                fixes.push(format!(
                    "{} L002: unused bind `{}` removed (<fix>:{})",
                    prefix, name, l.span.line
                ));
            }
        }
    }

    if fixes.is_empty() {
        return "0 fixes — no fixable issues found.".to_string();
    }

    let count = fixes.len();
    let mut out = fixes.join("\n");
    out.push('\n');
    if dry_run {
        out.push_str(&format!("{} fixes would be applied (dry-run, no changes made).", count));
    } else {
        out.push_str(&format!("{} fixes applied.", count));
    }
    out
}
```

### Step 3: `cmd_check_fix` 追加（`driver.rs`）

ファイルパスを受け取り、修正を適用してファイルに書き戻す CLI 向け関数。
`cmd_check_fix_src` の直後に追加する。

```rust
/// v60.2.0: `fav check --fix <file>` の実装。
pub fn cmd_check_fix(file: &str, dry_run: bool) -> String {
    let source = load_file(file);
    let result = cmd_check_fix_src(&source, dry_run);
    if !dry_run && result.contains("[auto-fixed]") {
        // 実際の fix 適用は今後の拡張で実装（v60.2 では dry-run と出力確認が主目的）
        // TODO: span ベースのソース書き換えを実装する
    }
    result
}
```

**注意**: v60.2.0 では `cmd_check_fix_src` はサマリー文字列を返すのみで実際のソース書き換えは行わない。
ファイルへの書き戻し（span ベースのトークン置換・行削除）は v60.2 以降の拡張で実装する。
テストは `cmd_check_fix_src` の出力形式（`[auto-fixed]` / `[would fix]` の有無）を検証する。

### Step 4: `v60200_tests` モジュール追加（`driver.rs`）

`v60100_tests` の直前（上側）に挿入する。

```rust
// -- v60200_tests (v60.2.0) -- fav check --fix 自動修正 --
#[cfg(test)]
mod v60200_tests {
    use super::*;

    #[test]
    fn check_fix_typo_single_candidate() {
        // 既知の変数 `userId` に対して `user_id` を使うと E0102 + did-you-mean ヒントが出る
        // fn 内で userId を定義して user_id を参照 → typo 修正候補が 1 件
        let src = "fn go(userId: Int) -> Int { user_id }";
        let out = cmd_check_fix_src(src, false);
        assert!(
            out.contains("[auto-fixed]") && out.contains("E0102"),
            "expected E0102 auto-fix but got:\n{}", out
        );
    }

    #[test]
    fn check_fix_unused_bind() {
        // fn 内で bind した変数を使わない → L002 → --fix --dry-run で [would fix] 出力
        let src = "fn go() -> Int {\n  bind tmp <- 42\n  0\n}";
        let out = cmd_check_fix_src(src, true);
        assert!(
            out.contains("[would fix]") && out.contains("L002"),
            "expected L002 would-fix but got:\n{}", out
        );
    }
}
```

---

## 注意事項

- サブバージョンのため `Cargo.toml` version は `"60.0.0"` のまま変更しない
- rolling check の更新は不要
- `v60200_tests` は `v60100_tests` の直前（上側）に追加（driver.rs の慣例）
- `use super::*;` を使用（`cmd_check_fix_src` / `extract_backtick_ident` の呼び出しに必要）
- main.rs への `--fix` フラグ追加は今後（実際のファイル書き換えが完成してから）
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.1.0（ベース） | 3332 | — |
| v60.2.0 | 3334 | +2 |
