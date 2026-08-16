# v72.7.0 実装計画 — Hot Reload 改善（`fav watch` 2.0）

Date: 2026-08-12

---

## 依存順序

```
T0: 事前確認
  ↓
T1: WatchSession 構造体 + watch_session_on_change_label 追加（driver.rs）
  ↓
T2: cmd_watch2 追加（driver.rs）
  ↓
T3: main.rs — --on-change フラグ対応
  ↓
T4: v727000_tests モジュール追加（driver.rs）
  ↓
T5: Cargo.toml バージョン更新 + driver.rs バージョンアサーション更新
  ↓
T6: 部分テスト確認（cargo test v727000）
  ↓
T7: 全体テスト確認（cargo test）
  ↓
T8: CHANGELOG.md 更新
T9: versions/current.md 更新
  ↓
T10: 最終確認
```

---

## ステップ詳細

### T0: 事前確認

- `fav/Cargo.toml` のバージョンが `72.6.0` であることを確認
- `cargo test` が 3635 tests pass（0 failures）であることを確認
- `driver.rs` に `v726000_tests` モジュールが存在することを確認
- `driver.rs` に `v727000_tests` が未存在であることを確認
- `driver.rs` 内の `cmd_watch` が存在することを確認（既存実装の把握）
- `driver.rs` 内の `"72.6.0"` 文字列（バージョンアサーション）の件数を grep で確認

### T1: `WatchSession` 構造体 + `watch_session_on_change_label` 追加

`playground_decode_url` の後、v72.6.0 make_* 関数ブロックの直後に追加:

```rust
// ── v72.7.0: fav watch 2.0 ──────────────────────────────────────────────────

pub struct WatchSession {
    pub file: Option<String>,
    pub on_change_cmd: String,
    pub debounce_ms: u64,
}

pub fn watch_session_on_change_label(session: &WatchSession) -> String {
    format!("[watch] Running: {}", session.on_change_cmd)
}
```

`cargo build` でエラーがないことを確認。

### T2: `cmd_watch2` 追加

`watch_session_on_change_label` の直後に追加:

```rust
pub fn cmd_watch2(file: Option<&str>, on_change: &str, debounce_ms: u64) {
    let session = WatchSession {
        file: file.map(|s| s.to_string()),
        on_change_cmd: on_change.to_string(),
        debounce_ms,
    };
    // collect_watch_paths は Vec<PathBuf> を返す（PathBuf は Ord 実装済み）
    let mut files = collect_watch_paths(session.file.as_deref());
    files.sort();
    files.dedup();

    if files.is_empty() {
        eprintln!("error: no .fav files found to watch");
        process::exit(1);
    }

    let mut dirs = BTreeSet::new();
    for file_path in &files {
        if let Some(parent) = file_path.parent() {
            dirs.insert(parent.to_path_buf());
        }
    }

    eprintln!("{}", watch_session_on_change_label(&session));
    eprintln!("[watch] watching {} files for changes...", files.len());

    // 変更検知ループ（既存 cmd_watch と同様の構造）
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| { let _ = tx.send(res); },
        Config::default(),
    ).unwrap_or_else(|e| {
        eprintln!("error: could not create watcher: {e}");
        process::exit(1);
    });

    for dir in &dirs {
        watcher.watch(dir, RecursiveMode::NonRecursive).unwrap_or_else(|e| {
            eprintln!("error: could not watch {}: {e}", dir.display());
            process::exit(1);
        });
    }

    let debounce = Duration::from_millis(session.debounce_ms);
    loop {
        let Ok(event) = rx.recv() else { break };
        let Ok(event) = event else { continue };
        let interesting = matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any
        );
        if !interesting { continue; }
        while rx.recv_timeout(debounce).is_ok() {}
        eprintln!("[watch] Change detected.");
        eprintln!("{}", watch_session_on_change_label(&session));
        // on_change_cmd をシェル経由で実行
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/C", &session.on_change_cmd])
            .status();
        #[cfg(not(target_os = "windows"))]
        let _ = std::process::Command::new("sh")
            .args(["-c", &session.on_change_cmd])
            .status();
        eprintln!("[watch] watching {} files for changes...", files.len());
    }
}
```

`cargo build` でエラーがないことを確認。

### T3: `main.rs` — `--on-change` フラグ対応

`Some("watch")` アームを確認し、`--on-change` フラグを検出して `cmd_watch2` を呼ぶ分岐を追加:

```rust
Some("watch") => {
    // --on-change フラグを検索
    if let Some(pos) = args.iter().position(|a| a == "--on-change") {
        let on_change = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("");
        let file = args.get(2).map(|s| s.as_str());
        crate::driver::cmd_watch2(file, on_change, 500);
    } else {
        // 既存処理
        ...
    }
}
```

`cargo build` でエラーがないことを確認。

### T4: `v727000_tests` モジュール追加

`v726000_tests` モジュールの直後に追加:

```rust
#[cfg(test)]
mod v727000_tests {
    use super::{WatchSession, watch_session_on_change_label};

    #[test]
    fn watch2_session_field_defaults() {
        // WatchSession 構造体のフィールドが正しく設定されることを確認
        let session = WatchSession {
            file: Some("pipeline.fav".to_string()),
            on_change_cmd: "fav check".to_string(),
            debounce_ms: 500,
        };
        assert_eq!(session.on_change_cmd, "fav check");
        assert_eq!(session.debounce_ms, 500);
    }

    #[test]
    fn watch2_runs_custom_command() {
        let session = WatchSession {
            file: None,
            on_change_cmd: "fav check && fav run --dry-run".to_string(),
            debounce_ms: 500,
        };
        let label = watch_session_on_change_label(&session);
        assert!(
            label.contains("fav check && fav run --dry-run"),
            "label should contain the on_change_cmd, got: {}",
            label
        );
    }
}
```

`cargo test v727000` で 2 件 pass することを確認。

### T5: バージョン更新

- `fav/Cargo.toml`: `version = "72.6.0"` → `version = "72.7.0"`
- `driver.rs` 内の `version = \"72.6.0\"` 文字列を `version = \"72.7.0\"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.6.0"` を `"72.7.0"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.6.0"` を `"72.7.0"` に replace_all
- 残存 72.6.0 はコメント・セクションヘッダーのみで意図的保持を確認

### T6: 部分テスト確認

```
cargo test v727000
```
2 件 pass を確認。

### T7: 全体テスト確認

```
cargo test
```
3637 tests pass（0 failures）を確認。

### T8: `CHANGELOG.md` 更新

- 先頭に `## [v72.7.0]` エントリを追加

### T9: `versions/current.md` 更新

- 「進行中バージョン」を `v72.7.0` に更新
- 「次に切る版」を `v72.8.0` に更新

### T10: 最終確認

- `cargo test v727000` 2 件 pass
- `cargo test` 全体 3637 pass
- `fav/Cargo.toml` バージョン = `72.7.0`
- `WatchSession` 構造体 + `watch_session_on_change_label` 存在
- `cmd_watch2` 存在
- `main.rs` に `--on-change` フラグ対応
- `versions/current.md` の「進行中バージョン」が `v72.7.0`、「次に切る版」が `v72.8.0`
