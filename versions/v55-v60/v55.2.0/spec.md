# Spec — v55.2.0 — セッションウィンドウ + ウォーターマーク本番品質化

## 概要

v55.2.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 2 弾。
v41.0 で実装済みの `Window.session` / `Watermark` に対し、
`fav.toml` の `[stream]` セクションから設定を受け取れるよう拡張する。
また、ウォーターマーク超過イベントのドロップ記録機構（`observe_late_drop` stub）と
`fav run --stream-stats` フラグのスタブ（`show_stream_stats` フィールド）を `vm.rs` に追加する。

具体的には以下 3 点を実装する：
1. `toml.rs` の `StreamConfig` に `session_gap_sec` / `watermark_max_late_sec` フィールドを追加し、既存 `[stream]` パーサーに対応キー解析を追加する
2. `vm.rs` の `VM` 構造体に `late_event_drops: u64` / `show_stream_stats: bool` フィールドを追加し、`observe_late_drop` / `print_stream_stats` stub メソッドを追加する
3. `driver.rs` に `v55200_tests` テストモジュールを追加する

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.2.0 セクション
- ベーステスト数: 3207（v55.1.0 完了時点の実績値）
- 目標テスト数: 3209（+2、削除なし）

> **注記**: ロードマップ上の完了条件は「3210 tests」（ベース 3208 + 2）だが、
> v55.1.0 の実績が 3207 だったため本バージョンの目標は **3209** とする。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.2.0"
```

### 2. `fav/src/toml.rs` — StreamConfig フィールド追加

既存の `StreamConfig` 構造体（v55.1.0 時点、`buffer_size` / `checkpoint_store` / `checkpoint_interval_sec` / `delivery` まで追加済み）に 2 フィールドを追加する。
既存フィールドは変更しない。`#[derive(Default)]` が各フィールドを `None` / 0 にするため手動 `Default` 実装は不要。

```rust
// --- v55.2.0 追加 ---
/// セッションウィンドウのギャップ秒数。(v55.2.0)
pub session_gap_sec: Option<u32>,
/// ウォーターマーク最大遅延許容秒数。遅延がこれを超えたイベントをドロップ対象とする。(v55.2.0)
pub watermark_max_late_sec: Option<u32>,
```

既存の `[stream]` パーサーブランチ（`_ => {}` の直前）に 2 キーを追加する。

```rust
"session_gap_sec" => {
    current.session_gap_sec = val.trim_matches('"').parse().ok();
}
"watermark_max_late_sec" => {
    current.watermark_max_late_sec = val.trim_matches('"').parse().ok();
}
```

> **注意**: `FavToml` 構造体にはすでに `pub stream: Option<StreamConfig>` が存在するため、
> `FavToml` 自体への変更は不要。

### 3. `fav/src/backend/vm.rs` — 遅延イベント観測 stub 追加

#### 3-a. `VM` 構造体にフィールド追加

```rust
/// v55.2: 遅延イベントドロップ回数カウンター（観測フック用）
pub(crate) late_event_drops: u64,
/// v55.2: --stream-stats フラグ有効時に true（v55.9 でフル実装）
pub(crate) show_stream_stats: bool,
```

#### 3-b. `VM::new_with_db_path` 初期化部分に追加

```rust
late_event_drops: 0,
show_stream_stats: false,
```

#### 3-c. stub メソッド追加（`impl VM` ブロック内）

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

> **注意**:
> - `late_event_drops` と `show_stream_stats` は `pub(crate)` とし、`stream_buffer_size` との可視性を揃える。
> - `CHECKPOINT_BACKEND` thread-local（vm.rs L75〜77）および v55.1.0 追加の `checkpoint_store` は変更しない。
> - `vm.rs` L75〜77 の既存 `CHECKPOINT_BACKEND` thread-local と共存させること。

---

## テスト仕様

テストは `parse_fav_toml_pub` を使用する。`v55100_tests` にはバージョン検証テストが存在しないため、削除タスクはない。

### `window_session_toml_config`

```rust
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
```

### `watermark_late_event_observe_effect`

> **注記**: テスト名は「遅延イベント観測エフェクト」を示すが、本バージョンではドロップ観測フックは stub のため、
> テスト本体は TOML パーサーが `late_policy` と `watermark_max_late_sec` を正しく組み合わせて解析できることを検証する。
> フル観測動作は v55.9 で検証する。

```rust
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
```

---

## 完了条件

- `cargo test` 全通過（3209 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `window_session_toml_config` pass
- `watermark_late_event_observe_effect` pass
- `toml.rs` の `StreamConfig` に `session_gap_sec` / `watermark_max_late_sec` が追加されている
- `vm.rs` に `observe_late_drop` / `print_stream_stats` stub が追加されている
- `CHANGELOG.md` に v55.2.0 エントリが追加されている
- `versions/current.md` が v55.2.0 / 3209 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.2.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.2.0 実績欄も COMPLETE に更新

---

## 備考

- `Effect` enum は v35.5.0 で削除済み。`!Observe` エフェクトを AST レベルで追加するのではなく、
  `vm.rs` の `late_event_drops` カウンターをその代替メカニズムとして位置付ける。
  フル実装（カウンター値の実際の増加トリガー）は v55.9「安定化」バージョンで行う。
- `VMStream::Session` / `VMStream::Watermark` は独立したバリアントとして存在しない
  （`VMStream::Window` に統合済み）。本バージョンでは設定値の受け皿（toml.rs フィールド）と
  観測フック（vm.rs stub）のみを追加する。
- `--stream-stats` CLI フラグの実際のパース追加は v55.9 で行う。本バージョンでは
  `vm.rs` に `show_stream_stats` フィールドを追加するのみ。
- ロードマップ（roadmap-v55.1-v56.0.md L56）は「`!Observe` エフェクト経由のドロップ記録」および
  「`fav run --stream-stats` フラグで統計表示」と記述しているが、`Effect` enum が v35.5.0 で削除されており
  実装不可のためスコープを縮小した。実装完了後に以下の 2 点をロードマップに反映すること：
  1. `!Observe` / `--stream-stats` の記述を `late_event_drops` カウンター stub / v55.9 後送りに更新する。
  2. 完了条件テスト数（roadmap-v55.1-v56.0.md L65「3210」）を実績値（3209）に訂正する。
- `driver.rs` への `v55200_tests` 挿入は `v55100_tests` の**直前**（逆順挿入の慣行に従う — 新しいテストほど上に配置される）。
- ドキュメント MDX は v55.8「ドキュメントサイト Streaming 2.0 記事」でまとめて追加するため、本バージョンでは不要。
- v55100_tests にはバージョン検証テスト（`cargo_toml_version_is_55_1_0`）が存在しないため、削除タスクは不要。
