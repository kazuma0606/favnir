# v77.3.0 仕様書 — 集約系不変条件

Date: 2026-08-15
Status: 計画中

---

## Background

集約結果（SUM・COUNT・AVG）が持つべき数学的性質（非負・非正・境界値内・非NULL）を不変条件として検証する型基盤を提供する。v77.1.0 の `InvariantViolation` を再利用し、`AggregateProperty` enum・`AggregateInvariant` 構造体・`check_aggregate_invariant` 関数を追加する。

---

## Goals

1. `AggregateProperty` enum（NonNegative / NonPositive / Bounded { min, max } / NonNull）を追加する
2. `AggregateInvariant` 構造体（column: String, property: AggregateProperty）を追加する
3. `check_aggregate_invariant(values: &[f64], inv: &AggregateInvariant) -> Result<(), InvariantViolation>` を追加する
4. Rust テスト 2 件を追加し 3742 tests に到達する

---

## 型・関数仕様

### `AggregateProperty` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateProperty {
    NonNegative,
    NonPositive,
    Bounded { min: f64, max: f64 },
    NonNull,
}
```

---

### `AggregateInvariant` 構造体

```rust
#[derive(Debug, Clone)]
pub struct AggregateInvariant {
    pub column:   String,
    pub property: AggregateProperty,
}
```

---

### `check_aggregate_invariant`

```rust
pub fn check_aggregate_invariant(
    values: &[f64],
    inv: &AggregateInvariant,
) -> Result<(), InvariantViolation>
```

**動作:**

| `property` | 検証ロジック | Err 時の `expected` | Err 時の `actual` |
|---|---|---|---|
| `NonNegative` | 全値 >= 0.0 | `"NonNegative (>= 0.0)"` | 違反値を `"{:.4}"` でフォーマット |
| `NonPositive` | 全値 <= 0.0 | `"NonPositive (<= 0.0)"` | 違反値を `"{:.4}"` でフォーマット |
| `Bounded { min, max }` | 全値が `[min, max]` 内 | `"[{:.4}, {:.4}]"` | 違反値を `"{:.4}"` でフォーマット |
| `NonNull` | `values.is_empty()` なら Err | `"NonNull (non-empty)"` | `"empty"` |

`invariant_name` は常に `inv.column.clone()`。

---

## テスト仕様

### `aggregate_invariant_non_negative_passes`

```rust
// Rust テスト（driver.rs 内）
// NonNegative: 全値 >= 0.0 → Ok
let inv = AggregateInvariant {
    column: "amount".to_string(),
    property: AggregateProperty::NonNegative,
};
let result = check_aggregate_invariant(&[1.0, 2.0, 3.0], &inv);
assert!(result.is_ok());

// NonPositive: 全値 <= 0.0 → Ok
let inv_np = AggregateInvariant {
    column: "delta".to_string(),
    property: AggregateProperty::NonPositive,
};
let result_np = check_aggregate_invariant(&[-1.0, -2.0, 0.0], &inv_np);
assert!(result_np.is_ok());

// NonNull: 非空スライス → Ok
let inv2 = AggregateInvariant {
    column: "score".to_string(),
    property: AggregateProperty::NonNull,
};
let result2 = check_aggregate_invariant(&[42.0], &inv2);
assert!(result2.is_ok());

// Bounded: 全値が [0.0, 100.0] 内 → Ok
let inv3 = AggregateInvariant {
    column: "score".to_string(),
    property: AggregateProperty::Bounded { min: 0.0, max: 100.0 },
};
let result3 = check_aggregate_invariant(&[0.0, 50.0, 100.0], &inv3);
assert!(result3.is_ok());
```

### `aggregate_invariant_bounded_violated`

```rust
// Rust テスト（driver.rs 内）
// Bounded: 150.0 は [0.0, 100.0] の範囲外 → Err
let inv = AggregateInvariant {
    column: "score".to_string(),
    property: AggregateProperty::Bounded { min: 0.0, max: 100.0 },
};
let result = check_aggregate_invariant(&[0.0, 50.0, 150.0], &inv);
assert!(result.is_err());
let violation = result.unwrap_err();
assert_eq!(violation.invariant_name, "score");
assert!(violation.expected.contains("100"));
assert_eq!(violation.actual, "150.0000");

// NonNull: 空スライス → Err
let inv2 = AggregateInvariant {
    column: "amount".to_string(),
    property: AggregateProperty::NonNull,
};
let result2 = check_aggregate_invariant(&[], &inv2);
assert!(result2.is_err());
assert_eq!(result2.unwrap_err().actual, "empty");
```

---

## Success Criteria

- `AggregateProperty` enum が定義されている（Debug / Clone / PartialEq 付き、Bounded はバリアント内フィールド）
- `AggregateInvariant` 構造体が定義されている（Debug / Clone 付き）
- `check_aggregate_invariant` が各 property の検証ロジックを正しく実装している
- `invariant_name` が常に `inv.column` から設定される
- `aggregate_invariant_non_negative_passes` が pass
- `aggregate_invariant_bounded_violated` が pass
- `cargo test` が 3742 tests all pass
- `driver.rs` 内の `77.2.0` バージョン文字列アサーションがすべて `77.3.0` に更新されている
- `CHANGELOG.md` の先頭に v77.3.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `AggregateProperty`, `AggregateInvariant`, `check_aggregate_invariant`, `v773000_tests` を追加
- `CHANGELOG.md` — v77.3.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.2.0` → `77.3.0` に更新

---

## 依存

- v77.1.0 の `InvariantViolation` 構造体を再利用（新規定義なし）

---

## 対象外

- ロードマップのコードサンプル（`contract AggregatePipeline { ... }`）は将来構文のイメージであり、v77.3.0 では `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `fav verify` CLI コマンド（v77.5.0 で実装予定）
- Join 系不変条件（v77.4.0 で実装予定）
