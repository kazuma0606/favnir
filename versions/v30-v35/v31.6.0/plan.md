# v31.6.0 実装計画 — fav test --watch

## 前提

- `fav/Cargo.toml` version = `31.5.0`
- `cargo test` — 2440 passed（0 failures）
- v31.5.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.5.0"` → `version = "31.6.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`** — `v315000_tests::cargo_toml_version_is_31_5_0` をスタブ化:

```rust
fn cargo_toml_version_is_31_5_0() {
    // Stubbed: version bumped to 31.6.0 in v31.6.0.
}
```

### Step 3: main.rs — `--watch` フラグ追加

**`fav/src/main.rs`** の `Some("test")` ブロックに以下の変更を行う。

**①** 変数宣言（`let mut file: Option<String> = None;` の直後に追加）:

```rust
let mut watch_mode = false;
let mut watch_dirs: Vec<String> = Vec::new();
```

**②** `match args[i].as_str()` ブロック内（`other =>` アームの直前）に追加:

```rust
"--watch" => {
    watch_mode = true;
    i += 1;
}
```

**③** `if let Some(n) = cases { ... }` ブロックの直後、`if (coverage_html || coverage_lcov)...` の前に追加:

```rust
if watch_mode {
    let file_for_watch: Option<&str>;
    if let Some(ref f) = file {
        let path = std::path::Path::new(f);
        if path.is_dir() {
            watch_dirs.push(f.clone());
            file_for_watch = None;   // ディレクトリは extra_dirs に委ねる
        } else {
            file_for_watch = Some(f.as_str());
        }
    } else {
        file_for_watch = None;
    }
    let dir_refs: Vec<&str> = watch_dirs.iter().map(|s| s.as_str()).collect();
    cmd_watch(file_for_watch, "test", &dir_refs, 80);
    return;
}
```

> **重要:** `file` がディレクトリの場合は `file_for_watch = None` + `extra_dirs = &["src/"]` にする。
> `cmd_watch(Some("src/"), "test", &[], 80)` とすると `collect_watch_paths(Some("src/"))` が
> `PathBuf::from("src/")` をファイルとして扱い、その parent が空文字列になって監視が壊れる。
>
> `watch_dirs` は `Vec<String>` — 将来の `--watch-dir` 複数指定拡張を見据えての型選択。
> v31.6.0 では位置引数 1 件のみのため実質 0 or 1 件。
>
> `"--watch"` は `i += 1` のみ（値なし）。続く `"src/"` は次ループの `other =>` で `file` にキャプチャされる。
> 誤って `i += 2` にすると `src/` が読み飛ばされるため注意。
>
> `cmd_test(...)` の呼び出しより前に配置すること。

### Step 4: v316000_tests 追加

`v315000_tests` の直前に追加:

```rust
// ── v31.6.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v316000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_6_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.6.0\""), "Cargo.toml must contain version = \"31.6.0\"");
    }
    #[test]
    fn benchmark_v31_6_0_exists() {
        let src = include_str!("../../benchmarks/v31.6.0.json");
        assert!(src.contains("31.6.0"), "benchmarks/v31.6.0.json must contain '31.6.0'");
    }
    #[test]
    fn collect_watch_paths_finds_fav_files() {
        use std::fs;
        let tmp = std::env::temp_dir().join("fav_v316_watch_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let fav_file = tmp.join("sample.fav");
        fs::write(&fav_file, "fn main() -> Bool { true }\n").unwrap();
        let paths = collect_watch_paths_from_dir(tmp.to_str().unwrap());
        assert!(!paths.is_empty(), "collect_watch_paths_from_dir should find .fav files");
        assert!(paths.iter().any(|p| p.extension().map(|e| e == "fav").unwrap_or(false)));
        let _ = fs::remove_dir_all(&tmp);
    }
}
```

### Step 5: CHANGELOG.md 追記

```markdown
## [v31.6.0] — 2026-07-03

### Added
- `fav test --watch <dir>` — ファイル変更検知による自動テスト再実行
- `benchmarks/v31.6.0.json` 追加

### Changed
- `Cargo.toml` version: `31.5.0` → `31.6.0`
```

### Step 6: benchmarks/v31.6.0.json 作成

```json
{
  "version": "31.6.0",
  "date": "2026-07-03",
  "milestone": "Real-World Readiness",
  "tests_passed": 2443,
  "tests_failed": 0,
  "notes": "fav test --watch: file-change-triggered automatic test re-run"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+3 件 = 2443 想定）。
> **必ず T12 で実測値に書き換えること。** 上記の 2443 は暫定値。JSON はコメント不可のため更新漏れに注意。

### Step 7: versions/current.md 更新

- 「最新安定版」欄を v31.6.0 に更新
- 「次に切る版」を `v31.7.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.5.0` → `31.6.0` |
| `fav/src/driver.rs` | 更新 | v315000 スタブ化 + v316000_tests（3件）追加 |
| `fav/src/main.rs` | 更新 | `--watch` フラグ追加 + `cmd_watch` 呼び出し |
| `CHANGELOG.md` | 更新 | [v31.6.0] セクション追加 |
| `benchmarks/v31.6.0.json` | 新規 | ベンチマーク結果（T12 で tests_passed を実測値に更新）|
| `versions/current.md` | 更新 | v31.6.0 に更新 |

---

## 完了判定

- `cargo test v316000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
