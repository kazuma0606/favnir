# Spec — v55.1.0 — タンブリング / スライディングウィンドウ + Exactly-once 統合

## 概要

v55.1.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の起点。
v41.0 以前に実装済みの `VMStream::Window`（`vm.rs` の `materialize_stream` で処理）に、
`fav.toml` の `[stream]` セクションへ Exactly-once 用フィールドを追加し、
将来の v55.3（Exactly-once チェックポイント）との統合インターフェースを先行整備する。

具体的には以下 3 点を実装する：
1. `toml.rs` の既存 `StreamConfig` に `checkpoint_store` / `checkpoint_interval_sec` / `delivery` フィールドを追加し、既存 `[stream]` パーサーに対応キー解析を追加する
2. `vm.rs` の `VM` 構造体に `checkpoint_store: Option<String>` フィールドを追加し、`VMStream::Window` ブランチに `checkpoint_hook` stub 呼び出しを挿入する
3. `driver.rs` に `v55100_tests` テストモジュールを追加する

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.1.0 セクション
- ベーステスト数: 3206（v55.0.0 完了時点）
- 目標テスト数: 3208（+2）

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.1.0"
```

### 2. `fav/src/toml.rs` — StreamConfig フィールド追加

既存の `StreamConfig` 構造体（v40.5.0 実装済み、L144〜152）に 3 フィールドを追加する。

```rust
// 既存フィールドはそのまま保持
pub struct StreamConfig {
    pub watermark_delay:        Option<u32>,   // 既存
    pub late_policy:            Option<String>, // 既存
    pub buffer_size:            Option<usize>,  // 既存 (v51.3.0)
    // --- v55.1.0 追加 ---
    pub checkpoint_store:       Option<String>, // "file://..." | "s3://..."
    pub checkpoint_interval_sec: Option<u32>,  // デフォルト None（10 秒相当を v55.3 で設定）
    pub delivery:               Option<String>, // "at-least-once" | "exactly-once"
}
```

既存の `[stream]` パーサーブランチ（L845〜863）の `_ => {}` 行（L860）の直前に 3 キーの解析を追加する。

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

> **注意**: `FavToml` 構造体にはすでに `pub stream: Option<StreamConfig>` が存在するため、
> `FavToml` 自体への変更は不要。

### 3. `fav/src/backend/vm.rs` — checkpoint_hook stub 追加

`VM` 構造体に `checkpoint_store: Option<String>` フィールドを追加し、
pipeline 実行時に `fav.toml` の `stream.checkpoint_store` を注入する。
`materialize_stream` の `VMStream::Window` ブランチ（L5986〜5994）に
`checkpoint_hook` stub 呼び出しを挿入する。

```rust
/// ウィンドウ境界でのチェックポイント保存フック（v55.3 でフル実装）
fn checkpoint_hook(&self, offset: u64) {
    if let Some(_store) = &self.checkpoint_store {
        // TODO(v55.3): checkpoint_store にオフセットを永続化する
        let _ = offset;
    }
}
```

`VMStream::Window` ブランチへの挿入:

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

## テスト仕様

テストは既存の `parse_fav_toml` 関数と `StreamConfig` を使用する（`toml` クレート追加は不要）。

### `window_tumbling_checkpoint_integration`

```rust
#[test]
fn window_tumbling_checkpoint_integration() {
    use crate::toml::parse_fav_toml_pub;
    let src = "[package]\nname=\"test\"\nversion=\"1.0\"\n\n\
               [stream]\nbuffer_size = 500\ncheckpoint_store = \"file://./checkpoints\"\n";
    let fav = parse_fav_toml_pub(src).expect("valid fav.toml");
    let cfg = fav.stream.unwrap_or_default();
    assert_eq!(cfg.buffer_size, Some(500),
        "buffer_size should be parsed from [stream]");
    assert_eq!(cfg.checkpoint_store, Some("file://./checkpoints".to_string()),
        "checkpoint_store should be parsed from [stream]");
}
```

### `window_sliding_resume_from_checkpoint`

```rust
#[test]
fn window_sliding_resume_from_checkpoint() {
    use crate::toml::parse_fav_toml_pub;
    let src = "[package]\nname=\"test\"\nversion=\"1.0\"\n\n\
               [stream]\ndelivery = \"exactly-once\"\ncheckpoint_interval_sec = 30\n";
    let fav = parse_fav_toml_pub(src).expect("valid fav.toml");
    let cfg = fav.stream.unwrap_or_default();
    assert_eq!(cfg.delivery, Some("exactly-once".to_string()),
        "delivery should be parsed from [stream]");
    assert_eq!(cfg.checkpoint_interval_sec, Some(30),
        "checkpoint_interval_sec should be parsed from [stream]");
}
```

---

## 完了条件

- `cargo test` 全通過（3208 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `window_tumbling_checkpoint_integration` pass
- `window_sliding_resume_from_checkpoint` pass
- `toml.rs` の `StreamConfig` に 3 フィールドが追加されている
- `vm.rs` に `checkpoint_hook` stub が追加されている
- `versions/current.md` が v55.1.0 / 3208 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.1.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.1.0 実績欄も COMPLETE に更新

---

## 備考

- `Window.tumbling` / `Window.sliding` は実際には単一の `VMStream::Window` バリアントとして実装されている（`materialize_stream` L5986〜5994）。別々の opcode は存在しない。
- `checkpoint_hook` は v55.3 でフル実装する。本バージョンでは no-op stub として挿入するのみ。
- `v55100_tests` モジュールは `v55000_tests` の直前に挿入する（逆順挿入の慣行に従う）。
- テストは既存の `parse_fav_toml_pub` を使用するため `toml` クレートの追加は不要。
  （`parse_fav_toml` は private 関数のため `driver.rs` から直接アクセス不可）。
- `vm.rs` L75〜77 に既存の `CHECKPOINT_BACKEND` thread-local が存在する。
  v55.1 では `VM.checkpoint_store` と共存させる（上書き・削除しない）。
  v55.3 でフル実装する際に `checkpoint_store` の値で `CHECKPOINT_BACKEND` を初期化する設計とする。
- ドキュメント MDX は v55.8「ドキュメントサイト Streaming 2.0 記事」でまとめて追加するため、本バージョンでは不要。
