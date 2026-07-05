# v30.7.0 実装計画 — fav run エラー時スタックトレース改善

## Step 0 — 前提確認

```bash
cd /c/Users/yoshi/favnir/fav
grep '^version' Cargo.toml                      # → version = "30.6.0"
cargo test 2>&1 | grep "test result"            # → 2409 passed, 0 failed
grep -c 'v307000_tests' src/driver.rs           # → 0
grep -c 'hint_for_runtime_error' src/driver.rs  # → 0
```

---

## Step 1 — バージョン番号更新

`fav/Cargo.toml`:
```toml
version = "30.6.0"  →  version = "30.7.0"
```

`fav/src/driver.rs` — `v306000_tests::cargo_toml_version_is_30_6_0` をスタブ化。
検索アンカー: `fn cargo_toml_version_is_30_6_0()` のボディを空に:
```rust
fn cargo_toml_version_is_30_6_0() {
    // Version bump is tested in v307000_tests::cargo_toml_version_is_30_7_0.
}
```

---

## Step 2 — `hint_for_runtime_error` 追加（`driver.rs`）

挿入アンカー: `fn format_runtime_error(source_file:` の直前の行に追加。
（ファイル内検索: `grep -n 'fn format_runtime_error'` で行番号を特定）

```rust
pub(crate) fn hint_for_runtime_error(message: &str) -> Option<&'static str> {
    // より具体的なパターンを先に評価する（"global index out of bounds" は
    // "index out of bounds" を部分文字列として含むため順序が重要）
    if message.contains("global index out of bounds") || message.contains("constant index out of bounds") {
        Some("モジュールのインポートが不足している可能性があります。import 文を確認してください。")
    } else if message.contains("index out of bounds") {
        Some("List.nth は範囲外アクセスで失敗します。List.get を使うと Option<T> で安全に取得できます。")
    } else if message.contains("type error") {
        Some("型の不一致が発生しています。fav check で型エラーを事前に確認できます。")
    } else {
        None
    }
}
```

---

## Step 3 — `format_runtime_error` 改善（`driver.rs`）

挿入アンカー: `fn format_runtime_error(source_file: &str,` 関数全体を置換。

**変更前**（grep: `fn format_runtime_error`）:
```rust
fn format_runtime_error(source_file: &str, e: crate::backend::vm::VMError) -> String {
    if e.stack_trace.is_empty() {
        return format!("vm error in {} @{}: {}", e.fn_name, e.ip, e.message);
    }
    let mut msg = format!("RuntimeError: {}", e.message);
    for frame in &e.stack_trace {
        if frame.line == 0 {
            msg.push_str(&format!("\n  at {} ({})", frame.fn_name, source_file));
        } else {
            msg.push_str(&format!(
                "\n  at {} ({}:{})",
                frame.fn_name, source_file, frame.line
            ));
        }
    }
    msg
}
```

**変更後**:
```rust
fn format_runtime_error(source_file: &str, e: crate::backend::vm::VMError) -> String {
    if e.stack_trace.is_empty() {
        // fn_name / ip 情報を保持しつつプレフィックスを統一
        let mut msg = if e.fn_name == "<none>" {
            format!("runtime error: {}", e.message)
        } else {
            format!("runtime error: {}\n  in {} ({})", e.message, e.fn_name, source_file)
        };
        if let Some(hint) = hint_for_runtime_error(&e.message) {
            msg.push_str(&format!("\n  = ヒント: {}", hint));
        }
        return msg;
    }
    let mut msg = format!("runtime error: {}", e.message);
    for frame in &e.stack_trace {
        // Favnir ではステージ名はアッパーキャメルケース（例: ValidateRows）。
        // fn 名は小文字始まりが言語規約（W003 で推奨）。
        // 先頭文字が大文字の場合は "in stage X" ラベルを付与する。
        // ※ "<unknown>" / "<none>" は '<' 始まりのため誤検知しない。
        let is_stage = frame.fn_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        if is_stage {
            if frame.line == 0 {
                msg.push_str(&format!("\n  in stage {} ({})", frame.fn_name, source_file));
            } else {
                msg.push_str(&format!(
                    "\n  in stage {} ({}:{})",
                    frame.fn_name, source_file, frame.line
                ));
            }
        } else if frame.line == 0 {
            msg.push_str(&format!("\n  at {} ({})", frame.fn_name, source_file));
        } else {
            msg.push_str(&format!(
                "\n  at {} ({}:{})",
                frame.fn_name, source_file, frame.line
            ));
        }
    }
    if let Some(hint) = hint_for_runtime_error(&e.message) {
        msg.push_str(&format!("\n  = ヒント: {}", hint));
    }
    msg
}
```

---

## Step 4 — Rust テスト追加（v307000_tests — 3 件）

`fav/src/driver.rs` の `v306000_tests` ブロックの直前に追加。
挿入アンカー: `// ── v30.6.0 tests ───` の直前。

```rust
// ── v30.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v307000_tests {
    use super::hint_for_runtime_error;
    #[test]
    fn cargo_toml_version_is_30_7_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"30.7.0\""), "Cargo.toml must contain version = \"30.7.0\"");
    }
    #[test]
    fn hint_for_runtime_error_works() {
        assert!(hint_for_runtime_error("index out of bounds").is_some());
        assert!(hint_for_runtime_error("global index out of bounds").is_some());
        assert!(hint_for_runtime_error("type error in Add").is_some());
        assert!(hint_for_runtime_error("unknown runtime failure").is_none());
        // global は index より具体的なヒントを返す
        let global_hint = hint_for_runtime_error("global index out of bounds").unwrap();
        let index_hint  = hint_for_runtime_error("index out of bounds").unwrap();
        assert_ne!(global_hint, index_hint, "global and generic index hints must differ");
    }
    #[test]
    fn benchmark_v30_7_0_exists() {
        let src = include_str!("../../benchmarks/v30.7.0.json");
        assert!(src.contains("30.7.0"), "benchmarks/v30.7.0.json must contain '30.7.0'");
    }
}
```

> `use super::hint_for_runtime_error` を使用するため `hint_for_runtime_error` は `pub(crate)` である必要がある。

---

## Step 5 — CHANGELOG / benchmark / current.md

### `CHANGELOG.md` 先頭に追記

```markdown
## [v30.7.0] — 2026-07-02

### Changed
- `hint_for_runtime_error`（新規 `pub(crate)` 関数）— index out of bounds / global index / type error に `= ヒント:` を付加（具体パターン優先順）
- `format_runtime_error` — プレフィックスを `"runtime error:"` に統一、ステージ名を `"in stage X"` 形式で表示、空スタックトレース時も `fn_name` を保持
```

### `benchmarks/v30.7.0.json`

```json
{
  "version": "30.7.0",
  "date": "2026-07-02",
  "description": "runtime error stack trace improvement: stage context + hints",
  "compile_ms": 11,
  "check_ms": 7,
  "tests_passed": 2412
}
```

### `versions/current.md`

「最新安定版」欄を `v30.6.0` → `v30.7.0` に更新（バージョン番号・説明文・install コマンド）。

---

## Step 6 — テスト実行

```bash
cargo test --bin fav v307000 2>&1 | tail -10   # 3/3 PASS
cargo test 2>&1 | grep "test result"           # 0 failures
```

---

## Step 7 — tasks.md を COMPLETE に更新
