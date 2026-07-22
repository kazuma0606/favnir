# Spec: v51.3.0 — ストリーミングバックプレッシャー制御

## 概要

`fav.toml` の `[stream]` セクションに `buffer_size` フィールドを追加し、
VM の `__streaming_pipeline` ハンドラに伝達して chunk サイズの上限制御（バックプレッシャー）を実装する。

```toml
# fav.toml
[stream]
buffer_size = 1000   # 1 チャンクあたりの最大要素数（chunk_size の上限）
```

- **`buffer_size`**: 1 チャンクあたりの最大要素数。`__streaming_pipeline` の chunk_size を `min(compiled_chunk_size, buffer_size)` に制約する。
- std::thread 環境での実装: chunk_size のキャッピングにより疑似的なバックプレッシャーを実現（tokio への移行後に真の bounded channel 化を予定）。

---

## 背景・現状確認

| 項目 | 現状 |
|---|---|
| `StreamConfig` struct (`toml.rs`) | `watermark_delay` / `late_policy` フィールドのみ。`buffer_size` は**未定義** |
| `[stream]` セクション解析 (`toml.rs` 行 842) | `watermark_delay` / `late_policy` のみ解析。`buffer_size` は**未解析** |
| VM struct | `stream_buffer_size` フィールド**未定義** |
| `__streaming_pipeline` ハンドラ (`vm.rs` 行 5536〜5584) | chunk_size を引数から取得（デフォルト 512）。config 参照**なし** |
| `compile_streaming_pipeline` (`compiler.rs` 行 1139) | `ann.chunk_size.unwrap_or(512)` を静的にコンパイル。config 参照**なし** |
| `parse_fav_toml_pub` (`toml.rs` 行 352) | 公開ラッパー（`parse_fav_toml` は private — テストはこちらを使用） |

---

## 成果物仕様

### 1. `toml.rs` — `StreamConfig.buffer_size` フィールド追加

```rust
pub struct StreamConfig {
    pub watermark_delay: Option<u32>,
    pub late_policy: Option<String>,
    /// chunk_size の上限（バックプレッシャー制御用）。デフォルト None（制約なし）。(v51.3.0)
    /// 0 は None 相当として扱う（chunks(0) によるパニック防止）。
    pub buffer_size: Option<usize>,
}
```

`parse_fav_toml` の `"stream"` セクション処理（行 842〜857）に `buffer_size` キーを追加:

```rust
"buffer_size" => {
    // 0 は制約なし（None 相当）として扱う
    current.buffer_size = val.trim_matches('"').parse::<usize>().ok().filter(|&n| n > 0);
}
```

### 2. `backend/vm.rs` — VM struct に `stream_buffer_size` フィールド追加

```rust
pub struct VM {
    // ... 既存フィールド ...
    /// fav.toml [stream].buffer_size から注入される chunk_size 上限 (v51.3.0).
    /// None = 制約なし（デフォルト）。wasm32 でも使用可能。
    stream_buffer_size: Option<usize>,
}
```

`VM::new_with_db_path` 初期化ブロックに `stream_buffer_size: None` を追加。

**新しい静的実行メソッド**（`impl VM` ブロック内に定義 — `invoke_function` が private のため）:

```rust
/// buffer_size を設定して streaming pipeline を実行する (v51.3.0).
pub fn run_with_stream_buffer_size(
    artifact: &FvcArtifact,
    fn_idx: usize,
    args: Vec<Value>,
    buffer_size: Option<usize>,
) -> Result<Value, VMError> {
    let mut vm = VM::new_with_db_path(artifact, None);
    vm.stream_buffer_size = buffer_size;
    let args_vm: Vec<VMValue> = args.into_iter().map(VMValue::from).collect();
    let result = vm.invoke_function(artifact, fn_idx, args_vm)?;
    Ok(Value::from(result))
}
```

### 3. `backend/vm.rs` — `__streaming_pipeline` ハンドラにバックプレッシャー適用

`__streaming_pipeline` の chunk_size 計算（行 5541〜5543）を拡張:

```rust
// 変更前（3行）:
let chunk_size = match args_iter.next() {
    Some(VMValue::Int(n)) if n > 0 => n as usize,
    _ => 512,
};

// 変更後（+4行）:
let compiled_chunk_size = match args_iter.next() {
    Some(VMValue::Int(n)) if n > 0 => n as usize,
    _ => 512,
};
// fav.toml [stream].buffer_size が設定されている場合 chunk_size の上限として適用 (v51.3.0)
// 将来 tokio 化時に FuturesUnordered / sync_channel による真のバックプレッシャーに置換予定
let chunk_size = if let Some(buf) = self.stream_buffer_size {
    compiled_chunk_size.min(buf)
} else {
    compiled_chunk_size
};
```

---

## テスト仕様

### `stream_buffer_size_config`

```rust
use crate::toml::parse_fav_toml_pub;

let toml_str = "[stream]\nbuffer_size = 500\n";
let config = parse_fav_toml_pub(toml_str);
assert_eq!(config.stream.as_ref().unwrap().buffer_size, Some(500));
```

### `stream_backpressure_blocks`

既存 `streaming_pipeline_executes` テスト（行 33940）のパターンに従う。
stage は `List<Int> -> List<Int>` シグネチャ（`__streaming_pipeline` が chunk を List として渡すため scalar-to-scalar は不可）。

```rust
let src = r#"
stage double_list: List<Int> -> List<Int> = |xs| {
  List.map(xs, |x| { x * 2 })
}
#[streaming]
seq Pipeline = double_list
"#;
// Parser → build_artifact → fn_idx_by_name("Pipeline")
// 入力: Value::List([1,2,3,4,5,6])
// buffer_size = Some(2) で実行
let result = VM::run_with_stream_buffer_size(&artifact, pipeline_idx, vec![input], Some(2));
// 戻り値: List([2,4,6,8,10,12]) — chunk_size が 2 に制約されても正しく処理される
assert!(result.is_ok());
if let Ok(Value::List(items)) = result {
    assert_eq!(items.len(), 6);
    let ints: Vec<i64> = ...;
    assert_eq!(ints, vec![2, 4, 6, 8, 10, 12]);
}
```

---

## バージョン要件

- `fav/Cargo.toml` version: `51.3.0`
- テスト数: 3117 → **3119**（純増 +2）
  - `v51300_tests` 3 件追加、`v51200_tests::cargo_toml_version_is_51_2_0` 1 件削除
  - 純増: +3 − 1 = **+2**

---

## 完了条件

- `fav.toml` の `[stream] buffer_size = N` が解析できる
- `VM::run_with_stream_buffer_size` が chunk_size を N で制約する
- `cargo test` 3119 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51300_tests` 3 件 pass

---

## リスク・制約

- `buffer_size = 0` は `None` 相当として扱う（`chunks(0)` によるパニック防止。`.filter(|&n| n > 0)` でガード）。
- `run_with_stream_buffer_size` は `impl VM` ブロック内に定義すること（`invoke_function` が private のため）。
- wasm32 環境: `chunk_arena` 非使用パスがあるが `stream_buffer_size` は chunk_size のキャッピングのみのため wasm32 でも安全。
- tokio への移行（v52.x+）まで真の bounded channel 実装はなし。コメントで明記する。
