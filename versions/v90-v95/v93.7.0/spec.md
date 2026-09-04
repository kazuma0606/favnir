# Spec: v93.7.0 — 生成コードの `fav fmt` 適用

## Background

v93.2.0〜v93.4.0 で実装した `entity_type_to_favnir` / `enum_type_to_favnir` 等の SAP 型生成関数は、
Favnir ソースコード文字列を直接組み立てる形式（手動インデント）で出力している。
v93.7.0 では、生成済みの Favnir ソースを `fav fmt`（`fmt_source_str`）に通して標準フォーマットに統一する
ヘルパー関数を `sap_metadata.rs` に追加する。

## Goals

1. `sap_metadata.rs` に `apply_fmt_to_generated(src: &str) -> String` ヘルパーを追加する。
   - `crate::compiler_fav_runner::fmt_source_str` を呼ぶ（VM の `fmt_source_raw` primitive と同じバックエンド）
   - フォーマット失敗時は元の文字列をフォールバックとして返す
2. `driver.rs` に `mod v93700_tests`（2 件）を追加し、4,134 tests を達成する。

## Implementation Details

### `fav/src/sap_metadata.rs`

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

注意: `sap_metadata.rs` と `compiler_fav_runner.rs` は同一クレート内のため `crate::` パス参照で呼び出せる。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | `apply_fmt_to_generated` 関数を追加 |
| `fav/src/driver.rs` | `mod v93700_tests` を追加（2 テスト） |
| `CHANGELOG.md` | v93.7.0 エントリを追加 |
| `versions/roadmap/roadmap-v93.1-v94.0.md` | v93.7.0 本文のテスト数を修正（T6b） |

## Success Criteria

- `cargo test 2>&1 | grep "test result"` → `4134 tests, 0 failures`
- `cargo clippy --locked -- -D warnings` → pass
- `sap_metadata_generator_applies_fmt`: `sap_metadata.rs` に `fmt_source_raw` が含まれる
- `infer_output_is_formatted`: `sap_metadata.rs` に `formatted` が含まれる

## Notes

- v93.7.0 では `apply_fmt_to_generated` の定義のみ。実際の生成関数（`entity_type_to_favnir` 等）への組み込みは v93.8.0 以降で行う。
- テスト `infer_output_is_formatted` は `formatted` 変数名の存在で確認する（`format!` マクロとの区別）。
