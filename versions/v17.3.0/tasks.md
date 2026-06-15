# v17.3.0 — コレクション内包表記 タスク

## ステータス: 未着手

---

## タスク一覧

### T1: AST 拡張

- [ ] `fav/src/ast.rs` に `CompClause` enum を追加
  ```rust
  pub enum CompClause {
      For { pat: Pattern, src: Box<Expr>, span: Span },
      Guard(Box<Expr>),
  }
  ```
- [ ] `Expr::ListComp { expr, clauses, span }` を追加
- [ ] `Expr::MapComp { key, val, clauses, span }` を追加
- [ ] `Expr::ResultComp { expr, clauses, span }` を追加

### T2: パーサー

- [ ] `parse_primary` の `LBracket` ハンドラに内包表記分岐を追加
  - `[` 消費後に expr をパース、次が `Pipe` なら ListComp へ
  - `?` があれば ResultComp
- [ ] `parse_comp_clauses` 関数を実装
  - `|` 消費後、カンマ区切りで clause をパース
  - `ident <- src` → `CompClause::For`
  - それ以外の expr → `CompClause::Guard`
- [ ] `parse_primary` の `LBrace` ハンドラにマップ内包分岐を追加
  - `{ key: val | ...` パターンを先読み確認
- [ ] `parse_comp_body_expr`（`|` で止まる expr パース）実装

### T3: 型チェッカー

- [ ] `checker.rs` の `infer_expr` に `Expr::ListComp` を追加
  - For 節ソース型 → `List<T>` 確認 (E0327)
  - パターン変数を T 型でスコープ追加
  - Guard 節を Bool 型チェック
  - 結果型: `List<expr_ty>`
- [ ] `Expr::ResultComp` を追加
  - expr 型 → `Result<T, E>` 確認 (E0328)
  - 結果型: `Result<List<T>, E>`
- [ ] `Expr::MapComp` を追加
  - 結果型: `Map<key_ty, val_ty>`

### T4: コンパイラ（デシュガー）

- [ ] `compiler.rs` の `compile_expr` に `Expr::ListComp` を追加
  - 単一 For + ガードなし → `List.map` に展開
  - 単一 For + ガードあり → `List.filter` + `List.map`
  - ガードのみ（expr == パターン変数）→ `List.filter`
  - 複数 For → `List.flat_map` のネスト
- [ ] `Expr::ResultComp` を追加
  - `List.fold_result` または相当の展開
- [ ] `Expr::MapComp` を追加
  - `List.map` + `Map.from_entries` に展開

### T5: stdlib 追加（必要な場合）

- [ ] `List.flat_map(list, fn)` が未実装なら `vm.rs` `call_builtin` / `compiler.rs` / `checker.rs` に追加
- [ ] `List.fold_result` または Result 内包用の builtin が必要な場合追加
- [ ] `Map.from_entries` が未実装なら追加

### T6: Exhaustive match 対応

- [ ] `fav/src/fmt.rs` — `Expr::ListComp` / `MapComp` / `ResultComp` / `CompClause` 追加
- [ ] `fav/src/middle/ast_lower_checker.rs` — 上記 variant の catch-all 追加
- [ ] `fav/src/emit_python.rs` — 簡易対応（`"# list comp"` コメント等）
- [ ] `fav/src/driver.rs` — 必要に応じて追加

### T7: テスト（driver.rs）

- [ ] `v173000_tests` モジュールを `driver.rs` に追加

```rust
#[cfg(test)]
mod v173000_tests {
    use super::*;

    #[test]
    fn version_is_17_3_0() { /* バージョン確認 */ }

    #[test]
    fn list_comp_map() {
        // bind ns <- Result.ok(List.push(List.push(List.singleton(1), 2), 3))
        // bind doubled <- Result.ok([x * 2 | x <- ns])
        // assert: List.length(doubled) == 3 かつ先頭が 2
    }

    #[test]
    fn list_comp_filter() {
        // bind ns <- Result.ok(...)
        // bind evens <- Result.ok([x | x <- ns, x % 2 == 0])
        // assert: evens は偶数のみ
    }

    #[test]
    fn list_comp_multi_source() {
        // bind as_ <- ...
        // bind bs  <- ...
        // bind pairs <- Result.ok([Pair(a, b) | a <- as_, b <- bs])
        // assert: List.length(pairs) == len(as_) * len(bs)
    }

    #[test]
    fn result_comp_propagation() {
        // 一部が err になる入力で [? f(x) | x <- xs] が Result.err を返すことを確認
    }
}
```

- [ ] 既存テストがリグレッションしないことを `cargo test` で確認

### T8: ドキュメント

- [ ] `site/content/docs/language/comprehensions.mdx` を新規作成
  - 基本 map / filter / 複数ソース / Result 内包 / マップ内包の例
  - Before / After 比較
  - 型チェック挙動の説明

### T9: バージョン更新

- [ ] `fav/Cargo.toml` のバージョンを `17.3.0` に更新
- [ ] `fav/Cargo.lock` を `cargo build` で更新

---

## 完了条件チェックリスト

- [ ] `[x * 2 | x <- numbers]` が `List.map` 相当の結果を返す
- [ ] `[x | x <- numbers, x > 0]` が `List.filter` 相当の結果を返す
- [ ] 複数ソース `[Pair(a, b) | a <- as, b <- bs]` が動作する
- [ ] `[? transform(row) | row <- rows]` のエラー伝播が動作する
- [ ] マップ内包 `{ k: v | (k, v) <- ... }` が動作する
- [ ] `cargo test v173000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし

---

## 優先度

T1（AST）→ T2（Parser）→ T3（Checker）→ T4（Compiler）→ T5（stdlib）→ T6（match）→ T7（test）→ T8（doc）→ T9（version）

T6 は T1〜T4 と並行でも可（clippy -D warnings 対応のため早めに行う）。
