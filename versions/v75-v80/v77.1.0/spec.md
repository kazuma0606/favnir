# v77.1.0 仕様書 — `PipelineInvariant` 型基盤

Date: 2026-08-15
Status: 計画中

---

## Background

パイプラインの不変条件（invariant）を型として表現する基盤を提供する。「フィルターは行数を増やさない」「出力件数は入力の上限以下」などの性質を `PipelineInvariant` 構造体で宣言し、`check_count_invariant` でランタイム検証する。将来の `fav verify` コマンドに向けた型基盤。

---

## Goals

1. `InvariantCheckPoint` enum（Pre / Post / Both）を追加する
2. `PipelineInvariant` 構造体（name: String, expression: String, check_point: InvariantCheckPoint）を追加する
3. `InvariantViolation` 構造体（invariant_name: String, expected: String, actual: String）を追加する
4. `check_count_invariant(expected_max: usize, actual: usize, name: &str) -> Result<(), InvariantViolation>` を追加する
5. Rust テスト 2 件を追加し 3738 tests に到達する

---

## 型・関数仕様

### `InvariantCheckPoint` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum InvariantCheckPoint {
    Pre,
    Post,
    Both,
}
```

---

### `PipelineInvariant` 構造体

```rust
#[derive(Debug, Clone)]
pub struct PipelineInvariant {
    pub name:        String,
    pub expression:  String,
    pub check_point: InvariantCheckPoint,
}
```

---

### `InvariantViolation` 構造体

```rust
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub invariant_name: String,
    pub expected:       String,
    pub actual:         String,
}
```

---

### `check_count_invariant`

```rust
pub fn check_count_invariant(
    expected_max: usize,
    actual: usize,
    name: &str,
) -> Result<(), InvariantViolation>
```

**動作:**
- `actual <= expected_max` → `Ok(())`
- `actual > expected_max` → `Err(InvariantViolation { invariant_name: name.to_string(), expected: format!("<= {}", expected_max), actual: actual.to_string() })`

---

## テスト仕様

### `invariant_count_passes`

```rust
// 不変条件を満たすケース: actual <= expected_max
let result = check_count_invariant(100, 80, "row_count_invariant");
assert!(result.is_ok());

// 境界値: actual == expected_max
let result2 = check_count_invariant(50, 50, "exact_boundary");
assert!(result2.is_ok());

// PipelineInvariant 構造体の構築確認
let inv = PipelineInvariant {
    name:        "row_count_invariant".to_string(),
    expression:  "output.row_count <= input.row_count".to_string(),
    check_point: InvariantCheckPoint::Post,
};
assert_eq!(inv.check_point, InvariantCheckPoint::Post);
```

### `invariant_count_violated`

```rust
// 不変条件違反: actual > expected_max
let result = check_count_invariant(100, 150, "row_count_invariant");
assert!(result.is_err());
let violation = result.unwrap_err();
assert_eq!(violation.invariant_name, "row_count_invariant");
assert!(violation.expected.contains("100"));
assert_eq!(violation.actual, "150");
```

---

## Success Criteria

- `InvariantCheckPoint` enum が定義されている（Pre / Post / Both、PartialEq 付き）
- `PipelineInvariant` / `InvariantViolation` 構造体が定義されている
- `check_count_invariant` が `actual <= expected_max` で Ok、超過で Err を返す
- 違反時の `InvariantViolation` に `invariant_name`・`expected`（`<= N` 形式）・`actual` が含まれる
- `invariant_count_passes` が pass
- `invariant_count_violated` が pass
- `cargo test` が 3738 tests all pass
- `driver.rs` 内の `77.0.0` バージョン文字列アサーションがすべて `77.1.0` に更新されている
- `CHANGELOG.md` の先頭に v77.1.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `InvariantCheckPoint`, `PipelineInvariant`, `InvariantViolation`, `check_count_invariant`, `v771000_tests` を追加
- `CHANGELOG.md` — v77.1.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.0.0` → `77.1.0` に更新

---

## 依存

- 新規スプリント（Verifiable Pipelines）の最初の版。既存型への直接依存なし

---

## 対象外

- `contract` 構文への Favnir 言語統合（将来バージョン）。ロードマップのコードサンプルは将来構文のイメージであり、v77.1.0 では `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `fav verify` CLI コマンド（v77.5.0 で実装予定）
- フィルター系不変条件（v77.2.0 で実装予定）
