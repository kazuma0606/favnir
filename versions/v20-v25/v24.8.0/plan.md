# v24.8.0 — 実装計画

## 概要

`fav/src/driver.rs` に `TEMPLATE_GALLERY` 定数と 4 つの `create_*_project` 関数を追加する。
既存の `try_cmd_new` match に 4 アームを追加するだけで CLI 統合は完了する（main.rs 変更不要）。

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep -n "version = " fav/Cargo.toml                              # "24.7.0" であること
cargo test --bin fav 2>&1 | grep "test result: ok"               # 現在の件数を実測（1962 件）
grep -n "mod v248000_tests" fav/src/driver.rs                    # 未存在
grep -n "etl-csv-to-db\|TEMPLATE_GALLERY" fav/src/driver.rs     # 全 0 件
```

---

### Step 1: `TEMPLATE_GALLERY` 定数 + `create_*_project` 関数（driver.rs）

既存の `create_postgres_etl_project` 関数の直後に追加する。

**Rust の注意事項（Step 1 全体）:**
- `format!(r#"...{name}..."#)` は正しく動作する。`format!` マクロは raw string 内の `{name}` も展開する
- `{{` / `}}` は raw string 内でも `{` / `}` にエスケープされる（Favnir コードのブロックを含む場合に必要）
- `write_text_file` は `path.parent()` に `create_dir_all` を呼ぶため、`root/` および `.github/workflows/` を含む多段ディレクトリは自動作成される。明示的な mkdir 不要
- `tempfile` は `[target.cfg(not(wasm32)).dependencies]` および `[dev-dependencies]` の両方に既に登録済み。追加不要

**1-1. `TEMPLATE_GALLERY` 定数**

```rust
pub const TEMPLATE_GALLERY: &[(&str, &str)] = &[
    ("etl-csv-to-db",    "CSV → DB ETL パイプライン"),
    ("api-gateway",      "HTTP API ゲートウェイ"),
    ("lambda-scheduled", "スケジュール実行 Lambda ジョブ"),
    ("distributed-etl",  "分散並列 ETL パイプライン"),
];
```

**1-2. `create_etl_csv_to_db_project`**

```rust
fn create_etl_csv_to_db_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("pipeline.fav"), &format!(
        "import csv\nimport postgres as db\n\nstage LoadCsv -> List[String] !Io {{\n    csv.read_file(\"data.csv\")\n}}\n\nstage InsertRows(rows: List[String]) -> Int !Db {{\n    rows |> List.map(|row| db.execute(\"INSERT INTO records (data) VALUES ($1)\", [row]))\n         |> List.length\n}}\n\npipeline {name} {{\n    LoadCsv |> InsertRows\n}}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname = \"{name}\"\n\n[runes]\ncsv      = \"1.0.0\"\npostgres = \"1.0.0\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nCSV to DB ETL pipeline.\n\n## Usage\n\n```bash\nDATABASE_URL=postgres://localhost/{name} fav run pipeline.fav\n```\n"
    ))?;
    write_text_file(&root.join(".github/workflows/ci.yml"),
        "name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo install fav\n      - run: fav check pipeline.fav\n"
    )?;
    Ok(())
}
```

**1-3. `create_api_gateway_project`**

```rust
fn create_api_gateway_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("api.fav"), &format!(
        "stage Server -> Unit !Http {{\n    Http.serve_raw(8080, |req| \"{{\\\"status\\\":\\\"ok\\\"}}\")\n}}\n\npipeline {name} {{ Server }}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname = \"{name}\"\n\n[runes]\nhttp = \"1.0.0\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nHTTP API gateway.\n\n## Usage\n\n```bash\nfav run api.fav\n```\n"
    ))?;
    write_text_file(&root.join(".github/workflows/ci.yml"),
        "name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo install fav\n      - run: fav check api.fav\n"
    )?;
    Ok(())
}
```

**1-4. `create_lambda_scheduled_project`**

```rust
fn create_lambda_scheduled_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("job.fav"), &format!(
        "stage Run -> Unit !Io {{\n    Io.println(\"Running: {name}\")\n}}\n\npipeline {name} {{ Run }}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname = \"{name}\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nScheduled Lambda job.\n\n## Usage\n\n```bash\nfav run job.fav\n```\n"
    ))?;
    write_text_file(&root.join(".github/workflows/ci.yml"),
        "name: CI\non:\n  schedule:\n    - cron: '0 * * * *'\n  push:\njobs:\n  run:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo install fav\n      - run: fav run job.fav\n"
    )?;
    Ok(())
}
```

**1-5. `create_distributed_etl_project`**

```rust
fn create_distributed_etl_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("pipeline.fav"), &format!(
        "stage FetchA -> List[String] !Http {{ Http.get(\"https://api.example.com/a\") |> String.split(\"\\n\") }}\nstage FetchB -> List[String] !Http {{ Http.get(\"https://api.example.com/b\") |> String.split(\"\\n\") }}\n\nstage Merge(a: List[String], b: List[String]) -> Int !Db {{\n    List.concat(a, b)\n        |> List.map(|row| Db.execute(\"INSERT INTO results (data) VALUES ($1)\", [row]))\n        |> List.length\n}}\n\npipeline {name} {{\n    par [FetchA, FetchB] |> Merge\n}}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname = \"{name}\"\n\n[runes]\npostgres = \"1.0.0\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nDistributed parallel ETL.\n\n## Usage\n\n```bash\nDATABASE_URL=postgres://localhost/{name} fav run pipeline.fav\n```\n"
    ))?;
    write_text_file(&root.join(".github/workflows/ci.yml"),
        "name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo install fav\n      - run: fav check pipeline.fav\n"
    )?;
    Ok(())
}
```

---

### Step 2: `try_cmd_new` match アームに 4 テンプレート追加 + エラーメッセージ更新（driver.rs）

```rust
fn try_cmd_new(name: &str, template: &str) -> Result<(), String> {
    let root = PathBuf::from(name);
    if root.exists() {
        return Err(format!("destination `{}` already exists", name));
    }
    match template {
        "script"           => create_script_project(&root, name),
        "pipeline"         => create_pipeline_project(&root, name),
        "lib"              => create_lib_project(&root, name),
        "postgres-etl"     => create_postgres_etl_project(&root, name),
        // v24.8.0: テンプレートギャラリー
        "etl-csv-to-db"    => create_etl_csv_to_db_project(&root, name),
        "api-gateway"      => create_api_gateway_project(&root, name),
        "lambda-scheduled" => create_lambda_scheduled_project(&root, name),
        "distributed-etl"  => create_distributed_etl_project(&root, name),
        other => Err(format!(
            "unknown template `{other}` \
             (expected script|pipeline|lib|postgres-etl|\
             etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl)"
        )),
    }
}
```

---

### Step 3: `fav/src/driver.rs` — v248000_tests 追加

v247000_tests には `version_is_X` テストが存在しないため削除対象なし。7 件を純粋追加する。

```rust
// ── v248000_tests (v24.8.0) — テンプレートギャラリー ───────────────────────────
#[cfg(test)]
mod v248000_tests {
    use super::*;

    #[test]
    fn template_gallery_has_4_entries() {
        assert_eq!(TEMPLATE_GALLERY.len(), 4,
            "TEMPLATE_GALLERY must have 4 entries, got {}", TEMPLATE_GALLERY.len());
        let names: Vec<&str> = TEMPLATE_GALLERY.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"etl-csv-to-db"),     "missing etl-csv-to-db");
        assert!(names.contains(&"api-gateway"),        "missing api-gateway");
        assert!(names.contains(&"lambda-scheduled"),   "missing lambda-scheduled");
        assert!(names.contains(&"distributed-etl"),    "missing distributed-etl");
    }

    #[test]
    fn fav_new_etl_csv_to_db_ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("myetl");
        let result = try_cmd_new(proj.to_str().unwrap(), "etl-csv-to-db");
        assert!(result.is_ok(), "etl-csv-to-db must succeed: {:?}", result);
        assert!(proj.join("pipeline.fav").exists(), "pipeline.fav missing");
        assert!(proj.join("fav.toml").exists(),     "fav.toml missing");
    }

    #[test]
    fn fav_new_api_gateway_ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("myapi");
        let result = try_cmd_new(proj.to_str().unwrap(), "api-gateway");
        assert!(result.is_ok(), "api-gateway must succeed: {:?}", result);
        assert!(proj.join("api.fav").exists(),  "api.fav missing");
        assert!(proj.join("fav.toml").exists(), "fav.toml missing");
    }

    #[test]
    fn fav_new_lambda_scheduled_ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("myjob");
        let result = try_cmd_new(proj.to_str().unwrap(), "lambda-scheduled");
        assert!(result.is_ok(), "lambda-scheduled must succeed: {:?}", result);
        assert!(proj.join("job.fav").exists(),  "job.fav missing");
        assert!(proj.join("fav.toml").exists(), "fav.toml missing");
    }

    #[test]
    fn fav_new_distributed_etl_ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("mybig");
        let result = try_cmd_new(proj.to_str().unwrap(), "distributed-etl");
        assert!(result.is_ok(), "distributed-etl must succeed: {:?}", result);
        assert!(proj.join("pipeline.fav").exists(), "pipeline.fav missing");
        assert!(proj.join("fav.toml").exists(),     "fav.toml missing");
    }

    #[test]
    fn fav_new_unknown_template_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("badproj");
        let result = try_cmd_new(proj.to_str().unwrap(), "no-such-template");
        assert!(result.is_err(), "unknown template must return Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("etl-csv-to-db"), "error must list etl-csv-to-db: {msg}");
    }

    #[test]
    fn changelog_has_v24_8_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v24.8.0]"),
            "CHANGELOG.md must contain [v24.8.0]");
    }
}
```

> **可視性**: `try_cmd_new` はプライベート関数だが、同一ファイル内の `#[cfg(test)] mod` からは `use super::*` でアクセス可能（既存テストと同じパターン）。

---

### Step 4: サイトドキュメント

`site/content/docs/tools/templates.mdx` を新規作成。

---

### Step 5: Cargo.toml + CHANGELOG + benchmarks

- `fav/Cargo.toml`: `"24.7.0"` → `"24.8.0"`
- `CHANGELOG.md` 先頭に v24.8.0 エントリ追加
- `benchmarks/v24.8.0.json` 作成（test_count: 1969）
