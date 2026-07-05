# v30.3.0 — 実装計画

## 前提確認（T0）

```bash
cd /c/Users/yoshi/favnir/fav

grep '^version' Cargo.toml
# → version = "30.2.0"

cargo test --bin fav 2>&1 | grep "^test result"
# → 2384 passed, 0 failed

grep -c 'v303000_tests' src/driver.rs || echo "not found"
# → not found
```

- [ ] `fav/Cargo.toml` の version が `30.2.0` であること
- [ ] テスト数が `2384 passed` であること
- [ ] `driver.rs` に `mod v303000_tests` が存在しないこと
- [ ] v30.2.0 が COMPLETE であること

---

## 実装ステップ

### Step 1 — Cargo.toml バージョン更新 + 旧バージョンテストスタブ化

**Cargo.toml:**
```toml
version = "30.2.0"  →  version = "30.3.0"
```

**`cargo_toml_version_is_30_2_0` をスタブ化（v302000_tests 内）:**
```rust
fn cargo_toml_version_is_30_2_0() {
    // Version bump is tested in v303000_tests::cargo_toml_version_is_30_3_0.
}
```

### Step 2 — フィクスチャプロジェクト作成

`fav/tests/fixtures/multifile_etl/` に以下 4 ファイルを作成する。

**注意事項（Favnir 構文制約）:**
- `let` キーワードは使用禁止（パースエラーになる）
- `String.to_int` / `String.to_float` は `Option<T>` を返す → `Some`/`None` でマッチ
- `Result.all(List.map(...))` の返値は `Result<List<T>, E>`

**`fav/tests/fixtures/multifile_etl/fav.toml`**:
```toml
[project]
name    = "multifile_etl"
version = "0.1.0"
edition = "2026"
src     = "src"
```

**`fav/tests/fixtures/multifile_etl/src/types.fav`**:
```favnir
type RawRow = {
    id:     String
    name:   String
    amount: String
}

type ValidRow = {
    id:     Int
    name:   String
    amount: Float
}

type RowError = {
    field:   String
    message: String
}
```

**`fav/tests/fixtures/multifile_etl/src/validators.fav`**:
```favnir
import src/types

public fn validate_row(row: RawRow) -> Result<ValidRow, RowError> {
    match String.to_int(row.id) {
        None     => Result.err({ field: "id", message: "id must be integer" })
        Some(id) => match String.to_float(row.amount) {
            None         => Result.err({ field: "amount", message: "amount must be float" })
            Some(amount) => Result.ok({ id: id, name: row.name, amount: amount })
        }
    }
}
```

**`fav/tests/fixtures/multifile_etl/src/main.fav`**:
```favnir
import src/types
import src/validators

public stage Pipeline: List<RawRow> -> Result<List<ValidRow>, RowError> = |rows| {
    Result.all(List.map(rows, |row| validators.validate_row(row)))
}
```

### Step 3 — ビルド + 手動検証

```bash
cd /c/Users/yoshi/favnir/fav

cargo build 2>&1 | tail -2

# 型チェック（各ファイルが通ること）
./target/debug/fav check tests/fixtures/multifile_etl/src/types.fav
./target/debug/fav check tests/fixtures/multifile_etl/src/validators.fav
./target/debug/fav check tests/fixtures/multifile_etl/src/main.fav

# lint（警告なし）
./target/debug/fav lint tests/fixtures/multifile_etl/src/main.fav
./target/debug/fav lint tests/fixtures/multifile_etl/src/validators.fav

# fmt --check（フォーマット確認）
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/types.fav
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/validators.fav
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/main.fav
```

エラーが出た場合はフィクスチャを修正し、エラーの種類を tasks.md に記録する。

### Step 4 — `v303000_tests` 追加（7 件）

**対象ファイル:** `fav/src/driver.rs`（末尾に追加）

```rust
// ── v30.3.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v303000_tests {
    use crate::frontend::parser::Parser;

    fn fixture_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/multifile_etl")
    }

    #[test]
    fn cargo_toml_version_is_30_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("30.3.0"), "Cargo.toml must contain '30.3.0'");
    }
    #[test]
    fn multifile_fixture_fav_toml_exists() {
        assert!(
            fixture_root().join("fav.toml").exists(),
            "tests/fixtures/multifile_etl/fav.toml not found"
        );
    }
    #[test]
    fn multifile_fixture_types_fav_parses() {
        let path = fixture_root().join("src/types.fav");
        let src = std::fs::read_to_string(&path).expect("types.fav not found");
        let result = Parser::parse_str(&src, "tests/fixtures/multifile_etl/src/types.fav");
        assert!(result.is_ok(), "src/types.fav parse failed: {:?}", result.err());
    }
    #[test]
    fn multifile_fixture_validators_fav_parses() {
        let path = fixture_root().join("src/validators.fav");
        let src = std::fs::read_to_string(&path).expect("validators.fav not found");
        let result = Parser::parse_str(&src, "tests/fixtures/multifile_etl/src/validators.fav");
        assert!(result.is_ok(), "src/validators.fav parse failed: {:?}", result.err());
    }
    #[test]
    fn multifile_fixture_main_fav_parses() {
        let path = fixture_root().join("src/main.fav");
        let src = std::fs::read_to_string(&path).expect("main.fav not found");
        let result = Parser::parse_str(&src, "tests/fixtures/multifile_etl/src/main.fav");
        assert!(result.is_ok(), "src/main.fav parse failed: {:?}", result.err());
    }
    #[test]
    fn changelog_has_v30_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v30.3.0]"), "CHANGELOG.md must contain '[v30.3.0]'");
    }
    #[test]
    fn benchmark_v30_3_0_exists() {
        let src = include_str!("../../benchmarks/v30.3.0.json");
        assert!(src.contains("30.3.0"), "benchmarks/v30.3.0.json must contain '30.3.0'");
    }
}
```

### Step 5 — CHANGELOG.md 更新

```markdown
## [v30.3.0] — 2026-07-01

### Added
- `fav/tests/fixtures/multifile_etl/` — マルチファイル E2E 検証用フィクスチャプロジェクト

### Verified
- `fav check` — マルチファイル `.fav` プロジェクト（3ファイル）で動作確認
- `fav lint` — マルチファイルプロジェクトで動作確認
- `fav fmt --check` — マルチファイルプロジェクトで動作確認
```

### Step 6 — benchmarks/v30.3.0.json 作成

```json
{
  "version": "30.3.0",
  "date": "2026-07-01",
  "test_count": 2391,
  "notes": "マルチファイルプロジェクト E2E 検証: fixtures/multifile_etl (+7 tests)"
}
```

### Step 7 — versions/current.md 更新

進行中バージョンを `v30.3.0` に更新する。

### Step 8 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

cargo test --bin fav v303000 2>&1 | tail -5
cargo test 2>&1 | grep -E "^test result|FAILED"
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

- [ ] フィクスチャの `.fav` コードに `let` が使われていないこと
- [ ] `String.to_int` / `String.to_float` に `Some`/`None` パターンを使っていること
- [ ] `Pipeline` stage の返値型が `Result<List<ValidRow>, RowError>` であること
- [ ] `Parser::parse_str` の第 2 引数に固定文字列を渡していること（OS パス差異回避）
- [ ] 手動検証（Step 3）が完了し、発見バグが修正されていること
