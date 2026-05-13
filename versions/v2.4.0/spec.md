# Favnir v2.4.0 Language Specification

作成日: 2026-05-13

---

## テーマ

エラーが起きたときに「どこで何が起きたか」を明確に示す。

v2.4.0 では 3 つの品質改善を行う。

1. **スタックトレース** — ランタイムエラー時に呼び出し履歴を表示する
2. **Unknown 型警告** — `fav check` で型が解決できない変数を警告する
3. **ignored テスト解消** — `#[ignore]` 付きテスト 2 件を通過させる

---

## 1. スタックトレース

### 出力形式

ランタイムエラー発生時、以下の形式でスタックトレースを表示する：

```
RuntimeError: division by zero
  at divide (math.fav:12)
  at process (pipeline.fav:34)
  at main (main.fav:5)
```

- 最新フレームが先頭
- `at <fn_name> (<source_file>:<line>)` の形式
- 行番号はソースコードの行番号（`TrackLine` opcode から追跡）
- ソースファイル名は実行時にドライバから VM へ渡す

### 現状と変更点

**現状の問題：**

- `CallFrame` に `line` フィールドがない → `TrackLine` opcode が `COVERED_LINES` を更新するが `CallFrame` 自身の行番号は追跡しない
- `VMError` は単一フレームのみ（`fn_name + ip`）でスタック全体を保持しない
- `vm_error_from_frames` は `frames.last()` だけを使用し、上位フレームを無視する
- `driver.rs` のエラー表示は `"vm error in {} @{}: {}"` の 1 行形式

**変更：**

- `CallFrame` に `line: u32` フィールドを追加。`TrackLine` ハンドラで更新する
- `VMError` に `stack_trace: Vec<TraceFrame>` を追加。`TraceFrame = { fn_name: String, line: u32 }` の新型
- `VM` 構造体に `source_file: String` フィールドを追加。ドライバから渡す
- `vm_error_from_frames` を全フレームからトレースを構築するよう修正
- `driver.rs` のエラー表示を複数行のスタックトレース形式に変更

### `TraceFrame` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraceFrame {
    pub fn_name: String,
    pub line: u32,
}
```

### VMError の変更

```rust
// 変更前
pub struct VMError {
    pub message: String,
    pub fn_name: String,
    pub ip: usize,
}

// 変更後
pub struct VMError {
    pub message: String,
    pub fn_name: String,   // 最上位フレーム（後方互換のため残す）
    pub ip: usize,
    pub stack_trace: Vec<TraceFrame>,
}
```

### driver.rs のエラー表示

```
// 変更前（1 行）
vm error in divide @42: division by zero

// 変更後（スタックトレース）
RuntimeError: division by zero
  at divide (math.fav:12)
  at process (pipeline.fav:34)
  at main (main.fav:5)
```

---

## 2. Unknown 型警告

### 概要

`fav check` 実行時、バインドした変数の型が `Type::Unknown` のままになっている場合に
警告 **W001** を出力する。エラーではなく警告なので、終了コードは 0 のまま。

### 出力形式

```
warning[W001]: type of `x` could not be resolved (Unknown)
  --> main.fav:5:10
```

### 対象

- `bind x <- expr` の `expr` が `Unknown` 型に解決された場合
- チェッカーが型を確定できなかった `Unknown` 型の変数

### 除外

- 内部コンパイラ用の `Unknown` 使用（`codegen.rs` 内のプレースホルダー等）はスキャン対象外
- ユーザーコードのバインド変数のみを対象とする

---

## 3. ignored テスト解消

### テスト 1: `legacy_vm_test_bind_record_destruct`

```rust
// v2.3.0 でコンパイラの分割 bind 対応が完成したため、通過するはず
fn sum(p: Point) -> Int { bind { x, y } <- p; x + y }
```

`#[ignore]` アトリビュートを削除し、テストが通ることを確認する。

### テスト 2: `legacy_vm_test_bind_variant_destruct`

```rust
// bind Val(v) <- w — Stmt::Bind + Pattern::Variant
fn unwrap(w: Wrap) -> Int { bind Val(v) <- w; v }
```

コンパイラの `compile_stmt_into` で `Stmt::Bind` + `Pattern::Variant` が未対応の場合、
`match w { Val(v) => v }` へ脱糖するか、直接対応する。

---

## 4. 互換性

- v2.3.0 までの全コードはそのまま有効
- `VMError` の新フィールド `stack_trace` はオプション（既存の `fn_name`/`ip` フィールドは残す）
- スタックトレースはエラーが発生した場合のみ表示（正常実行には影響しない）
- `fav check` の Unknown 警告はエラーではなく警告（既存のエラー判定に影響しない）
