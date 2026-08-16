# v72.7.0 Spec — Hot Reload 改善（`fav watch` 2.0）

Date: 2026-08-12
Status: 完了

---

## 背景

既存の `cmd_watch` は `check` / `test` / `run` のいずれかを固定コマンドとして受け取る。
ユーザーが「変更時に任意のシェルコマンドを実行したい」場合に対応できない。

v72.7.0 では `--on-change <cmd>` フラグを追加し、変更検知時に任意コマンドを実行できるようにする。
また、watch セッションの状態を `WatchSession` 構造体で管理し、テスト可能性を高める。

---

## 目標

```bash
$ fav watch pipeline.fav --on-change "fav check && fav run --dry-run"
Watching pipeline.fav... (Ctrl+C to stop)
[10:32:01] Change detected: pipeline.fav
[10:32:01] Running: fav check && fav run --dry-run
[10:32:01] Type check: OK (0.8s)
[10:32:01] Ready.
```

---

## API / 構文例

```bash
# 従来（変更なし）
$ fav watch pipeline.fav check

# 新機能: --on-change フラグ（ファイルパスの後に配置すること）
$ fav watch pipeline.fav --on-change "fav check && fav run --dry-run"
$ fav watch pipeline.fav --on-change "cargo test"
```

> **注意**: `--on-change` はファイルパスの後に置く。`fav watch --on-change "cmd" file.fav` は非対応（パーサーがファイルパスを `args[2]` で取得するため）。

---

## 実装詳細

### `driver.rs` — `WatchSession` 構造体追加

```rust
pub struct WatchSession {
    pub file: Option<String>,
    pub on_change_cmd: String,
    pub debounce_ms: u64,
}
```

フィールド:
- `file` — 監視対象ファイル（`None` の場合はカレントディレクトリ全体）
- `on_change_cmd` — 変更検知時に実行するシェルコマンド文字列
- `debounce_ms` — デバウンス時間（デフォルト 500ms）

### `driver.rs` — `watch_session_on_change_label` 追加

```rust
pub fn watch_session_on_change_label(session: &WatchSession) -> String
```

変更時のコンソール出力ラベルを構築する純粋関数。
`format!("[watch] Running: {}", session.on_change_cmd)` を返す。
ファイルシステム非依存でテスト可能。

### `driver.rs` — `cmd_watch2` 追加

```rust
pub fn cmd_watch2(file: Option<&str>, on_change: &str, debounce_ms: u64)
```

既存 `cmd_watch` の拡張版。`on_change` に任意コマンド文字列を受け取り、
変更検知時に `std::process::Command::new("cmd").args(["/C", on_change])` (Windows) /
`std::process::Command::new("sh").args(["-c", on_change])` (Unix) で実行する。

既存の `cmd_watch` はそのまま保持する（後方互換）。

### `main.rs` — `--on-change` フラグ対応

```rust
// fav watch pipeline.fav --on-change "fav check && fav run --dry-run"
Some("watch") => {
    // --on-change フラグがある場合は cmd_watch2 を呼ぶ
    // なければ既存の cmd_watch を呼ぶ
}
```

---

## テスト

### `v727000_tests` モジュール

```rust
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
    // watch_session_on_change_label が正しいラベルを返すことを確認
    let session = WatchSession {
        file: None,
        on_change_cmd: "fav check && fav run --dry-run".to_string(),
        debounce_ms: 500,
    };
    let label = watch_session_on_change_label(&session);
    assert!(label.contains("fav check && fav run --dry-run"),
        "label should contain the on_change_cmd, got: {}", label);
}

#[test]
fn watch2_on_change_label_format() {
    // ラベルの prefix と on_change_cmd 含有を確認
    let session = WatchSession {
        file: Some("main.fav".to_string()),
        on_change_cmd: "cargo test".to_string(),
        debounce_ms: 500,
    };
    let label = watch_session_on_change_label(&session);
    assert!(label.starts_with("[watch]"), "label should start with [watch], got: {}", label);
    assert!(label.contains("cargo test"), "label should contain the cmd, got: {}", label);
}
```

---

## 成功基準

- `cargo test v727000` で 3 件 pass
- `cargo test` 全体で 3638 tests pass（3635 + 3）
- `fav/Cargo.toml` のバージョンが `72.7.0` であること
- `WatchSession` 構造体と `watch_session_on_change_label` が pub
- `cmd_watch2` が pub で存在すること
- 既存 `cmd_watch` はリグレッションなし

---

## スコープ外

- 差分ステージ検出（変更されたステージの上流のみ再実行）— 複雑度が高く v73.x 以降
- `~/fav_watch_history` 等の永続化
- サイト側ドキュメント更新（v73.x 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — `WatchSession` 構造体 + `watch_session_on_change_label` + `cmd_watch2` + `v727000_tests` + バージョン更新
- `fav/src/main.rs` — `--on-change` フラグ対応（`cmd_watch2` 呼び出し追加）
- `fav/Cargo.toml` — version `72.6.0` → `72.7.0`
- `CHANGELOG.md` — v72.7.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v72.7.0 に更新
