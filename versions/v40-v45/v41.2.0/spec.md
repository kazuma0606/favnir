# v41.2.0 Spec — Refinement type `fav check` 統合・E0404 系

**バージョン**: v41.2.0
**テーマ**: Refinement type を `fav check` に統合し、違反検出の基盤整備を行う
**前バージョン**: v41.1.0（Refinement type 基盤 — parser `where |v| pred` 対応済み）
**目標テスト数**: 2851（前バージョン 2848 + 3）

---

## 概要

v41.1.0 で `type Age = Int where |v| v >= 0` の構文パース（`TypeDef.invariants`）を実装した。
checker.rs の `type_invariants: HashMap<String, Vec<Expr>>` は既に全 TypeDef（Alias 含む）の
invariants を `td.invariants.clone()` で収集しているため、checker.rs 側の変更は不要。

本バージョンでは:

1. **E0404〜E0406 エラーコード** を `error_catalog.rs` に追加（refinement 系）
   - E0401/E0402/E0403 は SLA アノテーション系として v22.6.0 より使用済みのため E0404〜 を使用
   - ロードマップ記載の E0400/E0401/E0402 はドラフト時の仮番号であり、本実装で E0404〜に変更
2. **`checker.fav`** の `TypeDef` に `invariants: List<String>` フィールドを追加し、
   `check_refinement_alias` stub（v41.1.0 追加済み）を型定義チェック関数から統合呼び出し

**checker.rs 変更なし理由**: `type_invariants` フィールド（line 912）が既存で存在し、
TypeDef 処理パス（line 2158-2159）で `td.invariants.clone()` を収集している。
v41.1.0 の parser 変更により Alias 型の invariants も自動収集されている。

**checker.fav stub コメント注意**: `check_refinement_alias`（v41.1.0 追加済み）のコメントに
`// TODO: v41.2.0 で E0400 違反検出を実装` と書かれているが、E0400 は仮番号。
実装時は `E0400` → `E0404` にコメントを修正すること。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0404 / E0405 / E0406 追加（E04xx 末尾） |
| `fav/self/checker.fav` | `TypeDef` に `invariants: List<String>` 追加 + `check_refinement_alias` 統合呼び出し |
| `fav/src/driver.rs` | `v41200_tests` 追加（3 件）、`v41100_tests::cargo_toml_version_is_41_1_0` スタブ化 |
| `fav/Cargo.toml` | `version = "41.2.0"` に bump |
| `CHANGELOG.md` | `[v41.2.0]` エントリ追加 |

---

## 詳細仕様

### 1. error_catalog.rs — E0404〜E0406

`E0403`（`invalid circuit_breaker annotation`）の直後・`// ── E05xx` セクションコメントの直前に追加:

```rust
// ── E04xx: Refinement type (v41.2.0) ───────────────────────────────
ErrorEntry {
    code: "E0404",
    title: "refinement constraint violation",
    category: "types",
    description: "A value was assigned to a refinement type alias but violates the `where` invariant.",
    example: "type Age = Int where |v| v >= 0\nlet x: Age = -1  // E0404",
    fix: "Ensure the assigned value satisfies the refinement invariant.",
},
ErrorEntry {
    code: "E0405",
    title: "ambiguous refinement type",
    category: "types",
    description: "A refinement type alias has conflicting or unsatisfiable invariants.",
    example: "type Never = Int where |v| v > 0 && v < 0  // E0405",
    fix: "Review the invariant conditions for logical consistency.",
},
ErrorEntry {
    code: "E0406",
    title: "refinement constraint type mismatch",
    category: "types",
    description: "The predicate in a refinement `where` clause uses a type inconsistent with the base type.",
    example: "type Age = Int where |v| String.len(v) > 0  // E0406: len() on Int",
    fix: "Ensure the predicate operates on the base type of the alias.",
},
```

---

### 2. checker.fav — TypeDef invariants 統合

**現在の `TypeDef` フィールド**（`fav/self/checker.fav` line 83-89）:
```favnir
type TypeDef = {
    name: String
    is_record: Bool
    type_params: List<String>
    variants: List<VariantDef>
    fields: List<Param>
}
```

**追加後**:
```favnir
type TypeDef = {
    name: String
    is_record: Bool
    type_params: List<String>
    variants: List<VariantDef>
    fields: List<Param>
    invariants: List<String>
}
```

型定義チェック関数（T0 で実際の関数名を確認すること）内で `check_refinement_alias` を呼び出す:

```favnir
// v41.2.0: refinement alias invariants チェック統合
if List.length(td.invariants) > 0 {
    check_refinement_alias(td.name, td.invariants)
} else {
    true
}
```

---

## テスト設計（v41200_tests）

`use super::*` 不要。

### T1: `cargo_toml_version_is_41_2_0`（NOTE コメント付き）
```rust
#[test]
fn cargo_toml_version_is_41_2_0() {
    // NOTE: この assert は次バージョン bump 時にスタブ化すること
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("41.2.0"), "Cargo.toml must contain version 41.2.0");
}
```

### T2: `changelog_has_v41_2_0`
```rust
#[test]
fn changelog_has_v41_2_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("[v41.2.0]"), "CHANGELOG.md must contain [v41.2.0]");
}
```

### T3: `error_catalog_has_e0404`
```rust
#[test]
fn error_catalog_has_e0404() {
    // driver.rs と error_catalog.rs は同じ fav/src/ ディレクトリ
    let src = include_str!("error_catalog.rs");
    assert!(src.contains("E0404"), "error_catalog.rs must define E0404");
}
```

---

## 完了条件

- `cargo test` が 2851 tests passed, 0 failed
- `v41200_tests` 3 件すべて pass
- `error_catalog.rs` に E0404 / E0405 / E0406 が存在する
- `checker.fav` の `TypeDef` に `invariants` フィールドが存在する
