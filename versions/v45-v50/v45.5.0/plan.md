# Plan: v45.5.0 — 型エイリアス完全化（E0413 opaque alias coerce）

Date: 2026-07-16

---

## 事前確認

1. `cargo test` 2977 passed, 0 failed を確認
2. `grep -r "opaque type" fav/src/` で既存テストに `opaque type` 使用が0件であることを確認
   - もし既存テストで opaque の inner 型を直接返している場合、その箇所が E0413 を発生させ壊れる可能性がある

---

## Step 1 — `checker.rs`: `opaque_alias_inner` フィールド追加

`Checker` struct に追加:

```rust
opaque_alias_inner: HashMap<String, Type>,
```

`Checker::new()` の初期化リストに追加:
```rust
opaque_alias_inner: HashMap::new(),
```

`Checker::new_with_resolver()` の初期化リストにも追加（project モード用）:
```rust
opaque_alias_inner: HashMap::new(),
```

---

## Step 2 — `checker.rs`: `register_item_signatures` 修正

現在の `TypeBody::Alias` 処理（`type_aliases` に無条件登録）を修正:

```rust
TypeBody::Alias(inner_te) => {
    if td.is_opaque {
        // opaque: transparent resolution を禁止
        // resolve_type_expr の戻り値は Type（Type::String など）
        let inner_ty = self.resolve_type_expr(inner_te);
        self.opaque_alias_inner.insert(td.name.clone(), inner_ty);
    } else {
        // transparent: 従来通り type_aliases に登録して resolve_type_expr の再帰解決を使う
        self.type_aliases.insert(td.name.clone(), inner_te.clone());
    }
}
```

注意: `resolve_type_expr` の戻り値は `Type` であり、`Type::Named("String", [])` ではなく `Type::String` などになる。

---

## Step 3 — `checker.rs`: `check_fn_def` に E0413 追加

戻り型チェック箇所（line ~3161、E0101 を発行している箇所の**前**）に挿入:

```rust
// opaque alias coerce チェック (E0413)
if let Type::Named(ref opaque_name, _) = expected_ret {
    if let Some(inner_ty) = self.opaque_alias_inner.get(opaque_name.as_str()) {
        if body_ty.is_compatible(inner_ty) {
            self.type_error(
                "E0413",
                format!(
                    "cannot coerce `{}` to opaque type `{}`; use an explicit constructor",
                    body_ty.display(),
                    opaque_name
                ),
                ret_span,
            );
            return;
        }
    }
}
// 既存の E0101 発行
```

`check_trf_def` にも同様のチェックを追加（line ~3227 付近の return 型チェック箇所）:

```rust
// opaque alias coerce チェック (E0413)
if let Type::Named(ref opaque_name, _) = expected_ret {
    if let Some(inner_ty) = self.opaque_alias_inner.get(opaque_name.as_str()) {
        if body_ty.is_compatible(inner_ty) {
            self.type_error(
                "E0413",
                format!(
                    "cannot coerce `{}` to opaque type `{}`; use an explicit constructor",
                    body_ty.display(),
                    opaque_name
                ),
                ret_span,
            );
            return;
        }
    }
}
// 既存の E0101 発行
```

`is_compatible` が `Type` の impl メソッドとして存在することを事前確認すること（`body_ty.is_compatible(&expected_ret)` の形で使われているはず）。

---

## Step 4 — `driver.rs`: `v455000_tests` モジュール追加

既存 `v454000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v455000_tests {
    use super::*;

    fn check_src(src: &str) -> Vec<String> {
        let diags = type_check_source(src);
        diags.iter().map(|d| d.code.to_string()).collect()
    }

    #[test]
    fn transparent_alias_compatible() {
        let src = r#"
type UserId = Int
fn get_id() -> UserId {
    42
}
"#;
        let codes = check_src(src);
        assert!(codes.is_empty(), "expected no errors, got: {:?}", codes);
    }

    #[test]
    fn opaque_alias_incompatible() {
        let src = r#"
opaque type Token = String
fn make_token() -> Token {
    "abc"
}
"#;
        let codes = check_src(src);
        assert!(
            codes.contains(&"E0413".to_string()),
            "expected E0413, got: {:?}",
            codes
        );
    }
}
```

注意: `type_check_source` が存在しない場合は、`v454000_tests` の `check_src` と同じ実装を参照すること。

---

## Step 5 — `Cargo.toml` + `CHANGELOG.md` + `versions/current.md` 更新

- `fav/Cargo.toml`: `version = "45.4.0"` → `"45.5.0"`
- `CHANGELOG.md`: v45.5.0 エントリ追加
- `versions/current.md`: 最新安定版を v45.5.0（2979 tests）に更新

---

## Step 6 — テスト実行

```bash
cd fav && cargo test -j 8 -- --test-threads=8
```

期待: 2979 passed, 0 failed。

```bash
cargo clippy -- -D warnings
```

クリーンであること。
