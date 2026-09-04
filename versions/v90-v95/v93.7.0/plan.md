# Plan: v93.7.0 — 生成コードの `fav fmt` 適用

## Implementation Steps

### Step 1: `fav/src/sap_metadata.rs` に `apply_fmt_to_generated` を追加

ファイル末尾（`enum_type_to_favnir` の直後）に関数を追加する。

```rust
/// 生成した Favnir ソースを fav fmt に通して標準フォーマットを適用する。
/// VM の `fmt_source_raw` primitive と同じバックエンド（`fmt_source_str`）を使用する。
/// フォーマット失敗時は元の `src` をそのまま返す。
pub fn apply_fmt_to_generated(src: &str) -> String {
    let formatted = crate::compiler_fav_runner::fmt_source_str(src)
        .unwrap_or_else(|_| src.to_string());
    formatted
}
```

`crate::compiler_fav_runner::fmt_source_str` は `pub fn` で公開済み（`compiler_fav_runner.rs:219`）。

### Step 2: `cargo build` でコンパイル確認

```bash
cargo build
```

### Step 3: `driver.rs` に `mod v93700_tests` を追加

`mod v93600_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v93700_tests {
    #[test]
    fn sap_metadata_generator_applies_fmt() {
        let src = std::fs::read_to_string("src/sap_metadata.rs").unwrap();
        assert!(
            src.contains("fmt_source_raw"),
            "sap_metadata.rs should reference fmt_source_raw"
        );
    }

    #[test]
    fn infer_output_is_formatted() {
        let src = std::fs::read_to_string("src/sap_metadata.rs").unwrap();
        assert!(
            src.contains("formatted"),
            "sap_metadata.rs should contain 'formatted' variable"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4134 tests, 0 failures`

### Step 5: CHANGELOG.md を更新する

### Step 6: ロードマップ本文を修正する（T6b）

`roadmap-v93.1-v94.0.md` で以下を修正する:
- v93.7.0 詳細セクション中の `4119 + 2 = 4121` → `4132 + 2 = 4134`
- v93.7.0 詳細セクション中の `formatted` または `format` が含まれる」 → `formatted` が含まれる」
- v93.8.0〜v94.0.0 詳細セクションのテスト数を version table の値に一括修正

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
