# v30.2.0 — 実装計画

## 前提確認（T0）

実装開始前に以下のコマンドを実行して確認すること:

```bash
cd /c/Users/yoshi/favnir/fav

grep '^version' Cargo.toml
# → version = "30.1.0"

cargo test --bin fav 2>&1 | grep "^test result"
# → 2378 passed, 0 failed

grep -c 'v302000_tests' src/driver.rs || echo "not found"
# → not found
```

- [ ] `fav/Cargo.toml` の version が `30.1.0` であること
- [ ] テスト数が `2378 passed` であること
- [ ] `driver.rs` に `mod v302000_tests` が存在しないこと
- [ ] v30.1.0 が COMPLETE であること

---

## 実装ステップ

### Step 1 — Cargo.toml バージョン更新

```toml
# 変更前
version = "30.1.0"
# 変更後
version = "30.2.0"
```

### Step 2 — scaffold 関数 6 件追加

`fav/src/driver.rs` の `scaffold_postgres_etl()` 直後に追加:

```rust
fn scaffold_postgres_etl_types() -> &'static str {
    r#"// 入力 CSV の生データ型
type RawRow = {
    id:     String
    name:   String
    amount: String
    date:   String
}

// バリデーション済みの型
type ValidRow = {
    id:     Int
    name:   String
    amount: Float
    date:   String
}

// エラー型
type RowError = {
    field:   String
    message: String
}
"#
}

fn scaffold_postgres_etl_validators() -> &'static str {
    r#"import src/types

public fn validate_row(row: RawRow) -> Result<ValidRow, RowError> {
    match String.to_int(row.id) {
        Err(_) => Result.err({ field: "id", message: "id must be integer" })
        Ok(id) => match String.to_float(row.amount) {
            Err(_) => Result.err({ field: "amount", message: "amount must be float" })
            Ok(amount) => Result.ok({ id: id, name: row.name, amount: amount, date: row.date })
        }
    }
}
"#
}

fn scaffold_postgres_etl_stages() -> &'static str {
    r#"import runes/postgres
import src/types
import src/validators

fn parse_csv_row(line: String) -> RawRow {
    let parts  = String.split(line, ",")
    let id     = Option.unwrap_or(List.first(parts),                    "0")
    let name   = Option.unwrap_or(List.first(List.drop(parts, 1)), "")
    let amount = Option.unwrap_or(List.first(List.drop(parts, 2)), "0.0")
    let date   = Option.unwrap_or(List.first(List.drop(parts, 3)), "")
    { id: id, name: name, amount: amount, date: date }
}

public stage LoadCsv: String -> List<RawRow> !IO = |path| {
    bind text <- IO.read_file_raw(path)
    Result.ok(
        String.lines(text)
            |> List.drop(1)
            |> List.map(parse_csv_row)
    )
}

public stage ValidateRows: List<RawRow> -> List<ValidRow> = |rows| {
    Result.all(List.map(rows, |row| validators.validate_row(row)))
}

public stage WriteToDb: List<ValidRow> -> Int !Postgres = |rows| {
    bind conn  <- Postgres.connect(IO.env("DATABASE_URL"))
    bind count <- Result.all(
        List.map(rows, |row| Postgres.execute(conn,
            "INSERT INTO records (id, name, amount, date) VALUES ($1, $2, $3, $4)",
            [Int.to_string(row.id), row.name, Float.to_string(row.amount), row.date]))
    )
    Result.ok(List.length(count))
}

public seq EtlPipeline = LoadCsv |> ValidateRows |> WriteToDb
"#
}

fn scaffold_postgres_etl_main_v2() -> &'static str {
    r#"import src/stages

public stage Main: String -> String !IO !Postgres = |_args| {
    bind args  <- IO.argv()
    let  path   = Option.unwrap_or(List.first(args), "data/sample.csv")
    bind count <- EtlPipeline(path)
    Result.ok($"Inserted {count} rows successfully.")
}
"#
}

fn scaffold_postgres_etl_test() -> &'static str {
    r#"import src/validators
import src/types

test "validate_row: valid input" {
    let row = { id: "1", name: "Alice", amount: "9.99", date: "2026-01-01" }
    let result = validators.validate_row(row)
    assert_eq(Result.is_ok(result), true)
}

test "validate_row: invalid id" {
    let row = { id: "abc", name: "Bob", amount: "5.00", date: "2026-01-01" }
    let result = validators.validate_row(row)
    assert_eq(Result.is_err(result), true)
}
"#
}

fn scaffold_postgres_etl_readme(name: &str) -> String {
    format!(
        "# {name}\n\n\
         Postgres ETL パイプライン — Favnir テンプレート v2\n\n\
         ## セットアップ\n\n\
         ```bash\n\
         export DATABASE_URL=\"postgres://user:pass@localhost:5432/mydb\"\n\
         ```\n\n\
         ## 実行\n\n\
         ```bash\n\
         fav check                              # 型チェック\n\
         fav run src/main.fav data/sample.csv  # ETL 実行\n\
         fav test                               # テスト実行\n\
         ```\n"
    )
}
```

### Step 3 — `create_postgres_etl_project` を更新

既存の `create_postgres_etl_project` 関数本体を差し替える:

```rust
fn create_postgres_etl_project(root: &Path, name: &str) -> Result<(), String> {
    let fav_toml = format!(
        "[project]\n\
         name    = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2026\"\n\
         src     = \"src\"\n\
         \n\
         [postgres]\n\
         # url     = \"${{DATABASE_URL}}\"\n\
         sslmode = \"require\"\n"
    );
    write_text_file(&root.join("fav.toml"), &fav_toml)?;
    write_text_file(&root.join("src").join("types.fav"),      scaffold_postgres_etl_types())?;
    write_text_file(&root.join("src").join("validators.fav"), scaffold_postgres_etl_validators())?;
    write_text_file(&root.join("src").join("stages.fav"),     scaffold_postgres_etl_stages())?;
    write_text_file(&root.join("src").join("main.fav"),       scaffold_postgres_etl_main_v2())?;
    write_text_file(&root.join("tests").join("pipeline_test.fav"), scaffold_postgres_etl_test())?;
    write_text_file(&root.join("README.md"),                  &scaffold_postgres_etl_readme(name))?;
    Ok(())
}
```

### Step 4 — `new_template_postgres_etl_creates_dir` テストを更新

`src/pipeline.fav` チェックを削除し、v2 ファイルのチェックに変更する:

```rust
#[test]
fn new_template_postgres_etl_creates_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("my-etl");
    create_postgres_etl_project(&dir, "my-etl").unwrap();
    assert!(dir.join("fav.toml").exists(),                           "fav.toml not created");
    assert!(dir.join("src").join("types.fav").exists(),              "src/types.fav not created");
    assert!(dir.join("src").join("validators.fav").exists(),         "src/validators.fav not created");
    assert!(dir.join("src").join("stages.fav").exists(),             "src/stages.fav not created");
    assert!(dir.join("src").join("main.fav").exists(),               "src/main.fav not created");
    assert!(dir.join("tests").join("pipeline_test.fav").exists(),    "tests/pipeline_test.fav not created");
    assert!(dir.join("README.md").exists(),                          "README.md not created");
    let toml = std::fs::read_to_string(dir.join("fav.toml")).unwrap();
    assert!(toml.contains("sslmode"), "expected sslmode in fav.toml");
}
```

### Step 5 — `v302000_tests` 追加（末尾）

```rust
// ── v30.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v302000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_30_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("30.2.0"), "Cargo.toml must contain '30.2.0'");
    }
    #[test]
    fn postgres_etl_v2_creates_types_fav() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("proj");
        create_postgres_etl_project(&dir, "proj").unwrap();
        assert!(dir.join("src").join("types.fav").exists(), "src/types.fav not created");
    }
    #[test]
    fn postgres_etl_v2_creates_validators_fav() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("proj");
        create_postgres_etl_project(&dir, "proj").unwrap();
        assert!(dir.join("src").join("validators.fav").exists(), "src/validators.fav not created");
    }
    #[test]
    fn postgres_etl_v2_creates_stages_fav() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("proj");
        create_postgres_etl_project(&dir, "proj").unwrap();
        assert!(dir.join("src").join("stages.fav").exists(), "src/stages.fav not created");
    }
    #[test]
    fn changelog_has_v30_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v30.2.0]"), "CHANGELOG.md must contain '[v30.2.0]'");
    }
    #[test]
    fn benchmark_v30_2_0_exists() {
        let src = include_str!("../../benchmarks/v30.2.0.json");
        assert!(src.contains("30.2.0"), "benchmarks/v30.2.0.json must contain '30.2.0'");
    }
}
```

### Step 6 — CHANGELOG.md 更新

```markdown
## [v30.2.0] — 2026-07-01

### Changed
- `fav new --template postgres-etl` — 4 ファイル構成（types / validators / stages / main）に更新
- `tests/pipeline_test.fav` と `README.md` を生成するように変更
- scaffold コードを VM 確認済みプリミティブ（`String.to_int` / `String.to_float` / `Option.unwrap_or`）で統一
```

### Step 7 — benchmarks/v30.2.0.json 作成

```json
{
  "version": "30.2.0",
  "date": "2026-07-01",
  "test_count": 2384,
  "notes": "postgres-etl テンプレート v2: 4 ファイル構成"
}
```

### Step 7.5 — versions/current.md 更新

`versions/current.md` の進行中バージョンを `v30.2.0` に更新し、最新安定版を `v30.1.0` のままにする。

### Step 8 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

cargo test --bin fav v302000 2>&1 | tail -5
cargo test 2>&1 | grep -E "test result|FAILED"
```

### Step 9 — tasks.md 更新

全チェックボックスを `[x]` にして COMPLETE にする。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## コードレビューチェックリスト

- [ ] scaffold 関数に HTML インジェクション・コマンドインジェクションがないこと
- [ ] `scaffold_postgres_etl()` を削除していないこと（`cmd_scaffold` で引き続き使用）
- [ ] `scaffold_postgres_etl_uses_chain` テストが引き続き通過すること
- [ ] `write_text_file` が `tests/` ディレクトリを自動作成すること（`create_dir_all` 内部使用）
- [ ] scaffold コードが VM 未実装プリミティブ（`List.get` / `List.map_indexed` / `Int.parse`）を使わないこと
