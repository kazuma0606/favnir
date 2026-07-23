# Plan: v48.4.0 — `fav install` コマンド（`[runes]` 対応）

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `install_rune_stubs` 関数追加・`cmd_install_runes` 関数追加・`v484000_tests` 追加 |
| `fav/src/main.rs` | `cmd_install_runes` を import 行に追加・`Some("install-rune")` アーム追加 |
| `fav/Cargo.toml` | version → `"48.4.0"` |
| `CHANGELOG.md` | v48.4.0 エントリ追加 |
| `versions/current.md` | v48.4.0 に更新、進行中 v48.5.0 |
| `versions/v45-v50/v48.4.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### Step 1: `driver.rs` — `install_rune_stubs` 関数追加

`cmd_install` 関数の直後に挿入する。

```rust
/// `[runes]` テーブルから runes/<name>/ ディレクトリを作成する（v48.4.0 MVP スタブ）。
/// `pkg_name` が Some の場合は対象 1 件のみ、None の場合は全件処理。
/// 返り値: 作成に成功したパッケージ名の Vec。
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

### Step 2: `driver.rs` — `cmd_install_runes` 関数追加

`install_rune_stubs` の直後に追加。

```rust
/// `fav install-rune [<name>]` のエントリポイント（v48.4.0）。
/// fav.toml の [runes] テーブルを読み、runes/<name>/ にスタブを作成する。
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

### Step 3: `main.rs` — `cmd_install_runes` を import 行に追加

現在の import 行（行 93 付近）に `cmd_install_runes` と `install_rune_stubs` を追加する。

**変更前（該当部分抜粋）:**
```rust
    cmd_infer, cmd_infer_delta, cmd_infer_iceberg, cmd_infer_postgres, cmd_infer_proto, cmd_infer_snowflake, cmd_install, cmd_lint,
```

**変更後:**
```rust
    cmd_infer, cmd_infer_delta, cmd_infer_iceberg, cmd_infer_postgres, cmd_infer_proto, cmd_infer_snowflake, cmd_install, cmd_install_runes, cmd_lint,
```

### Step 4: `main.rs` — `Some("install-rune")` アームを追加

`Some("install")` アームの直後に挿入:

```rust
        Some("install-rune") => {
            let pkg_name = args.get(2).map(|s| s.as_str());
            cmd_install_runes(pkg_name);
        }
```

### Step 5: `driver.rs` — `v484000_tests` 追加

挿入位置: `v483000_tests` の直前。

```rust
// -- v484000_tests (v48.4.0) -- fav install-rune コマンド --
#[cfg(test)]
mod v484000_tests {
    use crate::driver::install_rune_stubs;
    use crate::toml::parse_fav_toml_pub;

    #[test]
    fn fav_install_creates_rune_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
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
        let dir = tempfile::tempdir().expect("tempdir");
        let content = "[project]\nname = \"myapp\"\nversion = \"0.1.0\"\n[runes]\nkafka = \"2.1.0\"\npostgres = \"1.0.0\"\n";
        let toml = parse_fav_toml_pub(content);
        let installed = install_rune_stubs(None, dir.path(), &toml.runes);
        assert_eq!(installed.len(), 2, "both runes must be installed");
        assert!(dir.path().join("runes").join("kafka").exists());
        assert!(dir.path().join("runes").join("postgres").exists());
    }
}
```

### Step 6: `Cargo.toml` version → `"48.4.0"`

### Step 7: `CHANGELOG.md` 更新

```markdown
## [v48.4.0] — 2026-07-18 — `fav install` コマンド（`[runes]` 対応）

### Added
- `driver.rs`: `install_rune_stubs` 関数追加（`runes/<name>/` ディレクトリ作成 MVP）
- `driver.rs`: `cmd_install_runes` 関数追加（`fav install-rune` CLI エントリ）
- `main.rs`: `Some("install-rune")` アーム追加
- `driver.rs`: `v484000_tests` 追加（`fav_install_creates_rune_dir` / `fav_install_all_from_toml` 2テスト）

### Changed
- `Cargo.toml` version: `48.3.0` → `48.4.0`
```

---

## 実装順序

1. `driver.rs` — `install_rune_stubs` 関数追加（`cmd_install` の直後）
2. `driver.rs` — `cmd_install_runes` 関数追加（`install_rune_stubs` の直後）
3. `main.rs` — import 行に `cmd_install_runes` 追加
4. `main.rs` — `Some("install-rune")` アーム追加（`Some("install")` の直後）
5. `driver.rs` — `v484000_tests` を `v483000_tests` 直前に追加
6. `Cargo.toml` version → `"48.4.0"`
7. `CHANGELOG.md` v48.4.0 エントリ追加
8. `cargo test` で 3053 passed, 0 failed を確認
9. `cargo clippy -- -D warnings` クリーン確認
10. `versions/current.md` 更新（v48.4.0、次 v48.5.0）
11. `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.4.0 完了条件テスト数（3053）を実績として記入
12. `tasks.md` COMPLETE に更新

---

## 注意事項

- 既存の `cmd_install`（`[dependencies]` 専用）は**変更しない**。`install_rune_stubs` は別関数として追加する。
- `install_rune_stubs` の `pub` 修飾子を忘れないこと（テストから `crate::driver::install_rune_stubs` でアクセスするため）。
- `fav_install_all_from_toml` は `HashMap` の反復順が非決定的なため、`installed.len() == 2` で件数のみ検証し順序を問わない。
- `tempfile::tempdir()` は既存の `[dev-dependencies]` に登録済み。追加不要。
- `main.rs` の `cmd_install_runes` import: `install_rune_stubs` はテスト専用のため main.rs の import 行には不要（`#[cfg(test)]` の中で `crate::driver::install_rune_stubs` を参照する）。
