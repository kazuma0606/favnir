# v31.9.0 — 実装計画: ドッグフード修正 vol.2

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `31.8.0` → `31.9.0` |
| `fav/src/driver.rs` | `cargo_toml_version_is_31_8_0` スタブ化、`add_history` 修正、`check_all_files` 修正、`v319000_tests` 追加 |
| `CHANGELOG.md` | `[v31.9.0]` セクション追加 |
| `benchmarks/v31.9.0.json` | 新規作成 |
| `versions/current.md` | 最新安定版を v31.9.0 に更新 |

※ site/ MDX 更新なし（バグ修正パッチのため）

---

## 実装手順

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "31.8.0"` を `"31.9.0"` に変更。

---

### Step 2: `cargo_toml_version_is_31_8_0` スタブ化

driver.rs の `v318000_tests` 内の `cargo_toml_version_is_31_8_0` テストを空スタブに変更する。

```rust
#[test]
fn cargo_toml_version_is_31_8_0() {
    // stubbed: version has advanced to 31.9.0
}
```

---

### Step 3: `add_history` 修正

driver.rs:12020 の `add_history` メソッドに空行チェックを追加する。

**変更前:**
```rust
fn add_history(&mut self, line: &str) {
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

**変更後:**
```rust
fn add_history(&mut self, line: &str) {
    if line.trim().is_empty() {
        return;
    }
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

---

### Step 4: `check_all_files` 空ファイル警告追加

driver.rs:4153 の `let files = collect_fav_files_recursive(dir);` の直後（4154 行 `if json {` の前）に
非 JSON モードの空チェックを追加する。

**変更前（driver.rs:4153〜4154）:**
```rust
    let files = collect_fav_files_recursive(dir);
    if json {
```

**変更後:**
```rust
    let files = collect_fav_files_recursive(dir);
    if files.is_empty() && !json {
        eprintln!("no .fav files found in `{}`", dir.display());
        return 0;
    }
    if json {
```

注意: JSON モードで files が空のとき `if json {` に到達し、空配列 `[]` を出力して `0` を返す。
これは変更前と同じ動作（JSON モードは変更なし）。

---

### Step 5: `v319000_tests` 追加

`v318000_tests` の閉じ括弧の直後に追加する。

注意: struct 名は `ReplSession`（`ReplState` ではない）。`add_history` と `history` フィールドは
`driver.rs` 内で定義されており、`mod v319000_tests { use super::*; }` は driver.rs の子モジュールに
なるため、Rust のプライバシー規則（子モジュールは親モジュールの private アイテムにアクセス可能）
により `history` フィールドへの直接アクセスが可能。`pub` 修飾子の追加は不要。

```rust
#[cfg(test)]
mod v319000_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_31_9_0() {
        let src = include_str!("../../Cargo.toml");
        assert!(src.contains("31.9.0"), "Cargo.toml must contain '31.9.0'");
    }

    #[test]
    fn benchmark_v31_9_0_exists() {
        let src = include_str!("../../benchmarks/v31.9.0.json");
        assert!(src.contains("31.9.0"), "benchmarks/v31.9.0.json must contain '31.9.0'");
    }

    #[test]
    fn repl_add_history_skips_blank_lines() {
        let mut state = ReplSession::new();
        state.add_history("");
        state.add_history("   ");
        state.add_history("\t");
        assert!(state.history.is_empty(), "blank lines should not be added to history");
        state.add_history("List.length([1,2,3])");
        assert_eq!(state.history.len(), 1);
    }
}
```

---

### Step 6: `benchmarks/v31.9.0.json` 作成

```json
{
  "version": "31.9.0",
  "date": "2026-07-03",
  "milestone": "Real-World Readiness",
  "tests_passed": 2452,
  "tests_failed": 0,
  "notes": "REPL blank-line history skip; check --all empty-dir warning"
}
```

`tests_passed` は実測後に更新する（暫定: 2449 + 3 = 2452）。

---

### Step 7: `CHANGELOG.md` 更新

先頭に追記:

```markdown
## [v31.9.0] — 2026-07-03

### Fixed
- REPL: `add_history` が空行・空白行を履歴に追加しないよう修正（`ReplSession::add_history`）
- `fav check --all`: .fav ファイルが見つからない場合に警告メッセージを表示
```

---

### Step 8: `versions/current.md` 更新

「最新安定版」欄を v31.9.0 に更新し、「進行中バージョン」を「なし（v31.9.0 完了直後）」、
「次に切る版」を v32.0.0 に変更する。
