# Plan — v55.1.0 — タンブリング / スライディングウィンドウ + Exactly-once 統合

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.1.0` に更新。

```toml
[package]
version = "55.1.0"
```

---

### Step 2: `toml.rs` — StreamConfig フィールド追加

`fav/src/toml.rs` の `StreamConfig` 構造体（L144〜152）に 3 フィールドを追加する。
**既存フィールドは変更しない**。

```rust
#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    pub watermark_delay:         Option<u32>,   // 既存
    pub late_policy:             Option<String>, // 既存
    pub buffer_size:             Option<usize>,  // 既存 (v51.3.0)
    // --- v55.1.0 追加 ---
    pub checkpoint_store:        Option<String>,
    pub checkpoint_interval_sec: Option<u32>,
    pub delivery:                Option<String>,
}
```

次に、既存の `[stream]` パーサーブランチ（L845〜863 の `_ => {}` の直前）に 3 キーを追加する。

```rust
"checkpoint_store" => {
    current.checkpoint_store = Some(val.trim_matches('"').to_string());
}
"checkpoint_interval_sec" => {
    current.checkpoint_interval_sec = val.trim_matches('"').parse().ok();
}
"delivery" => {
    current.delivery = Some(val.trim_matches('"').to_string());
}
```

---

### Step 3: `vm.rs` — checkpoint_hook stub 追加

#### 3-a. `VM` 構造体にフィールド追加

`fav/src/backend/vm.rs` の `VM` 構造体に `checkpoint_store: Option<String>` を追加する。

```rust
pub struct VM {
    // ... 既存フィールド ...
    /// v55.1: ウィンドウチェックポイント保存先（v55.3 でフル実装）
    pub checkpoint_store: Option<String>,
}
```

`VM::new`（またはビルダー）の初期化部分に `checkpoint_store: None` を追加する。

#### 3-b. checkpoint_hook メソッド追加

`impl VM` ブロック内に追加する。

```rust
/// ウィンドウ境界でのチェックポイント保存フック（v55.3 でフル実装）
fn checkpoint_hook(&self, offset: u64) {
    if let Some(_store) = &self.checkpoint_store {
        // TODO(v55.3): checkpoint_store にオフセットを永続化する
        let _ = offset;
    }
}
```

#### 3-c. `VMStream::Window` ブランチへの挿入

`materialize_stream` の `VMStream::Window` ブランチ（L5986〜5994）を以下に変更する。

```rust
VMStream::Window { inner, size, window_fn } => {
    let items = self.materialize_stream(artifact, *inner)?;
    let chunk_size = if size <= 0 { 1 } else { size as usize };
    let mut out = Vec::new();
    for chunk in items.chunks(chunk_size) {
        let batch = VMValue::List(FavList::new(chunk.to_vec()));
        let result = self.call_value(artifact, window_fn.clone(), vec![batch])?;
        out.push(result);
        // v55.1: チェックポイントフック（stub — v55.3 でフル実装）
        self.checkpoint_hook(out.len() as u64);
    }
    Ok(out)
}
```

---

### Step 4: `driver.rs` — v55100_tests モジュール追加

`fav/src/driver.rs` の `v55000_tests` モジュールの直前に `v55100_tests` を挿入。
同時に `v55000_tests` から `cargo_toml_version_is_55_0_0` を削除する（Cargo.toml 更新の慣行）。

```rust
// -- v55100_tests (v55.1.0) -- タンブリング / スライディングウィンドウ + Exactly-once 統合 --
#[cfg(test)]
mod v55100_tests {
    use crate::toml::parse_fav_toml_pub;

    #[test]
    fn window_tumbling_checkpoint_integration() {
        let src = "[package]\nname=\"test\"\nversion=\"1.0\"\n\n\
                   [stream]\nbuffer_size = 500\ncheckpoint_store = \"file://./checkpoints\"\n";
        let fav = parse_fav_toml_pub(src).expect("valid fav.toml");
        let cfg = fav.stream.unwrap_or_default();
        assert_eq!(cfg.buffer_size, Some(500),
            "buffer_size should be parsed from [stream]");
        assert_eq!(cfg.checkpoint_store, Some("file://./checkpoints".to_string()),
            "checkpoint_store should be parsed from [stream]");
    }

    #[test]
    fn window_sliding_resume_from_checkpoint() {
        let src = "[package]\nname=\"test\"\nversion=\"1.0\"\n\n\
                   [stream]\ndelivery = \"exactly-once\"\ncheckpoint_interval_sec = 30\n";
        let fav = parse_fav_toml_pub(src).expect("valid fav.toml");
        let cfg = fav.stream.unwrap_or_default();
        assert_eq!(cfg.delivery, Some("exactly-once".to_string()),
            "delivery should be parsed from [stream]");
        assert_eq!(cfg.checkpoint_interval_sec, Some(30),
            "checkpoint_interval_sec should be parsed from [stream]");
    }
}
```

---

### Step 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3208 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 6: ポスト処理

- `CHANGELOG.md` に v55.1.0 エントリ追加
- `versions/current.md` を v55.1.0 / 3208 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.1.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.1.0 実績欄も COMPLETE に更新

---

## 注意事項

- `cargo_toml_version_is_55_0_0`（v55000_tests）が Cargo.toml を `55.0.0` で検証しているため、
  `55.1.0` に更新すると失敗する。`v55000_tests` から削除する（毎バージョンの慣行）。
- `StreamConfig` の既存フィールド（`watermark_delay` / `late_policy` / `buffer_size`）は変更しない。
  `Default` 実装は `#[derive(Default)]` で各フィールドが `None` になるため追加不要。
- `VM` 構造体のフィールド追加後、`VM::new` などの構造体初期化箇所が全て更新されていることを
  `cargo build` で確認する（不完全な初期化はコンパイルエラーになる）。
- テストは既存の `parse_fav_toml_pub` 関数を使用するため `toml` クレートの追加は不要。
  `parse_fav_toml` は private 関数のため `driver.rs` から直接アクセス不可。`parse_fav_toml_pub` を使うこと。
  `[package]` セクションも含めた最小 TOML 文字列を引数に渡す。
- `vm.rs` の `CHECKPOINT_BACKEND` thread-local（L75〜77）は変更しない。`VM.checkpoint_store` と共存させる。
