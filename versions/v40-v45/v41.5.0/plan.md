# v41.5.0 実装プラン — Row polymorphism 強化

**目標**: RecordSpread の checker.fav 統合バグを修正し、TeRecord 型ノードを追加する

---

## フェーズ 1 — ast_lower_checker.rs 変更（2箇所）

1. `RecordSpread` lowering を `sv("()")` から `v2("ERecordSpread", lower_expr(base), lower_field_list(updates))` に修正
2. `lower_te` の `RecordType(_, _)` を `v0("TeRecord")` に変更

---

## フェーズ 2 — checker.fav: 新バリアント追加

1. `Expr` 型に `| ERecordSpread(Expr, Expr)` 追加（`ERecordLit` の直後）
2. `TypeExpr` 型に `| TeRecord` 追加（`TeFn` の直後）

---

## フェーズ 3 — checker.fav: 関数更新

1. `infer_expr`: `ERecordSpread` ケース追加（`ERecordLit` 直後、`"Unknown"` を返す）
2. `type_expr_to_str`: `TeRecord => "Any"` 追加（`TeFn` ケース直後）
3. `collect_type_vars_from_te`: `TeRecord => List.empty()` 追加（`TeMap` ケース直後）

---

## フェーズ 4 — driver.rs + バージョン管理

1. `v41400_tests::cargo_toml_version_is_41_4_0` をスタブ化
2. `v41500_tests` モジュール（3 件）を末尾に追加
3. `Cargo.toml`: `version = "41.5.0"`
4. `CHANGELOG.md`: `[v41.5.0]` エントリ追加
5. `versions/current.md` / `roadmap` 更新

---

## 実装順序

```
ast_lower_checker.rs (RecordSpread + RecordType)
  → checker.fav (ERecordSpread + TeRecord バリアント追加)
  → checker.fav (infer_expr + type_expr_to_str + collect_type_vars_from_te)
  → cargo test（中間確認）
  → driver.rs (v41400 スタブ + v41500 追加)
  → Cargo.toml + CHANGELOG.md
  → cargo test（最終確認）
```

---

## リスク評価

| リスク | 影響度 | 対処 |
|---|---|---|
| `te_to_string`（Rust 側）の `RecordType` 変更漏れ | HIGH | spec §2 の注意: Rust側 `te_to_string` は変更しない |
| `TeRecord` が `collect_type_vars_from_te` でクラッシュ | HIGH | `TeRecord` は `v0`（引数なし）なのでフィールドアクセスしない |
| `ERecordSpread` が他の traversal 関数で `_ =>` に落ちる | LOW | `infer_expr_effects` / `check_rebind` / `check_w006_expr` は `_ =>` catch-all 持ちのため安全 |
| `type_expr_to_str` に `TeRecord` ケース追加漏れ | HIGH | `type_expr_to_str` と `collect_type_vars_from_te` は **`_ =>` catch-all なし**。T6/T7 の実装は必須。漏れると Favnir 実行時エラー |
| `TeRecord` 追加後の exhaustive match 漏れ | MED | `type_expr_to_str` と `collect_type_vars_from_te` の 2 関数のみ全バリアント列挙が必要。それ以外の `_ =>` 持ち関数は安全 |
