# Spec: v80.2.0 — `GoldenDataset` / ゴールデンデータセット比較

## Background

v80.1.0 で `TestCase` / `TestSuite` 型基盤を構築した。
本バージョンでは「期待値ファイルとパイプライン出力を比較する型」として `GoldenDataset` を追加する。
`GoldenDataset` はパイプラインの出力行を保持し、`compare_golden` でどの行が一致しないかを型として記録する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.2.0 セクション）

## Goals

- `GoldenDataset` 構造体を `test_framework.rs` に追加する
- `GoldenCompareResult` 構造体を追加する
- `compare_golden` / `format_golden_diff` 関数を実装する
- `load_golden_dataset` 関数を実装する（ファイル I/O あり → `#[cfg(not(target_arch = "wasm32"))]`）
- テスト 2 件を追加して **3811 + 2 = 3813** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

#[derive(Debug)]
pub struct GoldenDataset {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct GoldenCompareResult {
    pub matches: bool,
    pub diff_rows: Vec<usize>,
}

/// 実際の出力と期待値を行単位で比較する。
/// 行数が異なる場合は短い方を基準に比較し、超過分はすべて diff として記録する。
pub fn compare_golden(
    actual: &GoldenDataset,
    expected: &GoldenDataset,
) -> GoldenCompareResult;

/// diff 結果をテキスト形式で返す。
/// - 一致: `"OK: datasets match"`
/// - 不一致: `"DIFF: N row(s) differ: [0, 2, ...]"`
pub fn format_golden_diff(result: &GoldenCompareResult) -> String;

/// CSV ファイルを読み込んで GoldenDataset を構築する。
/// - 各行をカンマで分割し、`Vec<String>` を行として追加する
/// - 空行はスキップする
/// - ファイル I/O を使うため WASM 環境では利用不可
#[cfg(not(target_arch = "wasm32"))]
pub fn load_golden_dataset(path: &str) -> Result<GoldenDataset, String>;
```

### `compare_golden` の動作

1. `actual.rows` と `expected.rows` を同インデックスで比較
2. 行が異なる（`actual.rows[i] != expected.rows[i]`）場合は `diff_rows` に `i` を追加
3. 行数が異なる場合、超過行インデックスをすべて `diff_rows` に追加
4. `diff_rows.is_empty()` のとき `matches = true`

### `load_golden_dataset` の CSV 形式

```csv
alice,30,engineer
bob,25,designer
```

→ `rows = [["alice", "30", "engineer"], ["bob", "25", "designer"]]`

## Success Criteria

- `cargo test` が **3813 tests**, 0 failures
- `golden_dataset_compare_pass`: 同一内容の2データセットを比較して `matches = true`、`diff_rows` が空
- `golden_dataset_compare_fail_shows_diff`: 異なる行を持つ2データセットを比較して `matches = false`、`diff_rows` が正しい行インデックスを含む

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `GoldenDataset` / `GoldenCompareResult` / `compare_golden` / `format_golden_diff` / `load_golden_dataset` |
| `fav/src/driver.rs` | 追記 | `mod v80200_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`test_framework` モジュールはすでに宣言済み）。

## 注記

- テストモジュール（`mod v80200_tests`）からのインポートは `use fav_core::test_framework::*;` を使用する（v80.1.0 の慣例統一）。
- `load_golden_dataset` は `#[cfg(not(target_arch = "wasm32"))]` が付いているため WASM テストから直接呼べないが、`driver.rs` テストは非 WASM 環境で実行されるため問題なし。

## Error Codes

新規エラーコードなし。
