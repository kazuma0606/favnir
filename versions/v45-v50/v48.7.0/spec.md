# Spec: v48.7.0 — rune.toml 標準化

## 概要

全公式 rune の `rune.toml` を統一フォーマットに規定し、
`toml.rs` に `validate_rune_toml` ヘルパーを追加する。
必須フィールド（`[rune]` セクション + `name` / `version` / `entry`）の存在チェックと、
非標準セクション（`[connection]`）の除去確認を行う。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `validate_rune_toml(content: &str) -> Vec<String>` 追加 |
| `fav/src/driver.rs` | `v487000_tests` 追加（2テスト） |
| `fav/Cargo.toml` | version → `"48.7.0"` |
| `CHANGELOG.md` | v48.7.0 エントリ追加 |

---

## 実装詳細

### `validate_rune_toml` — rune.toml 検証ヘルパー

`toml.rs` のファイル末尾（既存 `parse_kv` の後）に追加する。

**標準フォーマット**:
- `[rune]` セクションが必須
- `name`・`version`・`entry` フィールドが必須（`[rune]` セクション内）
- `[connection]` セクションは非標準（存在するとエラー）

**返り値**: `Vec<String>` — バリデーションエラーのリスト。空なら合格。

```rust
/// rune.toml の標準フォーマット検証（v48.7.0）。
/// 必須: `[rune]` セクション + `name` / `version` / `entry` フィールド
/// 非標準: `[connection]` セクションが存在するとエラー
/// 返り値: バリデーションエラーの Vec（空なら合格）
pub fn validate_rune_toml(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut has_rune_section = false;
    let mut has_name = false;
    let mut has_version = false;
    let mut has_entry = false;
    let mut has_connection_section = false;
    let mut section = "";
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[rune]" {
            has_rune_section = true;
            section = "rune";
            continue;
        }
        if trimmed == "[connection]" {
            has_connection_section = true;
            section = "connection";
            continue;
        }
        if trimmed.starts_with('[') {
            section = "other";
            continue;
        }
        if section == "rune" {
            if let Some((k, _)) = parse_kv(trimmed) {
                match k {
                    "name" => has_name = true,
                    "version" => has_version = true,
                    "entry" => has_entry = true,
                    _ => {}
                }
            }
        }
    }
    if !has_rune_section {
        errors.push("missing [rune] section".to_string());
    }
    if !has_name {
        errors.push("missing required field: name".to_string());
    }
    if !has_version {
        errors.push("missing required field: version".to_string());
    }
    if !has_entry {
        errors.push("missing required field: entry".to_string());
    }
    if has_connection_section {
        errors.push("[connection] section is non-standard; remove it".to_string());
    }
    errors
}
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `rune_toml_standard_format` | `[rune]` + `name`/`version`/`entry`/`description` を持つ valid な rune.toml で `validate_rune_toml` が空 Vec を返す |
| `rune_toml_no_connection_section` | `[connection]` セクションを持つ rune.toml で `validate_rune_toml` がエラーを返し、エラーに `"connection"` が含まれる |

```rust
#[test]
fn rune_toml_standard_format() {
    use crate::toml::validate_rune_toml;
    let content = "[rune]\nname = \"kafka\"\nversion = \"2.1.0\"\nentry = \"kafka.fav\"\ndescription = \"Kafka rune\"\n";
    let errors = validate_rune_toml(content);
    assert!(errors.is_empty(), "standard rune.toml must pass validation: {:?}", errors);
}

#[test]
fn rune_toml_no_connection_section() {
    use crate::toml::validate_rune_toml;
    let content = "[rune]\nname = \"kafka\"\nversion = \"2.1.0\"\nentry = \"kafka.fav\"\n[connection]\nurl = \"localhost:9092\"\n";
    let errors = validate_rune_toml(content);
    assert!(!errors.is_empty(), "rune.toml with [connection] must fail validation");
    assert!(
        errors.iter().any(|e| e.contains("connection")),
        "errors must mention 'connection': {:?}", errors
    );
}
```

テスト数: 3058 → **3060**（+2）

---

## 注意事項

- `validate_rune_toml` は `toml.rs` 内の `parse_kv`（private 関数）を呼び出す — 同一ファイル内なのでアクセス可能。
- `description` フィールドは**オプション**（必須チェックの対象外）。rune の説明は空文字列でも動作するため、必須化はしない。バリデーターは `description` の有無を無視する。
- `validate_rune_toml` は `pub` で公開すること（`driver.rs` テストから `crate::toml::validate_rune_toml` として参照するため）。
- 全公式 rune の `rune.toml` ファイル更新は本バージョンのスコープ外（v48.4.0 で `install_rune_stubs` が生成するスタブは既に `entry` フィールドを含む標準フォーマット済み）。
- `site/` MDX 更新は不要（v48.9.0 のドキュメント整備スプリントで対応）。
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）。

---

## 完了条件

- `cargo test` 3060 passed, 0 failed（3058 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.7.0"`
- `CHANGELOG.md` に v48.7.0 エントリ追加
- `error_catalog.rs` の `ERROR_CATALOG` に変更なし（本バージョンはエラーコード追加なし）
- `versions/current.md` を v48.7.0（3060 tests）に更新、進行中バージョンを `v48.8.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
