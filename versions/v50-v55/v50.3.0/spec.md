# Spec: v50.3.0 — `explain-error` と `explain` の統合

Date: 2026-07-19
Status: Draft

---

## 概要

`fav explain --error <code>` を正式導線として追加し、既存の `fav explain-error <code>`
を後方互換 alias として維持する。
あわせて `error_catalog.rs` の全エントリに `description`・`fix` が揃っていることを
テストで保証するカバレッジアサーションを追加する。

> **テスト件数の注記**: ロードマップ v50.3.0 は機能テスト 2 件（`explain_error_flag_works`・
> `explain_error_all_codes_have_text`）を完了条件として記載している。
> 本バージョンではこれに加えてバージョン確認テスト 1 件（`cargo_toml_version_is_50_3_0`）を
> 追加するため、`v503000_tests` モジュールは合計 3 件となる。テスト総数は 3097。

---

## 背景

現在 Favnir にはエラーコード説明の導線が 2 つ存在する。

| コマンド | 実装 | 状態 |
|---|---|---|
| `fav explain-error <code>` | `main.rs` `Some("explain-error")` → `cmd_explain_error` | 実装済み（v12.5.0 以前） |
| `fav explain --error <code>` | — | **未実装** |

`fav explain` は既に `--lineage`・`--types`・`--sla`・`--verbose` 等の多くのサブフラグを
保有しており、`--error` を追加することでコマンド体系の一貫性が向上する。
`explain-error` は後方互換として残し、ドキュメントは `fav explain --error` を推奨。

> **`description` / `fix` の充足状況**: v50.1.0 で全 94 エントリの `suggestion` が補完された。
> `description` と `fix` はそれ以前から全エントリに非空テキストが設定済みであり、
> `explain_error_all_codes_have_text` は回帰テストとして追加する（新規テキスト追加作業は不要）。

---

## 仕様

### 変更 1: `main.rs` — `fav explain --error` フラグ追加

`Some("explain")` アームの先頭（`compiler` チェックの直後）に `--error` ガードを追加。

```rust
// Some("explain") の先頭に挿入（"compiler" チェックの直後）
if args.iter().any(|a| a == "--error") {
    let mut list   = false;
    let mut format = "text";
    let mut code: Option<&str> = None;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--error"  => { i += 1; }
            "--list"   => { list = true; i += 1; }
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].as_str();
                    i += 2;
                } else {
                    eprintln!("error: --format requires a value (text|json)");
                    process::exit(1);
                }
            }
            other => { code = Some(other); i += 1; }
        }
    }
    if list {
        if format == "json" { cmd_explain_error_list_json(); }
        else                { cmd_explain_error_list(); }
    } else if let Some(c) = code {
        cmd_explain_error(c);
    } else {
        eprintln!("error: fav explain --error requires a code (e.g. E0213) or --list");
        process::exit(1);
    }
    return;
}
```

### 変更 2: `driver.rs` — `cmd_explain_error_collect` ヘルパー追加

`cmd_explain_error` が出力を `println!` に直書きしているため、テスト可能にするため
出力を `String` として返す `pub(crate) fn cmd_explain_error_collect(code: &str) -> Option<String>`
を追加し、`cmd_explain_error` はその戻り値を print する形に変更する。

```rust
pub(crate) fn cmd_explain_error_collect(code: &str) -> Option<String> {
    crate::error_catalog::lookup(code).map(|e| {
        let mut out = String::new();
        out.push_str(&format!("  Code:  {}\n", e.code));
        out.push_str(&format!("  Title: {}\n", e.title));
        out.push('\n');
        out.push_str("  Description\n");
        out.push_str(&format!("  {}\n", e.description));
        out.push('\n');
        out.push_str("  Example\n");
        for line in e.example.lines() {
            out.push_str(&format!("    {}\n", line));
        }
        out.push('\n');
        out.push_str("  Fix\n");
        for line in e.fix.lines() {
            out.push_str(&format!("    {}\n", line));
        }
        if let Some(suggestion) = e.suggestion {
            out.push('\n');
            out.push_str("  Suggestion\n");
            out.push_str(&format!("    {}\n", suggestion));
        }
        out
    })
}

pub fn cmd_explain_error(code: &str) {
    match cmd_explain_error_collect(code) {
        Some(text) => print!("{}", text),
        None => {
            eprintln!("error: unknown error code `{}`", code);
            eprintln!("run `fav explain --error --list` to see all known codes");
            std::process::exit(1);
        }
    }
}
```

### テスト仕様

`v503000_tests` モジュールを `driver.rs` の `v502000_tests` 直前に追加（3 件）。

テスト総数: 3095（ベース）− 1（`cargo_toml_version_is_50_2_0` 削除）+ 3（v503000_tests 追加）= **3097**。

```rust
// -- v503000_tests (v50.3.0) -- explain --error 統合 --
#[cfg(test)]
mod v503000_tests {
    #[test]
    fn cargo_toml_version_is_50_3_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.3.0\""),
            "Cargo.toml version should be 50.3.0");
    }

    #[test]
    fn explain_error_flag_works() {
        // cmd_explain_error_collect を使って --error ルートの出力を検証
        let out = super::cmd_explain_error_collect("E0213");
        assert!(out.is_some(), "E0213 should be in error catalog");
        let text = out.unwrap();
        assert!(text.contains("E0213"),    "output should contain code");
        assert!(text.contains("Fix"),      "output should contain Fix section");
        assert!(text.contains("Suggestion"), "output should contain Suggestion section");
    }

    #[test]
    fn explain_error_all_codes_have_text() {
        let all = crate::error_catalog::list_all();
        for e in all {
            assert!(!e.description.is_empty(),
                "{}: description must be non-empty", e.code);
            assert!(!e.fix.is_empty(),
                "{}: fix must be non-empty", e.code);
        }
    }
}
```

---

## 完了条件

- `cargo test` 3097 passed, 0 failed
- `main.rs`: `fav explain --error <code>` / `--list` / `--list --format json` が動作する
- `driver.rs`: `cmd_explain_error_collect` 追加、`cmd_explain_error` が内部委譲に変更
- `fav explain-error <code>` は後方互換として引き続き動作する
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.3.0 エントリ追加
- `versions/current.md` を v50.3.0 に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.3.0 実績を記入
