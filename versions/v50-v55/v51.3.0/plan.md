# Plan: v51.3.0 — ストリーミングバックプレッシャー制御

## 実装方針

変更ファイルは 3 つのみ（`toml.rs` / `vm.rs` / `driver.rs`）。
AST/IR/コンパイラーには触れない最小変更で実装する。

---

## 実装ステップ

### Step 1: `toml.rs` — `StreamConfig.buffer_size` 追加 + 解析

`StreamConfig` struct に `buffer_size: Option<usize>` を追加:

```rust
pub struct StreamConfig {
    pub watermark_delay: Option<u32>,
    pub late_policy: Option<String>,
    /// chunk_size の上限（バックプレッシャー制御用）。デフォルト None（制約なし）。(v51.3.0)
    pub buffer_size: Option<usize>,
}
```

`parse_fav_toml` の `"stream"` セクション処理（行 842〜857）に `buffer_size` キーを追加:

```rust
"buffer_size" => {
    // 0 は制約なし（None 相当）として扱う（chunks(0) パニック防止）
    current.buffer_size = val.trim_matches('"').parse::<usize>().ok().filter(|&n| n > 0);
}
```

### Step 2: `backend/vm.rs` — `stream_buffer_size` フィールド追加 + 初期化

VM struct（行 1484 付近の `db_path` の後）に追加:

```rust
/// fav.toml [stream].buffer_size から注入される chunk_size 上限 (v51.3.0).
/// None = 制約なし（デフォルト）。
stream_buffer_size: Option<usize>,
```

`VM::new_with_db_path`（行 1670〜1714）の初期化ブロックに追加:

```rust
stream_buffer_size: None,
```

### Step 3: `backend/vm.rs` — `run_with_stream_buffer_size` 静的メソッド追加

`VM::run`（行 1774）の直後（`run_with_db_path` の前）に追加。
`invoke_function` が private のため、`impl VM` ブロック内に定義:

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

### Step 4: `backend/vm.rs` — `__streaming_pipeline` にバックプレッシャー適用

`__streaming_pipeline` ハンドラ（行 5541〜5543）の chunk_size 計算を拡張:

```rust
// 変更前:
let chunk_size = match args_iter.next() {
    Some(VMValue::Int(n)) if n > 0 => n as usize,
    _ => 512,
};

// 変更後:
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

### Step 5: `driver.rs` — v51300_tests 追加

3 件追加、1 件削除:
- 追加: `cargo_toml_version_is_51_3_0`, `stream_buffer_size_config`, `stream_backpressure_blocks`
- 削除: `v51200_tests::cargo_toml_version_is_51_2_0`

**`stream_buffer_size_config`** は `crate::toml::parse_fav_toml_pub`（private の `parse_fav_toml` ではなく公開ラッパー）を使用:

```rust
use crate::toml::parse_fav_toml_pub;
let config = parse_fav_toml_pub("[stream]\nbuffer_size = 500\n");
assert_eq!(config.stream.as_ref().unwrap().buffer_size, Some(500));
```

**`stream_backpressure_blocks`** は既存 `streaming_pipeline_executes`（行 33940）のパターンに従い、`List<Int> -> List<Int>` シグネチャの stage を使用:

```rust
let src = r#"
stage double_list: List<Int> -> List<Int> = |xs| {
  List.map(xs, |x| { x * 2 })
}
#[streaming]
seq Pipeline = double_list
"#;
// Parser → build_artifact → pipeline_idx
let input = Value::List(vec![Int(1), Int(2), Int(3), Int(4), Int(5), Int(6)]);
let result = VM::run_with_stream_buffer_size(&artifact, pipeline_idx, vec![input], Some(2));
assert!(result.is_ok());
if let Ok(Value::List(items)) = result {
    assert_eq!(items.len(), 6);
    let ints: Vec<i64> = items.iter()
        .filter_map(|v| if let Value::Int(n) = v { Some(*n) } else { None })
        .collect();
    assert_eq!(ints, vec![2, 4, 6, 8, 10, 12]);
}
```

---

## ファイル変更リスト

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `StreamConfig.buffer_size` 追加、`[stream]` 解析に `buffer_size` キー追加 |
| `fav/src/backend/vm.rs` | `VM.stream_buffer_size` 追加、`new_with_db_path` 初期化更新、`run_with_stream_buffer_size` 追加、`__streaming_pipeline` chunk_size キャッピング追加 |
| `fav/src/driver.rs` | `v51300_tests` 追加（3件）、`cargo_toml_version_is_51_2_0` 削除 |
| `fav/Cargo.toml` | version → `"51.3.0"` |
| `CHANGELOG.md` | v51.3.0 エントリ追加 |
| `versions/current.md` | v51.3.0、3119 tests に更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v51.3.0 実績欄更新 |

---

## リスク

1. **`parse_fav_toml` は private**: テストでは `parse_fav_toml_pub`（行 352）を使う。
2. **`invoke_function` は private**: `run_with_stream_buffer_size` を `impl VM` ブロック内に定義することで解決。
3. **`buffer_size = 0` のパニックリスク**: `.filter(|&n| n > 0)` でガード済み。
4. **wasm32 ビルド**: `stream_buffer_size` はスレッド/チャネル操作なし（chunk_size のキャッピングのみ）のため wasm32 でも安全。追加後に `cargo build` で確認。
