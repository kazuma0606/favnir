# v60.8.0 Tasks — `fav doc` 強化（HTML 出力・Rune ドキュメント統合）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo build` でコンパイルエラーがないことを確認（`doc_source_str` の VM アーティファクトが存在するために必須）
- [x] `cargo test` でベースラインが 3344 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60800_tests` がまだ存在しないことを確認
  - `grep -c 'v60800_tests' fav/src/driver.rs` = 0 件
- [x] `v60700_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v60700_tests' fav/src/driver.rs` ≥ 1 件
- [x] `cmd_doc_html_str` がまだ存在しないことを確認
  - `grep -c 'cmd_doc_html_str' fav/src/driver.rs` = 0 件
- [x] `cmd_doc_rune_description_str` がまだ存在しないことを確認
  - `grep -c 'cmd_doc_rune_description_str' fav/src/driver.rs` = 0 件
- [x] `--format html` が現行の doc match に存在しないことを確認
  - `grep -c '"site" | "html"' fav/src/main.rs` = 0 件

---

## T1: `main.rs` — `--format html` アーム追加

`Some("doc")` ブロック内の `match format.as_str()` を修正する。

```rust
// 旧
"site" => cmd_doc_site(&path, &out_dir),

// 新
"site" | "html" => cmd_doc_site(&path, &out_dir),
```

- [x] `"site" | "html"` アームが追加された
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T2: `driver.rs` — `cmd_doc_html_str` 追加

`// ── fav doc ──` セクション内、`pub fn cmd_doc(...)` の直前に挿入する。

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

- [x] `cmd_doc_html_str` を `cmd_doc` の直前に追加した
- [x] `pub fn` で宣言されている
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T3: `driver.rs` — `parse_rune_toml_description` + `cmd_doc_rune_description_str` 追加

`cmd_doc_html_str` の直後（`cmd_doc` の直前）に追加する。

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

- [x] `parse_rune_toml_description` を追加した（非公開）
- [x] `cmd_doc_rune_description_str` を追加した（`pub fn`）
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T4: `driver.rs` — `parse_doc_tags` 追加

`cmd_doc_rune_description_str` の直後（`cmd_doc` の直前）に追加する。

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

- [x] `parse_doc_tags` を追加した（`pub fn`）
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T5: `driver.rs` — `v60800_tests` モジュール追加

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

- [x] `v60800_tests` モジュールを `v60700_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `doc_html_output_generated` テストが含まれている
- [x] `doc_rune_description_included` テストが含まれている

---

## T6: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60800_tests::doc_html_output_generated` pass
- [x] `v60800_tests::doc_rune_description_included` pass
- [x] 総テスト数 **3347** tests passed, 0 failed を確認（XSS テスト追加で +1）

---

## T7: 事後処理

- [x] `versions/current.md` を v60.8.0 / 3346 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.8.0 実績欄を更新
  - 実績欄に以下を明記: 「`parse_doc_tags` は関数追加のみ。`@param`/`@returns` の HTML 統合表示は v60.9 以降。`runes/*/rune.toml` スキャンも v60.9 以降。」
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v61.0 でまとめて記載）
  - v61.0 記載範囲: v60.1〜v60.9 全機能
- [x] `parse_doc_tags` の単体テストは v60.9.0 の tasks.md に持ち越す（本バージョンではカバレッジなし）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

### コードレビュー指摘と対応

- **[HIGH] XSS — `cmd_doc_rune_description_str`**: `html_escape` を使用して `desc` をエスケープするように修正。XSS テスト `doc_rune_description_xss_escaped` を追加（テスト数 +1 → 3347）。
- **[HIGH] `cmd_doc_html_str` 不完全エスケープ**: カスタム replace チェーンを `html_escape()` に統一（`"` もエスケープ対象に）。
- **[MED] `parse_rune_toml_description` インラインコメント**: `trim_matches('"')` → クォートペアを検索して内部文字列を抽出するよう修正。
- **[MED] `parse_doc_tags` `//` 誤処理**: `trim_start_matches('/')` → `strip_prefix("///")` でフィルタし、`///` 以外は `continue` するよう修正。
- **[MED] `cmd_doc_html_str` パストラバーサル**: `Path::components()` で `ParentDir` を検出して `Err` を返すガードを追加。
- **[LOW] WASM cfg ガード**: `main.rs` はバイナリターゲット専用で WASM32 ビルドでは使われないため対応不要と判断。

### 実装中の問題と対応

- **`doc_source_str` が `stage` を非対応**: テスト入力を `stage Double` から `public fn add` に変更して解決。`doc_source_str` は `public fn` / `type` のみドキュメント生成する（v9.8 の設計）。

---

Status: COMPLETE
