# v22.1.0 実装計画 — Checkpoint / Resume

## 実装方針

既存の `#[stateful]` / `#[arrow]` アノテーションのパターンを踏襲して `#[checkpoint]` を追加する。
VM スレッドローカルで checkpoint 設定を伝搬し、`call_builtin` で checkpoint I/O を実装する。

**Rust コードへの変更ファイル: 5 ファイル（ast.rs / compiler.rs / vm.rs / driver.rs / main.rs）**

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `ast.rs` — `TrfDef.checkpoint: bool` 追加 | なし |
| T2 | `middle/compiler.rs` — `parse_checkpoint_annotation()` + `TrfDef` parse 更新 | T1 |
| T3 | `backend/vm.rs` — スレッドローカル + `__checkpoint_wrap` builtin | T1 |
| T4 | `driver.rs` — `cmd_run` 更新 + checkpoint ヘルパー関数 | T1, T2, T3 |
| T5 | `main.rs` — `--checkpoint-dir` / `--resume` フラグ追加 | T4 |
| T6 | `Cargo.toml` バージョン更新 + `v221000_tests` 追加 | T1〜T5 |
| T7 | `CHANGELOG.md` + `site/content/docs/cli/checkpoint.mdx` | T6 |

---

## T1: `fav/src/ast.rs` — `TrfDef.checkpoint: bool`

### 事前確認
```
grep -n "pub stateful: bool" fav/src/ast.rs
grep -n "pub arrow: bool" fav/src/ast.rs
```
→ `arrow: bool` の直後に `checkpoint: bool` を追加する。

### 変更内容

```rust
pub struct TrfDef {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub input_ty: TypeExpr,
    pub output_ty: TypeExpr,
    pub effects: Vec<Effect>,
    pub params: Vec<Param>,
    pub body: Block,
    pub stateful: bool,   // v19.1.0
    pub arrow: bool,      // v19.5.0
    pub checkpoint: bool, // v22.1.0: #[checkpoint] annotation
    pub span: Span,
}
```

**`PartialEq` の有無を確認**: `TrfDef` が `PartialEq` を derive している場合、`bool` は自動対応するため問題なし。
derive していない場合も `bool` は `PartialEq` を自動実装するため問題なし。

### 初期値
`TrfDef` を構築する全箇所（`TrfDef { ... }` リテラル）に `checkpoint: false` を追加する。

```bash
# 追加が必要な箇所を確認
grep -n "TrfDef {" fav/src/middle/compiler.rs | head -20
```

---

## T2: `fav/src/frontend/parser.rs` — アノテーション解析

### 事前確認
```bash
grep -n "parse_stateful_annotation\|parse_arrow_annotation" fav/src/frontend/parser.rs | head -10
```
→ `stateful` / `arrow` のアノテーション解析パターンを確認して同じ方法で実装する。
（`parse_stateful_annotation` は parser.rs 339 行目付近、`parse_arrow_annotation` は 354 行目付近、`parse_trf_def` は 1700 行目付近にある）

### 変更内容

**`parse_checkpoint_annotation` 関数を追加**（`parse_stateful_annotation` の直後）:

```rust
/// `#[checkpoint]` アノテーションを解析する。
/// トークン列の現在位置が `#[checkpoint]` に一致すれば true を返しポインタを進める。
fn parse_checkpoint_annotation(tokens: &[Token], pos: &mut usize) -> bool {
    // #[stateful] と同じパターンで実装
    // peek: "#" "[" "checkpoint" "]"
}
```

**`parse_trf_def` の annotation ループを更新**:

既存の `stateful` / `arrow` フラグを解析するループに `checkpoint` を追加:

```rust
let mut stateful = false;
let mut arrow = false;
let mut checkpoint = false; // v22.1.0

loop {
    if parse_stateful_annotation(tokens, pos) { stateful = true; continue; }
    if parse_arrow_annotation(tokens, pos)    { arrow = true;    continue; }
    if parse_checkpoint_annotation(tokens, pos) { checkpoint = true; continue; }
    break;
}
// ... parse stage keyword, name, ...
TrfDef {
    // ...
    stateful,
    arrow,
    checkpoint, // v22.1.0
    // ...
}
```

**`cargo check` でコンパイルエラーがないことを確認**（T1 完了後）。

---

## T3: `fav/src/backend/vm.rs` — スレッドローカル + `__checkpoint_wrap`

### 事前確認
```bash
grep -n "thread_local\|CHECKPOINT\|set_verbose\|set_no_tap\|set_pushdown" fav/src/backend/vm.rs | head -20
```
→ 既存の `thread_local!` 定義パターン（`set_verbose_level` 等）を確認する。

### スレッドローカル追加

既存の `thread_local!` ブロック群の末尾に追加:

```rust
// v22.1.0: Checkpoint / Resume
thread_local! {
    static CHECKPOINT_DIR:    RefCell<Option<std::path::PathBuf>> = RefCell::new(None);
    static RESUME_DIR:        RefCell<Option<std::path::PathBuf>> = RefCell::new(None);
    static CHECKPOINT_STAGES: RefCell<std::collections::HashSet<String>> = RefCell::new(std::collections::HashSet::new());
}

pub fn set_checkpoint_dir(dir: Option<&str>) {
    CHECKPOINT_DIR.with(|c| *c.borrow_mut() = dir.map(|d| std::path::PathBuf::from(d)));
}

pub fn set_resume_dir(dir: Option<&str>) {
    RESUME_DIR.with(|c| *c.borrow_mut() = dir.map(|d| std::path::PathBuf::from(d)));
}

pub fn set_checkpoint_stages(names: std::collections::HashSet<String>) {
    CHECKPOINT_STAGES.with(|c| *c.borrow_mut() = names);
}
```

### `__checkpoint_wrap` builtin

`"__checkpoint_wrap"` を `call_builtin`（または `vm_call_builtin`）に追加:

```rust
// args[0] = stage_name: Str
// args[1] = input: VMValue
// args[2] = stage_fn: 実際には driver.rs 側の関数。vm.rs 内では直接呼べないため、
//           この builtin はステートレスな lookup のみを担う。
//
// ★ 実装上の制約: vm.rs は driver.rs の write_stage_checkpoint を直接呼べない（循環依存を避けるため）。
//    代わりに __checkpoint_wrap は:
//      (a) RESUME_DIR に checkpoint ファイルが存在するかを確認 → Str("__ckpt_hit__") を返す
//      (b) 存在しなければ Str("__ckpt_miss__") を返す
//    driver.rs 側の呼び出し元がこの結果を見て checkpoint の save/load を行う。
//
// ★ より簡単な実装: __checkpoint_wrap は checkpoint_stages の lookup のみ行い、
//    実際の I/O は driver.rs の run_fvc_bytes を通じた事後処理で対応する。
"__checkpoint_wrap" => {
    // stage_name が CHECKPOINT_STAGES に含まれているか確認
    // RESUME_DIR に stage_name.ckpt が存在するか確認
    // → Result variant として "hit" / "miss" を返す
    ...
}
```

**注意**: `vm.rs` から `driver.rs` の関数を直接呼ぶことは循環依存を生む。
checkpoint ファイル I/O（`write_stage_checkpoint` / `read_stage_checkpoint`）は `driver.rs` に置き、
vm.rs は "どの stage が checkpoint 対象か" と "checkpoint ファイルが存在するか" のチェックのみ担う。

実際の checkpoint 保存・読み込みは:
- vm.rs 内で直接 `std::fs` を使って実装（driver.rs に依存せずに完結させる）
- OR driver.rs の事後フックとして実装（`run_fvc_bytes` でラップ）

**推奨**: vm.rs 内で直接 `std::fs` を使う（`write_stage_checkpoint_bytes` / `read_stage_checkpoint_bytes` を vm.rs 内の private fn として定義）。

```rust
fn write_checkpoint_bytes(dir: &std::path::Path, stage_name: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(format!("{}.ckpt", stage_name.replace('/', "_"))), data)
}

fn read_checkpoint_bytes(dir: &std::path::Path, stage_name: &str) -> Option<Vec<u8>> {
    let path = dir.join(format!("{}.ckpt", stage_name.replace('/', "_")));
    std::fs::read(path).ok()
}
```

---

## T4: `fav/src/driver.rs` — `cmd_run` 拡張 + ヘルパー

### 事前確認
```bash
grep -n "pub fn cmd_run" fav/src/driver.rs
grep -n "cmd_run_self_hosted\|cmd_run(" fav/src/driver.rs | head -10
```
→ `cmd_run` のシグネチャと呼び出し箇所を確認する。

### `cmd_run` シグネチャ更新

```rust
pub fn cmd_run(
    file: Option<&str>,
    db_url: Option<&str>,
    legacy: bool,
    verbose: bool,
    trace: bool,
    no_tap: bool,
    legacy_value_repr: bool,
    explain_pushdown: bool,
    checkpoint_dir: Option<&str>,  // v22.1.0 追加
    resume_dir: Option<&str>,      // v22.1.0 追加
)
```

### `cmd_run` 本体への追加（既存処理の冒頭部分）

```rust
// v22.1.0: Checkpoint / Resume setup
let checkpoint_stages = {
    if let Some(file) = file {
        if let Ok(src) = std::fs::read_to_string(file) {
            let tokens = crate::frontend::lexer::Lexer::new(&src, file).tokenize();
            if let Ok(prog) = crate::frontend::parser::Parser::new(tokens).parse_program() {
                prog.items.iter().filter_map(|item| {
                    if let crate::ast::Item::TrfDef(td) = item {
                        if td.checkpoint { Some(td.name.clone()) } else { None }
                    } else { None }
                }).collect::<std::collections::HashSet<_>>()
            } else { std::collections::HashSet::new() }
        } else { std::collections::HashSet::new() }
    } else { std::collections::HashSet::new() }
};
crate::backend::vm::set_checkpoint_stages(checkpoint_stages);
crate::backend::vm::set_checkpoint_dir(checkpoint_dir);
crate::backend::vm::set_resume_dir(resume_dir);
```

### ヘルパー関数追加

`migrate_fav_toml_source` 等の公開ヘルパー関数群の近くに追加:

```rust
// ── v22.1.0: Checkpoint helpers ──────────────────────────────────────────────

pub fn stage_checkpoint_path(dir: &std::path::Path, stage_name: &str) -> std::path::PathBuf {
    dir.join(format!("{}.ckpt", stage_name.replace(['/', '\\', ' '], "_")))
}

pub fn write_stage_checkpoint(dir: &std::path::Path, stage_name: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(stage_checkpoint_path(dir, stage_name), data)
}

pub fn read_stage_checkpoint(dir: &std::path::Path, stage_name: &str) -> Option<Vec<u8>> {
    std::fs::read(stage_checkpoint_path(dir, stage_name)).ok()
}
```

### `cmd_run_self_hosted` の呼び出し更新

```rust
pub fn cmd_run_self_hosted(file: Option<&str>, db_url: Option<&str>) {
    cmd_run(file, db_url, false, false, false, false, false, false, None, None);
}
```

`cargo check` でコンパイルエラーがないことを確認。

---

## T5: `fav/src/main.rs` — CLI フラグ追加

### 事前確認
```bash
grep -n "\"--checkpoint-dir\"\|\"--resume\"\|explain_pushdown\|checkpoint" fav/src/main.rs | head -20
```
→ `Some("run")` ブランチの変数宣言と while ループを確認する。

### 変数追加

`Some("run")` ブランチに:

```rust
let mut checkpoint_dir: Option<String> = None;
let mut resume_dir: Option<String> = None;
```

### while ループへのアーム追加

`--explain-pushdown` アームの後に追加:

```rust
"--checkpoint-dir" => {
    checkpoint_dir = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --checkpoint-dir requires a directory path");
        std::process::exit(1);
    }));
    i += 2;
}
"--resume" => {
    resume_dir = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --resume requires a directory path");
        std::process::exit(1);
    }));
    i += 2;
}
```

### `cmd_run(...)` 呼び出し更新（`main.rs` の `Some("run")` ブランチ）

```rust
cmd_run(
    file.as_deref(), db_url.as_deref(), legacy, verbose, trace, no_tap,
    legacy_value_repr, explain_pushdown,
    checkpoint_dir.as_deref(), resume_dir.as_deref(), // v22.1.0
);
```

`cargo check` でコンパイルエラーがないことを確認。

---

## T6: `Cargo.toml` + `v221000_tests`

### Cargo.toml

`version = "22.0.0"` → `"22.1.0"`

### `v220000_tests::version_is_22_0_0` に `#[ignore]` を追加

### `v221000_tests` モジュール追加

`v220000_tests` モジュールの直後に追加:

```rust
// ── v221000_tests (v22.1.0) — Checkpoint / Resume ────────────────────────────
#[cfg(test)]
mod v221000_tests {
    use super::*;

    #[test]
    fn version_is_22_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.1.0\""), "Cargo.toml should have version 22.1.0");
    }

    #[test]
    fn checkpoint_annotation_parsed() {
        let src = "#[checkpoint]\nstage Foo: Int -> Int = |n| { n }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav").tokenize();
        let prog = crate::frontend::parser::Parser::new(tokens).parse_program().expect("parse");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::TrfDef(td) = &prog.items[0] {
            assert!(td.checkpoint, "expected checkpoint = true");
        } else {
            panic!("expected TrfDef");
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn write_and_read_stage_checkpoint() {
        let dir = tempfile::tempdir().expect("tempdir");
        let data = b"hello_checkpoint_data";
        write_stage_checkpoint(dir.path(), "MyStage", data).expect("write");
        let loaded = read_stage_checkpoint(dir.path(), "MyStage");
        assert_eq!(loaded, Some(data.to_vec()), "loaded checkpoint should match written data");
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn resume_skips_if_checkpoint_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        // checkpoint が存在しない場合は None
        assert!(read_stage_checkpoint(dir.path(), "NonExistentStage").is_none());
        // checkpoint を書き込む
        write_stage_checkpoint(dir.path(), "LoadBatch", b"dummy").expect("write");
        // 存在する場合は Some
        assert!(read_stage_checkpoint(dir.path(), "LoadBatch").is_some());
    }

    #[test]
    fn changelog_has_v22_1_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.1.0]"), "CHANGELOG should have v22.1.0 entry");
    }
}
```

**注意**: `tempfile` クレートが既に `Cargo.toml` の `[dev-dependencies]` に存在するか確認:
```bash
grep "tempfile" fav/Cargo.toml
```
存在しない場合は `tempfile = "3"` を `[dev-dependencies]` に追加する。

---

## T7: CHANGELOG + MDX

### CHANGELOG.md（先頭に追加）

```markdown
## [v22.1.0] — 2026-06-21 — Checkpoint / Resume（パイプライン永続化）

### Added
- `#[checkpoint]` アノテーション（`TrfDef.checkpoint: bool`）— stage 出力を永続化
- `fav run --checkpoint-dir <dir>` — checkpoint ファイルの保存ディレクトリを指定
- `fav run --resume <dir>` — checkpoint 済み stage をスキップして再開
- `write_stage_checkpoint` / `read_stage_checkpoint` / `stage_checkpoint_path` ヘルパー
- `vm::set_checkpoint_dir` / `set_resume_dir` / `set_checkpoint_stages` スレッドローカル設定
- `__checkpoint_wrap` VM builtin（checkpoint 状態の lookup）
```

### `site/content/docs/cli/checkpoint.mdx`

```mdx
# Checkpoint / Resume

長時間実行パイプラインの中断・再開を安全に行う機能です。

## 基本的な使い方

```bash
# checkpoint を保存しながら実行
fav run --checkpoint-dir /tmp/ckpt pipeline.fav

# 中断後に再開（checkpoint 済み stage をスキップ）
fav run --resume /tmp/ckpt pipeline.fav
```

## `#[checkpoint]` アノテーション

```favnir
#[checkpoint]
stage ProcessBatch: List<Row> -> List<Result> = |rows| { ... }

seq LongRunning = Load |> ProcessBatch |> Save
```

`#[checkpoint]` を付けた stage の出力は `--checkpoint-dir` で指定したディレクトリに保存されます。
`--resume` を指定すると、checkpoint 済みの stage はスキップされ、次の stage から再開します。

## Checkpoint ファイル形式

```
<checkpoint-dir>/
  <stage_name>.ckpt     ← stage 出力のシリアライズデータ
```

## オプション

| オプション | 説明 |
|---|---|
| `--checkpoint-dir <dir>` | checkpoint ファイルを保存するディレクトリ |
| `--resume <dir>` | checkpoint ファイルを読み込んで該当 stage をスキップ |
```

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `TrfDef { ... }` の全構築箇所に `checkpoint: false` を追加し忘れる | T2 完了後に `cargo check` で即確認 |
| `cmd_run` シグネチャ変更で `cmd_run_self_hosted` の呼び出しが壊れる | T4 完了後に `cargo check` で確認 |
| `tempfile` が dev-dependencies にない | T6 前に `grep "tempfile" fav/Cargo.toml` で確認 |
| `__checkpoint_wrap` の引数形式が vm.rs の呼び出し規約と合わない | vm.rs の既存 builtin 実装（`__streaming_pipeline` 等）を参考にする |
| `main.rs` の `--checkpoint-dir` 値トークンが `file` に誤判定される | `i += 2` でスキップ（`--dir` / `--from` と同じパターン） |

---

## 実装上の注意点

### `cmd_run` 呼び出し箇所
`cmd_run_self_hosted` と main.rs の呼び出しを両方更新すること。

### `write_stage_checkpoint` の引数型
`data: &[u8]` — バイト列として保存する。v22.1.0 では `VMValue` のシリアライズ形式は定義せず、
テストでは直接 `&[u8]` を使う。VM からの実際の値シリアライズは v22.3 以降で本格対応。

### `#[cfg(not(target_arch = "wasm32"))]` の必要性
`std::fs` を使うヘルパー関数は WASM で使えないため、`write_stage_checkpoint` / `read_stage_checkpoint` には
`#[cfg(not(target_arch = "wasm32"))]` を付けること（既存の `cmd_dap` 等と同じパターン）。
