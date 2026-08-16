# v77.4.0 仕様書 — Join 系不変条件

Date: 2026-08-15
Status: 計画中

---

## Background

Join の種類に応じた不変条件（行数の保持・NULL 発生）を検証する型基盤を提供する。`JoinType` enum・`JoinNullPolicy` enum・`JoinInvariant` 構造体・`check_join_invariant` 関数を追加する。v77.1.0 の `InvariantViolation` を再利用。将来の `fav verify` コマンドの Join 不変条件サポートに向けた型基盤。

---

## Goals

1. `JoinType` enum（Inner / Left / Right / Full）を追加する
2. `JoinNullPolicy` enum（Fail / Warn / Allow）を追加する
3. `JoinInvariant` 構造体（join_type: JoinType, null_policy: JoinNullPolicy）を追加する
4. `check_join_invariant(left_count: usize, result_count: usize, null_count: usize, inv: &JoinInvariant) -> Result<(), InvariantViolation>` を追加する
5. Rust テスト 2 件を追加し 3744 tests に到達する

---

## 型・関数仕様

### `JoinType` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
```

---

### `JoinNullPolicy` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum JoinNullPolicy {
    Fail,
    Warn,
    Allow,
}
```

---

### `JoinInvariant` 構造体

```rust
#[derive(Debug, Clone)]
pub struct JoinInvariant {
    pub join_type:   JoinType,
    pub null_policy: JoinNullPolicy,
}
```

---

### `check_join_invariant`

```rust
pub fn check_join_invariant(
    left_count: usize,
    result_count: usize,
    null_count: usize,
    inv: &JoinInvariant,
) -> Result<(), InvariantViolation>
```

**動作（2 段階チェック）:**

**Step 1 — JoinType による行数チェック:**

| `join_type` | 条件 | Err 時の invariant_name | expected | actual |
|---|---|---|---|---|
| Left / Full | `result_count >= left_count` | `"join_row_count"` | `">= N (left_count)"` | `result_count.to_string()` |
| Inner / Right | チェックなし | — | — | — |

> **Full join の根拠**: Full outer join では左テーブルのアンマッチ行も NULL 付きで結果に保持されるため、result_count は必ず left_count 以上になる。Left と同じ条件を適用する。

**Step 2 — NullPolicy による NULL チェック:**

| `null_policy` | 条件 | Err 時の invariant_name | expected | actual |
|---|---|---|---|---|
| Fail | `null_count == 0` | `"join_null_count"` | `"0 nulls (Fail policy)"` | `null_count.to_string()` |
| Warn / Allow | チェックなし | — | — | — |

両チェックが通れば `Ok(())`。

---

## テスト仕様

### `join_invariant_inner_no_nulls`

```rust
// Rust テスト（driver.rs 内）
// Inner + Fail + null_count=0 → Ok
let inv = JoinInvariant {
    join_type:   JoinType::Inner,
    null_policy: JoinNullPolicy::Fail,
};
let result = check_join_invariant(100, 80, 0, &inv);
assert!(result.is_ok());

// Inner + Fail + null_count=5 → Err（null_policy Fail 違反）
let result2 = check_join_invariant(100, 80, 5, &inv);
assert!(result2.is_err());
let violation = result2.unwrap_err();
assert_eq!(violation.invariant_name, "join_null_count");
assert_eq!(violation.actual, "5");
```

### `join_invariant_left_preserves_rows`

```rust
// Rust テスト（driver.rs 内）
// Left + Allow + result_count >= left_count → Ok
let inv = JoinInvariant {
    join_type:   JoinType::Left,
    null_policy: JoinNullPolicy::Allow,
};
let result = check_join_invariant(100, 120, 20, &inv);
assert!(result.is_ok());

// Left + Allow + result_count < left_count → Err（行数保持違反）
let result2 = check_join_invariant(100, 80, 0, &inv);
assert!(result2.is_err());
let violation = result2.unwrap_err();
assert_eq!(violation.invariant_name, "join_row_count");
assert!(violation.expected.contains("100"));
assert_eq!(violation.actual, "80");
```

---

## Success Criteria

- `JoinType` / `JoinNullPolicy` enum が定義されている（Debug / Clone / PartialEq 付き）
- `JoinInvariant` 構造体が定義されている（Debug / Clone 付き）
- `check_join_invariant` が JoinType による行数チェックと NullPolicy による NULL チェックの 2 段階を正しく実装している
- Left / Full join で `result_count < left_count` → Err（invariant_name: `"join_row_count"`）
- Fail policy で `null_count > 0` → Err（invariant_name: `"join_null_count"`）
- `join_invariant_inner_no_nulls` が pass
- `join_invariant_left_preserves_rows` が pass
- `cargo test` が 3744 tests all pass
- `driver.rs` 内の `77.3.0` バージョン文字列アサーションがすべて `77.4.0` に更新されている
- `CHANGELOG.md` の先頭に v77.4.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `JoinType`, `JoinNullPolicy`, `JoinInvariant`, `check_join_invariant`, `v774000_tests` を追加
- `CHANGELOG.md` — v77.4.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.3.0` → `77.4.0` に更新

---

## 依存

- v77.1.0 の `InvariantViolation` 構造体を再利用（`fav/src/driver.rs` 内 `// --- v77.1.0: PipelineInvariant 型基盤 ---` ブロック参照）

---

## 対象外

- ロードマップのコードサンプル（`contract JoinPipeline { ... }`）は将来構文のイメージであり、v77.4.0 では `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `fav verify` CLI コマンド（v77.5.0 で実装予定）
- Right join の右テーブル行数保証チェック（`result_count >= right_count`）: 現在の `check_join_invariant` シグネチャに `right_count` パラメータが存在しないため v77.4.0 では実装しない。将来バージョンで拡張予定
