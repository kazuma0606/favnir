# v72.4.0 実装プラン — REPL 2.0

Date: 2026-08-12

---

## 依存関係

```
Step 1 (driver.rs: repl_tab_complete + needs_continuation pub化)
  └─ Step 2 (ReplSession.timing_enabled + :timing ハンドラ)
       └─ Step 3 (v724000_tests)
            └─ Step 4 (バージョン更新)
                 └─ Step 5 (テスト確認)
                      └─ Step 6 (ドキュメント更新)
```

---

## Step 1: `driver.rs` — `repl_tab_complete` 追加 + `needs_continuation` pub 化

対象: `fav/src/driver.rs`

### 1-1. `needs_continuation` を pub fn に変更

現在 `fn needs_continuation(input: &str) -> bool` として定義されている。
`pub fn needs_continuation(input: &str) -> bool` に変更する。

### 1-2. `repl_tab_complete` 追加

```rust
pub fn repl_tab_complete(prefix: &str, scope: &[&str]) -> Vec<String> {
    scope.iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| s.to_string())
        .collect()
}
```

`cmd_repl` の直前（または直後）に追加する。

### 1-3. `cargo build` で確認

---

## Step 2: `ReplSession` に `timing_enabled` 追加 + `:timing` ハンドラ追加

対象: `fav/src/driver.rs`

### 2-1. `ReplSession` 構造体に `timing_enabled: bool` フィールド追加

```rust
pub struct ReplSession {
    // 既存フィールド（変更しない）
    pub timing_enabled: bool,   // 追加
}
```

### 2-2. `ReplSession::new()` に `timing_enabled: false` を追加

### 2-3. `cmd_repl` の `match line` ブロックに `:timing` ハンドラ追加

```rust
":timing on"  => { session.timing_enabled = true;  println!("Timing enabled."); }
":timing off" => { session.timing_enabled = false; println!("Timing disabled."); }
```

### 2-4. `handle_expression` にタイミング計測を追加

`handle_expression` のシグネチャを確認し、`timing_enabled: bool` を追加引数として渡すか、
`session` 参照を渡す。最小限の変更に留める。

呼び出し側（`cmd_repl` の `_ => handle_expression(line, &session)` アーム）も更新する。

### 2-5. `cargo build` で確認

---

## Step 3: `v724000_tests` 追加（`driver.rs`）

`v723000_tests` モジュールの直後に追加:

```rust
#[cfg(test)]
mod v724000_tests {
    use super::{needs_continuation, repl_tab_complete};

    #[test]
    fn repl2_tab_completion() {
        let completions = repl_tab_complete("Li", &["List", "Csv", "linq"]);
        assert_eq!(completions, vec!["List".to_string()]);
    }

    #[test]
    fn repl2_multiline_input() {
        assert!(needs_continuation("fn main() {"), "open brace should need continuation");
        assert!(!needs_continuation("let x = 1"), "complete statement should not need continuation");
    }
}
```

- `cargo test v724000` で 2 件 pass することを確認（早期フィードバック）

---

## Step 4: バージョン更新

- `fav/Cargo.toml`: `version = "72.3.0"` → `version = "72.4.0"`
- `driver.rs` 内 `version = \"72.3.0\"` → `version = \"72.4.0\"`（replace_all）
- `driver.rs` 内 `"Cargo.toml version should be 72.3.0"` → `"72.4.0"`（replace_all）
- `driver.rs` 内 `"Cargo.toml should declare version 72.3.0"` → `"72.4.0"`（replace_all）
- `grep -c "72\.3\.0" driver.rs` の結果が T0 で記録した件数と一致することを確認し、漏れがあればパターンを追加する
- T0 で記録した件数と置換後の `"72.4.0"` grep 件数が一致することを確認する

---

## Step 5: テスト確認

- `cargo test v724000` → 2 件 pass
- `cargo test` 全体 → 3622 tests pass（0 failures）

---

## Step 6: ドキュメント更新

- `CHANGELOG.md`: `## [v72.4.0]` エントリを先頭に追加
- `versions/current.md`: 進行中バージョンを v72.4.0、次を v72.5.0 に更新
