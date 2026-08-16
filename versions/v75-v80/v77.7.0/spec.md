# v77.7.0 仕様書 — 反例自動生成

Date: 2026-08-16
Status: 計画中

---

## Background

不変条件を「破る」サンプルデータ（反例）を自動生成し、不変条件の設計ミスを早期に発見する機能を追加する。`CounterExampleResult` 構造体と `generate_counter_example_values` 関数を追加する。v77.3.0 の `AggregateInvariant` / `check_aggregate_invariant` を活用し、境界値付近の候補を生成して不変条件が実際に破れるかを検証する基盤を構築する。

---

## Goals

1. `CounterExampleResult` 構造体（invariant_name: String, example: Vec<f64>, violates: bool）を追加する
2. `generate_counter_example_values(inv: &AggregateInvariant, seed: u64) -> CounterExampleResult` を追加する
3. Rust テスト 2 件を追加し 3750 tests に到達する

---

## 型・関数仕様

### `CounterExampleResult` 構造体

```rust
#[derive(Debug, Clone)]
pub struct CounterExampleResult {
    pub invariant_name: String,
    pub example:        Vec<f64>,
    pub violates:       bool,
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `invariant_name` | String | 対象の不変条件名（`AggregateInvariant::column` の値） |
| `example` | Vec<f64> | 生成された候補値のスライス（`check_aggregate_invariant` に渡したもの） |
| `violates` | bool | `check_aggregate_invariant` が `Err` を返した場合 `true` |

> **設計注記**: `f64` フィールド（`example: Vec<f64>`）を含むため `Eq` は derive しない。`PartialEq` も同様に derive しない（`f64` の NaN 等価性の問題）。`Debug` / `Clone` のみ。

---

### `generate_counter_example_values`

```rust
pub fn generate_counter_example_values(inv: &AggregateInvariant, seed: u64) -> CounterExampleResult
```

**動作:**

`seed % 2 == 0`（偶数シード）の場合: adversarial 候補を生成する
```
candidates = [0.0, -0.001, -1.0, 1.0]
```
— 負値を含むため、`NonNegative` 不変条件では `check_aggregate_invariant` が `Err` を返す（`violates: true`）

`seed % 2 == 1`（奇数シード）の場合: 安全な正値候補を生成する
```
candidates = [0.0, 0.001, 1.0, 100.0]
```
— 正値のみのため、`NonNegative` 不変条件では `check_aggregate_invariant` が `Ok` を返す（`violates: false`）

その後 `check_aggregate_invariant(&candidates, inv)` を呼び出し、結果が `Err` なら `violates: true`、`Ok` なら `violates: false`。

**返却値:**
```rust
CounterExampleResult {
    invariant_name: inv.column.clone(),
    example:        candidates,
    violates,
}
```

> **設計注記 1**: `seed` パラメータは将来の本格的な乱数生成（v78.x 以降）への拡張点として用意する。v77.7.0 では `seed % 2` による 2 パターンの擬似ランダム化のみ実装する。
>
> **設計注記 2**: ロードマップは候補値例として「0.0, -0.001, f64::MIN 等」を記載しているが、v77.7.0 では `f64::MIN`（約 -1.797e308）の代わりに `-1.0` を使用する。`f64::MIN` を使うと算術演算（`min - 1.0` 等）でアンダーフローが発生する懸念があり、テストの読みやすさのためシンプルな `-1.0` で代替する。違反の検出結果（`Err` / `Ok`）は同じ。
>
> **設計注記 3**: `check_aggregate_invariant(&candidates, inv)` の `inv` は `generate_counter_example_values` の引数（`inv: &AggregateInvariant`）をそのまま渡す（`&inv` は二重参照になるため誤り）。

---

## テスト仕様

### `counter_example_finds_violation`

```rust
// AggregateInvariant(NonNegative) + seed=0（偶数） → 負値候補を含む → violates=true
let inv = AggregateInvariant {
    column:   "amount".to_string(),
    property: AggregateProperty::NonNegative,
};
let result = generate_counter_example_values(&inv, 0);
assert!(result.violates);
assert_eq!(result.invariant_name, "amount");
assert!(!result.example.is_empty());
```

### `counter_example_none_for_trivially_valid`

```rust
// AggregateInvariant(NonNegative) + seed=1（奇数） → 正値のみ → violates=false
let inv = AggregateInvariant {
    column:   "score".to_string(),
    property: AggregateProperty::NonNegative,
};
let result = generate_counter_example_values(&inv, 1);
assert!(!result.violates);
assert_eq!(result.invariant_name, "score");
assert!(!result.example.is_empty());
```

---

## Success Criteria

- `CounterExampleResult` 構造体が定義されている（Debug / Clone 付き）
- `generate_counter_example_values` が `seed % 2 == 0` で `violates: true`（adversarial 候補を生成）、`seed % 2 == 1` で `violates: false`（安全な候補を生成）を返す
- `counter_example_finds_violation` が pass
- `counter_example_none_for_trivially_valid` が pass
- `cargo test` が 3750 tests all pass
- `driver.rs` 内の `77.6.0` バージョン文字列アサーション（`cargo_toml_version_is_X` 系テスト）がすべて `77.7.0` に更新されている（セクションコメント `// --- v77.6.0: 証明付き CI 統合 ---` は変更しない）
- `CHANGELOG.md` の先頭に v77.7.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `CounterExampleResult`, `generate_counter_example_values`, `v777000_tests` を追加
- `CHANGELOG.md` — v77.7.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.6.0` → `77.7.0` に更新
- `fav/Cargo.lock` — `Cargo.toml` バージョン更新に伴い自動更新（手動編集不要）

---

## 依存

- v77.3.0 の `AggregateInvariant` / `AggregateProperty` / `check_aggregate_invariant` を再利用（`fav/src/driver.rs` 内 `// --- v77.3.0: 集約系不変条件 ---` ブロック参照）
- `InvariantViolation`（v77.1.0 定義済み）は `check_aggregate_invariant` の返却型 `Result<(), InvariantViolation>` 経由で間接的に使用する（`CounterExampleResult` には含まない）

---

## 対象外

- CLI コマンド `fav verify --generate-counter-examples`: 将来の CLI 統合（v78.x 以降）
- `NonNegative` 以外の `AggregateProperty` バリアント（`NonPositive` / `Bounded` / `NonNull`）への個別テスト追加: v77.7.0 では `NonNegative` のみで 2 件のテストを実装
- 本格的な乱数生成（PRNG / QuickCheck 相当）: 将来の拡張点
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
