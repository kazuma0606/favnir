# v32.1.0 — 実装計画: 境界付きジェネリクス T with Ord

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `32.0.0` → `32.1.0` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_0_0` スタブ化、`v321000_tests` 追加 |
| `CHANGELOG.md` | `[v32.1.0]` セクション追加 |
| `benchmarks/v32.1.0.json` | 新規作成 |
| `versions/current.md` | 最新安定版を v32.1.0 に更新 |

---

## 実装手順

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "32.0.0"` を `"32.1.0"` に変更。

---

### Step 2: `cargo_toml_version_is_32_0_0` スタブ化

`v320000_tests` 内の `cargo_toml_version_is_32_0_0` テストを空スタブに変更。
`v320000_tests` は `use super::*` なしのため、スタブ化による副作用はなし。

```rust
#[test]
fn cargo_toml_version_is_32_0_0() {
    // stubbed: version has advanced to 32.1.0
}
```

---

### Step 3: `v321000_tests` 追加

挿入位置: `v320000_tests` の閉じ括弧（`}`）の直後、`// ── v31.7.0 tests` コメントの前。

**重要**: `check_errors` は driver.rs 内の各テストモジュールにローカル定義されるヘルパーで、
`super::*` では参照できない。`v321000_tests` 内でモジュール内ローカル関数として定義する。
`use super::*` は**不要**（`use crate::...` でインポートを完結させる）。

```rust
// ── v32.1.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v321000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v321000_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.1.0"), "Cargo.toml must contain '32.1.0'");
    }

    #[test]
    fn benchmark_v32_1_0_exists() {
        let src = include_str!("../../benchmarks/v32.1.0.json");
        assert!(src.contains("32.1.0"), "benchmarks/v32.1.0.json must contain '32.1.0'");
    }

    #[test]
    fn bounded_generics_display_and_hash_bounds() {
        // Display bound: String を渡してもエラーなし
        let display_errors = check_errors(r#"
fn show<T with Display>(val: T) -> String {
    f"{val}"
}
fn main() -> String {
    show("hello")
}
"#);
        assert!(
            display_errors.is_empty(),
            "Display bound should pass for String: {:?}",
            display_errors
        );

        // Hash bound: Int を渡してもエラーなし
        let hash_errors = check_errors(r#"
fn hash_it<T with Hash>(val: T) -> Int {
    42
}
fn main() -> Int {
    hash_it(99)
}
"#);
        assert!(
            hash_errors.is_empty(),
            "Hash bound should pass for Int: {:?}",
            hash_errors
        );
    }
}
```

---

### Step 4: `CHANGELOG.md` 更新

先頭に追記:

```markdown
## [v32.1.0] — 2026-07-03

### Added
- 境界付きジェネリクス（bounded generics）の確認・テスト補強
- `Display` / `Hash` 境界の動作を `v321000_tests` で明示的に検証
- 組み込み 4 Interface（Ord / Eq / Display / Hash）が仕様通り動作することを確認

### Notes
- 実装自体は v17.1.0 で完了済み。v32.1.0 は Language Power フェーズの起点として記録。
```

---

### Step 5: `benchmarks/v32.1.0.json` 作成

```json
{
  "version": "32.1.0",
  "date": "2026-07-03",
  "milestone": "Language Power",
  "tests_passed": 2459,
  "tests_failed": 0,
  "notes": "Bounded generics: Display/Hash bound verification (impl since v17.1.0)"
}
```

`tests_passed` は実測後に更新する（暫定: 2456 + 3 = 2459）。

---

### Step 6: `versions/current.md` 更新

「最新安定版」欄を v32.1.0 に更新し、「進行中バージョン」を「なし（v32.1.0 完了直後）」、
「次に切る版」を v32.2.0 に変更する。
