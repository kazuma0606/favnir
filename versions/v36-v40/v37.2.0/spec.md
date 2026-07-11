# v37.2.0 spec — 行多相実用強化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.2.0 |
| テーマ | 行多相実用強化 — 複数フィールド行制約の型チェック保証 + ネスト行型パース確認 |
| 前提 | v37.1.0 COMPLETE — 境界付きジェネリクス実用強化済み |
| 完了条件 | `v37200_tests` 全テスト pass・`cargo test` 0 failures（≥ 2711 件） |

## 背景と目的

v18.2.0（v32.2 ベースライン）で基本的な行多相 `R with { id: Int }` を実装済み。
v16.3.0 で RecordSpread `{ ...r, field: val }` も実装済み。

**現状の未検証領域:**
- 複数フィールド行制約 `R with { id: Int, name: String }` が call-site で型チェックを通ること
- ネスト行型 `R with { address: { city: String } }` のパース

**今バージョンで行うこと（実際のスコープ）:**
1. 複数フィールド行制約の call-site 型チェックをテストで保証（呼び出しサイトあり）
2. ネスト行型 `R with { address: { city: String } }` がパースを通ることを確認

**スコープ縮小の理由:**
- RecordSpread は v16.3.0 で実装済み → 重複テスト不要
- ネスト行型の完全型チェック（`type_has_field` の再帰チェック）は v37.3.0 に持ち越し
- ロードマップの「Spread が型チェックを通る」は既実装のため達成済みとみなす

## 実装スコープ

### 1. `fav/src/driver.rs` — `v37200_tests` モジュール追加

```rust
// ── v37200_tests (v37.2.0) — 行多相実用強化 ─────────────────────────────────
#[cfg(test)]
mod v37200_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v37200_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_37_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.2.0"), "Cargo.toml must contain version 37.2.0");
    }

    #[test]
    fn changelog_has_v37_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.2.0]"), "CHANGELOG.md must contain [v37.2.0]");
    }

    #[test]
    fn row_poly_multi_field_checks() {
        // R with { id: Int, name: String } — call-site で具体型を渡し制約が型チェックを通ることを確認
        // 呼び出しサイトを含めることで HasField 制約の call-site check が実際に実行される
        let errors = check_errors(r#"
type UserRow = { id: Int, name: String }
fn display<R with { id: Int, name: String }>(row: R) -> String {
    row.name
}
fn main() -> String {
    display(UserRow { id: 1, name: "Alice" })
}
"#);
        assert!(
            errors.is_empty(),
            "R with multiple field constraints must type-check at call-site: {:?}",
            errors
        );
    }

    #[test]
    fn nested_row_type_parseable() {
        // R with { address: { city: String } } — ネスト行型がパースを通ることを確認
        let result = Parser::parse_str(r#"
fn get_city<R with { address: { city: String } }>(row: R) -> String {
    row.address.city
}
"#, "v37200_nested_test.fav");
        assert!(result.is_ok(), "Nested row type must parse without error: {:?}", result.err());
    }
}
```

**重要:** `row_poly_multi_field_checks` は `type UserRow = { id: Int, name: String }` を定義し、
`display(UserRow { ... })` の call-site を含める。これにより `HasField` 制約の
call-site check が実際に実行される（関数宣言のみでは制約チェックがスキップされるため）。

## 注意事項

### 複数フィールド行制約の実装確認

`R with { id: Int, name: String }` がパーサーでどう処理されるか確認する:
- `{ id: Int, name: String }` が record type として `HasField("id", Int)` + `HasField("name", String)` の複数 `TypeConstraint` を生成するか確認
- もし未対応の場合、`parse_type_bounds` の実装を確認して必要な対応を追加する

### ネスト行型のスコープ

`R with { address: { city: String } }` については:
- **パースが通る（T4 対象）**: パーサーがネスト record type を `HasField { name: "address", ty: RecordType { ... } }` として認識する
- **型チェックが通る（v37.3 以降）**: `type_has_field` がネスト型を再帰的にチェックする実装は、スコープ外

### `v37100_tests` のスタブ化

`v37100_tests::cargo_toml_version_is_37_1_0` のライブアサーションを
`// Stubbed: version bumped to 37.2.0` に変更する（T3）。

### スコープ外（v37.3 以降）

- `R with { address: { city: String, .. }, .. }` の `..`（open row）構文
- ネスト行型制約の型チェック完全統合（`type_has_field` の再帰チェック）
- 複数フィールド制約の違反ケース（E0337）テスト

## ロードマップとの整合

ロードマップ v37.2.0 当初:「ネスト行型・Spread が型チェックを通る / Rust テスト 4 件」

**実際のスコープ（ロードマップを更新して記録）:**
- Spread: v16.3.0 で実装済み → 達成済みとみなす
- ネスト行型: パース確認まで（完全型チェックは v37.3.0 へ）
- 複数フィールド制約の call-site 型チェック保証を追加
- Rust テスト 4 件（ロードマップ指定通り）

ロードマップ文書 `roadmap-v37.1-v38.0.md` の v37.2.0 完了条件を T7 で更新する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.2.0` | `cargo_toml_version_is_37_2_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.2.0]` が含まれる | `changelog_has_v37_2_0` テスト |
| 3 | 複数フィールド行制約が call-site 型チェックを通る（`UserRow { id: Int, name: String }` を渡して 0 エラー） | `row_poly_multi_field_checks` テスト |
| 4 | ネスト行型 `R with { address: { city: String } }` がパースを通る | `nested_row_type_parseable` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2711） | `cargo test` 実行結果（v37.1.0 実績 2707 + v37200_tests 4 件 = 2711） |
