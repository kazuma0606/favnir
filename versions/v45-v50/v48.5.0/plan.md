# Plan: v48.5.0 — import エイリアス完全化 + 旧構文 deprecation

## 作業順序

### Step 1: `lint.rs` に W035 追加

1. `run_lint` 関数の `errors` return 行の直前（W034 コメントの直後）に登録行を追加
2. ファイル末尾に `check_w035_legacy_import_rune` 関数を追加

変更箇所:

```
// v45.4.0: W034 ...
errors                   ← この行を
↓
// v45.4.0: W034 ...
// v48.5.0: W035
check_w035_legacy_import_rune(program, &mut errors);
errors
```

追加する関数:

```rust
fn check_w035_legacy_import_rune(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::ImportDecl { kind, path, span, .. } = item {
            if *kind == ImportKind::Legacy {
                errors.push(LintError {
                    code: "W035".to_string(),
                    message: format!(
                        "legacy import syntax `import rune \"{}\"` is deprecated; \
                         use `import {}` (package) or `import \"./path\"` (local) instead",
                        path, path
                    ),
                    span: span.clone(),
                });
            }
        }
    }
}
```

### Step 2: `driver.rs` にテスト追加

`v484000_tests` の直前に `v485000_tests` モジュールを挿入（2テスト）。

### Step 3: `Cargo.toml` version 更新

`"48.4.0"` → `"48.5.0"`

### Step 4: 完了処理

- `cargo test` 3055 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v48.5.0 エントリ追加
- `versions/current.md` 更新（v48.5.0・3055 tests・進行中 v48.6.0）
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.5.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/lint.rs` | `check_w035_legacy_import_rune` 追加 + `run_lint` 登録 |
| `fav/src/driver.rs` | `v485000_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v48.5.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/v45-v50/v48.5.0/tasks.md` | COMPLETE 更新 |
