# v31.7.0 実装計画 — fav check --all

## 前提

- `fav/Cargo.toml` version = `31.6.0`
- `cargo test` — 2443 passed（0 failures）
- v31.6.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.6.0"` → `version = "31.7.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`** — `v316000_tests::cargo_toml_version_is_31_6_0` をスタブ化:

```rust
fn cargo_toml_version_is_31_6_0() {
    // Stubbed: version bumped to 31.7.0 in v31.7.0.
}
```

### Step 3: driver.rs — `check_all_files` + `cmd_check_all` 追加

`cmd_check_dir`（行 4142）の直後、`write_ambient_report` の前に追加する。

```rust
pub(crate) fn check_all_files(dir: &std::path::Path, json: bool) -> usize {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| cwd.clone());
    let toml = FavToml::load(&root);   // Option<FavToml> — .ok() 不要（Result ではない）
    let resolver = make_resolver(toml, Some(root));
    let files = collect_fav_files_recursive(dir);

    if json {
        let mut entries: Vec<serde_json::Value> = Vec::new();
        for fav_file in &files {
            let path_str = fav_file.to_string_lossy().to_string();
            let source = load_file(&path_str);
            let errors = match Parser::parse_str(&source, &path_str) {
                Err(e) => vec![serde_json::json!({"code": "PARSE", "message": e.to_string(), "line": 0})],
                Ok(program) => {
                    let mut checker = Checker::new_with_resolver(resolver.clone(), fav_file.clone());
                    let (errs, _) = checker.check_with_self(&program);
                    // TypeError（checker.rs）: code: &'static str, message: String, span: Span
                    errs.iter().map(|e| serde_json::json!({
                        "code": e.code,
                        "message": e.message,
                        "line": e.span.line,
                    })).collect()
                }
            };
            entries.push(serde_json::json!({
                "file": path_str,
                "ok": errors.is_empty(),
                "errors": errors,
            }));
        }
        // JSON モード: 「ok: false のファイル数」を返す（エラー件数の合計ではない）
        let total_errors: usize = entries.iter()
            .filter(|e| !e["ok"].as_bool().unwrap_or(true))
            .count();
        println!("{}", serde_json::to_string_pretty(&entries).unwrap_or_default());
        total_errors
    } else {
        let mut total_errors = 0usize;
        for fav_file in &files {
            let path_str = fav_file.to_string_lossy().to_string();
            eprint!("checking {}... ", path_str);
            let source = load_file(&path_str);
            let errors = match Parser::parse_str(&source, &path_str) {
                Err(e) => {
                    eprintln!();
                    eprintln!("  parse error: {}", e);
                    total_errors += 1;
                    continue;
                }
                Ok(program) => {
                    let mut checker = Checker::new_with_resolver(resolver.clone(), fav_file.clone());
                    let (errs, _) = checker.check_with_self(&program);
                    errs
                }
            };
            if errors.is_empty() {
                eprintln!("ok");
            } else {
                eprintln!();
                for e in &errors {
                    eprintln!("{}", format_diagnostic(&source, e));
                }
                total_errors += errors.len();
            }
        }
        if total_errors > 0 {
            eprintln!("\n{} エラーが見つかりました", total_errors);
        }
        total_errors
    }
}

pub fn cmd_check_all(json: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| cwd.clone());
    // FavToml::load は Option<FavToml> を返す — unwrap_or_else の引数は || (Result の |_| ではない)
    let src_dir = FavToml::load(&root)
        .map(|t| root.join(&t.src))
        .unwrap_or_else(|| cwd.clone());
    let error_count = check_all_files(&src_dir, json);
    if error_count > 0 {
        process::exit(1);
    }
}
```

> **注意:** `check_all_files` の `total_errors` は JSON モードでは `entries` 内の `ok: false` のファイル数を返す（エラー件数の合計ではなくファイル数）。
> 非 JSON モードでは `errors.len()` の合計（エラーの総件数）を返す。
> テストで扱いやすいよう `pub(crate)` にする。

### Step 4: main.rs — `--all` フラグ追加

**`fav/src/main.rs`** の `Some("check")` ブロックに以下の変更を行う。

**①** 変数宣言（`let mut sample: Option<usize> = None;` の直後）:

```rust
let mut all_mode = false;
```

**②** `match args[i].as_str()` ブロック内（`other =>` アームの直前）:

```rust
"--all" => {
    all_mode = true;
    i += 1;
}
```

**③** ディスパッチ部分（`} else if let Some(dir) = dir {` の後）:

```rust
} else if all_mode {
    driver::cmd_check_all(json);
} else {
    cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report, show_effects, refresh_schemas);
}
```

変更前:
```rust
} else if let Some(dir) = dir {
    driver::cmd_check_dir(dir);
} else {
    cmd_check(file, ...);
}
```

変更後:
```rust
} else if let Some(dir) = dir {
    driver::cmd_check_dir(dir);
} else if all_mode {
    driver::cmd_check_all(json);
} else {
    cmd_check(file, ...);
}
```

### Step 5: v317000_tests 追加

`v316000_tests` の直前に追加:

```rust
// ── v31.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v317000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_7_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.7.0\""), "Cargo.toml must contain version = \"31.7.0\"");
    }
    #[test]
    fn benchmark_v31_7_0_exists() {
        let src = include_str!("../../benchmarks/v31.7.0.json");
        assert!(src.contains("31.7.0"), "benchmarks/v31.7.0.json must contain '31.7.0'");
    }
    #[test]
    fn check_all_files_valid_fav_returns_zero_errors() {
        use std::fs;
        let tmp = std::env::temp_dir().join("fav_v317_check_all_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("main.fav"), "fn main() -> Bool { true }\n").unwrap();
        let errors = check_all_files(&tmp, false);
        assert_eq!(errors, 0, "check_all_files should return 0 errors for valid .fav file");
        let _ = fs::remove_dir_all(&tmp);
    }
}
```

### Step 6: CHANGELOG.md 追記

```markdown
## [v31.7.0] — 2026-07-03

### Added
- `fav check --all` — fav.toml src ディレクトリ内の全 .fav を一括型チェック
- `fav check --all --json` — JSON 形式でエラー出力
- `cmd_check_all()` / `check_all_files()` を `driver.rs` に追加
- `benchmarks/v31.7.0.json` 追加

### Changed
- `Cargo.toml` version: `31.6.0` → `31.7.0`
```

### Step 7: benchmarks/v31.7.0.json 作成

```json
{
  "version": "31.7.0",
  "date": "2026-07-03",
  "milestone": "Real-World Readiness",
  "tests_passed": 2446,
  "tests_failed": 0,
  "notes": "fav check --all: project-wide cross-file type check"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+3 件 = 2446 想定）。
> **必ず T12 で実測値に書き換えること。** 上記の 2446 は暫定値。

### Step 8: versions/current.md 更新

- 「最新安定版」欄を v31.7.0 に更新
- 「次に切る版」を `v31.8.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.6.0` → `31.7.0` |
| `fav/src/driver.rs` | 更新 | v316000 スタブ化 + `check_all_files` + `cmd_check_all` + v317000_tests（3件）|
| `fav/src/main.rs` | 更新 | `--all` フラグ + `cmd_check_all` ディスパッチ |
| `CHANGELOG.md` | 更新 | [v31.7.0] セクション追加 |
| `benchmarks/v31.7.0.json` | 新規 | ベンチマーク結果（T12 で実測値に更新）|
| `versions/current.md` | 更新 | v31.7.0 に更新 |

---

## 完了判定

- `cargo test v317000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
