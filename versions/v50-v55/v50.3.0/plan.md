# Plan: v50.3.0 — `explain-error` と `explain` の統合

Date: 2026-07-19

---

## 実装方針

### Step 1: 現状確認

```bash
# explain-error の既存実装を確認
grep -n "Some(\"explain-error\")" fav/src/main.rs
grep -n "cmd_explain_error\b" fav/src/driver.rs | head -5

# explain コマンドの --error フラグが未実装であることを確認
grep -n "\"--error\"" fav/src/main.rs

# error_catalog の全エントリ数を確認
grep -c "code:" fav/src/error_catalog.rs
```

### Step 2: `driver.rs` — `cmd_explain_error_collect` ヘルパー追加

`cmd_explain_error` の `println!` 出力ロジックを `cmd_explain_error_collect` に抽出する。
`cmd_explain_error` 自体は `cmd_explain_error_collect` を呼び出す薄いラッパーに変更。

追加箇所: `cmd_explain_error` 関数（行 17441 付近）の直前に `cmd_explain_error_collect` を挿入。

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
```

`cmd_explain_error` を以下に置き換える:

```rust
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

### Step 3: `main.rs` — `fav explain --error` フラグ追加

`Some("explain")` アームの `compiler` チェック直後（752 行付近）に `--error` ガードを挿入:

```rust
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

挿入場所: `if args.get(2).map(|s| s.as_str()) == Some("compiler")` ブロック終了直後（755 行の `}` の直後）。
`--verbose` チェックブロック（`args.iter().any(|a| a == "--verbose")`）よりも**前**に挿入すること。
これにより `fav explain --error E0213 --verbose` 等の複合引数での `--verbose` 誤判定を防ぐ。

### Step 4: `v503000_tests` モジュール追加

`driver.rs` の `v502000_tests` モジュール直前に挿入する。

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
        let out = super::cmd_explain_error_collect("E0213");
        assert!(out.is_some(), "E0213 should be in error catalog");
        let text = out.unwrap();
        assert!(text.contains("E0213"),      "output should contain code");
        assert!(text.contains("Fix"),        "output should contain Fix section");
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

### Step 5: バージョン更新・完了

順序を守ること:
1. `fav/Cargo.toml` version → `"50.3.0"`
2. `v502000_tests::cargo_toml_version_is_50_2_0` を削除
3. `cargo test` 通過確認（3097）
4. `cargo clippy -- -D warnings` クリーン確認
5. `CHANGELOG.md` に v50.3.0 エントリ追加
6. `versions/current.md` 更新
7. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.3.0 実績を記入

---

## 注意事項

- `cmd_explain_error` の既存テスト（`explain_single_known_code` 等）は lookup を直接使っているため
  `cmd_explain_error_collect` 追加の影響を受けない。
- `main.rs` の `--error` ガードは `return;` で終わるため、既存の `--lineage` / `--types` 等の
  フローに影響しない。
- `fav explain-error` の `Some("explain-error")` アームは変更不要（alias として残す）。
  `cmd_explain_error_collect` は `cmd_explain_error` を共通化しているだけで動作は同一。
- self-hosted ファイル（`compiler.fav` / `checker.fav`）への変更なし。
