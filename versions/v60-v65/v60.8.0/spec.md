# v60.8.0 Spec — `fav doc` 強化（HTML 出力・Rune ドキュメント統合）

Date: 2026-07-31
Status: 未着手

---

## 概要

既存の `fav doc`（v9.8、`///` コメント → Markdown）に以下 3 機能を追加する。

1. **`--format html` 出力バックエンド**: `fav doc --format html --out <dir>` で静的 HTML を生成
2. **Rune ドキュメント統合**: `runes/*/rune.toml` の `description` フィールドを読み込んで Rune ドキュメントページを生成
3. **`@param` / `@returns` タグパーサー**: `/// @param <name> <desc>` / `/// @returns <desc>` を解析し型情報と統合表示

---

## 現行状態の整理

| 機能 | 現状 | 本バージョンの扱い |
|---|---|---|
| `fav doc`（Markdown 生成） | 実装済み（v9.8、`cmd_doc`） | 変更なし |
| `fav doc --format site`（HTML サイト生成） | 実装済み（v21.7.0、`cmd_doc_site`） | `--format html` をエイリアスとして追加 |
| `fav doc --builtins` | 実装済み（v12.7.0） | 変更なし |
| `--format html` | 未実装（`match` のデフォルトで Markdown へフォールバック） | **追加**（`cmd_doc_site` へのエイリアス） |
| Rune toml `description` 読み込み | 未実装 | **ヘルパー関数追加のみ**（実際の `runes/*/rune.toml` スキャンは将来バージョンへ延期）|
| `@param` / `@returns` タグパーサー | 未実装 | **`parse_doc_tags` 関数追加のみ**（HTML 出力への統合は将来バージョンへ延期）|

## スコープ整理（ロードマップとの差分）

ロードマップ v60.8.0 の一部機能は本バージョンで実装を延期する。

| ロードマップ項目 | 本バージョンの扱い |
|---|---|
| `--format html` CLI 対応 | `cmd_doc_site` へのエイリアスとして追加（CLI 動線は既存 `cmd_doc_site` を再利用） |
| `cmd_doc_html_str` | テスト専用ヘルパー（CLI から直接は呼ばれない）。`<!DOCTYPE html>` を含む簡易ラッパー |
| `runes/*/rune.toml` スキャン・Rune ページ自動生成 | v60.9 以降へ延期。本バージョンは `parse_rune_toml_description` ヘルパーを追加するのみ |
| `@param` / `@returns` の HTML への統合表示 | v60.9 以降へ延期。本バージョンは `parse_doc_tags` 関数を追加するのみ |

---

## 実装方針

### 1. `--format html` の追加（`main.rs`）

`main.rs` の `Some("doc")` ディスパッチ内の `match format.as_str()` に `"html"` アームを追加する。

```rust
match format.as_str() {
    "site" | "html" => cmd_doc_site(&path, &out_dir),
    _ => cmd_doc(&path, &out_dir),
}
```

### 2. `cmd_doc_html_str` 追加（`driver.rs`）

テスト専用ヘルパー。CLI の `--format html` は `cmd_doc_site` を呼ぶため、`cmd_doc_html_str` は CLI から直接呼ばれない。
`doc_source_str` で Markdown を生成し、`<!DOCTYPE html>` を含む簡易 HTML ラッパーを付与する。

**注意**: `doc_source_str` は `compiler.fav` の VM 実行に依存するため、テスト実行前に `cargo build` が完了していることが前提。

```rust
/// v60.8.0: HTML 形式のドキュメントを String として返す（テスト容易性）
pub fn cmd_doc_html_str(path: &str) -> Result<String, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path, e))?;
    let md = crate::compiler_fav_runner::doc_source_str(&src)
        .map_err(|e| format!("doc failed for '{}': {}", path, e))?;
    let escaped = md.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    Ok(format!(
        "<!DOCTYPE html><html><head><title>Favnir Docs</title></head><body><pre>{}</pre></body></html>",
        escaped
    ))
}
```

### 3. `parse_rune_toml_description` + `cmd_doc_rune_description_str` 追加（`driver.rs`）

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

### 4. `parse_doc_tags` 追加（`driver.rs`）

`/// @param <name> <desc>` / `/// @returns <desc>` をパースして Markdown に変換するヘルパー。
将来的に `cmd_doc_html_str` の Markdown 生成前パスに組み込む（本バージョンでは関数追加のみ）。

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

---

## テスト仕様

### `doc_html_output_generated`

```
1. tempdir に pipeline.fav を作成:
     /// Stage that doubles a value
     stage Double: Int -> Int = |x| { x + x }
2. cmd_doc_html_str(path) を呼び出す
3. 期待:
   - Ok を返す
   - 出力に "<!DOCTYPE html>" が含まれる
   - 出力に "Double" が含まれる
```

### `doc_rune_description_included`

```
1. rune.toml 文字列を用意:
     [rune]
     name = "postgres"
     description = "PostgreSQL integration for Favnir"
2. cmd_doc_rune_description_str(content) を呼び出す
3. 期待:
   - 出力に "PostgreSQL integration for Favnir" が含まれる
```

---

## 完了条件

- `doc_html_output_generated` pass
- `doc_rune_description_included` pass
- 総テスト数: **3346** tests passed, 0 failed（ベース 3344 + 2）
- `cargo build` でコンパイルエラーなし
