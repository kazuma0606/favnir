# Spec: v48.8.0 — `fav rune` コマンド群（純粋ヘルパー関数追加）

## 概要

`fav rune list` / `fav rune info` / `fav rune remove` の CLI ルーティングは
`main.rs` `Some("rune")` → `rune_cmd.rs` `cmd_rune` として**既に実装済み**（`rune_modules/` 対象）。

v48.8.0 では、v48.4.0 系の `runes/` ディレクトリ（`install_rune_stubs` が作成）に対応した
**テスト可能な純粋ヘルパー関数**を `driver.rs` に追加する。

- `list_installed_runes(root: &Path) -> Vec<String>` — `runes/` 以下のサブディレクトリを列挙
- `get_rune_version(root: &Path, name: &str) -> Option<String>` — `runes/<name>/rune.toml` から version を取得

`main.rs` / `rune_cmd.rs` への変更は行わない。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `list_installed_runes` / `get_rune_version` 追加 + `v488000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.8.0"` |
| `CHANGELOG.md` | v48.8.0 エントリ追加 |

---

## 実装詳細

### `list_installed_runes`

```rust
/// v48.4.0 系の `runes/` ディレクトリ内のインストール済み rune 名一覧を返す。
/// ディレクトリ名のみを対象とし、ソート済みの Vec を返す。
pub fn list_installed_runes(root: &std::path::Path) -> Vec<String> {
    let runes_dir = root.join("runes");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&runes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names
}
```

### `get_rune_version`

`[rune]` セクション内の `version` フィールドを `split_once('=')` で取得する。
`strip_prefix("version")` は `version_tag` 等に誤マッチするため使用しない。

```rust
/// `runes/<name>/rune.toml` の `[rune]` セクションから version を返す。
/// ファイル不在またはフィールド不在の場合は None。
pub fn get_rune_version(root: &std::path::Path, name: &str) -> Option<String> {
    let rune_toml = root.join("runes").join(name).join("rune.toml");
    let content = std::fs::read_to_string(&rune_toml).ok()?;
    let mut in_rune_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[rune]" {
            in_rune_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_rune_section = false;
            continue;
        }
        if in_rune_section {
            if let Some((k, v)) = trimmed.split_once('=') {
                if k.trim() == "version" {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `fav_rune_list_shows_installed` | tempdir に `runes/kafka/` と `runes/postgres/` を作成し `list_installed_runes` が `["kafka", "postgres"]` を返すことを確認 |
| `fav_rune_info_shows_version` | tempdir に `runes/kafka/rune.toml`（`version = "2.1.0"`）を作成し `get_rune_version` が `Some("2.1.0")` を返すことを確認 |

```rust
#[test]
fn fav_rune_list_shows_installed() {
    use crate::driver::list_installed_runes;
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("runes").join("kafka")).unwrap();
    std::fs::create_dir_all(dir.path().join("runes").join("postgres")).unwrap();
    // list_installed_runes はソート済みを返すため、テスト側でのソートは不要
    let names = list_installed_runes(dir.path());
    assert_eq!(names, vec!["kafka".to_string(), "postgres".to_string()]);
}

#[test]
fn fav_rune_info_shows_version() {
    use crate::driver::get_rune_version;
    let dir = tempfile::tempdir().expect("tempdir");
    let kafka_dir = dir.path().join("runes").join("kafka");
    std::fs::create_dir_all(&kafka_dir).unwrap();
    std::fs::write(
        kafka_dir.join("rune.toml"),
        "[rune]\nname = \"kafka\"\nversion = \"2.1.0\"\nentry = \"kafka.fav\"\n",
    ).unwrap();
    let ver = get_rune_version(dir.path(), "kafka");
    assert_eq!(ver, Some("2.1.0".to_string()));
}
```

テスト数: 3061 → **3063**（+2）

---

## 注意事項

- **既存 `rune_cmd.rs` との共存**: `rune_cmd.rs` の `cmd_rune_list` / `cmd_rune_info` / `cmd_rune_uninstall` は `rune_modules/` ディレクトリを参照する旧系統。本バージョンで追加する `list_installed_runes` / `get_rune_version` は v48.4.0 系の `runes/` ディレクトリを参照する新系統。`main.rs` / `rune_cmd.rs` への変更は行わない。
- **`fav rune info` の「関数一覧」表示**: ロードマップの `fav rune info kafka` の説明に「関数一覧」とあるが、本バージョンでは version 取得のみ実装する。関数一覧は v48.8.0 のスコープ外。
- `get_rune_version` で `strip_prefix("version")` を使わず `split_once('=')` を使う理由: `strip_prefix("version")` は `version_tag = ...` 等の接頭辞一致に誤マッチするため。
- `list_installed_runes` は内部で `names.sort()` を呼ぶ。テスト側では重複ソートしないこと（実装のソート保証をテストが検証できなくなるため）。
- `site/` MDX 更新は不要（v48.9.0 のドキュメント整備スプリントで対応）。
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）。

---

## 完了条件

- `cargo test` 3063 passed, 0 failed（3061 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.8.0"`
- `CHANGELOG.md` に v48.8.0 エントリ追加
- `versions/current.md` を v48.8.0（3063 tests）に更新、進行中バージョンを `v48.9.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
