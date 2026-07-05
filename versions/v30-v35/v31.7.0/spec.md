# v31.7.0 仕様書 — fav check --all

## 概要

`fav check --all` でプロジェクト内の全 `.fav` ファイルを一括型チェックし、
クロスファイルのエラーを一括報告する。
`fav.toml` の `src` ディレクトリを自動的に走査し、各ファイルを独立してチェック（import 解決込み）する。

---

## 背景

ロードマップ v31.7 より:

```bash
$ fav check --all
checking src/types.fav... ok
checking src/validators.fav... ok
checking src/stages.fav...
  error[E0009] src/stages.fav:34:5 — 型不一致: Int が必要ですが String が返っています
checking src/main.fav... ok

1 エラーが見つかりました
```

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `try_cmd_check_dir(dir)` | **実装済み** (`driver.rs:4081`) — ディレクトリ内全 .fav を独立チェック |
| `collect_fav_files_recursive(dir)` | **実装済み** — .fav 再帰収集 |
| `FavToml::find_root(&cwd)` | **実装済み** — cwd から fav.toml ルートを探索 |
| `FavToml::load(&root) -> Option<FavToml>` | **実装済み** — fav.toml 読み込み（`src` フィールドあり、戻り値は `Option`）|
| `make_resolver(Option<FavToml>, Option<PathBuf>) -> Arc<Mutex<Resolver>>` | **実装済み** — import 解決用 Resolver 構築 |
| `load_file(path: &str) -> String` | **実装済み** (`driver.rs`) — ファイル読み込み |
| `format_diagnostic(&source, error)` | **実装済み** — エラー表示 |
| `Checker::new_with_resolver(resolver, path)` | **実装済み** — `try_cmd_check_dir` で使用済み |
| `checker.check_with_self(&program) -> (Vec<TypeError>, _)` | **実装済み** — `TypeError` は `code: &'static str`, `message: String`, `span: Span` を持つ |
| `fav check --all` フラグ | **未実装** — `main.rs` の `check` パーサに `--all` が存在しない |
| `cmd_check_all(json)` | **未実装** |
| `check_all_files(dir, json)` | **未実装** |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.6.0` → `31.7.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_6_0` をスタブ化
- `fav/src/driver.rs` — `pub fn cmd_check_all(json: bool)` 追加
- `fav/src/driver.rs` — `pub(crate) fn check_all_files(dir: &Path, json: bool) -> usize` 追加
- `fav/src/main.rs` — `Some("check")` パーサに `--all` フラグを追加
- `fav/src/main.rs` — `--all` 時に `cmd_check_all(json)` を呼ぶ
- `fav/src/driver.rs` — `v317000_tests`（3 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.7.0]` セクション追加
- `benchmarks/v31.7.0.json` 新規作成
- `versions/current.md` — v31.7.0 に更新

### OUT OF SCOPE

- クロスファイル型依存追跡（各ファイルは独立してチェック — import は個別解決）
- インクリメンタル・並列チェック
- `--all` + `--filter` フラグの組み合わせ
- site/ MDX 更新（v32.x ドキュメント整備スプリントで対応）

---

## 実装詳細

### driver.rs — `check_all_files(dir, json)` 追加

```rust
pub(crate) fn check_all_files(dir: &std::path::Path, json: bool) -> usize {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| cwd.clone());
    let toml = FavToml::load(&root);  // Option<FavToml> — .ok() 不要
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
                    // TypeError は code: &'static str, message: String, span: Span を持つ
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
        // JSON モードでは「エラーのあるファイル数」を返す（エラー件数の合計ではない）
        // → cmd_check_all は error_count > 0 で exit(1) するため「1ファイルでもエラーあれば終了」
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
```

### driver.rs — `cmd_check_all(json)` 追加

```rust
pub fn cmd_check_all(json: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| cwd.clone());
    // FavToml::load は Option<FavToml> を返す（Result ではない）
    let src_dir = FavToml::load(&root)
        .map(|t| root.join(&t.src))
        .unwrap_or_else(|| cwd.clone());  // |_| ではなく || （Option の unwrap_or_else）
    let error_count = check_all_files(&src_dir, json);
    if error_count > 0 {
        process::exit(1);
    }
}
```

> `cmd_check_all` は `cmd_check_dir`（行 4142）の直後、`write_ambient_report` の前に追加する。

### main.rs — `--all` フラグ追加

`Some("check")` の変数宣言に追加:

```rust
let mut all_mode = false;
```

`match args[i].as_str()` ブロック内（`other =>` アームの直前）に追加:

```rust
"--all" => {
    all_mode = true;
    i += 1;
}
```

ディスパッチ部分（`} else if let Some(dir) = dir {` の後）に追加:

```rust
} else if all_mode {
    driver::cmd_check_all(json);
} else {
```

---

## テスト設計（v317000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_7_0` | `Cargo.toml` に `version = "31.7.0"` |
| 2 | `benchmark_v31_7_0_exists` | `benchmarks/v31.7.0.json` に `"31.7.0"` |
| 3 | `check_all_files_valid_fav_returns_zero_errors` | 有効な `.fav` ファイルを含む一時ディレクトリで `check_all_files` を呼び、エラー件数 0 を確認 |

> テスト#3: `check_all_files(tmpdir, false)` が 0 を返すことを確認する。
> 一時ディレクトリに `fn main() -> Bool { true }` を書いた `.fav` ファイルを配置して使用する。
> `v317000_tests` は `use super::*` あり。

---

## 完了条件

- `Cargo.toml` version = `"31.7.0"`
- `fav check --all` で fav.toml src ディレクトリを走査してチェックする
- `fav check --all --json` で JSON 形式出力される（エラーのあるファイルが 1 つでも存在すれば exit 1）
- `fav check <file>` 等の既存動作が変わらないこと
- `cargo test v317000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.7.0]` セクション
- `benchmarks/v31.7.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v31.7.0 に更新
- `tasks.md` が COMPLETE
