# v60.8.0 Plan — `fav doc` 強化

Date: 2026-07-31
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/main.rs` | 修正 | `--format html` アームを `match format.as_str()` に追加 |
| `fav/src/driver.rs` | 追加 | `cmd_doc_html_str` / `parse_rune_toml_description` / `cmd_doc_rune_description_str` / `parse_doc_tags` |
| `fav/src/driver.rs` | 追加 | `v60800_tests` モジュール（テスト 2 件） |

---

## 実装ステップ

### Step 1: `main.rs` — `--format html` アーム追加

`Some("doc")` ブロック内の `match format.as_str()` を以下に更新する。

```rust
// 旧
match format.as_str() {
    "site" => cmd_doc_site(&path, &out_dir),
    _ => cmd_doc(&path, &out_dir),
}

// 新
match format.as_str() {
    "site" | "html" => cmd_doc_site(&path, &out_dir),
    _ => cmd_doc(&path, &out_dir),
}
```

### Step 2: `driver.rs` — 4 関数追加

挿入位置: `// ── fav doc ──` セクション内、`pub fn cmd_doc(...)` の**直前**に追加する。

#### 2-A: `cmd_doc_html_str`

```rust
/// v60.8.0: HTML 形式のドキュメントを String として返す（テスト容易性）
pub fn cmd_doc_html_str(path: &str) -> Result<String, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path, e))?;
    let md = crate::compiler_fav_runner::doc_source_str(&src)
        .map_err(|e| format!("doc failed for '{}': {}", path, e))?;
    let escaped = md
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    Ok(format!(
        "<!DOCTYPE html><html><head><title>Favnir Docs</title></head><body><pre>{}</pre></body></html>",
        escaped
    ))
}
```

#### 2-B: `parse_rune_toml_description` + `cmd_doc_rune_description_str`

```rust
/// v60.8.0: rune.toml の `description = "..."` を抽出する
fn parse_rune_toml_description(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("description") {
            if let Some(val) = rest.trim_start().strip_prefix('=') {
                return val.trim().trim_matches('"').to_string();
            }
        }
    }
    String::new()
}

/// v60.8.0: rune.toml の description を HTML スニペットとして返す
pub fn cmd_doc_rune_description_str(rune_toml_content: &str) -> String {
    let desc = parse_rune_toml_description(rune_toml_content);
    if desc.is_empty() {
        String::new()
    } else {
        format!("<p class=\"rune-description\">{}</p>", desc)
    }
}
```

#### 2-C: `parse_doc_tags`

```rust
/// v60.8.0: `/// @param name desc` / `/// @returns desc` タグを解析して Markdown に変換する
pub fn parse_doc_tags(doc_comment: &str) -> String {
    let mut params: Vec<(String, String)> = Vec::new();
    let mut returns: Option<String> = None;
    let mut body_lines: Vec<String> = Vec::new();

    for line in doc_comment.lines() {
        let t = line.trim().trim_start_matches('/').trim();
        if let Some(rest) = t.strip_prefix("@param ") {
            let mut parts = rest.splitn(2, ' ');
            let name = parts.next().unwrap_or("").to_string();
            let desc = parts.next().unwrap_or("").to_string();
            params.push((name, desc));
        } else if let Some(rest) = t.strip_prefix("@returns ") {
            returns = Some(rest.to_string());
        } else {
            body_lines.push(t.to_string());
        }
    }

    let mut out = body_lines.join("\n");
    if !params.is_empty() {
        out.push_str("\n\n**Parameters**\n");
        for (name, desc) in &params {
            out.push_str(&format!("- `{}`: {}\n", name, desc));
        }
    }
    if let Some(ret) = &returns {
        out.push_str(&format!("\n**Returns**: {}\n", ret));
    }
    out
}
```

### Step 3: `driver.rs` — `v60800_tests` モジュール追加

`v60700_tests` モジュールの直前（上側）に挿入する。

```rust
// -- v60800_tests (v60.8.0) -- fav doc 強化 --
#[cfg(test)]
mod v60800_tests {
    use super::*;

    #[test]
    fn doc_html_output_generated() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("pipeline.fav");
        std::fs::write(
            &path,
            "/// Stage that doubles a value\nstage Double: Int -> Int = |x| { x + x }\n",
        )
        .expect("write pipeline.fav");
        let html = cmd_doc_html_str(path.to_str().unwrap())
            .expect("cmd_doc_html_str failed");
        assert!(
            html.contains("<!DOCTYPE html>"),
            "output should be HTML; got: {:?}", html
        );
        assert!(
            html.contains("Double"),
            "output should contain stage name; got: {:?}", html
        );
    }

    #[test]
    fn doc_rune_description_included() {
        let content = "[rune]\nname = \"postgres\"\ndescription = \"PostgreSQL integration for Favnir\"\n";
        let html = cmd_doc_rune_description_str(content);
        assert!(
            html.contains("PostgreSQL integration for Favnir"),
            "description should appear in output; got: {:?}", html
        );
    }
}
```

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `"html"` アーム | `main.rs` の `match format.as_str()` 内 `"site"` の隣 |
| `cmd_doc_html_str` / `parse_rune_toml_description` / `cmd_doc_rune_description_str` / `parse_doc_tags` | `driver.rs` の `// ── fav doc ──` セクション、`pub fn cmd_doc` の直前 |
| `v60800_tests` | `driver.rs` の `v60700_tests` の直前（上側） |

---

## 注意点

- `cmd_doc_html_str` は `crate::compiler_fav_runner::doc_source_str` を使用する。既存実装のため追加インポートは不要。ただし `doc_source_str` は `compiler.fav` の VM 実行に依存するため、`cargo test` の前に `cargo build` が完了している必要がある（T0 に確認項目あり）。
- `cmd_doc_html_str` は **テスト専用ヘルパー**。CLI の `--format html` は `cmd_doc_site` を呼ぶため、CLI 動線に `cmd_doc_html_str` は現れない。
- `parse_doc_tags` は本バージョンでは単独テストを持たない（v60.9 以降でテストを追加する）。`pub fn` で宣言するため `dead_code` 警告は発生しない（将来 `pub(crate)` に変更しても可）。
- `cmd_doc_rune_description_str` は `pub fn` で宣言する（テストから `super::*` で参照するため）。
- `parse_rune_toml_description` は非公開ヘルパーで十分（テストは `cmd_doc_rune_description_str` を通して間接検証）。
- `main.rs` の `use driver::{...}` 行に `cmd_doc_html_str` / `cmd_doc_rune_description_str` / `parse_doc_tags` の追加は不要（CLI ディスパッチでは `cmd_doc_site` を呼ぶため）。
- `runes/*/rune.toml` の実際のスキャン・Rune ページ生成は v60.9 以降へ延期。本バージョンは `parse_rune_toml_description` ヘルパーのみ追加する。
