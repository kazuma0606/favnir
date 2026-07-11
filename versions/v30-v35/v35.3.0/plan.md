# v35.3.0 実装計画 — `fav ci init`

## 依存関係順序

```
driver.rs — generate_ci_yaml 追加
    ↓
driver.rs — cmd_ci_init 追加
    ↓
main.rs — Some("ci") アーム追加 + ヘルプテキスト更新
    ↓
driver.rs — v35300_ci_tests 追加（v35300_tests の cargo_toml_version_is_35_3_0 をスタブ化）
    ↓
Cargo.toml バージョン → 35.3.0
    ↓
cargo test 全通過確認
    ↓
CHANGELOG 更新
```

---

## Step 1: `fav/src/driver.rs` — `generate_ci_yaml` 追加

v35.2.0 の `cmd_deploy_docker` 周辺の後ろ（または deploy 関連関数群の末尾）に追加する。
`write_deploy_file` などのユーティリティとの整合性のため、deploy helpers セクション付近に配置する。

```rust
/// v35.3.0: GitHub Actions CI ワークフロー YAML を生成する。
/// check + lint + test の 3 ステップを含む。
pub fn generate_ci_yaml(_project_name: &str) -> String {
    "name: CI\n\
     on:\n\
     \x20 push:\n\
     \x20   branches: [main]\n\
     \x20 pull_request:\n\
     \n\
     jobs:\n\
     \x20 ci:\n\
     \x20   runs-on: ubuntu-latest\n\
     \x20   steps:\n\
     \x20     - uses: actions/checkout@v4\n\
     \x20     - name: Install fav\n\
     \x20       run: cargo install fav\n\
     \x20     - name: Check\n\
     \x20       run: fav check\n\
     \x20     - name: Lint\n\
     \x20       run: fav lint\n\
     \x20     - name: Test\n\
     \x20       run: fav test\n"
        .to_string()
}
```

`_project_name` は現バージョンでは未使用（ジョブ名は `ci` 固定）。将来の拡張のためシグネチャに残す。
`_` プレフィックスにより Clippy の `unused_variables` 警告は抑制される。`pub fn` のため `dead_code` 警告も発生しない。既存コード（`driver.rs`）でも同パターンが使われており問題なし。

---

## Step 2: `fav/src/driver.rs` — `cmd_ci_init` 追加

`generate_ci_yaml` の直後に追加する。

```rust
/// v35.3.0: `fav ci init` のエントリポイント。
/// `.github/workflows/ci.yml` を生成する。
pub fn cmd_ci_init(out_dir: Option<&str>, dry_run: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| cwd.clone());
    let toml = FavToml::load(&root);
    let project_name = toml
        .as_ref()
        .map(|t| t.name.as_str())
        .unwrap_or("fav-project");

    let yaml = generate_ci_yaml(project_name);

    if dry_run {
        println!("[ci] Preview — .github/workflows/ci.yml:");
        println!("{}", yaml);
        println!("[ci] Done (dry-run — no files written)");
        return;
    }

    let base = out_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.clone());
    let target = base.join(".github").join("workflows").join("ci.yml");

    if let Err(e) = write_text_file(&target, &yaml) {
        eprintln!("error: failed to write {}: {}", target.display(), e);
        process::exit(1);
    }
    println!("[ci] Generated → {}", target.display());
}
```

---

## Step 3: `fav/src/main.rs` — `Some("ci")` アーム追加

既存の `Some("deploy")` アームの直後に追加する。
`cmd_ci_init` は `driver.rs` に `pub fn` として追加されるため、`crate::driver::cmd_ci_init` で呼べる。

```rust
Some("ci") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("init") => {
            let mut out_dir: Option<String> = None;
            let mut dry_run = false;
            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--out-dir" => {
                        out_dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --out-dir requires a directory path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--dry-run" => { dry_run = true; i += 1; }
                    other => {
                        eprintln!("error: unexpected ci init argument `{}`", other);
                        process::exit(1);
                    }
                }
            }
            cmd_ci_init(out_dir.as_deref(), dry_run);
        }
        Some(other) => {
            eprintln!("error: unknown ci subcommand `{}`", other);
            process::exit(1);
        }
        None => {
            // サブコマンドなし → usage を stderr に出力して exit 1（他のサブコマンド制御コマンドと同方針）
            eprintln!("usage: fav ci init [--out-dir <dir>] [--dry-run]");
            process::exit(1);
        }
    }
}
```

ヘルプテキスト（`--help` 出力）にも `ci` を追記する:
```
ci init            Generate .github/workflows/ci.yml (check + lint + test)
```

---

## Step 4: `fav/src/driver.rs` — v35300_ci_tests 追加

`v35300_tests` の `cargo_toml_version_is_35_3_0` テストをスタブ化し、
`v35300_ci_tests` モジュールを `v35300_tests` の直後に追加する。

```rust
// ── v35.3.0 — Deployment Story: ci init tests ────────────────────────────────
#[cfg(test)]
mod v35300_ci_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_35_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("35.3.0"), "Cargo.toml must contain version 35.3.0");
    }

    #[test]
    fn ci_command_exists_in_main() {
        let src = include_str!("main.rs");
        assert!(src.contains("Some(\"ci\")"), "main.rs must contain Some(\"ci\") arm");
    }

    #[test]
    fn generate_ci_yaml_has_check_step() {
        let yaml = generate_ci_yaml("my-project");
        assert!(yaml.contains("fav check"), "CI yaml must contain 'fav check' step");
    }

    #[test]
    fn generate_ci_yaml_has_lint_step() {
        let yaml = generate_ci_yaml("my-project");
        assert!(yaml.contains("fav lint"), "CI yaml must contain 'fav lint' step");
    }

    #[test]
    fn generate_ci_yaml_has_test_step() {
        let yaml = generate_ci_yaml("my-project");
        assert!(yaml.contains("fav test"), "CI yaml must contain 'fav test' step");
    }

    #[test]
    fn changelog_has_v35_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[35.3.0]"), "CHANGELOG.md must contain [35.3.0]");
    }
}
```

---

## Step 5: `fav/Cargo.toml` バージョン更新

```toml
version = "35.3.0"
```

---

## Step 6: `cargo test` 全通過確認

```
cargo test 2>&1 | grep "test result"
# expected: ok. XXXX passed; 0 failed
```

v35300_ci_tests の全テストが pass することを確認する。

---

## Step 7: `CHANGELOG.md` 更新

テスト全通過確認後に追加する。先頭に以下を追加:

```markdown
## [35.3.0] — 2026-07-06

### Added
- `fav ci init` — GitHub Actions CI ワークフロー自動生成コマンド
- `fav ci init --dry-run` — ファイル書き出しなしのプレビューモード
- `fav ci init --out-dir <dir>` — 出力先ディレクトリ指定
- `generate_ci_yaml` — check + lint + test の 3 ステップを含む CI YAML テンプレート生成関数
- `.github/workflows/ci.yml` の自動生成（親ディレクトリ自動作成）
```
