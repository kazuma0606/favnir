# v60.5.0 Spec — `fav repl` 強化（`:load` / `:debug` / マルチライン入力）

Date: 2026-07-30

---

## 概要

v9.10.0 で実装した `fav repl` に 3 つの機能を追加する。

1. `:load <file>` — pipeline.fav を読み込み stage 定義をセッションに登録
   （基盤は v9.10 実装済み。`stage` 定義に対するテストを追加する）
2. `:debug <stage>` — 指定 stage のシグネチャ情報をターミナルに表示（NEW）
3. マルチライン入力 — 行末 `\` または未閉じカッコ/ブレースで次行に継続（NEW）

---

## 既存実装との関係

| 機能 | 実装状況 | 今バージョンの作業 |
|---|---|---|
| `:load <file>` | v9.10 で実装済み（`handle_load_cmd`） | stage 定義ロードのテスト追加のみ |
| `:debug <stage>` | 未実装 | `handle_debug_cmd` を新規追加 |
| マルチライン入力 | 未実装 | `needs_continuation` を新規追加、`cmd_repl` に組み込む |

---

## `:load <file>` の現状確認

`handle_load_cmd` は v9.10 でフル実装済み:
- ファイルを読み込み `check_source_str` で型チェック
- `extract_top_level_names` で `fn`/`stage`/`seq`/`type`/`effect` 定義名を抽出
- `session.def_names` と `session.definitions` に追加

既存テスト `repl_load_file`（v91000_tests）は `fn` 定義のみ検証。
本バージョンで `stage` 定義の正常ロードを検証するテスト `repl_load_pipeline_file` を追加する。

`stage` 定義の正しい Favnir 構文（実装済みパーサー準拠）:
```
stage Double: Int -> Int = |x| { x + x }
```

---

## 新規追加関数

### `needs_continuation(line: &str) -> bool`

行末 `\` または未閉じカッコ/ブレース/ブラケットを検出し、次行継続が必要かを返す。

```rust
pub fn needs_continuation(line: &str) -> bool {
    let t = line.trim_end();
    if t.ends_with('\\') { return true; }
    let opens: i64 = t.chars().filter(|&c| c == '(' || c == '{' || c == '[').count() as i64;
    let closes: i64 = t.chars().filter(|&c| c == ')' || c == '}' || c == ']').count() as i64;
    opens > closes
}
```

追加位置: `is_definition` 関数の直前。

### `handle_debug_cmd(stage_name: &str, session: &ReplSession) -> String`

`session.definitions` を走査し、指定 stage のシグネチャ行（`=` より前の部分）を返す。

**スコープ注記**: ロードマップ記載の `effects=[!IO]` 形式のエフェクト表示は本バージョンではスコープ外とする。
`stage Name: InputType -> OutputType` のシグネチャ部分のみを表示する（エフェクト情報の静的抽出は v60.6 以降）。
また、`:debug` は `stage` 定義のみを対象とし、`fn` 定義には非対応（スコープ外）。

```rust
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

追加位置: `handle_load_cmd` の直前。

---

## `cmd_repl` への変更

### 1. マルチライン継続ループ

初期行読み取り後、`needs_continuation` でチェックしながら継続行を結合する。

```rust
// 初期行読み取り後
let initial = line.trim_end_matches(['\n', '\r']).trim().to_string();
let mut acc = initial.clone();
// マルチライン継続: 行末 `\` または未閉じカッコ/ブレースで次行を結合
while needs_continuation(&acc) {
    // 行末 `\` を除去して継続
    if acc.trim_end().ends_with('\\') {
        let t = acc.trim_end();
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
    let cont = cont.trim_end_matches(['\n', '\r']).trim();
    acc.push(' ');
    acc.push_str(cont);
}
let line: &str = acc.trim();
```

### 2. `:debug` ディスパッチ追加

`:save ` アームの直後に挿入する。

```rust
_ if line.starts_with(":debug ") => {
    let out = handle_debug_cmd(line[7..].trim(), &session);
    println!("{}", out);
}
```

### 3. `REPL_COMMANDS` 更新

```rust
const REPL_COMMANDS: &[&str] = &[
    ":help", ":h", ":quit", ":q", ":reset", ":env",
    ":history", ":paste", ":type ", ":doc ", ":load ", ":save ", ":debug ",
];
```

### 4. `print_repl_help` 更新

`:paste ... :end` 行（最終コマンド行）の直後、`println!()` 空行の前に追加:
```rust
println!("  :debug <stage>     show stage signature from current session");
```
（実際の `print_repl_help` 末尾: `:save` → `:history` → `:paste` → **挿入位置** → 空行 → 説明文）

---

## ターミナル表示例

```
favnir> :load pipeline.fav
loaded: Double, AddOne
favnir> :debug Double
[debug] stage Double: Int -> Int
favnir> bind x <-
     |   42 +
     |   58
x = 100
```

---

## テスト

対象ファイル: `fav/src/driver.rs`

テスト数: ベース **3338** + 2 = **3340** tests passed, 0 failed

テストモジュール名: `v60500_tests`（`v60400_tests` の直前に挿入）

### `repl_load_pipeline_file`

`stage` 定義を含む pipeline ファイルを `:load` でセッションに登録できることを確認。

```rust
#[test]
fn repl_load_pipeline_file() {
    // use super::* で handle_load_cmd / ReplSession にアクセス（plan.md / tasks.md と統一）
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
```

### `repl_multiline_input`

`needs_continuation` が行末 `\` / 未閉じカッコ / バランス済み行を正しく判定することを確認。

```rust
#[test]
fn repl_multiline_input() {
    // use super::* で needs_continuation にアクセス
    // Known limitation: 文字列リテラル内の末尾 `\` も継続扱いされる（フルパーサー非使用の制限）
    // 行末 `\` → 継続
    assert!(needs_continuation("bind x <- \\"), "backslash at end should continue");
    // 未閉じカッコ → 継続
    assert!(needs_continuation("fn f("), "unclosed paren should continue");
    // 未閉じブレース → 継続
    assert!(needs_continuation("stage S: Int -> Int = |x| {"), "unclosed brace should continue");
    // 完結行 → 継続なし
    assert!(!needs_continuation("bind x <- 42"), "complete line should not continue");
    // バランス済み → 継続なし
    assert!(!needs_continuation("fn f(x: Int) -> Int { x }"), "balanced line should not continue");
}
```

---

## 注意事項

- `Cargo.toml` version は `"60.0.0"` のまま変更しない
- `handle_debug_cmd` と `needs_continuation` は `pub fn` とする（テストモジュールから `use super::*` でアクセスするため）
- `v60500_tests` は `v60400_tests` の直前（上側）に挿入する
- `stage` 構文: `stage Name: InputType -> OutputType = |param| { body }` を使用（Favnir 仕様準拠）
- **スコープ縮小 — `:debug` エフェクト表示なし**: ロードマップ記載の `effects=[!IO]` 形式は本バージョンではスコープ外。シグネチャ（型情報のみ）を表示する
- **`:debug` は `stage` 定義専用**: `fn` 定義は対象外（`:debug doublefn` → "not found" が正常動作）
- **Known limitation — `needs_continuation` の `\` 検出**: 文字列リテラル内の末尾 `\` も継続扱いされる（フルパーサー非使用の制約。修正は将来スコープ）
- `stdin.lock()` / `stdout.lock()` は継続ループ内でも一時式として取得・解放する（既存 `cmd_repl` の L13629 パターン踏襲）
- テスト実行: `cargo test -j 8 -- --test-threads=8`
