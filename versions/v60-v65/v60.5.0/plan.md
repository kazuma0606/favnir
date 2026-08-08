# v60.5.0 Plan — `fav repl` 強化（`:load` / `:debug` / マルチライン入力）

Date: 2026-07-30

---

## 実装方針

4 箇所の変更（関数 2 件追加・`cmd_repl` 2 箇所更新・定数更新）と
`v60500_tests` モジュール追加を行う。`:load` は実装済みのためテストのみ追加する。

---

## ステップ詳細

### Step 1: `needs_continuation` を追加（`driver.rs`）

追加位置: `is_definition` 関数（L13489 付近）の**直前**。

```rust
/// v60.5.0: マルチライン継続が必要な行かを判定する
pub fn needs_continuation(line: &str) -> bool {
    let t = line.trim_end();
    if t.ends_with('\\') { return true; }
    let opens: i64 = t.chars().filter(|&c| c == '(' || c == '{' || c == '[').count() as i64;
    let closes: i64 = t.chars().filter(|&c| c == ')' || c == '}' || c == ']').count() as i64;
    opens > closes
}
```

### Step 2: `handle_debug_cmd` を追加（`driver.rs`）

追加位置: `handle_load_cmd` 関数（L13443 付近）の**直前**。

```rust
/// v60.5.0: `:debug <stage>` — セッション内の stage シグネチャを表示する
pub fn handle_debug_cmd(stage_name: &str, session: &ReplSession) -> String {
    for line in session.definitions.lines() {
        let t = line.trim();
        if t.starts_with("stage ") {
            let name = extract_def_name(t);
            if name == stage_name {
                let sig = t.split('=').next().unwrap_or(t).trim();
                return format!("[debug] {}", sig);
            }
        }
    }
    format!(
        "[debug] stage '{}' not found in session; use :load <file> to load definitions",
        stage_name
    )
}
```

### Step 3: `cmd_repl` 更新 — マルチライン継続ループ（`driver.rs`）

`cmd_repl` 内の初期行読み取り後（`let line = line.trim_end_matches...` の直後）を置き換える。

既存:
```rust
let line = line.trim_end_matches(['\n', '\r']).trim();
if line.is_empty() { continue; }
session.add_history(line);
match line {
```

変更後:
```rust
let initial = line.trim_end_matches(['\n', '\r']).trim().to_string();
if initial.is_empty() { continue; }
// v60.5.0: マルチライン継続
let mut acc = initial.clone();
while needs_continuation(&acc) {
    if acc.trim_end().ends_with('\\') {
        let t = acc.trim_end().to_string();
        acc = t[..t.len() - 1].to_string();
    }
    {
        let mut out = stdout.lock();
        let _ = write!(out, "     | ");
        let _ = out.flush();
    }
    let mut cont = String::new();
    match stdin.lock().read_line(&mut cont) {
        Ok(0) | Err(_) => break,
        Ok(_) => {}
    }
    let cont_trim = cont.trim_end_matches(['\n', '\r']).trim().to_string();
    acc.push(' ');
    acc.push_str(&cont_trim);
}
let line: &str = acc.trim();
if line.is_empty() { continue; }
session.add_history(line);
match line {
```

### Step 4: `cmd_repl` 更新 — `:debug` ディスパッチ追加（`driver.rs`）

`:save ` アームの直後に挿入:

```rust
_ if line.starts_with(":debug ") => {
    println!("{}", handle_debug_cmd(line[7..].trim(), &session));
}
```

### Step 5: `REPL_COMMANDS` に `:debug ` を追加（`driver.rs`）

```rust
const REPL_COMMANDS: &[&str] = &[
    ":help", ":h", ":quit", ":q", ":reset", ":env",
    ":history", ":paste", ":type ", ":doc ", ":load ", ":save ", ":debug ",
];
```

### Step 6: `print_repl_help` 更新（`driver.rs`）

`:paste ... :end` 行（最終コマンド行、L13611 付近）の直後、`println!()` 空行の前に追加:

```rust
println!("  :debug <stage>     show stage signature from current session");
```

（実際の関数末尾順序: `:save` → `:history` → `:paste ... :end` → **ここに挿入** → `println!()` → 説明文）

### Step 7: `v60500_tests` モジュール追加（`driver.rs`）

`v60400_tests` の直前（上側）に挿入する。

```rust
// -- v60500_tests (v60.5.0) -- fav repl 強化 --
#[cfg(test)]
mod v60500_tests {
    use super::*;

    #[test]
    fn repl_load_pipeline_file() {
        // handle_load_cmd がステージ定義を含む pipeline ファイルを正しくロードする
        // （既存 repl_load_file は fn 定義のみ検証; こちらは stage 定義を検証）
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("pipeline.fav");
        std::fs::write(
            &path,
            "stage Double: Int -> Int = |x| { x + x }\nstage AddOne: Int -> Int = |x| { x + 1 }\n",
        ).expect("write pipeline.fav");
        let mut session = ReplSession::new();
        handle_load_cmd(path.to_str().unwrap(), &mut session);
        assert!(
            session.def_names.contains(&"Double".to_string()),
            "Double stage should be loaded, got: {:?}", session.def_names
        );
        assert!(
            session.def_names.contains(&"AddOne".to_string()),
            "AddOne stage should be loaded, got: {:?}", session.def_names
        );
    }

    #[test]
    fn repl_multiline_input() {
        // needs_continuation が行末 `\` / 未閉じカッコ / バランス済み行を正しく判定する
        assert!(needs_continuation("bind x <- \\"), "backslash at end should continue");
        assert!(needs_continuation("fn f("), "unclosed paren should continue");
        assert!(needs_continuation("stage S: Int -> Int = |x| {"), "unclosed brace should continue");
        assert!(!needs_continuation("bind x <- 42"), "complete line should not continue");
        assert!(!needs_continuation("fn f(x: Int) -> Int { x }"), "balanced line should not continue");
    }
}
```

---

## 注意事項

- `handle_debug_cmd` と `needs_continuation` は `pub fn` とすること
  （`v60500_tests` が `use super::*` でアクセスするため）
- `stage` 定義構文は `stage Name: InputType -> OutputType = |param| { body }` を使用
  （`check_source_str` が通ることを確認済み — L7154, L9356 の既存テストより；`x + x` vs `n * 2` のボディ差異は型チェックに影響しない）
- マルチライン継続ループ: `acc` は `String` で管理し、`line: &str` として再バインドする
  （所有権の問題を避けるため）
- `stdin.lock()` / `stdout.lock()` は継続ループ内でも一時式（`match stdin.lock().read_line(...)` 形式）として取得・解放する（既存 `cmd_repl` L13629 パターン踏襲）
- テスト `repl_load_pipeline_file` で `handle_load_cmd` が型チェック失敗時に `def_names` を更新しない点に注意：ステージ構文が正しければ問題なし
- `v60500_tests` のインポートは `use super::*;`（spec.md / tasks.md と統一）
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.4.0（ベース） | 3338 | — |
| v60.5.0 | 3340 | +2 |
