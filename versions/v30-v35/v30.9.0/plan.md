# v30.9.0 実装計画 — ドッグフード発見修正

## Step 0 — 前提確認

```bash
cd /c/Users/yoshi/favnir/fav
grep '^version' Cargo.toml                  # → version = "30.8.0"
cargo test 2>&1 | grep "test result"        # → 2415 passed, 0 failed
grep -c 'v309000_tests' src/driver.rs       # → 0
grep -c '"project"' src/toml.rs            # → 0
```

---

## Step 1 — バージョン番号更新

`fav/Cargo.toml`:
```toml
version = "30.8.0"  →  version = "30.9.0"
```

`fav/src/driver.rs` — `v308000_tests::cargo_toml_version_is_30_8_0` をスタブ化:
```rust
fn cargo_toml_version_is_30_8_0() {
    // Version bump is tested in v309000_tests::cargo_toml_version_is_30_9_0.
}
```

---

## Step 2 — Fix 1: `toml.rs` — `[project]` セクション認識

### 2-a: section 認識追加

挿入アンカー: `if trimmed.starts_with('[') {` の直前。

```rust
if trimmed == "[project]" {
    section = "project";
    continue;
}
if trimmed.starts_with('[') {
    section = "";
    continue;
}
```

### 2-b: `"project"` アーム追加（`match section` ブロック）

挿入アンカー: `"rune" => {` ブロックの直後。

```rust
"project" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "name" => name = val.to_string(),
            "version" => version = val.to_string(),
            "description" => description = Some(val.to_string()),
            "license" => license = Some(val.to_string()),
            "authors" => {
                authors = val.split(',').map(|s| s.trim().to_string()).collect()
            }
            "src" => src = val.to_string(),
            _ => {}
        }
    }
}
```

---

## Step 3 — Fix 2: `driver.rs` — 非 rune import の `root` ベース解決

`load_all_items` 内 `load_rec` ヘルパーの `ImportDecl { is_rune: false }` アーム（行 703 付近）。

grep アンカー: `grep -n 'src_dir.join(import_name)' src/driver.rs`

```rust
// 変更前
src_dir.join(import_name).with_extension("fav")

// 変更後
root.join(import_name).with_extension("fav")
```

> **注意**: `is_rune: true` のアーム（`rune_modules/` / `runes/` 解決）は変更しない。
> 変数 `src_dir` は同関数内で `program.uses` 解決にも使われるため削除しない。

---

## Step 4 — Fix 3: `driver.rs` — `fav test` false 返却時ヒント

`cmd_test` 内の `Ok(crate::value::Value::Bool(false))` アーム（行 4454 付近）。

grep アンカー: `grep -n '"test returned false"' src/driver.rs`

```rust
// 変更前
error_msg: Some("test returned false".into()),

// 変更後
error_msg: Some("test returned false\n  hint: use assert_eq! or assert! for descriptive error messages".into()),
```

---

## Step 5 — Fix 4: `main.rs` — `fav new` 引数なし時ヒント

行 1262 付近の `Some("new")` ハンドラ内 `unwrap_or_else` クロージャ。

grep アンカー: `grep -n '"error: new requires a project name"' src/main.rs`

```rust
// 変更前
let name = args.get(2).unwrap_or_else(|| {
    eprintln!("error: new requires a project name");
    process::exit(1);
});

// 変更後
let name = args.get(2).unwrap_or_else(|| {
    eprintln!("error: new requires a project name");
    eprintln!("  hint: run 'fav new --list' to see available templates");
    process::exit(1);
});
```

---

## Step 6 — Rust テスト追加（v309000_tests — 3 件）

`driver.rs` の `v308000_tests` ブロックの直前に追加。
挿入アンカー: `// ── v30.8.0 tests ───` の直前。

```rust
// ── v30.9.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v309000_tests {
    #[test]
    fn cargo_toml_version_is_30_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"30.9.0\""), "Cargo.toml must contain version = \"30.9.0\"");
    }
    #[test]
    fn project_section_sets_src_dir() {
        let toml = crate::toml::parse_fav_toml_pub("[project]\nname=\"myapp\"\nversion=\"0.1.0\"\nsrc=\"src\"\n");
        assert_eq!(toml.src, "src", "[project] src field must be parsed as 'src'");
    }
    #[test]
    fn benchmark_v30_9_0_exists() {
        let src = include_str!("../../benchmarks/v30.9.0.json");
        assert!(src.contains("30.9.0"), "benchmarks/v30.9.0.json must contain '30.9.0'");
    }
}
```

---

## Step 7 — CHANGELOG / benchmark / current.md

### `CHANGELOG.md` 先頭に追記

```markdown
## [v30.9.0] — 2026-07-02

### Fixed
- `toml.rs` — `[project]` セクションを認識して `src` フィールドを正しくパースする
- `driver.rs` — 非 rune import を `src_dir` ではなく `root` ベースで解決（`import src/types` が `src/src/types.fav` にならない）
- `driver.rs` — `fav test` false 返却時に `assert_eq!` / `assert!` 使用を促すヒントを追加
- `main.rs` — `fav new`（引数なし）に `fav new --list` ヒントを追加
```

### `benchmarks/v30.9.0.json`

```json
{
  "version": "30.9.0",
  "date": "2026-07-02",
  "description": "dogfood fixes: [project] toml parsing, import resolution, test hint, new hint",
  "compile_ms": 11,
  "check_ms": 7,
  "tests_passed": 2418
}
```

> **注意**: `tests_passed` は `cargo test` 実行後の実際の通過数で更新すること。

### `versions/current.md`

「最新安定版」欄を `v30.8.0` → `v30.9.0` に更新。

---

## Step 8 — テスト実行

```bash
cargo test --bin fav v309000 2>&1 | tail -8   # 3/3 PASS
cargo test 2>&1 | grep "test result"          # 0 failures
```

---

## Step 9 — tasks.md を COMPLETE に更新
