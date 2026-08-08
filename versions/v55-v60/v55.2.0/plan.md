# Plan — v55.2.0 — セッションウィンドウ + ウォーターマーク本番品質化

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.2.0` に更新。

```toml
[package]
version = "55.2.0"
```

---

### Step 2: `toml.rs` — StreamConfig フィールド追加

`fav/src/toml.rs` の `StreamConfig` 構造体（v55.1.0 追加済みの `delivery` フィールドの直後）に 2 フィールドを追加する。
**既存フィールドは変更しない**。

```rust
#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    pub watermark_delay:         Option<u32>,   // 既存
    pub late_policy:             Option<String>, // 既存
    pub buffer_size:             Option<usize>,  // 既存 (v51.3.0)
    pub checkpoint_store:        Option<String>, // 既存 (v55.1.0)
    pub checkpoint_interval_sec: Option<u32>,   // 既存 (v55.1.0)
    pub delivery:                Option<String>, // 既存 (v55.1.0)
    // --- v55.2.0 追加 ---
    pub session_gap_sec:         Option<u32>,
    pub watermark_max_late_sec:  Option<u32>,
}
```

次に、既存の `[stream]` パーサーブランチ（`_ => {}` の直前）に 2 キーを追加する。

```rust
"session_gap_sec" => {
    current.session_gap_sec = val.trim_matches('"').parse().ok();
}
"watermark_max_late_sec" => {
    current.watermark_max_late_sec = val.trim_matches('"').parse().ok();
}
```

---

### Step 3: `vm.rs` — 遅延イベント観測 stub 追加

#### 3-a. `VM` 構造体にフィールド追加

`fav/src/backend/vm.rs` の `VM` 構造体（`checkpoint_store` フィールドの直後）に 2 フィールドを追加する。

```rust
/// v55.2: 遅延イベントドロップ回数カウンター（観測フック用）
pub(crate) late_event_drops: u64,
/// v55.2: --stream-stats フラグ有効時に true（v55.9 でフル実装）
pub(crate) show_stream_stats: bool,
```

#### 3-b. `VM::new_with_db_path` 初期化部分に追加

`checkpoint_store: None,` の直後に追加する。

```rust
late_event_drops: 0,
show_stream_stats: false,
```

#### 3-c. `impl VM` ブロックに stub メソッド追加

`checkpoint_hook` メソッドの直後に追加する。

```rust
/// ウォーターマーク超過イベントのドロップ記録フック（v55.9 でフル実装）
pub(crate) fn observe_late_drop(&mut self) {
    self.late_event_drops += 1;
}

/// ストリーム統計を標準出力に表示（v55.9 でフル実装）
pub(crate) fn print_stream_stats(&self) {
    if self.show_stream_stats {
        // TODO(v55.9): ウィンドウ / ウォーターマーク統計を表示する
        let _ = self.late_event_drops;
    }
}
```

---

### Step 4: `driver.rs` — v55200_tests モジュール追加

`fav/src/driver.rs` の `v55100_tests` モジュールの直前に `v55200_tests` を挿入。
v55100_tests にはバージョン検証テストが存在しないため削除タスクはない。

```rust
// -- v55200_tests (v55.2.0) -- セッションウィンドウ + ウォーターマーク本番品質化 --
#[cfg(test)]
mod v55200_tests {
    use crate::toml::parse_fav_toml_pub;

    #[test]
    fn window_session_toml_config() {
        let src = "[rune]\nname=\"test\"\nversion=\"1.0\"\n\n\
                   [stream]\nsession_gap_sec = 30\nwatermark_max_late_sec = 5\n";
        let fav = parse_fav_toml_pub(src);
        let cfg = fav.stream.unwrap_or_default();
        assert_eq!(cfg.session_gap_sec, Some(30),
            "session_gap_sec should be parsed from [stream]");
        assert_eq!(cfg.watermark_max_late_sec, Some(5),
            "watermark_max_late_sec should be parsed from [stream]");
    }

    #[test]
    fn watermark_late_event_observe_effect() {
        let src = "[rune]\nname=\"test\"\nversion=\"1.0\"\n\n\
                   [stream]\nlate_policy = \"drop\"\nwatermark_max_late_sec = 10\n";
        let fav = parse_fav_toml_pub(src);
        let cfg = fav.stream.unwrap_or_default();
        assert_eq!(cfg.late_policy.as_deref(), Some("drop"),
            "late_policy should be parsed from [stream]");
        assert_eq!(cfg.watermark_max_late_sec, Some(10),
            "watermark_max_late_sec should be parsed from [stream]");
    }
}
```

---

### Step 5: テスト実行・確認

まず `cargo build` でコンパイルエラー（VM 構造体初期化漏れ等）がないことを確認する。

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished` — エラーなし

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3209 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 6: ポスト処理

- `CHANGELOG.md` に v55.2.0 エントリ追加
- `versions/current.md` を v55.2.0 / 3209 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.2.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.2.0 実績欄も COMPLETE に更新

---

## 注意事項

- `StreamConfig` の既存フィールド（`watermark_delay` / `late_policy` / `buffer_size` / `checkpoint_store` / `checkpoint_interval_sec` / `delivery`）は変更しない。
  `Default` 実装は `#[derive(Default)]` で各フィールドが `None` / `false` / `0` になるため追加不要。
- `VM` 構造体のフィールド追加後、`VM::new_with_db_path` の初期化リストに `late_event_drops: 0` と `show_stream_stats: false` を追加すること。
  `cargo build` で初期化漏れを確認する。
- テストは `parse_fav_toml_pub`（`toml.rs` の public wrapper）を使用する。`parse_fav_toml` は private のため `driver.rs` からアクセス不可。
- `v55100_tests` には `cargo_toml_version_is_55_1_0` テストが存在しないため、削除タスクは不要。
- `observe_late_drop` と `print_stream_stats` はカウンターをインクリメントする処理 / show フラグをチェックする処理を含むため Clippy の `unused` 警告は発生しない。
- `vm.rs` L75〜77 の `CHECKPOINT_BACKEND` thread-local は変更しない。
