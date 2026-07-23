# Plan: v48.7.0 — rune.toml 標準化

## 作業順序

### Step 1: `toml.rs` に `validate_rune_toml` 追加

ファイル末尾（`parse_kv` 関数の直後）に追加:

```rust
pub fn validate_rune_toml(content: &str) -> Vec<String> {
    // [rune] セクション必須チェック、name/version/entry フィールド必須チェック
    // [connection] セクション非標準チェック
}
```

### Step 2: `driver.rs` に `v487000_tests` 追加

`v486000_tests` の直前に挿入（2テスト）:
- `rune_toml_standard_format`: valid な rune.toml で空 Vec を返すことを確認
- `rune_toml_no_connection_section`: `[connection]` セクション付きで errors に `"connection"` が含まれることを確認

### Step 3: `Cargo.toml` version 更新

`"48.6.0"` → `"48.7.0"`

### Step 4: 完了処理

- `cargo test` 3060 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v48.7.0 エントリ追加
- `versions/current.md` 更新（v48.7.0・3060 tests・進行中 v48.8.0）
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.7.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/toml.rs` | `validate_rune_toml` 追加 |
| `fav/src/driver.rs` | `v487000_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v48.7.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/v45-v50/v48.7.0/tasks.md` | COMPLETE 更新 |
