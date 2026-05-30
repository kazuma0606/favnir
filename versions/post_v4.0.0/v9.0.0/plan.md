# Favnir v9.0.0 実装計画

Date: 2026-05-30

---

## Phase A: `--legacy` フラグ非推奨化（main.rs / driver.rs）

### A-1: driver.rs — 非推奨警告追加

`cmd_run` の legacy ブランチ冒頭に警告を追加:

```rust
pub fn cmd_run(file: Option<&str>, db_url: Option<&str>, legacy: bool) {
    load_run_config(file);

    let (source_path, proj) = find_entry(file);
    let use_favnir = !legacy;

    if legacy {
        eprintln!(
            "warning: --legacy is deprecated since v9.0.0 and will be removed in a future version."
        );
        eprintln!(
            "         The Favnir pipeline (checker.fav + compiler.fav) handles all modes."
        );
    }

    if use_favnir { ... }
```

### A-2: main.rs — `--help` テキスト更新

`--legacy` のヘルプ文字列に `[deprecated]` を追加:

```rust
// Before:
.arg(clap::Arg::new("legacy").long("legacy").help("Use Rust pipeline"))

// After:
.arg(clap::Arg::new("legacy").long("legacy")
    .help("[deprecated since v9.0.0] Use Rust compiler pipeline instead of Favnir"))
```

---

## Phase B: バージョン定数更新（Cargo.toml）

```toml
# Before:
version = "5.0.0"

# After:
version = "9.0.0"
```

※ バイナリの `fav --version` 出力が `fav 9.0.0` になる。

---

## Phase C: 宣言テスト追加（driver.rs）

新モジュール `self_hosting_complete_tests` を追加:

```rust
#[cfg(test)]
mod self_hosting_complete_tests {
    /// v9.0.0 milestone: all fav run modes use Favnir pipeline.
    ///
    /// Covered by existing dispatch tests:
    ///   - dispatch_single_file_uses_favnir        (v8.5.0)
    ///   - dispatch_rune_import_uses_favnir_pipeline (v8.6.0)
    ///   - dispatch_project_uses_favnir_pipeline    (v8.11.0)
    ///
    /// This test is a compile-time assertion that the key functions exist and are public.
    #[test]
    fn v900_self_hosting_apis_exist() {
        // compile_src_str_to_bytes — shared compile helper
        let _: fn(&str) -> Result<Vec<u8>, String> =
            crate::compiler_fav_runner::compile_src_str_to_bytes;
        // compile_project_to_bytes — project mode Favnir compile
        let _: fn(&str, &std::path::Path, &crate::toml::FavToml) -> Result<Vec<u8>, String> =
            crate::compiler_fav_runner::compile_project_to_bytes;
    }
}
```

---

## Phase D: 確認・ドキュメント

```
cargo test v900                  ← 新規テスト通ること
cargo test dispatch              ← 全 dispatch テスト通ること
cargo test checker_fav           ← self-check 通ること
cargo test                       ← 全件通ること（1135+）
```

tasks.md 完了・MEMORY.md 更新・commit。

---

## 実装上の注意

### Cargo.toml バージョン変更の影響

`version = "9.0.0"` に変更すると `fav --version` の出力が変わるが、
テストやバイナリの動作には影響しない。

### `--legacy` 削除は v9.0.0 では行わない

非推奨警告のみ。完全削除は後続バージョン（v9.x.0）で検討。
既存のスクリプトや CI が `--legacy` を使っている場合の移行期間を確保する。
