# Spec: v48.4.0 — `fav install` コマンド（`[runes]` 対応）

## 概要

`fav.toml [runes]` テーブルを読んで `runes/<name>/` ディレクトリをローカルに作成する MVP 実装。
`driver.rs` に `install_rune_stubs`（テスト可能な内部関数）と `cmd_install_runes`（CLI エントリ）を追加する。
既存の `cmd_install`（`[dependencies]` 専用）は変更しない。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `install_rune_stubs` 関数追加（ディレクトリ作成ロジック）・`cmd_install_runes` 関数追加・`v484000_tests` 追加（2テスト） |
| `fav/src/main.rs` | `Some("install-rune")` アーム追加（`cmd_install_runes` へのディスパッチ） |
| `fav/Cargo.toml` | version → `"48.4.0"` |
| `CHANGELOG.md` | v48.4.0 エントリ追加 |

---

## 実装詳細

### `install_rune_stubs` — テスト可能な純粋関数

```rust
/// `[runes]` テーブルから runes/<name>/ ディレクトリを作成する（MVP スタブ）。
/// `pkg_name` が Some の場合は対象 1 件のみ、None の場合は全件処理。
/// 返り値: インストールしたパッケージ名の Vec。
pub fn install_rune_stubs(
    pkg_name: Option<&str>,
    root: &std::path::Path,
    runes: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let runes_dir = root.join("runes");
    let mut installed = Vec::new();
    for (name, version) in runes {
        if let Some(target) = pkg_name {
            if name != target {
                continue;
            }
        }
        let dest = runes_dir.join(name);
        if std::fs::create_dir_all(&dest).is_ok() {
            // MVP: rune.toml スタブを作成
            let rune_toml = dest.join("rune.toml");
            if !rune_toml.exists() {
                let _ = std::fs::write(
                    &rune_toml,
                    format!("[rune]\nname = \"{name}\"\nversion = \"{version}\"\n"),
                );
            }
            installed.push(name.clone());
        }
    }
    installed
}
```

### `cmd_install_runes` — CLI エントリポイント

```rust
pub fn cmd_install_runes(pkg_name: Option<&str>) {
    use crate::toml::FavToml;
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        std::process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        std::process::exit(1);
    });
    if toml.runes.is_empty() {
        println!("[install-rune] No runes declared in fav.toml [runes]");
        return;
    }
    let installed = install_rune_stubs(pkg_name, &root, &toml.runes);
    if installed.is_empty() {
        if let Some(n) = pkg_name {
            eprintln!("error: rune '{}' not found in fav.toml [runes]", n);
            std::process::exit(1);
        }
        println!("[install-rune] No runes installed.");
    } else {
        for name in &installed {
            println!("[install-rune] {} → runes/{}/", name, name);
        }
        println!("[install-rune] {} rune(s) installed.", installed.len());
    }
}
```

### `main.rs` — `"install-rune"` アーム追加

`"install"` アームの直後に挿入:

```rust
Some("install-rune") => {
    let pkg_name = args.get(2).map(|s| s.as_str());
    cmd_install_runes(pkg_name);
}
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `fav_install_creates_rune_dir` | `[runes] kafka = "2.1.0"` を持つ HashMap + tempdir で `install_rune_stubs(Some("kafka"), ...)` → `runes/kafka/` が作成され `rune.toml` が存在する |
| `fav_install_all_from_toml` | `parse_fav_toml_pub` で `[runes] kafka / postgres` をパース → `install_rune_stubs(None, ...)` → 両ディレクトリが存在し返り値 len == 2 |

```rust
#[test]
fn fav_install_creates_rune_dir() {
    use crate::driver::install_rune_stubs;
    use tempfile::tempdir;
    let dir = tempdir().expect("tempdir");
    let mut runes = std::collections::HashMap::new();
    runes.insert("kafka".to_string(), "2.1.0".to_string());
    let installed = install_rune_stubs(Some("kafka"), dir.path(), &runes);
    assert_eq!(installed, vec!["kafka".to_string()]);
    assert!(dir.path().join("runes").join("kafka").exists(),
        "runes/kafka/ must be created");
    assert!(dir.path().join("runes").join("kafka").join("rune.toml").exists(),
        "rune.toml stub must be created");
}

#[test]
fn fav_install_all_from_toml() {
    use crate::driver::install_rune_stubs;
    use crate::toml::parse_fav_toml_pub;
    use tempfile::tempdir;
    let dir = tempdir().expect("tempdir");
    let content = "[project]\nname = \"myapp\"\nversion = \"0.1.0\"\n[runes]\nkafka = \"2.1.0\"\npostgres = \"1.0.0\"\n";
    let toml = parse_fav_toml_pub(content);
    let installed = install_rune_stubs(None, dir.path(), &toml.runes);
    assert_eq!(installed.len(), 2, "both runes must be installed");
    assert!(dir.path().join("runes").join("kafka").exists());
    assert!(dir.path().join("runes").join("postgres").exists());
}
```

テスト数: 3051 → **3053**（+2）

---

## 注意事項

- **ロードマップとのコマンド名乖離について**: ロードマップ v48.4.0 セクションの旧記述（`cmd_install` 追加・`"install"` アーム追加）は誤記だった。正しくは `cmd_install_runes` / `"install-rune"` アーム。ロードマップは修正済み。
- 既存の `cmd_install`（`[dependencies]` 専用）は**変更しない**。
- `install_rune_stubs` は `pub` で公開し、driver.rs テストから直接呼べるようにする。
- `main.rs` の import に `cmd_install_runes` を追加すること（既存の `cmd_install` と並べて追記）。
- MVP のため実際のダウンロードは行わない。`runes/<name>/` ディレクトリと最小 `rune.toml` を作成するのみ。
- `fav_install_creates_rune_dir` の `installed` ベクターの要素順は HashMap の反復順に依存するため、`fav_install_all_from_toml` テストでは `contains` ではなく `len == 2` で件数のみ確認する。

---

## 完了条件

- `cargo test` 3053 passed, 0 failed（3051 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.4.0"`
- `CHANGELOG.md` に v48.4.0 エントリ追加
- `versions/current.md` を v48.4.0（3053 tests）に更新、進行中バージョンを `v48.5.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T2 全 `[x]`）
