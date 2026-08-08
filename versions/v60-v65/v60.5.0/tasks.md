# v60.5.0 Tasks — `fav repl` 強化（`:load` / `:debug` / マルチライン入力）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3338 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60500_tests` がまだ存在しないことを確認
  - `grep -c 'v60500_tests' fav/src/driver.rs` = 0 件
- [x] `v60400_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v60400_tests' fav/src/driver.rs` ≥ 1 件
- [x] `needs_continuation` がまだ存在しないことを確認
  - `grep -c 'needs_continuation' fav/src/driver.rs` = 0 件
- [x] `handle_debug_cmd` がまだ存在しないことを確認
  - `grep -c 'handle_debug_cmd' fav/src/driver.rs` = 0 件
- [x] `stage Double: Int -> Int = |x| { x + x }` が既存テストで使われていることを確認（構文有効性）
  - `grep -c 'stage Double: Int -> Int' fav/src/driver.rs` ≥ 1 件

---

## T1: `needs_continuation` 追加（`driver.rs`）

`is_definition` 関数の直前に追加する。

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

- [x] `needs_continuation` 関数を `is_definition` の直前に追加した
- [x] `pub fn` で宣言している

---

## T2: `handle_debug_cmd` 追加（`driver.rs`）

`handle_load_cmd` 関数の直前に追加する。

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

- [x] `handle_debug_cmd` 関数を `handle_load_cmd` の直前に追加した
- [x] `pub fn` で宣言している
- [x] シグネチャは `=` より前の部分を抽出している

---

## T3: `REPL_COMMANDS` 定数更新（`driver.rs`）

`:debug ` を末尾に追加する。

```rust
const REPL_COMMANDS: &[&str] = &[
    ":help", ":h", ":quit", ":q", ":reset", ":env",
    ":history", ":paste", ":type ", ":doc ", ":load ", ":save ", ":debug ",
];
```

- [x] `":debug "` を `REPL_COMMANDS` に追加した

---

## T4: `print_repl_help` 更新（`driver.rs`）

`:save` 行の直後に追加する。

```rust
println!("  :debug <stage>     show stage signature from current session");
```

- [x] `:debug <stage>` の説明行を `print_repl_help` に追加した

---

## T5: `cmd_repl` 更新 — マルチライン継続ループ（`driver.rs`）

`let line = line.trim_end_matches...` から `session.add_history(line);` までを置き換える。

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
```

- [x] マルチライン継続ループを `cmd_repl` に追加した
- [x] `needs_continuation` を使用している
- [x] `acc` は `String` で管理し `line: &str` として再バインドしている

---

## T6: `cmd_repl` 更新 — `:debug` ディスパッチ追加（`driver.rs`）

`:save ` アームの直後に挿入する。

```rust
_ if line.starts_with(":debug ") => {
    println!("{}", handle_debug_cmd(line[7..].trim(), &session));
}
```

- [x] `:debug ` ディスパッチアームを `:save ` の直後に追加した
- [x] `handle_debug_cmd` を呼び出している

---

## T7: `v60500_tests` モジュール追加（`driver.rs`）

`v60400_tests` の直前（上側）に挿入する。

```rust
// -- v60500_tests (v60.5.0) -- fav repl 強化 --
#[cfg(test)]
mod v60500_tests {
    use super::*;

    #[test]
    fn repl_load_pipeline_file() {
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
        assert!(needs_continuation("bind x <- \\"), "backslash at end should continue");
        assert!(needs_continuation("fn f("), "unclosed paren should continue");
        assert!(needs_continuation("stage S: Int -> Int = |x| {"), "unclosed brace should continue");
        assert!(!needs_continuation("bind x <- 42"), "complete line should not continue");
        assert!(!needs_continuation("fn f(x: Int) -> Int { x }"), "balanced line should not continue");
    }
}
```

- [x] `v60500_tests` モジュールを `v60400_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている（`handle_load_cmd` / `ReplSession` / `needs_continuation` アクセス用）
- [x] `repl_load_pipeline_file` テストが含まれている
  - `stage Double` と `stage AddOne` の両方が `session.def_names` に含まれることを確認
- [x] `repl_multiline_input` テストが含まれている
  - `needs_continuation` の 5 パターンを検証

---

## T8: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60500_tests::repl_load_pipeline_file` pass
- [x] `v60500_tests::repl_multiline_input` pass
- [x] 総テスト数 **3340** tests passed, 0 failed を確認

---

## T9: 事後処理

- [x] `versions/current.md` を v60.5.0 / 3340 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.5.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v61.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

Status: COMPLETE
