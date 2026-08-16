# v71.8.0 spec — 型推論強化（型注釈省略可能範囲の拡大）

Date: 2026-08-11

---

## Background

Favnir の型チェッカー（`checker.rs`）は Hindley-Milner 型推論を実装しており、
`bind` 束縛では `annotated_ty` が `None` の場合でも `effective_ty`（RHS の推論型）を使って
パターンに型を割り当てる（line 4330-4331）。

クロージャ引数は現在 `Type::Unknown` で初期化され（line 5473-5474）、
`unify` において `Unknown` は任意の型と互換となる（line 503）ため、
引数型注釈なしのクロージャもコンパイルできる。

v71.8.0 はこの既存挙動を `v718000_tests` 2 件で明示的に確認し、
型注釈省略が正式にサポートされることをテストカバレッジとして記録する。

---

## Goals

1. `type_infer_local_var_omit_annotation`: `bind x <- fn()` 型注釈省略が `check_program` で
   エラーなし（型は RHS から推論）
2. `type_infer_closure_arg_omit`: `|acc, x|` 引数型注釈省略のクロージャが `check_program` で
   エラーなし（`Type::Unknown` → `unify` による互換解決）
3. Cargo.toml バージョンを `71.8.0` に更新

---

## テスト詳細

### `type_infer_local_var_omit_annotation`

```rust
// bind items <- fn() — 型注釈なし。checker が RHS の List<Int> を推論すること。
let src = r#"
fn get_values() -> List<Int> { List.of(1, 2, 3) }
fn main() -> Int {
    bind items <- get_values()
    List.length(items)
}
"#;
let program = Parser::parse_str(src, "test.fav").expect("parse");
let (errors, _) = Checker::check_program(&program);
assert!(errors.is_empty(), "bind without annotation should infer type: {:?}", errors);
```

### `type_infer_closure_arg_omit`

```rust
// |acc, x| — 引数型注釈なし。Unknown → unify による互換解決でエラーなし。
let src = r#"
fn main() -> Int {
    bind items <- List.of(1, 2, 3)
    bind total <- List.fold(items, 0, |acc, x| acc + x)
    total
}
"#;
let program = Parser::parse_str(src, "test.fav").expect("parse");
let (errors, _) = Checker::check_program(&program);
assert!(errors.is_empty(), "closure args without annotation should type-check: {:?}", errors);
```

---

## 使用する内部 API

```rust
// テスト内で使用（driver.rs 内 mod のため super:: 不要、crate:: で参照）
use crate::middle::checker::Checker;
use crate::frontend::parser::Parser;
```

## List.* の挙動に関する注意

- `List.fold` / `List.length` は `check_builtin_apply`（checker.rs line ~6416, ~6353）の match アームで処理される
- `List.of` は `check_builtin_apply` に専用アームがなく `None` を返す → `Type::Unknown` として扱われる
  - `expect_list_arg`（line ~7861）は `Type::Unknown` を受け入れてエラーにしないため、テストは pass する
- `register_builtins` は名前空間登録のみ（`List.of` の型は登録しない）

---

## Success Criteria

- `cargo test v718000` で 2 件 pass（0 failures）
- `cargo test` 全体で 3606 tests pass（3604 + 2）
- `fav/Cargo.toml` のバージョンが `71.8.0`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v718000_tests` モジュール追加（2 テスト）+ cargo_toml_version 更新 |
| `fav/Cargo.toml` | バージョン `71.7.0` → `71.8.0` |
| `CHANGELOG.md` | `## [v71.8.0]` エントリ追加 |
| `versions/current.md` | 進行中: v71.8.0 / 次: v71.9.0 |

---

## スコープ外

- `fresh_var` / `unify` の大規模リファクタリング: 既存実装で動作するため対象外
- `fav check --show-types` での推論型表示の拡張: v12.5.0 で実装済み、対象外
- ポリモーフィック関数の推論強化: v72.x 以降
- `checker.fav`（セルフホスト型チェッカー）への反映: スコープ外
