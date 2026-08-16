# v77.2.0 仕様書 — フィルター系不変条件

Date: 2026-08-15
Status: 計画中

---

## Background

フィルター操作が持つべき性質（行数が減る・比率の上限など）を不変条件として検証する型基盤を提供する。v77.1.0 で導入した `InvariantViolation` を再利用し、フィルター固有の `FilterInvariant` 構造体と検証関数・レポート関数を追加する。将来の `fav verify` コマンドのフィルター不変条件サポートに向けた型基盤。

---

## Goals

1. `FilterInvariant` 構造体（expected_ratio_min: f64, expected_ratio_max: f64）を追加する
2. `check_filter_invariant(input_count: usize, output_count: usize, inv: &FilterInvariant) -> Result<(), InvariantViolation>` を追加する
3. `format_filter_invariant_report(inv: &FilterInvariant, result: &Result<(), InvariantViolation>) -> String` を追加する
4. Rust テスト 2 件を追加し 3740 tests に到達する

---

## 型・関数仕様

### `FilterInvariant` 構造体

```rust
#[derive(Debug, Clone)]
pub struct FilterInvariant {
    pub expected_ratio_min: f64,
    pub expected_ratio_max: f64,
}
```

`expected_ratio_min`〜`expected_ratio_max` はフィルター後の行数比率（output_count / input_count）の許容範囲。

---

### `check_filter_invariant`

```rust
pub fn check_filter_invariant(
    input_count: usize,
    output_count: usize,
    inv: &FilterInvariant,
) -> Result<(), InvariantViolation>
```

**動作:**
- `input_count == 0` → `Ok(())` （ゼロ除算回避）
- `ratio = output_count as f64 / input_count as f64`
- `ratio >= expected_ratio_min && ratio <= expected_ratio_max` → `Ok(())`
- それ以外 → `Err(InvariantViolation { invariant_name: "filter_ratio".to_string(), expected: format!("[{:.4}, {:.4}]", min, max), actual: format!("{:.4}", ratio) })`

---

### `format_filter_invariant_report`

```rust
pub fn format_filter_invariant_report(
    inv: &FilterInvariant,
    result: &Result<(), InvariantViolation>,
) -> String
```

**動作:**
- `Ok(())` → `"filter_ratio OK: ratio in [{:.4}, {:.4}]"` （min・max を埋め込む）
- `Err(v)` → `"filter_ratio VIOLATED: expected {}, actual {}"` （violation フィールドを埋め込む）

---

## テスト仕様

### `filter_invariant_ratio_valid`

```rust
// Rust テスト（driver.rs 内）
// 比率が許容範囲内: input=100, output=50 → ratio=0.5, bounds=[0.01, 1.0]
let inv = FilterInvariant { expected_ratio_min: 0.01, expected_ratio_max: 1.0 };
let result = check_filter_invariant(100, 50, &inv);
assert!(result.is_ok());

// format_filter_invariant_report が "OK" を含む
let report = format_filter_invariant_report(&inv, &result);
assert!(report.contains("OK"));

// input_count == 0 → Ok（ゼロ除算なし）
let result2 = check_filter_invariant(0, 0, &inv);
assert!(result2.is_ok());
```

### `filter_invariant_ratio_violated`

```rust
// Rust テスト（driver.rs 内）
// 比率が許容範囲外: input=100, output=0 → ratio=0.0, min=0.01
let inv = FilterInvariant { expected_ratio_min: 0.01, expected_ratio_max: 1.0 };
let result = check_filter_invariant(100, 0, &inv);
assert!(result.is_err());
let violation = result.as_ref().unwrap_err();
assert_eq!(violation.invariant_name, "filter_ratio");
assert!(violation.expected.contains("0.01"));
assert_eq!(violation.actual, "0.0000");

// format_filter_invariant_report が "VIOLATED" を含む
let report = format_filter_invariant_report(&inv, &result);
assert!(report.contains("VIOLATED"));
```

---

## Success Criteria

- `FilterInvariant` 構造体が定義されている（Debug / Clone 付き）
- `check_filter_invariant` が ratio in bounds で Ok、範囲外で Err を返す
- input_count == 0 の場合 Ok（ゼロ除算なし）
- 違反時の `InvariantViolation.invariant_name` が `"filter_ratio"`
- `format_filter_invariant_report` が Ok 時に "OK"、Err 時に "VIOLATED" を含む
- `filter_invariant_ratio_valid` が pass
- `filter_invariant_ratio_violated` が pass
- `cargo test` が 3740 tests all pass
- `driver.rs` 内の `77.1.0` バージョン文字列アサーションがすべて `77.2.0` に更新されている
- `CHANGELOG.md` の先頭に v77.2.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `FilterInvariant`, `check_filter_invariant`, `format_filter_invariant_report`, `v772000_tests` を追加
- `CHANGELOG.md` — v77.2.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.1.0` → `77.2.0` に更新

---

## 依存

- v77.1.0 の `InvariantViolation` 構造体を再利用（新規定義なし）

---

## 対象外

- ロードマップのコードサンプル（`contract FilterPipeline { ... }`）は将来構文のイメージであり、v77.2.0 では `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `fav verify` CLI コマンド（v77.5.0 で実装予定）
- 集約系不変条件（v77.3.0 で実装予定）
