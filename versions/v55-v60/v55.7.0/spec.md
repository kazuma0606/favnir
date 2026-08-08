# Spec — v55.7.0 — Checkpoint / Replay API

## 概要

v55.7.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 7 弾。
v55.3.0 で追加した `checkpoint_store` / `checkpoint_delivery` インフラを活用し、
`fav run --resume-from <name>` で再開ポイントを指定する API を実装する。

具体的には以下を実装する：
1. `vm.rs` に `RESUME_FROM_CHECKPOINT` thread-local と `set/get/clear_resume_from_checkpoint` API を追加
2. `driver.rs` に `v55700_tests` を追加（`cmd_checkpoint_list` / `cmd_resume_from_checkpoint`）

> **スコープ外（v56.x 予定）**: ロードマップに記載のある `--replay-from / --replay-until`
> 時刻範囲リプレイの完全実装（VM exec ループでの実際の状態復元）は v56.x で行う。
> v55.7.0 では再開ポイント格納 API のインターフェース確立のみを実施する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.7.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v55.7.0 行
- ベーステスト数: **3217**（v55.6.0 完了時点の実績値）
- 目標テスト数: **3219**（+2、削除なし）

> **注記**: ロードマップ上のベース値が 3218（3217 + 1 のずれ）と記載されているため、
> 完了条件が 3220 と記載されているが、v55.6.0 の実績が 3217 のため
> 本バージョンの目標は **3219**（3217 + 2）とする。
> ロードマップの 3220 記載は実装前に訂正する。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `checkpoint_store` / `checkpoint_delivery` フィールド | v55.3.0 | 実装済み（VM struct） |
| `checkpoint_save_direct` / `checkpoint_meta` / `checkpoint_reset_direct` | v55.3.0 | 実装済み（vm.rs pub fn） |
| `checkpoint_list` / `checkpoint_list_string` / `cmd_checkpoint_list` | v55.3.0以前 | 実装済み（driver.rs） |
| `processed_offsets` / `is_duplicate_offset` | v55.3.0 | 実装済み（VM struct） |
| `RESUME_FROM_CHECKPOINT` thread-local | — | **未実装（v55.7.0 で追加）** |
| `set/get/clear_resume_from_checkpoint` API | — | **未実装（v55.7.0 で追加）** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.7.0"
```

---

### 2. `fav/src/backend/vm.rs` — Resume-from-checkpoint API 追加

`STATE_VALUE_STORE` thread-local ブロックの直後（v55.5.0 で追加済み）に追加する。

```rust
// v55.7.0: Checkpoint / Replay API — resume-from 再開ポイント格納
thread_local! {
    static RESUME_FROM_CHECKPOINT: std::cell::RefCell<Option<String>>
        = std::cell::RefCell::new(None);
}

pub fn set_resume_from_checkpoint(name: &str) {
    RESUME_FROM_CHECKPOINT.with(|c| {
        *c.borrow_mut() = Some(name.to_string());
    });
}

pub fn get_resume_from_checkpoint() -> Option<String> {
    RESUME_FROM_CHECKPOINT.with(|c| c.borrow().clone())
}

pub fn clear_resume_from_checkpoint() {
    RESUME_FROM_CHECKPOINT.with(|c| *c.borrow_mut() = None);
}
```

> **注記**: `--resume-from` フラグの完全実装（VM exec ループでの実際の再開処理）は
> Streaming Native 2.0 本番化フェーズ（v56.x）で行う。
> v55.7.0 ではリプレイ API の公開インターフェースを確立する。

---

### 3. `fav/src/driver.rs` — `v55700_tests` 追加

`v55600_tests` の直前に挿入する（逆順挿入の慣行に従う）。

```rust
// -- v55700_tests (v55.7.0) -- Checkpoint / Replay API --
#[cfg(test)]
mod v55700_tests {
    use crate::backend::vm::{
        CheckpointBackend, checkpoint_save_direct,
        clear_resume_from_checkpoint, get_resume_from_checkpoint,
        set_checkpoint_backend, set_resume_from_checkpoint,
    };
    use super::checkpoint_list_string;

    #[test]
    fn cmd_checkpoint_list() {
        // tempfile で一時ディレクトリを作成し、File バックエンドとして使用
        let dir = tempfile::tempdir().expect("temp dir");
        set_checkpoint_backend(CheckpointBackend::File {
            dir: dir.path().to_path_buf(),
        });
        checkpoint_save_direct("v55700_cp1", "offset=1000").expect("save ok");
        let output = checkpoint_list_string().expect("list ok");
        assert!(
            output.contains("v55700_cp1"),
            "checkpoint list should contain v55700_cp1, got {:?}", output
        );
        assert!(
            output.contains("offset=1000"),
            "checkpoint list should contain checkpoint value, got {:?}", output
        );
    }

    #[test]
    fn cmd_resume_from_checkpoint() {
        clear_resume_from_checkpoint();
        set_resume_from_checkpoint("2026-07-23T09:10:00Z");
        let got = get_resume_from_checkpoint();
        assert_eq!(
            got.as_deref(),
            Some("2026-07-23T09:10:00Z"),
            "get_resume_from_checkpoint should return the set value, got {:?}", got
        );
        clear_resume_from_checkpoint();
    }
}
```

---

## テスト仕様

### `cmd_checkpoint_list`

`checkpoint_save_direct` で一時 File バックエンドにチェックポイントを保存し、
`checkpoint_list_string()` の出力にチェックポイント名と値が含まれることを検証。

- 一時ディレクトリ（`tempfile::tempdir()`）を File バックエンドとして設定
- `checkpoint_save_direct("v55700_cp1", "offset=1000")` で保存
- `checkpoint_list_string()` を呼び出し
- 出力に `"v55700_cp1"` が含まれること
- 出力に `"offset=1000"` が含まれること

### `cmd_resume_from_checkpoint`

`set_resume_from_checkpoint` / `get_resume_from_checkpoint` API の round-trip を検証。

- `clear_resume_from_checkpoint()` でクリア（汚染防止）
- `set_resume_from_checkpoint("2026-07-23T09:10:00Z")` で設定
- `get_resume_from_checkpoint()` → `Some("2026-07-23T09:10:00Z")` を検証
- `clear_resume_from_checkpoint()` でクリア（後始末）

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（3219 tests passed, 0 failed）
- `cargo clippy --all-targets -- -D warnings` クリーン
- ドキュメント MDX は v55.8 でまとめて追加（本バージョンはスキップ）
- `cmd_checkpoint_list` pass
- `cmd_resume_from_checkpoint` pass
- `vm.rs` に `RESUME_FROM_CHECKPOINT` thread-local が追加されている
- `vm.rs` に `set/get/clear_resume_from_checkpoint` が追加されている
- `CHANGELOG.md` に v55.7.0 エントリが追加されている
- `versions/current.md` が v55.7.0 / 3219 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.7.0 実績欄も COMPLETE に更新

---

## 備考

- `checkpoint_list_string()` は `driver.rs` のモジュールプライベート関数（可視性修飾子なし）。
  `v55700_tests` は `driver.rs` ファイル内に `mod v55700_tests` として配置されるため、
  `super::checkpoint_list_string()` でアクセス可能（同一ファイル内モジュールのため）。
- `checkpoint_save_direct` / `set_checkpoint_backend` は `vm.rs` の `pub fn` として公開済み。
- `tempfile` crate は既に `[dev-dependencies]` に登録済み（v24.8.0 時点で確認）。
- `RESUME_FROM_CHECKPOINT` の完全活用（exec ループでのチェックポイントロードと状態復元）は
  v56.x で実装予定。v55.7.0 はインターフェース確立のみ。
- `is_duplicate_offset` の VM レベルテストも v55.3.0 コメントで v55.7 予定と記載されていたが、
  今回は `cmd_checkpoint_list` / `cmd_resume_from_checkpoint` の 2 件に絞る。
