# v61.3.0 Plan — パターンガード拡張（OR パターン各アームへの個別ガード）

Date: 2026-07-31
Status: COMPLETE

---

## 実装順序

AST 変更 → IR 変更 → パーサー → 既存 OR パターン参照箇所の一括修正（コンパイルエラー駆動）→ checker ガード型検証 → codegen ガード評価 → error_catalog.rs → tests

---

## Phase 1: コアデータ構造変更（AST / IR）

### P1-1: `ast.rs` — `Pattern::Or` 型変更

`Pattern::Or(Vec<Pattern>, Span)` → `Pattern::Or(Vec<(Pattern, Option<Expr>)>, Span)`

この変更でコンパイルエラーが多数発生する → Phase 2 以降で順次修正。

### P1-2: `ir.rs` — `IRPattern::Or` 型変更

`IRPattern::Or(Vec<IRPattern>)` → `IRPattern::Or(Vec<(IRPattern, Option<IRExpr>)>)`

---

## Phase 2: コンパイルエラー修正（タプル分解対応）

ast.rs / ir.rs の変更後、`cargo build` でエラーが出る箇所を順次修正する。
修正方針: 既存の `pats` → `arms` へリネームし、`(p, _)` でタプル分解。

修正対象ファイル（ロジック変更なし、タプル分解のみ）:

| ファイル | 変更箇所 |
|---|---|
| `checker.rs` | `collect_pattern_variants` の `Pattern::Or` アーム |
| `compiler.rs` | `pattern_binds` の `Pattern::Or` アーム |
| `fmt.rs` | `Pattern::Or` フォーマット |
| `lint.rs` | `pattern_lit_keys_all` と `collect_pattern_bound_names` の `Pattern::Or` |
| `emit_python.rs` | `Pattern::Or` Python エミット |
| `ast_lower_checker.rs` | `Pattern::Or` の first alt 取得 |
| `driver.rs` | `remap_ir_pattern` の `IRPattern::Or` |

---

## Phase 3: パーサー変更（`parser.rs`）

### P3-1: `parse_or_alternative` 新規追加

`(pat if guard)` 形式を `(Pattern, Option<Expr>)` として返す新規メソッド。

### P3-2: `parse_match_arm` 更新

`parse_pattern()` 呼び出しを `parse_or_alternative()` に変更。
`Pattern::Or` 構築時に `Vec<(Pattern, Option<Expr>)>` を使用。

---

## Phase 4: 型検証（`checker.rs`）

### P4: `check_pattern_bindings` の `Pattern::Or` アーム更新

- タプル分解 `(pat, guard)` に対応
- ガード式が `Some(guard_expr)` の場合: `infer_expr` で型を取得し、Bool でなければ E0395 を発行

---

## Phase 5: コードゲン（`codegen.rs`）

### P5: `IRPattern::Or(arms)` — ガード評価ロジック追加

- 各アームのパターンマッチ成功後、guard が `Some` なら `emit_expr` + `JumpIfFalse` で次アームへ分岐
- guard が `None` なら従来通り成功として扱う

---

## Phase 6: コンパイラ IR 変換（`compiler.rs`）

### P6: `compile_pattern` の `Pattern::Or` アーム更新

- `guard.as_ref().map(|g| compile_expr(g, ctx))` で `Option<IRExpr>` を生成
- `(ir_pat, ir_guard)` タプルを `IRPattern::Or` に渡す

---

## Phase 7: エラーカタログ（`error_catalog.rs`）

### P7: E0395 追加

`E0394` の次に `E0395` エントリを追加。

---

## Phase 8: テスト（`driver.rs`）

### P8: `v61300_tests` モジュール追加

`v61200_tests` の直前（上側）に挿入。

- `guard_or_pattern_per_arm`: `(y if y > 90) | (y if y > 50)` が型チェックを通過することを確認
- `guard_or_pattern_fallthrough`: 3 アーム + ワイルドカードの組み合わせを確認

---

## Phase 9: テスト実行・確認

- `cargo test -j 8 -- --test-threads=8`
- 総テスト数 **3360** tests passed, 0 failed
- 既存 OR パターンテスト (`v61100_tests`) が引き続き pass

---

## Phase 10: 事後処理

- `versions/current.md` を v61.3.0 / 3360 tests に更新
- `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.3.0 実績欄を更新
- このファイル（plan.md）と tasks.md を COMPLETE に更新

---

## リスク・注意事項

1. **`cargo build` でのコンパイルエラーが多数発生する**: Phase 1 後に `cargo build` を実行し、エラーリストを確認してから Phase 2 を一括修正するのが効率的。
2. **`fmt.rs` の OR パターンフォーマット変更**: ガードありの場合は `(pat if expr)` 形式でラップする。ガードなしは従来通り。
3. **codegen.rs のガード評価**: `JumpIfFalse` は Bool スタック値を pop して評価する Opcode。ガード評価後の pop 数に注意。
4. **`emit_python.rs` は guard を無視**: Python トランスパイラはガードをサポートしないため、first alt のパターンのみ出力。
