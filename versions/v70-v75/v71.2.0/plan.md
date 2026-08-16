# v71.2.0 Plan — Refined Types（型レベル制約 `where self`）

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: 事前確認

```bash
# 現行パーサーが `type X = T where expr` をパースできるか確認
# TypeDef.invariants に式が入ることを確認
# E0425 が未定義であることを確認
```

- `type PositiveFloat = Float where self > 0.0` → パーサーは v41.1.0 から対応済みのはず
- チェッカーに E0425 が未定義であることを grep で確認

---

### Step 2: checker.rs に `fn_alias_refinements` フィールドを追加

**注意**: `alias_type_invariants` は不要。既存の `type_invariants: HashMap<String, Vec<Expr>>` で代替できる。

`Checker` 構造体に 1 つのフィールドを追加:

```rust
/// v71.2.0: 関数パラメータの alias refined type 対応（呼び出し時チェック用）
fn_alias_refinements: HashMap<String, Vec<(usize, Vec<Expr>)>>,
```

`Checker::new()` と `new_with_resolver()` の初期化に `HashMap::new()` を追加。

---

### Step 3: `register_item_signatures` で alias 型の `type_invariants` 登録を修正

現行コードでは `TypeBody::Alias` の場合、line 2324 の `continue` で早期リターンしており、`type_invariants` への登録（line 2338-2339）が実行されない。

`TypeBody::Alias` ブランチの `continue` より**前**に invariants を登録する:

```rust
if let TypeBody::Alias(target) = &td.body {
    // 既存コード: type_aliases への登録 etc.
    // ...

    // v71.2.0: alias 型の where 制約を type_invariants に登録（continue 前に実行）
    if !td.invariants.is_empty() {
        self.type_invariants.insert(td.name.clone(), td.invariants.clone());
    }
    continue; // ← ここより前に登録する
}
```

---

### Step 4: `register_item_signatures` で関数シグネチャに alias 制約を伝播

`Item::FnDef(fd)` の処理（line ~2418）の直後に追加:

```rust
// v71.2.0: パラメータに refined alias 型が使われている場合、呼び出し時チェック用に登録
let alias_refs: Vec<(usize, Vec<Expr>)> = fd.params.iter().enumerate()
    .filter_map(|(idx, p)| {
        if let TypeExpr::Named(type_name, args, _) = &p.ty {
            if args.is_empty() {
                if let Some(invariants) = self.type_invariants.get(type_name) {
                    return Some((idx, invariants.clone()));
                }
            }
        }
        None
    })
    .collect();
if !alias_refs.is_empty() {
    self.fn_alias_refinements.insert(fd.name.clone(), alias_refs);
}
```

**重要**: `type_invariants` への登録（Step 3）は items の順序通りに処理される。TypeDef が FnDef より前に定義されている場合のみ `fn_alias_refinements` が正しく登録される。テストコードでは必ず型定義を関数定義の前に記述すること。

---

### Step 5: `check_type_def` でエイリアス型の制約を型チェック

`check_type_def` に `TypeBody::Alias` ブランチを追加:

```rust
fn check_type_def(&mut self, td: &TypeDef) {
    if let TypeBody::Record(fields) = &td.body {
        // 既存コード...
    }
    // v71.2.0: alias 型の where 制約を型チェック
    if let TypeBody::Alias(target) = &td.body {
        if !td.invariants.is_empty() {
            self.env.push();
            let target_ty = self.resolve_type_expr(target);
            self.env.define("self".to_string(), target_ty);
            for invariant in &td.invariants {
                let inv_ty = self.check_expr(invariant);
                if !inv_ty.is_compatible(&Type::Bool) {
                    self.type_error(
                        "E0245",
                        format!(
                            "`where` constraint for type `{}` must be of type Bool, got `{}`",
                            td.name,
                            inv_ty.display()
                        ),
                        invariant.span(),
                    );
                }
            }
            self.env.pop();
        }
    }
    // 既存コード（interface_synthesis）...
}
```

---

### Step 6: `Expr::Apply` で呼び出し時制約チェック（E0425）

既存の `fn_refinement_registry` チェック（line ~5036）の直後に追加。
既存パターン（2 段階: arg 式→静的 Lit 取り出し→constraint 評価）に合わせる:

```rust
// v71.2.0: alias refined type の制約チェック（E0425）
if let Some(alias_refinements) = self.fn_alias_refinements.get(&fn_name).cloned() {
    for (param_idx, invariants) in &alias_refinements {
        if let Some(arg_expr) = args.get(*param_idx) {
            // Step 1: 引数式から静的 Lit を取り出す
            if let Some(static_val) = self.eval_static_expr(arg_expr, &HashMap::new()) {
                let lit = match static_val {
                    StaticValue::Int(v) => Lit::Int(v),
                    StaticValue::Float(v) => Lit::Float(v),
                    StaticValue::Str(v) => Lit::Str(v),
                    StaticValue::Bool(v) => Lit::Bool(v),
                };
                // Step 2: "self" に Lit を束縛して constraint を評価
                let mut values = HashMap::new();
                values.insert("self".to_string(), lit);
                let violated = invariants.iter().any(|inv| {
                    matches!(self.eval_static_expr(inv, &values), Some(StaticValue::Bool(false)))
                });
                if violated {
                    self.type_error(
                        "E0425",
                        "literal does not satisfy refined type constraint".to_string(),
                        arg_expr.span(),
                    );
                }
            }
        }
    }
}
```

---

### Step 6.5: error_catalog.rs に E0425 エントリを追加

既存の E0424（RBAC アクセス拒否）エントリの直後に追加:

```rust
ErrorEntry {
    code: "E0425",
    title: "Refined type constraint violation",
    description: "A literal value passed to a function does not satisfy the `where` constraint of the refined type alias.",
    fix: "Ensure the value satisfies the constraint defined by `type X = T where self <expr>`.",
    since: "v71.2.0",
},
```

---

### Step 7: driver.rs に `v712000_tests` を追加

`v711000_tests` の直後に追加:

```rust
// ── v71.2.0: Refined Types（型レベル制約 where self） ────────────────────────

#[cfg(test)]
mod v712000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn refined_type_positive_float() {
        let src = concat!(
            "type PositiveFloat = Float where self > 0.0\n",
            "fn safe_log(x: PositiveFloat) -> Float { 1.0 }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "refined type definition should typecheck cleanly; errors: {:?}",
            errors
        );
    }

    #[test]
    fn refined_type_violation_compile_error() {
        let src = concat!(
            "type PositiveFloat = Float where self > 0.0\n",
            "fn safe_log(x: PositiveFloat) -> Float { 1.0 }\n",
            "fn bad() -> Float { safe_log(0.0) }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.iter().any(|e| e.code == "E0425"),
            "constraint violation should produce E0425; errors: {:?}",
            errors
        );
    }
}
```

---

### Step 8: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "71.1.0"` → `"71.2.0"`
- driver.rs 内の全バージョン文字列（`"71.1.0"`）を `"71.2.0"` に一括更新（replace_all）

---

### Step 9: CHANGELOG.md 更新

ヘッダー形式: `## [v71.2.0] — 2026-08-09 — Refined Types（型レベル制約 where self）`

---

### Step 10: 最終確認

- `cargo test v712000` で 2 件 pass
- `cargo test` 全体で 3588 tests pass（0 failures）
- `versions/current.md` を v71.2.0 に更新

---

## 注意事項

### TypeDef 登録順序の問題

`register_item_signatures` は items を順番に処理するため、`type PositiveFloat = ...` が `fn safe_log(x: PositiveFloat)` より前に定義されている場合のみ `fn_alias_refinements` が正しく登録される。

テストコードでは必ず型定義を関数定義の前に記述する。

実際のユーザーコードでも同様の制約があるが、これは v71.2.0 の最小実装スコープ内として許容する（将来の multi-pass 対応で解消可能）。

### `self` 識別子との衝突

Favnir の `self` は通常の識別子（キーワードではない）。`check_type_def` で `self` を env に定義しても、他の文脈での `self` と衝突しない（スコープを push/pop するため）。

### eval_static_expr の制約

`eval_static_expr` は `Lit` と `BinOp` のみ対応。複雑な式（関数呼び出しを含む制約）は静的評価できないため、違反を検出できない（`None` を返して無視）。これは最小実装での許容範囲。
