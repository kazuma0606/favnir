# v31.4.0 実装計画 — REPL 品質向上

## 前提

- `fav/Cargo.toml` version = `31.3.0`
- `cargo test` — 2433 passed（0 failures）
- v31.3.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.3.0"` → `version = "31.4.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`** — `v313000_tests::cargo_toml_version_is_31_3_0` をスタブ化（コメント付き）:

```rust
fn cargo_toml_version_is_31_3_0() {
    // Stubbed: version bumped to 31.4.0 in v31.4.0.
}
```

### Step 3: REPL プロンプト変更

`cmd_repl()` 内のプロンプト出力を変更:

```rust
// before
let _ = write!(out, "> ");

// after
let _ = write!(out, "favnir> ");
```

### Step 4: 履歴上限（100 件）

`ReplSession::add_history()` を修正:

```rust
fn add_history(&mut self, line: &str) {
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

### Step 5: repl_complete_with_defs() 追加

`repl_complete_prefix()` の直後（`extract_top_level_names()` の直前）に追加:

```rust
pub fn repl_complete_with_defs(prefix: &str, def_names: &[String]) -> Vec<String> {
    let mut result = repl_complete_prefix(prefix);
    for name in def_names {
        if name.starts_with(prefix) && !result.contains(name) {
            result.push(name.clone());
        }
    }
    result.sort();
    result
}
```

### Step 6: v314000_tests 追加

`v313000_tests` の直前に追加:

```rust
// ── v31.4.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v314000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_4_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.4.0\""), "Cargo.toml must contain version = \"31.4.0\"");
    }
    #[test]
    fn benchmark_v31_4_0_exists() {
        let src = include_str!("../../benchmarks/v31.4.0.json");
        assert!(src.contains("31.4.0"), "benchmarks/v31.4.0.json must contain '31.4.0'");
    }
    #[test]
    fn repl_complete_with_defs_delegates_to_prefix() {
        // repl_complete_prefix への委譲パスを検証（def_names が空でも BUILTIN_DOCS 補完が返る）
        let result = repl_complete_with_defs("List.", &[]);
        assert!(result.contains(&"List.map".to_string()), "should delegate to repl_complete_prefix and include List.map");
    }
    #[test]
    fn repl_complete_with_defs_returns_session_defs() {
        let def_names = vec!["my_fn".to_string(), "other_fn".to_string()];
        let result = repl_complete_with_defs("my", &def_names);
        assert!(result.contains(&"my_fn".to_string()), "should complete 'my' to 'my_fn'");
        assert!(!result.contains(&"other_fn".to_string()), "should not include non-matching 'other_fn'");
    }
}
```

### Step 7: CHANGELOG.md 追記

```markdown
## [v31.4.0] — 2026-07-02

### Added
- `driver.rs` — `repl_complete_with_defs()` 追加（セッション定義名をタブ補完に含める）
- `benchmarks/v31.4.0.json` 追加

### Changed
- `driver.rs::cmd_repl()` — REPL プロンプトを `> ` → `favnir> ` に変更
- `driver.rs::ReplSession::add_history()` — 履歴上限を 100 件に制限
- `Cargo.toml` version: `31.3.0` → `31.4.0`
```

### Step 8: benchmarks/v31.4.0.json 作成

```json
{
  "version": "31.4.0",
  "date": "2026-07-02",
  "milestone": "Real-World Readiness",
  "tests_passed": 2436,
  "tests_failed": 0,
  "notes": "REPL quality: favnir> prompt + history cap 100 + repl_complete_with_defs"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+3 件 = 2436 想定）。
> **T12 で必ず実測値に書き換えること。** 上記の 2436 は暫定値。

### Step 9: versions/current.md 更新

- 「最新安定版」欄を v31.4.0 に更新
- 「次に切る版」を `v31.5.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.3.0` → `31.4.0` |
| `fav/src/driver.rs` | 更新 | v313000 スタブ化 + プロンプト変更 + add_history 上限 + repl_complete_with_defs + v314000_tests |
| `CHANGELOG.md` | 更新 | [v31.4.0] セクション追加 |
| `benchmarks/v31.4.0.json` | 新規 | ベンチマーク結果（T12 で tests_passed を実測値に更新すること）|
| `versions/current.md` | 更新 | v31.4.0 に更新 |

---

## 完了判定

- `cargo test v314000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
