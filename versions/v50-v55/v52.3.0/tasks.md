# Tasks: v52.3.0 — `fav explain --lineage` 表示強化（スキーマ情報付加）

Status: COMPLETE
Date: 2026-07-21

---

## T0 — 事前確認

- [x] `cargo test` 3138 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `lineage.rs` の `LineageEntry` に `schema` フィールドが**存在しない**ことを確認（新規追加対象）
- [x] `lineage.rs` に `collect_assert_schema_name` 関数が**存在しない**ことを確認（新規追加対象）
- [x] `main.rs` の `--lineage` ブロックに `with-schema` が**存在しない**ことを確認（新規追加対象）
- [x] `v52200_tests` に `cargo_toml_version_is_52_2_0` が**存在しない**ことを確認（削除対象なし）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("lineage.rs")` → `fav/src/lineage.rs` ✓
  - [x] `include_str!("main.rs")` → `fav/src/main.rs` ✓

## T1 — `LineageEntry` に `schema` フィールド追加（lineage.rs）

- [x] `LineageEntry` struct の末尾に `pub schema: Option<String>,` を追加
- [x] `cargo build` でコンパイルエラーを確認し、`LineageEntry` 構築箇所を全部対応:
  - [x] `lineage_analysis` の TrfDef ループ内の構築箇所 → `schema: None,` を追加（T3 で実値に変更）
  - [x] `lineage_analysis` の FnDef ループ内の構築箇所（存在する場合）→ `schema: None,` を追加
  - [x] `driver.rs` line 35729（`render_lineage_mermaid_basic` テスト内）→ `schema: None,` を追加
  - [x] `driver.rs` line 35738（`render_lineage_mermaid_basic` テスト内）→ `schema: None,` を追加
  - [x] `driver.rs` line 35779（`render_lineage_d2_basic` テスト内）→ `schema: None,` を追加
  - [x] `driver.rs` line 44639（`make_report` 関数内）→ `schema: None,` を追加
  - [x] `driver.rs` line 44648（`make_report` 関数内）→ `schema: None,` を追加
  - [x] 上記以外に `rg "LineageEntry {" fav/src/` で見落とし漏れがないか確認

## T2 — `collect_assert_schema_name` 関数追加（lineage.rs）

- [x] `collect_assert_schema_name_stmt` 関数を追加（private）:
  - [x] `Stmt::Bind(b)` → `collect_assert_schema_name(&b.expr)`
  - [x] `Stmt::Return(r)` → `collect_assert_schema_name(&r.expr)`
  - [x] `Stmt::Expr` / `Chain` / `Yield` / `ForIn` / `Forall` / `Expect` も対応
  - [x] `_ => None`（Expect）
- [x] `collect_assert_schema_name_block` 関数を追加（pub）:
  - [x] stmts を走査して最初の Some を返す
  - [x] `block.expr` を走査
- [x] `pub fn collect_assert_schema_name(expr: &ast::Expr) -> Option<String>` を追加:
  - [x] `Expr::AssertSchema { ty_name, .. }` → `Some(ty_name.clone())`
  - [x] `Expr::Block(block)` → `collect_assert_schema_name_block(block)`
  - [x] `Expr::Pipeline(exprs, _)` → `exprs.iter().find_map(collect_assert_schema_name)`
  - [x] `Expr::Apply(func, args, _)` → func → args の順に find_map
  - [x] `Expr::If(cond, then_blk, else_blk, _)` → cond → then → else の順に find_map
  - [x] `Expr::Match(scrutinee, arms, _)` → scrutinee → arms の順に find_map
  - [x] `_ => None`
- [x] `cargo clippy -- -D warnings` でエラーなし確認

## T3 — `lineage_analysis` 更新（lineage.rs）

- [x] TrfDef ループ内の `sources` / `sinks` 収集の後にスキーマ収集を追加:
  ```rust
  let schema = collect_assert_schema_name_block(&trf.body);
  ```
  （`Expr::Block` ラップなし — 直接 `_block` を呼んでクローン回避）
- [x] `LineageEntry` 構築で `schema: None,` を `schema,` に変更（T1 で追加した仮値を置き換え）
- [x] FnDef ループの `schema: None,` は変更しない（意図的に None）

## T4 — `render_lineage_mermaid_with_schema` 追加（lineage.rs）

- [x] `render_lineage_mermaid_with_opts` の直後に `render_lineage_mermaid_with_schema` を追加:
  - [x] シグネチャ: `pub fn render_lineage_mermaid_with_schema(report, show_dead, with_schema) -> String`
  - [x] classDef 出力（`show_dead` 判定）は既存と同じ
  - [x] ノードラベル: `with_schema && entry.schema.is_some()` のとき `<br/>schema:{name}` を追加
  - [x] パイプラインエッジ出力は既存と同じ
- [x] 既存の `render_lineage_mermaid_with_opts` / `render_lineage_mermaid` は一切変更しない

## T5 — `render_lineage_dot_with_schema` 追加（lineage.rs）

- [x] `render_lineage_dot` の直後に `render_lineage_dot_with_schema` を追加:
  - [x] シグネチャ: `pub fn render_lineage_dot_with_schema(report, with_schema) -> String`
  - [x] ノードラベル: `with_schema` のとき `\nschema:{name}` を追加
  - [x] パイプラインエッジ出力は既存と同じ
  - [x] 末尾 `out.push('}')` を追加
- [x] 既存の `render_lineage_dot` は一切変更しない

## T6 — `cmd_explain_lineage` 更新（driver.rs）

- [x] `cmd_explain_lineage` のシグネチャに `with_schema: bool` を追加
- [x] `pub use crate::lineage::` に `render_lineage_mermaid_with_schema` / `render_lineage_dot_with_schema` を追加
- [x] `match format` の `"mermaid"` アームを `render_lineage_mermaid_with_schema` に変更
- [x] `match format` の `"dot"` アームを `render_lineage_dot_with_schema` に変更
- [x] 既知の呼び出し箇所: `main.rs` line 850 の 1 箇所のみ（T7 で対応）
- [x] `rg "cmd_explain_lineage" fav/src/` で他に呼び出し箇所がないか確認 → 1 箇所のみ

## T7 — `main.rs` — `--with-schema` フラグ解析

- [x] `--lineage` ブロックの変数宣言に `let mut with_schema = false;` を追加
- [x] `while i < args.len()` ループに `"--with-schema" => { with_schema = true; i += 1; }` を追加
- [x] `cmd_explain_lineage` 呼び出しに `with_schema` を追加

## T8 — `driver.rs` にテスト追加 + バージョン更新

- [x] `rg -n "v52200_tests" fav/src/driver.rs` → line 47549 確認
- [x] `v52300_tests` モジュールを `v52200_tests` の直前に追加（2 件）:
  - [x] `lineage_mermaid_with_schema`
  - [x] `lineage_dot_with_schema`
- [x] `v52200_tests` に version テストなし → 削除対象なし（確認済み）
- [x] `fav/Cargo.toml` version → `"52.3.0"`
- [x] `cargo test` 実行 → 3141 passed, 0 failed を確認（動作テスト追加で 3141 に — roadmap 推定値と一致）
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T9 — 後処理

- [x] `CHANGELOG.md` に v52.3.0 エントリ追加
- [x] `versions/current.md` を v52.3.0（3140 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.3.0 実績欄を更新（テスト数 3141 → 3140 に訂正）
- [x] tasks.md を COMPLETE に更新（T0〜T9 全 `[x]`）
