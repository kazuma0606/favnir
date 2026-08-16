# v71.4.0 タスクリスト — Const / Compile-Time Evaluation

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.3.0` であることを確認
- [x] `cargo test` が全 pass（3592 tests）であることを確認
- [x] `const` が文脈キーワード（識別子として認識）であることを確認（`grep -rn '"const"' fav/src/frontend/` → 結果なし）
- [x] `TypeExpr::ConstInt` の利用箇所を確認（`[N]` サフィックス処理の場所 — `parser.rs:1979`）
- [x] `eval_static_expr` のシグネチャを確認: `(&self, expr: &Expr, values: &HashMap<String, Lit>)` — 第2引数は `Lit` であることに注意
- [x] E0247/E0250 が既存コードで未使用であることを確認（`grep -rn '"E0247"\|"E0250"' fav/src/` → 結果なし）
- [x] E0248/E0249 が既存 checker.rs で使用中であることを確認（使用不可）
- [x] dim 抽出箇所（checker.rs ~line 4919: `TypeExpr::ConstInt(n, _)` マッチ）の構造を確認

---

## T1: AST — `ConstDef`・`Item::ConstDef`・`TypeExpr::ConstName` 追加

- [x] `fav/src/ast.rs` に `ConstDef` 構造体を追加した
  - `pub name: String`
  - `pub ty: TypeExpr`
  - `pub value: Expr`
  - `pub span: Span`
- [x] `Item::ConstDef(ConstDef)` バリアントを追加した
- [x] `Item::span()` に `Item::ConstDef(c) => &c.span` を追加した
- [x] `TypeExpr::ConstName(String, Span)` バリアントを追加した（`ConstInt` 直後）
- [x] `TypeExpr::span()` に `TypeExpr::ConstName(_, s) => s` を追加した
- [x] `cargo build` でコンパイルエラー箇所の一覧を取得した（次 Step の作業リスト）

---

## T2: パーサー — `parse_const_def` と `ConstName` パース追加

- [x] `parse_base_type` の `[N]` サフィックス処理に `Ident` → `ConstName` ブランチを追加した
- [x] `parse_item` に `Ident("const")` ブランチを追加した
- [x] `parse_const_def` 関数を実装した（`name: Type = expr`）
- [x] `cargo test` で既存テスト（3592 件）が全 pass することを確認

---

## T3: チェッカー — `const_env` フィールド追加

- [x] `Checker` 構造体に `const_env: HashMap<String, StaticValue>` を追加した
- [x] `Checker::new()` の初期化に `const_env: HashMap::new()` を追加した
- [x] `new_with_resolver()` の初期化にも追加した

---

## T4: チェッカー — const pre-pass（`register_item_signatures`）

- [x] `register_item_signatures` 先頭（alias invariant pre-pass の直後）に const pre-pass を追加した
  - `const_lit_values: HashMap<String, Lit>` を宣言順に増分構築
  - `Item::ConstDef` → `eval_static_expr(&cd.value, &const_lit_values)` → `const_env.insert` + `env.define`
  - 型不一致 → E0250 エラー（E0248/E0249 は既存用途あり）
  - 評価不能（未定義参照等） → E0250 エラー
- [x] `cargo test` で既存テスト（3592 件）が全 pass することを確認

---

## T5: チェッカー — `ConstName` 次元解決と `resolve_type_expr` 対応

- [x] `resolve_type_expr` に `TypeExpr::ConstName` アームを追加した
  - 未定義 → E0247 エラー
  - → `Type::Int` を返す（型推論用）
- [x] checker.rs の dim 抽出箇所（`TypeExpr::ConstInt(n, _)` マッチ、~line 4919）に `ConstName` ブランチを追加した
  - `TypeExpr::ConstName(name, _)` → `const_env.get(name)` → `i64` として解決
  - 未定義 → E0247 エラー
  - **`is_dim_annotated_name_mismatch` の変更は不要**（dim が `i64` として解決されれば既存の文字列エンコードで比較可能）
- [x] `cargo test` で既存テスト（3592 件）が全 pass することを確認

---

## T6: fmt.rs — `ConstName`・`Item::ConstDef` フォーマット

- [x] `TypeExpr::ConstName(name, _) => format!("{}", name)` の arm を追加した
- [x] `Item::ConstDef` のフォーマット出力を追加した（`const {name}: {ty} = {value}`）
- [x] `cargo test` で既存テスト（3592 件）が全 pass することを確認

---

## T7: その他ファイル — コンパイルエラー解消

- [x] `fav/src/middle/compiler.rs` に `TypeExpr::ConstName` arm を追加した
- [x] `fav/src/middle/ast_lower_checker.rs` に `TypeExpr::ConstName` arm を追加した
- [x] `fav/src/lint.rs` に `TypeExpr::ConstName` arm を追加した
- [x] `fav/src/emit_python.rs` に `TypeExpr::ConstName` arm を追加した
- [x] `fav/src/driver.rs` の各 `ty_to_str` 系 match に `TypeExpr::ConstName` arm を追加した
- [x] `Item::ConstDef` のコンパイルエラー箇所に空アームを追加した
- [x] `fav/src/lsp/references.rs` — `collect_in_item` が `_ => {}` で自動スキップされることを確認した
- [x] `fav/src/lineage.rs` — `Item::ConstDef` が透過（skip）されることを確認した
- [x] `cargo build` が通ることを確認

---

## T8: error_catalog.rs — E0247・E0250 追加

- [x] E0247 エントリを追加した（未定義定数参照）— E0246 の直後
- [x] E0250 エントリを追加した（定数型不一致）— E0249 の後（E0248/E0249 は既存 checker.rs で使用中）

---

## T9: driver.rs — `v714000_tests` 追加

- [x] `v714000_tests` モジュールを追加した（`v713000_tests` の直後）
- [x] `const_eval_int_expr` テストを実装した（`const EMBED_DIM` + `HALF_DIM` + fn 呼び出し）
- [x] `const_used_in_dependent_type` テストを実装した（`Vec<Float>[EMBED_DIM]` 型注釈）
- [x] `cargo test v714000` で 2 件 pass することを確認

---

## T10: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.3.0"` → `"71.4.0"` に変更した
- [x] `driver.rs` 内の `"71.3.0"` 文字列を `"71.4.0"` に一括更新した（`replace_all: true`）

---

## T11: CHANGELOG.md 更新

- [x] `## [v71.4.0]` エントリを先頭に追加した（E0247/E0250、テスト数 3592→3594）

## T11b: site ドキュメント（スコープ外とする）

- [x] `const` 構文の言語リファレンス MDX 更新は v71.4.0 スコープ外とする（今後 v72.x のドキュメント整備フェーズで対応）

---

## T12: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.4.0`（Const / Compile-Time Evaluation）に更新した
- [x] 「次に切る版」を `v71.5.0` に更新した

---

## T13: 最終確認

- [x] `cargo test v714000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3594 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.4.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### [HIGH] E0247 が未定義定数参照時に発行されていない
- `None` ブランチで E0250 のみ発行していたため、未定義変数参照が「コンパイル時評価不能」という誤メッセージになっていた
- 対応: `collect_undefined_idents` / `walk_expr_idents` ヘルパーを追加し、`None` ブランチで undefined ident が存在する場合は E0247 を発行するよう分岐

### [MED] `parse_const_def` の span が `const` キーワードを含まない
- `self.advance()` の後に `let start = self.peek_span()` を取っていたため、span が定数名の位置を指していた
- 対応: `let start = self.peek_span().clone()` を `self.advance()` の前に移動

### [LOW] ロードマップのテスト数カウントが実績と不一致
- `roadmap-v71.1-v72.0.md` に `3587 + 2 = 3589` と記載されているが実績は 3592 → 3594
- ドキュメント上の不一致のみ、実装には影響なし（スコープ外）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T13）が完了している
- [x] `const_eval_int_expr` が pass
- [x] `const_used_in_dependent_type` が pass
- [x] テスト総数: 3594（+2、実績ベース: 3592 + 2）
- [x] `const EMBED_DIM: Int = 1536` が parse できる
- [x] `const HALF_DIM: Int = EMBED_DIM / 2` が `768` にコンパイル時評価される（宣言順評価）
- [x] `Vec<Float>[EMBED_DIM]` の次元位置で定数名が解決される
- [x] E0247（未定義定数参照）が実装済み
- [x] E0250（定数型不一致）が実装済み（E0248/E0249 は既存用途あり）
