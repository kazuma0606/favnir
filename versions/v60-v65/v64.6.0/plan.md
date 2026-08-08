# v64.6.0 Plan — `fav lint --perf`

Version: 64.6.0
Status: 未着手

---

## 作業順序

### Step 1: `toml.rs` — `LintTomlConfig` に `perf` フィールド追加

**① `LintTomlConfig` 構造体定義に `perf: Option<bool>` を追加**

```rust
pub struct LintTomlConfig {
    pub warn_as_error: Option<Vec<String>>,
    pub allow: Option<Vec<String>>,
    pub strict: Option<bool>,
    /// v64.6.0: `perf = true` で W041 等のパフォーマンス lint を有効化。
    pub perf: Option<bool>,
}
```

**② `parse_fav_toml` の `[lint]` セクション struct literal（`unwrap_or(LintTomlConfig { ... })`）に `perf: None` を追加**

```rust
let mut current = lint_cfg.take().unwrap_or(LintTomlConfig {
    warn_as_error: None,
    allow:         None,
    strict:        None,
    perf:          None,   // ← 追加
});
```

**③ `[lint]` セクションの `parse_kv` 処理に `perf` キーを追加**

`strict` キーのすぐ下（または `if key == "strict"` と同じ `if/else if` チェーンに追加）:

```rust
if key == "strict" {
    current.strict = Some(val.trim() == "true");
} else if key == "perf" {
    current.perf = Some(val.trim() == "true");
} else {
    // Parse comma-separated list: warn_as_error = ["W006", "W007"]
    ...
}
```

### Step 2: `driver.rs` — `cmd_lint` の `perf` を toml 設定から読み取るよう更新

`cmd_lint` 内の `LintConfig` 構築箇所（行 ~11185）を変更:

```rust
// 変更前:
let run_config = LintConfig { strict: strict_mode, perf: false };

// 変更後:
let perf_mode = lint_config.as_ref().and_then(|c| c.perf).unwrap_or(false);
let run_config = LintConfig { strict: strict_mode, perf: perf_mode };
```

### Step 3: `driver.rs` — `v64600_tests` 追加

`v64500_tests` の直前（`// -- v64500_tests` コメント行の前）に挿入:

```rust
// -- v64600_tests (v64.6.0) -- lint --perf --
#[cfg(test)]
mod v64600_tests {
    use super::*;
    use crate::lint::{lint_program_with_config, LintConfig};

    #[test]
    fn lint_perf_flag_enables_w039_w040() { ... }

    #[test]
    fn lint_toml_perf_setting() { ... }
}
```

### Step 4: ビルド・テスト

```bash
cargo build 2>&1 | tail -5
cargo test --bin fav v64600_tests 2>&1 | tail -10
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

---

## 注意事項

- `LintTomlConfig` の struct literal は `parse_fav_toml` 内の 1 箇所のみ（`BenchTomlConfig` と異なり driver.rs/checker.rs/resolver.rs に FavToml struct literal は不要）
- `lint_program_with_config` のシグネチャ: `(program: &Program, config: &LintConfig) -> Vec<LintError>`（引数 2 つ）
- `lint_toml_perf_setting` テストは `parse_fav_toml_pub` が `pub` であることを前提（v64.2.0 で確認済み）
