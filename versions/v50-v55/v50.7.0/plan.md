# Plan: v50.7.0 — `fav run --trace` / `fav run --watch` 強化

## 作業ステップ

### Step 1: `vm.rs` — 構造化 trace ログ

**対象**: `fav/src/backend/vm.rs` — `SeqStageCheck` ハンドラ Ok 分岐（line 2205-2208 付近）

既存の emit 行を変更:
```rust
// 変更前
trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: exit Ok({})", stage_name, display));

// 変更後
trace_emit(&mut vm.trace_lines, format!("[trace] stage={}  out={}", stage_name, display));
```

Err 分岐は変更しない。

### Step 2: `vm.rs` — `WATCH_FIELDS` スレッドローカル

**対象**: `fav/src/backend/vm.rs`（`VERBOSE_LEVEL` 定義の近傍）

`VERBOSE_LEVEL` は `Cell<u8>` を使用（`Copy` 可能）。`WATCH_FIELDS` は `Vec<String>`（`Copy` 不可）なので `RefCell` を使用。

```rust
thread_local! {
    static WATCH_FIELDS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(vec![]) };
}

pub fn set_watch_fields(fields: Vec<String>) {
    WATCH_FIELDS.with(|f| *f.borrow_mut() = fields);
}

fn watch_fields() -> Vec<String> {
    WATCH_FIELDS.with(|f| f.borrow().clone())
}
```

### Step 3: `vm.rs` — `SeqStageCheck` の `uvm` 生成条件拡張

**対象**: `fav/src/backend/vm.rs` — SeqStageCheck Ok 分岐の `uvm` 生成行（line 2200）

`--watch` が `verbose_level` と独立して動作するため、`uvm` 生成条件に watch チェックを追加:

```rust
// 変更前
let uvm = if vlevel > 0 || needs_otel {
    Some(unwrapped.clone().to_vmvalue())
} else {
    None
};

// 変更後
let has_watch = !watch_fields().is_empty();
let uvm = if vlevel > 0 || needs_otel || has_watch {
    Some(unwrapped.clone().to_vmvalue())
} else {
    None
};
```

### Step 4: `vm.rs` — `SeqStageCheck` に watch フック挿入

**対象**: `fav/src/backend/vm.rs` — OTel span 終了コード（line 2210）の直前

```rust
// watch フック（uvm が Some の場合のみ実行）
if has_watch {
    if let Some(ref uval) = uvm {
        if let VMValue::Record(ref map) = *uval {
            for target in watch_fields() {
                // "order.amount" → "amount"（最後のドット以降）
                let field_name = target.rsplit('.').next().unwrap_or(target.as_str());
                if let Some(field_val) = map.get(field_name) {
                    let field_display = truncate_for_trace(field_val, 1);
                    trace_emit(
                        &mut vm.trace_lines,
                        format!("[watch] {}: \u{2014} \u{2192} {}   (stage: {})", target, field_display, stage_name),
                    );
                }
            }
        }
    }
}
```

`—` は U+2014 EM DASH、`→` は U+2192 RIGHTWARDS ARROW。

### Step 5: `driver.rs` — `run_with_watch` テストヘルパー

**対象**: `fav/src/driver.rs`（`run_verbose` ヘルパーの直後）

`run_verbose` と同パターン（`VM::run_with_trace` 使用）:

```rust
#[cfg(test)]
fn run_with_watch(source: &str, watch_targets: &[&str]) -> Vec<String> {
    use crate::backend::vm::{set_watch_fields, set_verbose_level};
    set_watch_fields(watch_targets.iter().map(|s| s.to_string()).collect());
    let program = Parser::parse_str(source, "test_watch.fav").expect("parse");
    let artifact = build_artifact(&program);
    let main_idx = artifact.fn_idx_by_name("main").expect("main");
    let result = VM::run_with_trace(&artifact, main_idx, vec![], None, Some("test_watch.fav"));
    set_watch_fields(vec![]); // テスト後にリセット（並列テスト汚染防止）
    match result {
        Ok((_, _, traces)) => traces,
        Err(_) => vec![],
    }
}
```

### Step 6: `driver.rs` — `v507000_tests` 追加

**対象**: `fav/src/driver.rs`（`v506000_tests` モジュールの直前）

3 件:

1. `cargo_toml_version_is_50_7_0`:
```rust
let cargo_toml = include_str!("../Cargo.toml");
assert!(cargo_toml.contains("version = \"50.7.0\""));
```

2. `run_trace_structured_output`:
```rust
let source = r#"
pipeline P {
  stage Double = |n: Int| -> Int { n * 2 }
  stage Triple = |n: Int| -> Int { n * 3 }
}
fn main() -> Int { P.run(1) }
"#;
let (_, traces) = run_verbose(source, 1);
assert!(traces.iter().any(|l| l.contains("[trace] stage=Double") && l.contains("out=")));
assert!(traces.iter().any(|l| l.contains("[trace] stage=Triple") && l.contains("out=")));
```

3. `run_watch_tracks_variable`:
```rust
let source = r#"
pipeline Q {
  stage Parse = |n: Int| -> { amount: Int } { { amount: n * 10 } }
}
fn main() -> { amount: Int } { Q.run(5) }
"#;
let traces = run_with_watch(source, &["amount"]);
assert!(traces.iter().any(|l| l.contains("[watch] amount:")));
```

`v506000_tests::cargo_toml_version_is_50_6_0` を削除（`lsp_hover_builtin_fn` / `lsp_hover_rune_method` は保持）。

### Step 7: `Cargo.toml` バージョン更新

`fav/Cargo.toml`: `version = "50.6.0"` → `version = "50.7.0"`

### Step 8: テスト・Lint 確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -10
```

期待: 3105 tests passed, 0 failed

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/backend/vm.rs` | SeqStageCheck 構造化 trace + WATCH_FIELDS + uvm 条件拡張 + watch フック |
| `fav/src/driver.rs` | run_with_watch ヘルパー + v507000_tests + cargo_toml version 削除 |
| `fav/Cargo.toml` | version → `50.7.0` |

---

## リスク・注意点

- `SeqStageCheck` の Err 分岐を誤って変更しないこと
- `watch_fields()` は clone を返すため呼び出し回数を最小化（`has_watch` で事前チェック）
- `set_watch_fields(vec![])` を test teardown で必ず呼ぶ（並列テスト汚染防止）
- Record フィールドマッチは case-sensitive（`amount` と `Amount` は別物）
- `RefCell` の `borrow()` と `borrow_mut()` の混在による実行時 panic に注意（同スレッド内での二重借用不可）
