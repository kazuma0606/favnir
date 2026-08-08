# Plan — v55.7.0 — Checkpoint / Replay API

## ステップ

### Step 0: 事前作業 — ロードマップのテスト数訂正（実装開始前に実施）

`versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 完了条件テスト数を `3220` → `3219` に訂正（v55.6.0 実績 3217 + 2）。

> **注**: spec-reviewer 対応として訂正済み。

---

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.7.0` に更新。

```toml
[package]
version = "55.7.0"
```

---

### Step 2: `vm.rs` — `RESUME_FROM_CHECKPOINT` thread-local と API 追加

`STATE_VALUE_STORE` thread-local ブロック（v55.5.0 で追加、L1430 付近）の直後に追加する。

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

---

### Step 4: `driver.rs` — `v55700_tests` モジュール追加

`v55600_tests` の直前（`// -- v55600_tests` コメント行の前）に挿入する。

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

### Step 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3219 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 6: ポスト処理

- `CHANGELOG.md` に v55.7.0 エントリ追加
- `versions/current.md` を v55.7.0 / 3219 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.7.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.7.0 実績欄も COMPLETE に更新

---

## 注意事項

- `checkpoint_list_string()` は `driver.rs` の `fn`（`pub` ではなく `pub(crate)` でもない）。
  テストモジュールが同一 crate 内のため `super::checkpoint_list_string()` でアクセス可能。
- `tempfile::tempdir()` は `[dev-dependencies]` に既存登録済みのため追加不要。
- thread-local の汚染防止: `cmd_resume_from_checkpoint` テストは冒頭で `clear_resume_from_checkpoint()` を呼ぶ。
- `cmd_checkpoint_list` テストは `tempfile::tempdir()` を使うことで
  ファイルシステムの汚染（他テストへの影響）を防ぐ。`TempDir` は drop 時に自動削除される。
- `RESUME_FROM_CHECKPOINT` の完全活用（exec ループでの再開処理）は v56.x 予定。
  v55.7.0 ではインターフェース確立のみ実施する。
