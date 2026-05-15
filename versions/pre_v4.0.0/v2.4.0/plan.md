# Favnir v2.4.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.4.0"` に変更。
`src/main.rs` の HELP テキストを `v2.4.0` に更新。

---

## Phase 1 — スタックトレース

### Phase 1-1: `TraceFrame` 型と `VMError` の拡張 (`src/backend/vm.rs`)

新しい構造体 `TraceFrame` を追加し、`VMError` に `stack_trace` フィールドを加える。

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraceFrame {
    pub fn_name: String,
    pub line: u32,
}

// VMError に追加
pub struct VMError {
    pub message: String,
    pub fn_name: String,   // 後方互換のため残す
    pub ip: usize,
    pub stack_trace: Vec<TraceFrame>,  // 新フィールド
}
```

`VMError` のデフォルト構築箇所（`fn_name: "<invalid>"` 等）に `stack_trace: vec![]` を追加する。

### Phase 1-2: `CallFrame` に `line` フィールドを追加 (`src/backend/vm.rs`)

```rust
pub struct CallFrame {
    pub fn_idx: usize,
    pub ip: usize,
    pub base: usize,
    pub n_locals: usize,
    pub line: u32,   // 追加: TrackLine opcode で更新される現在行
}
```

初期値は `0`。`CallFrame` を構築している全箇所（主に `frames.push(CallFrame { ... })`）に `line: 0` を追加する。

### Phase 1-3: TrackLine ハンドラで `CallFrame.line` を更新 (`src/backend/vm.rs`)

既存の `Opcode::TrackLine` ハンドラは `COVERED_LINES` のみ更新している。
`frame.line` も同時に更新するよう修正する。

```rust
x if x == Opcode::TrackLine as u8 => {
    // ... 既存のバイト読み出し処理 ...
    let line = u32::from_le_bytes([b0, b1, b2, b3]);
    frame.line = line;   // ← 追加
    COVERED_LINES.with(|c| {
        if let Some(set) = c.borrow_mut().as_mut() {
            set.insert(line);
        }
    });
}
```

### Phase 1-4: `VM` 構造体に `source_file` を追加 (`src/backend/vm.rs`)

```rust
pub struct VM {
    globals: Vec<VMValue>,
    stack: Vec<VMValue>,
    frames: Vec<CallFrame>,
    collect_frames: Vec<Vec<VMValue>>,
    emit_log: Vec<VMValue>,
    db_path: Option<String>,
    source_file: String,   // 追加
}
```

`VM::new_with_db_path` に `source_file: String` 引数を追加（または別メソッド `set_source_file`）。
既存テストに影響する場合は `String::new()` でデフォルト値を持つよう調整する。

### Phase 1-5: `vm_error_from_frames` を全スタック出力に修正 (`src/backend/vm.rs`)

```rust
fn vm_error_from_frames(
    artifact: &FvcArtifact,
    frames: &[CallFrame],
    message: String,
    source_file: &str,
) -> VMError {
    // 全フレームからトレースを構築（最新フレームが先頭）
    let stack_trace: Vec<TraceFrame> = frames
        .iter()
        .rev()
        .map(|frame| {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            TraceFrame { fn_name, line: frame.line }
        })
        .collect();

    let top = stack_trace.first().cloned().unwrap_or(TraceFrame {
        fn_name: "<none>".to_string(),
        line: 0,
    });

    VMError {
        message,
        fn_name: top.fn_name.clone(),
        ip: frames.last().map(|f| f.ip).unwrap_or(0),
        stack_trace,
    }
}
```

`vm_error_from_frames` を呼び出している全箇所に `&vm.source_file`（または適切な参照）を渡す。

### Phase 1-6: driver.rs のエラー表示を更新 (`src/driver.rs`)

```rust
// 変更前
.map_err(|e| format!("vm error in {} @{}: {}", e.fn_name, e.ip, e.message))

// 変更後
.map_err(|e| {
    let mut msg = format!("RuntimeError: {}", e.message);
    for frame in &e.stack_trace {
        msg.push_str(&format!("\n  at {} ({}:{})", frame.fn_name, source_file, frame.line));
    }
    msg
})
```

`source_file` は `cmd_run` / `cmd_exec` 等が持つファイルパス文字列から取得する。

---

## Phase 2 — Unknown 型警告

### Phase 2-1: チェッカーに Unknown 変数スキャンを追加 (`src/middle/checker.rs`)

`check_program` 完了後（エラーがない場合）、全バインド変数の型をスキャンする。
`Type::Unknown` になっているものを収集して `FavWarning` として返す。

```rust
// 新型
pub struct FavWarning {
    pub code: String,
    pub message: String,
    pub span: Span,
}

// check_program の戻り値を拡張
pub fn check_program(program: &Program) -> (Vec<TypeError>, Vec<FavWarning>)
```

または、チェッカーに `warnings: Vec<FavWarning>` フィールドを追加して蓄積する。

### Phase 2-2: `fav check` コマンドで警告を表示 (`src/driver.rs`)

```rust
// cmd_check 内
let (errors, warnings) = checker.check_program(&program);
for w in &warnings {
    eprintln!("warning[{}]: {}", w.code, w.message);
}
if !errors.is_empty() {
    // エラー表示...
    std::process::exit(1);
}
// 警告があっても exit code は 0
```

### Warning コード

| コード | 意味 |
|---|---|
| W001 | バインド変数の型が Unknown のまま解決されなかった |

---

## Phase 3 — ignored テスト解消

### Phase 3-1: `legacy_vm_test_bind_record_destruct` の `#[ignore]` 削除

v2.3.0 でコンパイラの分割 bind（`Pattern::Record`）対応が完成した。
`src/backend/vm_legacy_coverage_tests.rs` の当該テストから `#[ignore]` 行を削除し、
`cargo test` で通ることを確認する。

### Phase 3-2: `legacy_vm_test_bind_variant_destruct` の対応

テスト内容：
```rust
fn unwrap(w: Wrap) -> Int { bind Val(v) <- w; v }
```

`compiler.rs` の `compile_stmt_into` が `Stmt::Bind` + `Pattern::Variant` を処理しているか確認する。

**未対応の場合**: `Pattern::Variant(name, inner)` を以下に脱糖する：

```rust
// bind Val(v) <- w  →  match w { Val(v) => v  _ => panic }
Pattern::Variant(name, Some(inner_pat)) => {
    // 右辺を $tmp スロットに格納
    // match $tmp { name(inner) => { inner_bindings } _ => runtime_error }
    let tmp_slot = ctx.define_anon_slot();
    out.push(IRStmt::Bind(tmp_slot, rhs_ir));
    // match 式に変換
    // ...
}
```

または、パーサーが `bind Pat <- expr` を `match expr { Pat => { rest } }` に変換する
desugar レイヤーを追加する（より汎用的）。

---

## Phase 4 — テスト追加

### `src/backend/vm_stdlib_tests.rs`

**スタックトレーステスト**：

```rust
#[test]
fn test_runtime_error_shows_stack_trace() {
    // ゼロ除算でスタックトレースが出ることを確認
    // エラーメッセージに "at divide" と "at main" が含まれることを検証
}

#[test]
fn test_stack_trace_has_three_frames() {
    // 3 段以上のネスト呼び出しでトレースが 3 フレーム以上あることを確認
}
```

**ignored テスト解消**：

```rust
// vm_legacy_coverage_tests.rs から #[ignore] を削除して 2 件が通ることを確認
```

### `src/middle/checker.rs`

**Unknown 警告テスト**：

```rust
#[test]
fn test_w001_unknown_type_warning() {
    // 型解決できない bind 変数に W001 警告が出ることを確認
}
```

---

## Phase 5 — ドキュメント

- `versions/v2.4.0/langspec.md` を作成
  - スタックトレースの出力形式
  - W001 警告コードの説明
  - ignored テスト解消の説明（v2.4.0 での変化）
  - v2.3.0 との互換性

---

## テスト数の見込み

v2.3.0 ベースライン: 579

- ignored テスト 2 件が通過に変わる: +2（passed カウントが増加）
- スタックトレーステスト: +2
- Unknown 警告テスト: +1
- 合計目標: **584**（+5 程度）

---

## 注意点

### `vm_error_from_frames` の呼び出し箇所

`vm_error_from_frames` は複数の場所で呼ばれている。
`source_file` 引数を追加する場合、全呼び出し箇所を更新すること。
または `VM` のメソッドとして移動し、`self.source_file` を使う形にする方が安全。

### TrackLine が挿入されていない関数

コンパイラが `TrackLine` を挿入していない関数では `frame.line = 0` のまま。
スタックトレース表示で `line = 0` の場合は行番号を省略（`at fn (file)`）するか、
`<unknown>` と表示する。

### 既存テストの `VMError` 比較

`VMError` に `stack_trace` フィールドを追加すると、`PartialEq` で比較している
テストが `stack_trace: vec![]` を期待してしまう可能性がある。
`VMError` を構築している全テスト箇所に `stack_trace: vec![]` を追加すること。

### `VM::new_with_db_path` のシグネチャ変更

`source_file` 引数を追加すると、直接呼び出しているテストが多数あるため、
`source_file` は別途 `vm.set_source_file(path)` メソッドで設定する形にするか、
既存呼び出しに `String::new()` を渡すデフォルトを用意するのが安全。
