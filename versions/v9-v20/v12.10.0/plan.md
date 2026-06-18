# Favnir v12.10.0 実装計画

Date: 2026-06-09

---

## Phase A — `get_help_text` + エラー出力への `help:` 注入（driver.rs）

### A-1: `get_help_text(code: &str) -> &'static [&'static str]` を定義

```rust
fn get_help_text(code: &str) -> &'static [&'static str] {
    match code {
        "E0001" => &[
            "check the variable name for typos",
            "introduce the variable with `bind x <- expr`",
        ],
        "E0007" => &[
            "see available primitives: `fav doc --builtins`",
        ],
        "E0008" => &[
            "check the number of arguments matches the function signature",
        ],
        "E0009" => &[
            "the declared return type and the inferred body type must match",
            "add a type annotation or adjust the return value",
        ],
        "E0013" => &[
            "fix the `where` validator expression",
        ],
        "E0014" => &[
            "add the missing `fn` declaration to the interface",
        ],
        "E0015" => &[
            "implement all required `fn` entries listed in the interface",
        ],
        "E0018" => &[
            "use a different name: `bind x2 <- ...`",
            "or discard the value: `bind _ <- ...`",
        ],
        "W001" => &[
            "prefix the name with `_` to suppress: `_unused`",
        ],
        "W004" => &[
            "use `chain x <- expr` to propagate errors automatically",
        ],
        "W006" => &[
            "use `chain _ <- expr` to propagate errors automatically",
            "or handle explicitly: `match expr { Ok(_) => ... Err(e) => ... }`",
        ],
        "W007" => &[
            "the returned Result must be handled or propagated",
        ],
        _ => &[],
    }
}
```

### A-2: `cmd_check` の型エラー出力ループに `get_help_text` を差し込む

既存の `fmt_error_with_source` / エラー表示箇所を特定し、
エラーコードが確定した後に:

```rust
for hint in get_help_text(err.code) {
    eprintln!("  = help: {}", hint);
}
```

を追加。

### A-3: `cmd_lint` の lint 出力ループに `get_help_text` を差し込む

```rust
for hint in get_help_text(&lint.code) {
    eprintln!("  = help: {}", hint);
}
```

---

## Phase B — `fav check --strict`（driver.rs + main.rs）

### B-1: `cmd_check` シグネチャに `strict: bool` を追加

```rust
pub fn cmd_check(
    file: Option<&str>,
    no_warn: bool,
    legacy_check: bool,
    json: bool,
    show_types: bool,
    strict: bool,   // ← 追加
)
```

### B-2: `strict` 時の動作

`collect_binding_types`（W006 検出）を `strict` 時は常に実行し、
W006 マークが付いた binding が 1 件以上ある場合は exit 1:

```rust
if strict {
    let bindings = collect_binding_types(path);
    let w006_count = bindings.iter().filter(|b| b.warning.as_deref() == Some("W006")).count();
    if w006_count > 0 {
        eprintln!("error: --strict: {} W006 warning(s) treated as errors", w006_count);
        process::exit(1);
    }
}
```

### B-3: `main.rs` に `--strict` フラグ追加

```rust
Some("check") => {
    // ... 既存パース ...
    let strict = args.iter().any(|a| a == "--strict");
    cmd_check(file, no_warn, legacy_check, json, show_types, strict);
}
```

---

## Phase C — `fav lint --deny-warnings`（driver.rs + main.rs）

### C-1: `cmd_lint` の `warn_only` を `deny_warnings` に整理

現在: `warn_only=false`（デフォルト）→ 警告で exit 1
新規: `--deny-warnings` フラグを追加して CI での意図を明示

```rust
pub fn cmd_lint(file: Option<&str>, warn_only: bool, deny_warnings: bool)
```

`deny_warnings=true` は `warn_only=false` と同義（exit 1）。
両方の内部ロジックは共通:

```rust
let should_exit = deny_warnings || !warn_only;
if total_warnings > 0 && should_exit {
    process::exit(1);
}
```

### C-2: `main.rs` に `--deny-warnings` パース追加

```rust
"--deny-warnings" => { deny_warnings = true; i += 1; }
```

### C-3: CI 更新（ci.yml）

```yaml
- name: Self-lint (fav lint)
  working-directory: fav
  run: |
    ./target/debug/fav lint --deny-warnings self/compiler.fav
    ./target/debug/fav lint --deny-warnings self/checker.fav
```

---

## Phase D — `fav.toml [lint]` セクション（toml.rs + driver.rs）

### D-1: `LintTomlConfig` を `toml.rs` に追加

```rust
#[derive(serde::Deserialize, Default, Clone)]
pub struct LintTomlConfig {
    pub warn_as_error: Option<Vec<String>>,
    pub allow:         Option<Vec<String>>,
}
```

### D-2: `FavToml` に `lint` フィールドを追加

```rust
pub struct FavToml {
    // ... 既存フィールド ...
    pub lint: Option<LintTomlConfig>,
}
```

### D-3: `cmd_lint` で `fav.toml` の `lint` セクションを読む

```rust
// allow リストに含まれるコードをフィルタ
let filtered_lints: Vec<_> = lints.iter().filter(|l| {
    !allow_codes.contains(&l.code.to_string())
}).collect();

// warn_as_error リストに含まれるコードは exit 1
let has_error_level = filtered_lints.iter().any(|l| {
    warn_as_error_codes.contains(&l.code.to_string())
});
```

---

## Phase E — テスト追加（driver.rs）

### `v121000_tests` モジュール

```rust
#[cfg(test)]
mod v121000_tests {
    // --strict で W006 が exit 1
    fn check_strict_w006_exits_1()         { ... }
    // --strict で警告なし → exit 0
    fn check_strict_no_warning_exits_0()   { ... }
    // --deny-warnings で exit 1
    fn lint_deny_warnings_exits_1()        { ... }
    // help text E0001 に "= help:" が含まれる
    fn help_text_e0001_present()           { ... }
    // help text W006 に "= help:" が含まれる
    fn help_text_w006_present()            { ... }
    // lint allow で W001 が抑制される
    fn lint_allow_suppresses_w001()        { ... }
    // version
    fn version_is_12_10_0()               { ... }
}
```

テスト実装方針:
- `check_strict_*`: `std::process::Command::new(env!("CARGO_BIN_EXE_fav"))` で
  プロセス起動して exit code を確認（integration test に置く）
- `help_text_*`: `get_help_text("E0001")` を直接呼び出して内容を検証（unit test）
- `lint_*`: `cmd_lint` 相当の出力を capture、または process で確認

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.10.0"`
- `versions/v12.10.0/tasks.md` の version テストコードを `12.10.0` に
- `cargo test` 全通過確認
- `git commit -m "feat: v12.10.0 — help: for errors + --strict + --deny-warnings"`
- `git push` → CI 通過確認

---

## 実装上の注意

### 1. `cmd_check` の既存呼び出し箇所

`cmd_check` は `main.rs` の `Some("check")` 分岐のみから呼ばれる。
シグネチャに `strict: bool` を追加しても呼び出し元は 1 箇所なので変更は局所的。

### 2. W006 と `--strict` の関係

W006 は `collect_binding_types`（v12.5.0 実装）でのみ検出される。
`--strict` 時は `collect_binding_types` を呼んで W006 カウントを確認する。
`--show-types` との組み合わせも可能（`--strict --show-types`）。

### 3. `warn_only` の後方互換性

既存の `--warn-only` フラグは変更しない。
`--deny-warnings` は新規フラグとして追加し、`warn_only=false` と同じ効果にする。
将来的に `--warn-only` を deprecated にする予定（v13.0.0 以降）。

### 4. `fav.toml [lint]` の `allow` と `--deny-warnings` の優先度

`allow` でコードを抑制 → そのコードは `--deny-warnings` の対象外。
`warn_as_error` でコードをエラー化 → `--warn-only` でも exit 1 になる。

### 5. テストでの help text 検証

`get_help_text` 関数が public でない場合は `pub(crate)` にするか、
`#[cfg(test)]` ブロックから呼べるようにする。
