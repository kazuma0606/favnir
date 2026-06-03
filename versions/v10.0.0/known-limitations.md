# Favnir v10.0.0 既知の制限事項

## compiler.fav pipeline での par コンパイル（スタックオーバーフロー）

### 症状

`compile_file_to_bytes`（compiler.fav Favnir pipeline）で
`par` を含む seq をコンパイルしようとすると Rust スタックオーバーフローが発生する。

```
# 失敗するケース
stage Double: Int -> Int = |n| { n * 2 }
stage AddTen: Int -> Int = |n| { n + 10 }
stage SumList: List<Int> -> Int = |xs| { List.fold(xs, 0, |acc, x| acc + x) }
seq TestPar = par [Double, AddTen] |> SumList
```

### 根本原因

`List.fold` + lambda を含む stage body の再帰コンパイルが
Favnir VM のコールスタックを深くしすぎる（Rust デフォルトスレッドスタックサイズ: 8MB）。

compiler.fav の `compile_expr` → `compile_stmt` → `compile_block` → ... の再帰が
ネストした lambda 式で深くなり、Rust の stack overflow を引き起こす。

### 確認方法

v9.13.0 の `par_compiles_with_favnir_pipeline` テストで確認済み。
このテストは Rust pipeline（`compile_program` + `codegen_program` 直接呼び出し）を使うよう
回避策を適用している。

### 回避策

par を含む seq のコンパイルには Rust pipeline を使用する:
- `fav run --legacy src/file.fav` — Rust pipeline での実行
- または CI テストで Rust pipeline の `compile_program` / `codegen_program` を直接呼び出す

---

## fav check self/compiler.fav が checker.fav 経由で失敗する

### 症状

CI の Self-check ステップで `./target/debug/fav check self/compiler.fav` を実行すると
`E0001: undefined variable: _unsupported_` が発生する。

### 根本原因

`compiler.fav` は `collect { yield ... }` コルーチン構文を使っている。
`ast_lower_checker.rs` の `lower_expr` はこの構文を未対応として
`EVar("_unsupported_")` に変換してしまい、checker.fav が E0001 を報告する。

`checker.fav` / `cli.fav` はこの構文を使っていないため、これらの self-check は正常に動く。

### 回避策

CI では `compiler.fav` のみ `--legacy` フラグを使って Rust チェッカーで確認する:

```yaml
./target/debug/fav check --legacy self/compiler.fav
```

### 将来の修正方針

- `ast_lower_checker.rs` に `Expr::Collect` / `Expr::Yield` の lowering を追加する
- v10.x 以降で対応予定（優先度: 低）

---

### 将来の修正方針（par スタックオーバーフロー）

- `RUST_MIN_STACK` 環境変数でスレッドスタックサイズを拡大する（`RUST_MIN_STACK=16777216 fav run ...`）
- またはコンパイラ内の再帰を trampoline パターンに書き換える
- v10.x 以降で対応予定（優先度: 中）
