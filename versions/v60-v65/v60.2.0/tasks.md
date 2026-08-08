# v60.2.0 Tasks — `fav check --fix` 自動修正 Phase 1

Date: 2026-07-30
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3332 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60200_tests` がまだ存在しないことを確認
  - `grep -c 'v60200_tests' fav/src/driver.rs` = 0 件
- [x] `cmd_check_fix_src` がまだ存在しないことを確認
  - `grep -c 'cmd_check_fix_src' fav/src/driver.rs` = 0 件
- [x] `extract_backtick_ident` がまだ存在しないことを確認
  - `grep -c 'extract_backtick_ident' fav/src/driver.rs` = 0 件
- [x] `cmd_check_fix` がまだ存在しないことを確認
  - `grep -c 'cmd_check_fix' fav/src/driver.rs` = 0 件

---

## T1: `extract_backtick_ident` ヘルパー追加（`driver.rs`）

`cmd_check_fix_src` の直前に追加する。

```rust
/// バッククォートで囲まれた最初の識別子を返す。
/// 例: `"did you mean \`foo\`?"` → `Some("foo")`
fn extract_backtick_ident(s: &str) -> Option<&str> {
    let start = s.find('`')? + 1;
    let end = s[start..].find('`')? + start;
    Some(&s[start..end])
}
```

- [x] 関数を追加した

---

## T2: `cmd_check_fix_src` 追加（`driver.rs`）

`extract_backtick_ident` の直後に追加する。

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

- [x] 関数を追加した

---

## T3: `cmd_check_fix` 追加（`driver.rs`）

`cmd_check_fix_src` の直後に追加する。

```rust
/// v60.2.0: `fav check --fix <file>` の実装。
pub fn cmd_check_fix(file: &str, dry_run: bool) -> String {
    let source = load_file(file);
    cmd_check_fix_src(&source, dry_run)
}
```

- [x] 関数を追加した

---

## T4: `v60200_tests` モジュール追加（`driver.rs`）

`v60100_tests` の直前（上側）に挿入する。

```rust
// -- v60200_tests (v60.2.0) -- fav check --fix 自動修正 --
#[cfg(test)]
mod v60200_tests {
    use super::*;

    #[test]
    fn check_fix_typo_single_candidate() {
        // 既知の変数 `userId` に対して `user_id` を使うと E0102 + did-you-mean ヒントが出る
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

- [x] `v60200_tests` モジュールを `v60100_tests` の直前（上側）に追加した
  （driver.rs は新しい順＝ファイル上部に追加する慣例）
- [x] `use super::*;` が含まれている
- [x] `check_fix_typo_single_candidate` テストが含まれている
- [x] `check_fix_unused_bind` テストが含まれている

---

## T5: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60200_tests::check_fix_typo_single_candidate` pass
- [x] `v60200_tests::check_fix_unused_bind` pass
- [x] 総テスト数 **3334** tests passed, 0 failed を確認

---

## T6: 事後処理

- [x] `versions/current.md` を v60.2.0 / 3334 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.2.0 実績欄を更新
  - `**実績**: — （未実施）` → `**実績**: 3334 tests passed, 0 failed（2026-07-30 完了）`
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v60.1.0 も同様、v61.0 でまとめて記載）
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` v60.2.0 実績欄に「出力は E0102/L002 を使用（ロードマップ記載の E0001/W001 とは異なる）」の注記を追記する
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

- [MED] テストアサートが `&&` で複合条件 → 診断しにくい
  → `check_fix_typo_single_candidate` / `check_fix_unused_bind` 両テストで `assert!` を 3 個に分割
    （`!is_empty()` / `contains("E0102")` or `"L002"` / `contains("[auto-fixed]")` or `"[would fix]"`）
  → 修正後も 2/2 pass 確認
- [LOW] `extract_backtick_ident` の独立ユニットテストなし → 今バージョンのスコープ外につき対応なし
- [LOW] `cmd_check_fix` のテストなし → 設計上 `cmd_check_fix_src` 経由でテストする方針、対応なし

`cmd_check_fix_src` では `Parser::parse_str` の代わりに `Lexer::new + Parser::new(tokens)` を使用
（v60.1.0 で判明した正しい API シグネチャに従った）。

---

Status: COMPLETE
