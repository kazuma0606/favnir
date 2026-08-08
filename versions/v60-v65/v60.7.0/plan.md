# v60.7.0 Plan — `fav fmt` ルール拡張

Date: 2026-07-31
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/fmt.rs` | 追加 | `FmtConfig`・`format_with_config`・`reinsert_comments` |
| `fav/src/driver.rs` | 修正 | `cmd_fmt` で `.favfmt` 読み込み + `format_with_config` 呼び出し |
| `fav/src/driver.rs` | 追加 | `v60700_tests` モジュール（テスト 2 件） |

---

## 実装ステップ

### Step 1: `fmt.rs` — `FmtConfig` 構造体追加

`format_program` 関数の直前（`// ── public API ──` セクション冒頭）に挿入する。

```rust
/// v60.7.0: フォーマット設定（.favfmt ファイルから読み込む）
pub struct FmtConfig {
    pub max_line_length: usize,
    pub indent_width: usize,
    pub preserve_comments: bool,
    pub trailing_comma: String,
}

impl Default for FmtConfig {
    fn default() -> Self {
        FmtConfig {
            max_line_length: 100,
            indent_width: 4,
            preserve_comments: true,
            trailing_comma: "always".to_string(),
        }
    }
}

impl FmtConfig {
    /// `.favfmt` TOML 文字列をパースして FmtConfig を生成する
    pub fn from_toml_str(s: &str) -> Self {
        let mut cfg = FmtConfig::default();
        for line in s.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "max_line_length" => {
                        if let Ok(n) = v.trim().parse::<usize>() {
                            cfg.max_line_length = n;
                        }
                    }
                    "indent_width" => {
                        if let Ok(n) = v.trim().parse::<usize>() {
                            cfg.indent_width = n;
                        }
                    }
                    "preserve_comments" => {
                        cfg.preserve_comments = v.trim() == "true";
                    }
                    "trailing_comma" => {
                        cfg.trailing_comma =
                            v.trim().trim_matches('"').to_string();
                    }
                    _ => {}
                }
            }
        }
        cfg
    }
}
```

### Step 2: `fmt.rs` — `format_with_config` + `reinsert_comments` 追加

`format_program` 関数の直後に追加する。

```rust
/// v60.7.0: コメント保持・設定適用付きフォーマット
pub fn format_with_config(prog: &Program, source: &str, config: &FmtConfig) -> String {
    let formatted = format_program(prog);
    if config.preserve_comments {
        reinsert_comments(source, &formatted)
    } else {
        formatted
    }
}

/// オリジナルソース中の `//` コメント行をフォーマット済みソースに再挿入する
fn reinsert_comments(original: &str, formatted: &str) -> String {
    use std::collections::{HashMap, HashSet};

    // anchor_line_trimmed → コメント行リスト のマップを構築
    let mut comment_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut pending: Vec<String> = Vec::new();

    for line in original.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            pending.push(line.to_string());
        } else {
            if !pending.is_empty() {
                comment_map
                    .entry(trimmed.to_string())
                    .or_default()
                    .extend(pending.drain(..));
            } else {
                pending.clear();
            }
        }
    }
    // ファイル末尾のコメント（次の非コメント行なし）
    let trailing: Vec<String> = pending;

    // フォーマット済みソースにコメントを挿入
    let mut result: Vec<String> = Vec::new();
    let mut inserted: HashSet<String> = HashSet::new();

    for line in formatted.lines() {
        let key = line.trim().to_string();
        if !inserted.contains(&key) {
            if let Some(comments) = comment_map.get(&key) {
                for c in comments {
                    result.push(c.clone());
                }
                inserted.insert(key.clone());
            }
        }
        result.push(line.to_string());
    }
    for c in &trailing {
        result.push(c.clone());
    }

    let mut out = result.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
```

### Step 3: `driver.rs` — `cmd_fmt` 更新

`use crate::fmt::format_program;` を
`use crate::fmt::{format_program, format_with_config, FmtConfig};` に変更。

`format_program(&program)` 呼び出し（通常フォーマットパス）の直前に `.favfmt` 読み込みを追加し、
呼び出し自体を `format_with_config(&program, &source, &config)` に置き換える。

具体的な差分:

```rust
// 旧
let formatted = format_program(&program);

// 新
let config = load_favfmt_config();
let formatted = format_with_config(&program, &source, &config);
```

`load_favfmt_config` ヘルパーを `cmd_fmt` の直前に追加:

```rust
fn load_favfmt_config() -> FmtConfig {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let favfmt_path = cwd.join(".favfmt");
    if favfmt_path.exists() {
        if let Ok(s) = std::fs::read_to_string(&favfmt_path) {
            return FmtConfig::from_toml_str(&s);
        }
    }
    FmtConfig::default()
}
```

**注意**: `source` は既に `load_file(path)` で取得済みのため追加読み込みは不要。

### Step 4: `driver.rs` — `v60700_tests` モジュール追加

`v60600_tests` モジュールの直前（上側）に挿入する。

```rust
// -- v60700_tests (v60.7.0) -- fav fmt ルール拡張 --
#[cfg(test)]
mod v60700_tests {
    use super::*;

    #[test]
    fn fmt_preserves_comments() {
        let source =
            "// pipeline comment\nstage Foo: Int -> Int = |x| { x + 1 }\n";
        let prog = crate::frontend::parser::Parser::parse_str(source, "test.fav")
            .expect("parse failed");
        let config = crate::fmt::FmtConfig {
            preserve_comments: true,
            ..crate::fmt::FmtConfig::default()
        };
        let out = crate::fmt::format_with_config(&prog, source, &config);
        assert!(
            out.contains("// pipeline comment"),
            "comment should be preserved; got: {:?}", out
        );
        assert!(out.contains("stage Foo"), "stage should still be present");
    }

    #[test]
    fn fmt_respects_favfmt_config() {
        let toml = "max_line_length = 80\nindent_width = 2\npreserve_comments = true\n";
        let config = crate::fmt::FmtConfig::from_toml_str(toml);
        assert_eq!(config.max_line_length, 80);
        assert_eq!(config.indent_width, 2);
        assert!(config.preserve_comments);
    }
}
```

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `FmtConfig` 構造体 | `fmt.rs` の `// ── public API ──` 直後（`format_program` の直前） |
| `format_with_config` / `reinsert_comments` | `format_program` 関数の直後 |
| `load_favfmt_config` | `driver.rs` の `// ── fav fmt ──` セクション、`cmd_fmt` の直前 |
| `cmd_fmt` の `use` / 呼び出し変更 | 既存コードの 2 点を Edit で修正 |
| `v60700_tests` | `driver.rs` の `v60600_tests` の直前（上側） |

---

## 注意点

- `fmt.rs` の `reinsert_comments` は `std::collections` を直接使う（`fmt.rs` は `use crate::ast::*;` のみ）
- `pending.clear()` は `pending` が空の場合に無駄だが副作用なし（コードレビュー時に `.drain(..)` で統合可）
- `format_program` は既存関数のまま変更しない（後方互換）
- テストは `crate::fmt::FmtConfig` / `crate::fmt::format_with_config` を直接参照する（`use super::*` で `driver.rs` スコープ内から到達可能）
