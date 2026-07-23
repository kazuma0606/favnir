# Plan: v48.6.0 — 循環 import 検出 + E0418

## 作業順序

### Step 1: `error_catalog.rs` に E0418 追加

既存の予約コメント行を `ErrorEntry` に差し替える:

```
// ── E0418〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```
↓
```rust
ErrorEntry {
    code: "E0418",
    title: "circular import detected",
    ...
},
// ── E0419: 予約（将来拡張用） ─────────────────────────────────────────────────
```

### Step 2: `driver.rs` に `detect_circular_imports` 追加

`cmd_install_runes` の直後（`install_rune_stubs` / `cmd_install_runes` ブロックの後）に追加。

```rust
pub fn detect_circular_imports(
    graph: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> { ... }
```

DFS カラーリング（0=white / 1=gray / 2=black）で循環を検出し、循環パスを返す。

### Step 3: `driver.rs` に `v486000_tests` 追加

`v485000_tests` の直前に挿入（2テスト）:
- `circular_import_e0418`: a→b→a の循環グラフで `Some(cycle)` が返ることを確認
- `non_circular_import_ok`: a→b→c の非循環グラフで `None` が返ることを確認

### Step 4: `Cargo.toml` version 更新

`"48.5.0"` → `"48.6.0"`

### Step 5: 完了処理

- `cargo test` 3058 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v48.6.0 エントリ追加
- `versions/current.md` 更新（v48.6.0・3058 tests・進行中 v48.7.0）
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.6.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/error_catalog.rs` | E0418 `ErrorEntry` 追加 |
| `fav/src/driver.rs` | `detect_circular_imports` 追加 + `v486000_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v48.6.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/v45-v50/v48.6.0/tasks.md` | COMPLETE 更新 |
