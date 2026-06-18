# v18.3.0 — Refinement Types タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/ast.rs` — `Param.constraint` フィールド追加

- [x] `Param` 構造体に `constraint: Option<Box<Expr>>` を追加
- [x] `Param::span()` など参照箇所を確認（変更不要なら OK）

### T2: 波及ファイル更新（`Param` 構造体変更対応）

- [x] `fav/src/frontend/parser.rs` — `parse_params` の `Param { name, ty, span }` → `Param { name, ty, constraint: None, span }` に修正
- [x] `fav/src/fmt.rs` — `Param` pretty-print: `constraint.is_some()` なら ` where { <expr> }` を出力
- [x] `fav/src/emit_python.rs` — `Param` 参照箇所（`constraint` は無視して OK）
- [x] `fav/src/lineage.rs` — `Param` 参照箇所を確認（`constraint: None` 追加）
- [x] `fav/src/middle/compiler.rs` — `Param` 構築・参照箇所を確認
- [x] その他 `Param { ... }` リテラルが存在するファイルを `cargo build` で特定して修正

### T3: `fav/src/frontend/parser.rs` — `parse_params` 拡張

- [x] `parse_params` 内で `TokenKind::Where` を検出したら `where` ブロックを解析
- [x] `where` の後に `{` を期待する（`expect(&TokenKind::LBrace)`）
- [x] `parse_expr()` で制約式を解析
- [x] `}` を `expect(&TokenKind::RBrace)` で閉じる
- [x] `Param.constraint = Some(Box::new(expr))` にセット

### T4: `fav/src/middle/checker.rs` — コンパイル時 E0331 チェック

- [x] 関数呼び出しのチェック箇所（`check_apply` または `check_fn_call`）を特定
- [x] 引数に `constraint` がある param に対して:
  - [x] 呼び出し引数式を `eval_static_expr` で評価
  - [x] 評価できた場合、制約式を静的に評価（引数名を `StaticValue` で代入）
  - [x] 制約違反（`false`）なら E0331 を発行
  - [x] 評価できない場合は何もしない（ランタイムチェックに委ねる）
- [x] E0331 メッセージ: `refinement violated: argument \`{name}\` must satisfy \`{constraint_text}\`, got {value}`
- [x] `fn_params_registry` に制約情報を保存（呼び出し時に参照できるよう）

### T5: `fav/src/backend/codegen.rs` — `RefinementAssert` opcode 追加

- [x] `Opcode` enum に `RefinementAssert { param_name: String }` を追加
- [x] `remap_string_operands`（または同等の文字列テーブル参照）に `RefinementAssert` を追加（必要な場合）
- [x] exhaustive match が必要なすべての箇所に `RefinementAssert` の arm を追加

### T6: `fav/src/middle/compiler.rs` — ランタイムアサーション生成

- [x] `compile_fn_def` の prologue 部分に refinement アサーションを追加
- [x] 各 param で `constraint.is_some()` なら:
  - [x] 引数のスロット番号を確認（引数は順に `Local(0)`, `Local(1)`, ... に割り当て）
  - [x] `Load(Local(slot))` を emit して引数値をスタックに積む
  - [x] `StoreLocal(tmp_slot)` で引数名をスコープに追加（制約式から参照可能にする）
  - [x] 制約式を `compile_expr` でコンパイル
  - [x] `RefinementAssert { param_name: name.clone() }` を emit
- [x] VM インタープリタ（`interpreter.rs` または `vm.rs`）に `RefinementAssert` の実行ロジックを追加:
  - [x] スタックから bool を pop
  - [x] `true` なら何もしない
  - [x] `false` なら error value をスタックに push

### T7: `fav/src/driver.rs` — `v183000_tests` 追加

- [x] `v182000_tests` の `version_is_18_2_0` テストを削除
- [x] `v183000_tests` モジュールを追加（5件）:
  - [x] `version_is_18_3_0`
  - [x] `refinement_literal_pass`（`divide(10, 2)` — 制約 `b != 0` を満たす → 正常）
  - [x] `refinement_literal_fail`（`divide(10, 0)` → E0331）
  - [x] `refinement_runtime_check`（変数引数 → RefinementAssert が注入される、正常実行）
  - [x] `refinement_range_constraint`（`age >= 0 && age <= 150` 複合制約、リテラル合格）

### T8: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.2.0` → `18.3.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T9: `site/content/docs/language/refinement-types.mdx` 作成

- [x] `fn f(x: Int where { x > 0 })` の基本構文を記載
- [x] コンパイル時チェック（E0331）の説明と例を記載
- [x] ランタイムチェックの動作を記載
- [x] 複合制約（`&&`）の例を記載
- [x] E0331 エラーの説明を記載

---

## テスト（v183000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_3_0` | Cargo.toml に "18.3.0" が含まれる |
| `refinement_literal_pass` | `divide(10, 2)` — 制約 `b != 0` を満たす → 正常コンパイル・実行 |
| `refinement_literal_fail` | `divide(10, 0)` → E0331 コンパイルエラー |
| `refinement_runtime_check` | 変数引数で RefinementAssert が注入される（正常実行） |
| `refinement_range_constraint` | `age >= 0 && age <= 150` 複合制約がパース・チェックされる |

---

## 完了条件チェックリスト

- [x] `fav/Cargo.toml` のバージョンが `18.3.0`
- [x] `Param.constraint: Option<Box<Expr>>` が `ast.rs` に存在する
- [x] `fn f(x: Int where { x > 0 })` がパースされる
- [x] リテラル違反で E0331 が発行される
- [x] 変数引数で `RefinementAssert` opcode が注入される
- [x] `site/content/docs/language/refinement-types.mdx` が存在する
- [x] `cargo test v183000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（ast.rs Param 変更）                ← 最初
T2（波及ファイル修正）                  ← T1 完了後すぐ（`cargo build` エラーを解消）
T3（parser.rs where 解析）              ← T1, T2 完了後
T4（checker.rs E0331）                  ← T3 完了後
T5（codegen.rs opcode）                 ← T1 完了後（T3, T4 と並列可）
T6（compiler.rs コード生成）            ← T3, T5 完了後
→ T7（v183000_tests）                   ← T4, T6 完了後
T8（バージョン更新）                    ← T7 完了後
T9（ドキュメント）                      ← T8 と並列可

**重要**: T1 の `Param` 変更はコンパイルエラーを多数発生させる。
T2 の波及ファイル更新で全て解消してから次フェーズに進む。
