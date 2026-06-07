# Favnir v12.2.0 Plan

Date: 2026-06-07
Theme: lint 強化 — W006（Result を `bind _` で捨てる）+ W007（深い match ネスト）

---

## 実装対象

### W006: `bind _` で Result を捨てる（checker.fav）

`checker.fav` の `infer_hm` — `EBind` 分岐内で型推論後に実施。
型推論で `Result<...>` 型と判定された場合に W006 を warn_list に追加。

実装フロー:
1. `infer_hm` の `EBind({ _0: "_", ... })` 分岐に追加
2. `val_expr` を型推論 → `inferred_ty` が `"Result<"` で始まるかチェック
3. 始まる場合、`warn_list` に W006 メッセージを追加
4. `warn_list` は既存 W001-W005 と同じ方式（`Result.ok("ok\nW006: ...")` で返却）

注意: `chain _` は既存のエラー伝播ロジックで処理されるため W006 対象外。
`bind _` に限定してチェックする（AST レベルで `EBind` にフラットに lowered される点は同じだが、
ソースキーワードを `is_chain` フラグで区別する必要がある可能性あり — AST を確認すること）。

### W007: 深い match ネスト（compiler.fav）

`compiler.fav` の lint エンジン（`lint_fn` 付近）に追加。
AST 走査で `EMatch` に到達するたびに depth をインクリメントし、depth >= 3 で W007。

実装フロー:
1. `lint_fn_w007(expr: Expr, depth: Int) -> Option<String>` を追加
2. `lint_arms_w007(arms: Expr, depth: Int) -> Option<String>` を追加
3. `lint_fn` から `lint_fn_w007(body, 1)` を呼ぶ
4. `fav.toml [lint] allow` — 既存の allow リスト処理に W007 を追加

---

## 技術的注意点

### checker.fav での W006 実装

`EBind` の `_0` が `"_"` であることに加え、`val_expr` の型推論結果で `Result<` 判定が必要。
`infer_hm` の `EBind` ハンドラは既に `val_expr` を型推論しているので、
その結果の型文字列を `String.starts_with(ty_str, "Result<")` で判定する。

warn_list の蓄積は既存パターン:
- `infer_hm` は `Result<(Type, InfState), String>` を返す
- 警告はチェック後に `Result.ok("ok\nW006: ...")` 形式で上位に返す
- 実際の warn 蓄積パターンは既存 W001〜W005 のコードを参照

### compiler.fav での W007 実装

既存の `lint_fn` は `fn_def.body` を再帰的に走査している。
W007 追加時は depth パラメータを持つ再帰関数を別途定義し、`lint_fn` から呼ぶ。

`EMatch` ネストのカウント: 各 arm body の中に `EMatch` があれば depth+1。
`ELambda` や `EBind` の中の `EMatch` も対象（関数境界は depth リセット）。

---

## テスト方針

`fav/src/driver.rs` に `v12200_tests` モジュールを追加。
全テストは `check_with_checker_fav` / コンパイルテスト関数を使用。

W006 テスト (5件):
- `bind _ <- Postgres.execute_raw(...)` → W006
- `bind _ <- IO.println(...)` → 警告なし（Unit）
- `chain _ <- Postgres.execute_raw(...)` → 警告なし
- `match Postgres.execute_raw(...) { Ok(_) => ... Err(e) => ... }` → 警告なし
- fav.toml `allow = ["W006"]` → 警告なし

W007 テスト (5件):
- match 2段 → 警告なし
- match 3段 → W007
- match 4段 → W007
- ヘルパー関数に切り出し → 警告なし
- fav.toml `allow = ["W007"]` → 警告なし

バージョン確認 (1件):
- `version_is_12_2_0`

合計 11 件。

---

## 実装優先順位

1. W007（compiler.fav）— lint エンジン追加のみ、型推論不要で単純
2. W006（checker.fav）— 型推論結果を利用するため慎重に実装
3. fav.toml allow 対応（両方）
4. テスト追加
5. バージョン更新 + コミット
