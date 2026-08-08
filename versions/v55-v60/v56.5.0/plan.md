# Plan — v56.5.0 — OR パターン + パターンガード強化

## ゴール

- `lint.rs` に W037 `check_unreachable_patterns` を追加し `run_lint` に統合
- `v56500_tests` 3 件追加（OR パターン回帰テスト 2 件 + W037 unit test 1 件）
- 3235 → 3237 tests（+2 net: バージョンチェックテスト更新 -0 +3 件 - 以前の +1 = net +2）

---

## 実装ステップ

### Phase 1: Cargo.toml バージョン更新

`56.4.0` → `56.5.0`

---

### Phase 2: `lint.rs` — W037 `check_unreachable_patterns` 追加

W コードは `lint.rs` にのみ実装する（`error_catalog.rs` は E コード専用）。
既存の W036 コメントブロックの直後（ファイル末尾付近）に追加する。

新規追加関数:
- `pub fn check_unreachable_patterns(program: &Program) -> Vec<LintError>` — FnDef / StageDef をウォーク
- `fn check_block_for_unreachable(block: &Block, errors: &mut Vec<LintError>)` — Block ウォーク
- `fn check_stmt_for_unreachable(stmt: &Stmt, errors: &mut Vec<LintError>)` — **全 Stmt バリアント網羅**:
  `Bind` / `Chain` / `Expr` / `Yield` / `Return` / `ForIn`（iter + body）/ `Forall`（body）/ `Expect`（expr）
- `fn check_expr_for_unreachable(expr: &Expr, errors: &mut Vec<LintError>)` — spec.md 完全コードに従う
- `fn pattern_is_catch_all(pat: &Pattern) -> bool` — `Wildcard` | `Bind` のみ true
- `fn pattern_lit_key(pat: &Pattern) -> Option<String>` — `Lit` のみ Some、OR パターンは None

実装方針（spec.md 完全コード参照）:
1. 全 FnDef / StageDef をウォーク
2. 各 `Expr::Match` の arms を走査:
   - ガードなしの `_` / bind が非末尾に現れたら直後アームに W037 → `break`
   - 同一 match 内でリテラルが重複したら W037
3. `_ if cond` はガード付きのため catch-all 扱いしない

---

### Phase 3: `lint.rs` — `run_lint` への統合

`run_lint` 関数の既存 lint 呼び出しブロックに追加する:

```rust
errors.extend(check_unreachable_patterns(program));
```

---

### Phase 4: `driver.rs` — `v56500_tests` 追加

`v56400_tests` の直前に挿入する（3 件）:

```rust
mod v56500_tests {
    // match_or_pattern: Checker テスト — OR パターン型チェック
    // match_or_with_guard: Checker テスト — OR + guard 型チェック
    // w037_unreachable_after_wildcard: lint unit test — W037 発行確認
}
```

---

### Phase 5: `driver.rs` — バージョンチェックテスト更新

**更新対象**: `v56300_tests` モジュール内の `cargo_toml_version_is_56_3_0` テスト。
**変更内容**: 期待値を `"56.4.0"` → `"56.5.0"` に更新。

（注: このテストは v56.3.0 時点のままバージョン番号の名前が維持されており、
各バージョンでその期待値のみ更新する運用となっている）

---

### Phase 6: ポスト処理

- `CHANGELOG.md` に v56.5.0 エントリを追加
- `versions/current.md` を v56.5.0 / 3237 tests に更新
- `roadmap-v56.1-v57.0.md` の v56.5.0 実績を COMPLETE に更新し、
  「`PatternOr` AST ノード — 新規追加」記述を「`Pattern::Or` は v17.2.0 実装済み」に修正
- `roadmap-v55.1-v60.0.md` の v56.5.0 実績欄も COMPLETE に更新

---

## テスト数計算

| 操作 | 件数 |
|------|------|
| v56.4.0 実績 | 3235 |
| バージョンチェックテスト期待値更新（削除なし、数変化なし） | ±0 |
| `v56500_tests` 新規追加 3 件 | +3 |
| (将来バージョンの期待値更新のため v56400 の 1 件は変化なし) | ±0 |
| **目標合計** | **3237** |

（net +2: 既存の `v56300_tests::cargo_toml_version_is_56_3_0` は削除せず期待値を更新するため
テスト件数は変化せず、純粋に `v56500_tests` 3 件 - v56400 が残した既存テストの変化 = net +2 に相当するが、
実際の計算は 3235 + 2 = 3237 で正確）

---

## リスク管理

| リスク | 対策 |
|--------|------|
| `check_stmt_for_unreachable` の Stmt 網羅漏れ | spec.md の完全コードに全 8 バリアントを明示済み。`Stmt::ForIn`（iter + body）も対象 |
| `check_expr_for_unreachable` の Expr 分岐漏れ | spec.md の完全コードを使用。`Expr::Lit` / `Expr::Ident` は match 式を持たないため `{}` で安全 |
| ガード付き catch-all を誤検出 | `pattern_is_catch_all` は `arm.guard.is_none()` と組み合わせて使用（spec.md 明示済み） |
| `run_lint` 統合で既存テストが影響を受ける | W037 は新規追加のため既存 lint テストは影響を受けない |
| バージョンチェックテストの更新対象誤り | 更新対象は `v56300_tests::cargo_toml_version_is_56_3_0`（`v56400_tests` 内ではない）。tasks.md T6 に明記 |
| error_catalog.rs への誤登録 | W コードは lint.rs のみで管理。error_catalog.rs は変更しない |
| ロードマップのテスト数 3238 との乖離 | spec.md 備考に説明済み。実績ベースで 3237 が正確 |
