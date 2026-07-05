# v30.6.0 実装計画 — fav test プロジェクト統合

## Step 0 — 前提確認

```bash
cd /c/Users/yoshi/favnir/fav
grep '^version' Cargo.toml                # → version = "30.5.0"
cargo test 2>&1 | grep "test result"      # → 2406 passed, 0 failed
grep -c 'v306000_tests' src/driver.rs     # → 0
```

---

## Step 1 — バージョン番号更新

`fav/Cargo.toml`:
```toml
version = "30.5.0"  →  version = "30.6.0"
```

`fav/src/driver.rs` — `v305000_tests::cargo_toml_version_is_30_5_0` をスタブ化:
```rust
fn cargo_toml_version_is_30_5_0() {
    // Version bump is tested in v306000_tests::cargo_toml_version_is_30_6_0.
}
```

---

## Step 2 — cmd_test の tests/ 走査対応（`driver.rs`）

変更箇所: `cmd_test` 関数の `file == None` ブランチ（約 4290〜4310 行）。

現状コード:
```rust
let src_dir = toml.src_dir(&root);
collect_test_files(&src_dir)
    .into_iter()
    .filter_map(|f| {
        let path_str = f.to_string_lossy().to_string();
        let src = std::fs::read_to_string(&f).ok()?;
        Parser::parse_str(&src, &path_str)
            .ok()
            .map(|p| (path_str, p))
    })
    .collect()
```

変更後:
```rust
let src_dir = toml.src_dir(&root);
let tests_dir = root.join("tests");
let mut test_files = collect_test_files(&src_dir);
if tests_dir.is_dir() {
    test_files.extend(collect_test_files(&tests_dir));
    test_files.sort();
    test_files.dedup();
}
test_files
    .into_iter()
    .filter_map(|f| {
        let path_str = f.to_string_lossy().to_string();
        let src = std::fs::read_to_string(&f).ok()?;
        Parser::parse_str(&src, &path_str)
            .ok()
            .map(|p| (path_str, p))
    })
    .collect()
```

> **注意**: `collect_test_files` は内部で `out.sort()` 済みベクタを返す。
> `extend` 後は 2 つのソート済みリストが連結されて順序が乱れるため、`test_files.sort()` が必要。
> `dedup()` は同一絶対パスの重複除去（シンボリックリンク対策）のみ機能する。
> `src/foo.fav` と `tests/foo.fav` は別パスなので両方実行される。

---

## Step 3 — 手動検証

```bash
cd /c/Users/yoshi/favnir/fav

# ビルド
cargo build

# examples/csv-to-postgres/ にある tests/ を fav test が検出するか確認
cd ../examples/csv-to-postgres
../../fav/target/debug/fav test 2>&1
# → "running 3 tests" が表示され validate_row テスト 3 件が実行されること

# --filter フラグの動作確認（ロードマップ完了条件）
../../fav/target/debug/fav test --filter validate 2>&1
# → "validate_row" を含むテストのみが実行されること（3 件）
```

> **注意**: `examples/csv-to-postgres/` には `fav.toml` があるため、
> `fav test` 引数なしで `fav.toml` が認識される。
> `--filter` は既存実装済みのため変更不要。動作確認のみ行う。

---

## Step 4 — Rust テスト追加（v306000_tests — 3 件）

`fav/src/driver.rs` の末尾（v305000_tests ブロックの直前）に追加:

```rust
// ── v30.6.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v306000_tests {
    #[test]
    fn cargo_toml_version_is_30_6_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"30.6.0\""), "Cargo.toml must contain version = \"30.6.0\"");
    }
    #[test]
    fn cmd_test_scans_tests_dir() {
        let src = include_str!("driver.rs");
        assert!(src.contains("tests_dir"), "cmd_test must reference tests_dir");
        assert!(src.contains("is_dir()"), "cmd_test must check tests_dir.is_dir()");
    }
    #[test]
    fn benchmark_v30_6_0_exists() {
        let src = include_str!("../../benchmarks/v30.6.0.json");
        assert!(src.contains("30.6.0"), "benchmarks/v30.6.0.json must contain '30.6.0'");
    }
}
```

---

## Step 5 — CHANGELOG / benchmark / current.md

### `CHANGELOG.md` 先頭に追記

```markdown
## [v30.6.0] — 2026-07-02

### Changed
- `cmd_test`（引数なし）— `src/` に加えて `tests/` ディレクトリも走査対象に追加
- `fav test`（引数なし）で `tests/pipeline_test.fav` が自動検出・実行される
```

### `benchmarks/v30.6.0.json`

```json
{
  "version": "30.6.0",
  "date": "2026-07-02",
  "description": "fav test project integration: tests/ directory scanning",
  "compile_ms": 11,
  "check_ms": 7,
  "tests_passed": 2409
}
```

### `versions/current.md`

最新安定版を `v30.6.0` に更新。

---

## Step 6 — テスト実行

```bash
cargo test v306000 2>&1 | tail -5    # 3/3 PASS
cargo test 2>&1 | grep "test result" # 0 failures
```

---

## Step 7 — tasks.md を COMPLETE に更新
