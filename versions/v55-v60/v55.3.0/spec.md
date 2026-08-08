# Spec — v55.3.0 — Exactly-once 意味論（冪等チェックポイント）

## 概要

v55.3.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 3 弾。
v55.1.0 で追加した `checkpoint_hook` stub と `checkpoint_store` フィールドを昇格し、
ウィンドウ境界ごとに処理済みオフセットを in-memory で追跡する冪等チェックポイント機構を実装する。
ファイル / S3 への永続化は v55.7（Checkpoint / Replay API）で行う。

具体的には以下を実装する：
1. `vm.rs` の `VM` 構造体に `checkpoint_delivery: Option<String>` / `processed_offsets: HashSet<u64>` フィールドを追加する
2. `checkpoint_hook` を `&self` stub から `&mut self` 実装に昇格し、`delivery = "exactly-once"` 時に `processed_offsets` へオフセットを記録する
3. `is_duplicate_offset` メソッドを追加する（重複排除クエリ用）
4. `driver.rs` に `v55300_tests` テストモジュールを追加する

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.3.0 セクション
- ベーステスト数: 3209（v55.2.0 完了時点の実績値）
- 目標テスト数: 3211（+2、削除なし）

> **注記**: ロードマップ上のベース値が 3210 と記載されていたが、v55.2.0 の実績が 3209 のため
> 実装前にロードマップを 3209 ベースに訂正済み。本バージョンの目標は **3211**（3209 + 2）とする。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.3.0"
```

### 2. `fav/src/toml.rs` — 変更なし

`checkpoint_store` / `checkpoint_interval_sec` / `delivery` フィールドは v55.1.0 で追加済み。
`session_gap_sec` / `watermark_max_late_sec` は v55.2.0 で追加済み。
`toml.rs` への追加変更は不要。

### 3. `fav/src/backend/vm.rs` — 冪等チェックポイント機構

#### 3-a. `VM` 構造体にフィールド追加

`show_stream_stats` フィールドの直後に追加する。

```rust
/// v55.3: delivery セマンティクス（"exactly-once" | "at-least-once"）。
/// run_with_stream_buffer_size 等で fav.toml の stream.delivery から注入する。
pub(crate) checkpoint_delivery: Option<String>,
/// v55.3: 処理済みウィンドウオフセットの in-memory セット（冪等重複排除用）。
/// 永続化は v55.7 で実装。
pub(crate) processed_offsets: HashSet<u64>,
```

> **注意**: `HashSet` は vm.rs L23 の `use std::collections::{HashMap, HashSet};` で既にインポート済み。
> 追加インポートは不要。

#### 3-b. `VM::new_with_db_path` 初期化部分に追加

`show_stream_stats: false,` の直後に追加する。

```rust
checkpoint_delivery: None,
processed_offsets: HashSet::new(),
```

#### 3-c. `checkpoint_hook` を stub から実装に昇格

シグネチャを `&self` から `&mut self` に変更し、`processed_offsets` への記録を追加する。

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

#### 3-d. `is_duplicate_offset` メソッド追加

`checkpoint_hook` の直後に追加する。

```rust
/// 指定オフセットが処理済みかどうかを検証する（Exactly-once 重複排除クエリ）
pub(crate) fn is_duplicate_offset(&self, offset: u64) -> bool {
    self.processed_offsets.contains(&offset)
}
```

> **注意**:
> - `checkpoint_hook` の `&self` → `&mut self` 変更は、呼び出し元の `VMStream::Window` ブランチ
>   （`materialize_stream` 内）において `self` が既に `&mut Self` であるため問題ない。
> - `v55200_tests` にはバージョン検証テストが存在しないため削除タスクは不要。
> - `run_with_stream_buffer_size` の `TODO(v55.3)` コメントは残す（`checkpoint_store` と
>   `checkpoint_delivery` の外部注入は v55.7 の CLI フラグ実装時に行う）。

---

## テスト仕様

テストは `parse_fav_toml_pub` を使用する。`v55200_tests` にはバージョン検証テストが存在しないため削除タスクはない。

### `exactly_once_checkpoint_saved`

`delivery = "exactly-once"` と `checkpoint_store` の組み合わせが `[stream]` パーサーで正しく解析されることを検証する。

```rust
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
```

### `exactly_once_no_duplicate_on_restart`

`delivery = "exactly-once"` と `checkpoint_interval_sec` の組み合わせが正しく解析されることを検証する。
冪等リトライに必要な設定値のパーサー整合性を確認する。

```rust
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
```

---

## 完了条件

- `cargo test` 全通過（3211 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `exactly_once_checkpoint_saved` pass
- `exactly_once_no_duplicate_on_restart` pass
- `vm.rs` の `checkpoint_hook` が `&mut self` になっている
- `vm.rs` に `processed_offsets: HashSet<u64>` フィールドが追加されている
- `vm.rs` に `is_duplicate_offset` メソッドが追加されている
- `CHANGELOG.md` に v55.3.0 エントリが追加されている
- `versions/current.md` が v55.3.0 / 3211 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.3.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.3.0 実績欄も COMPLETE に更新

---

## 備考

- `HashSet<u64>` は vm.rs L23 の既存 `use std::collections::{HashMap, HashSet};` で利用可能。
  追加インポートは不要。
- `checkpoint_hook` の `&self` → `&mut self` 変更後、`VMStream::Window` ブランチの呼び出し
  `self.checkpoint_hook(out.len() as u64)` はシグネチャ変更のみでソースコード上の変更は不要。
- `processed_offsets` は in-memory のみ（VM インスタンスのライフタイムで消える）。
  永続化（ファイル書き込み / S3 アップロード）は v55.7「Checkpoint / Replay API」で実装する。
- `checkpoint_delivery` は現時点で外部から注入する経路がないため常に `None` 動作となる。
  v55.7 で `run_with_stream_buffer_size` / `cmd_run` から注入する予定。
- `is_duplicate_offset` は `pub(crate)` とし、v55.5（Stateful stage）や v55.7（Replay API）から参照できるようにする。
  v55.3.0 時点では呼び出し元が存在しないため、`cargo clippy -- -D warnings` で `dead_code` 警告が出ないことを実装後に確認すること
  （`pub(crate)` フィールドは通常 Clippy の `dead_code` 対象外だが念のため確認する）。
- `checkpoint_delivery` は v55.3.0 時点で外部から注入する経路がなく常に `None` 動作となる。
  `checkpoint_hook` 内の `as_deref() == Some("exactly-once")` 分岐が dead code とならないか
  `cargo clippy -- -D warnings` で確認すること（`pub(crate)` フィールドへの書き込みは可能なため警告は出ないはずだが要確認）。
- `checkpoint_store = None` の場合は `checkpoint_delivery = Some("exactly-once")` であっても
  `checkpoint_hook` の外側 `if let Some(_store)` でガードされるため `processed_offsets` への記録は行われない
  （`checkpoint_store` が設定前提の設計）。v55.7 で `run_with_stream_buffer_size` から注入する際に両フィールドを同時に設定すること。
- ドキュメント MDX は v55.8 でまとめて追加するため本バージョンでは不要。
