# Plan — v55.3.0 — Exactly-once 意味論（冪等チェックポイント）

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.3.0` に更新。

```toml
[package]
version = "55.3.0"
```

---

### Step 2: `vm.rs` — 冪等チェックポイント機構

#### 2-a. `VM` 構造体にフィールド追加

`fav/src/backend/vm.rs` の `VM` 構造体（`show_stream_stats: bool` の直後）に 2 フィールドを追加する。

```rust
/// v55.3: delivery セマンティクス（"exactly-once" | "at-least-once"）。
/// run_with_stream_buffer_size 等で fav.toml の stream.delivery から注入する。
pub(crate) checkpoint_delivery: Option<String>,
/// v55.3: 処理済みウィンドウオフセットの in-memory セット（冪等重複排除用）。
/// 永続化は v55.7 で実装。
pub(crate) processed_offsets: HashSet<u64>,
```

> `HashSet` は vm.rs L23 の `use std::collections::{HashMap, HashSet};` で既にインポート済み。

#### 2-b. `VM::new_with_db_path` 初期化部分に追加

`show_stream_stats: false,` の直後に追加する。

```rust
checkpoint_delivery: None,
processed_offsets: HashSet::new(),
```

#### 2-c. `checkpoint_hook` を昇格

既存の `checkpoint_hook` メソッド（`&self` stub）を `&mut self` 実装に置き換える。

**変更前:**
```rust
fn checkpoint_hook(&self, offset: u64) {
    if let Some(_store) = &self.checkpoint_store {
        // TODO(v55.3): checkpoint_store にオフセットを永続化する
        let _ = offset;
    }
}
```

**変更後:**
```rust
/// ウィンドウ境界でのチェックポイント保存フック（v55.3: in-memory 追跡、v55.7 で永続化）
/// `offset` = これまでに処理したウィンドウ数（`out.len()` の値）。
fn checkpoint_hook(&mut self, offset: u64) {
    if let Some(_store) = &self.checkpoint_store {
        // v55.3: exactly-once の場合、処理済みオフセットを in-memory で記録する
        // （永続化は v55.7 Checkpoint / Replay API で実装）
        if self.checkpoint_delivery.as_deref() == Some("exactly-once") {
            self.processed_offsets.insert(offset);
        }
    }
}
```

#### 2-d. `is_duplicate_offset` メソッド追加

`checkpoint_hook` の直後に追加する。

```rust
/// 指定オフセットが処理済みかどうかを検証する（Exactly-once 重複排除クエリ）
pub(crate) fn is_duplicate_offset(&self, offset: u64) -> bool {
    self.processed_offsets.contains(&offset)
}
```

---

### Step 3: `driver.rs` — v55300_tests モジュール追加

`fav/src/driver.rs` の `v55200_tests` モジュールの直前に `v55300_tests` を挿入（逆順挿入の慣行に従う）。
`v55200_tests` にはバージョン検証テストが存在しないため削除タスクは不要。

```rust
// -- v55300_tests (v55.3.0) -- Exactly-once 意味論（冪等チェックポイント）--
#[cfg(test)]
mod v55300_tests {
    use crate::toml::parse_fav_toml_pub;

    #[test]
    fn exactly_once_checkpoint_saved() {
        let src = "[rune]\nname=\"test\"\nversion=\"1.0\"\n\n\
               [stream]\ndelivery = \"exactly-once\"\n\
               checkpoint_store = \"file://./checkpoints\"\n";
        let fav = parse_fav_toml_pub(src);
        let cfg = fav.stream.expect("[stream] section should be parsed");
        assert_eq!(cfg.delivery.as_deref(), Some("exactly-once"),
            "delivery should be parsed as exactly-once");
        assert_eq!(cfg.checkpoint_store.as_deref(), Some("file://./checkpoints"),
            "checkpoint_store should be parsed from [stream]");
    }

    #[test]
    fn exactly_once_no_duplicate_on_restart() {
        let src = "[rune]\nname=\"test\"\nversion=\"1.0\"\n\n\
               [stream]\ndelivery = \"exactly-once\"\ncheckpoint_interval_sec = 10\n";
        let fav = parse_fav_toml_pub(src);
        let cfg = fav.stream.expect("[stream] section should be parsed");
        assert_eq!(cfg.delivery.as_deref(), Some("exactly-once"),
            "delivery should be parsed as exactly-once");
        assert_eq!(cfg.checkpoint_interval_sec, Some(10),
            "checkpoint_interval_sec should be parsed from [stream]");
    }
}
```

---

### Step 4: テスト実行・確認

まず `cargo build` でコンパイルエラー（VM 構造体初期化漏れ、シグネチャ不整合等）がないことを確認する。

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished` — エラーなし

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3211 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 5: ポスト処理

- `CHANGELOG.md` に v55.3.0 エントリ追加
- `versions/current.md` を v55.3.0 / 3211 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.3.0 実績を COMPLETE に更新（テスト数訂正含む）
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.3.0 実績欄も COMPLETE に更新

---

## 注意事項

- `HashSet` は vm.rs L23 で既にインポート済みのため追加インポート不要。
- `checkpoint_hook` の `&self` → `&mut self` 変更後、呼び出し元 `VMStream::Window` ブランチの
  `self.checkpoint_hook(out.len() as u64)` はソースコード上の変更不要（`materialize_stream` は `&mut self` を取るため）。
- `VM::new_with_db_path` が唯一の VM 構造体初期化箇所である（`VM::new` は委譲のみ）。
  `cargo build` で初期化漏れを確認すること。
- `v55200_tests` にはバージョン検証テストが存在しないため削除タスクは不要。
- `run_with_stream_buffer_size` の `TODO(v55.3)` コメントは残す（外部注入は v55.7 で実装）。
- `checkpoint_delivery` は `None` 固定のため `checkpoint_hook` 内の `exactly-once` 分岐が dead code になりうるが、
  `pub(crate)` フィールドへの書き込みは外部から可能なため Clippy は通常 `dead_code` を報告しない。
  実装後 `cargo clippy -- -D warnings` で警告ゼロを必ず確認すること。
- `is_duplicate_offset` は v55.3.0 時点で呼び出し元がないが `pub(crate)` のため `dead_code` 警告対象外になる。
  実装後 Clippy で確認すること。
