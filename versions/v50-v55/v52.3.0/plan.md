# Plan: v52.3.0 — `fav explain --lineage` 表示強化（スキーマ情報付加）

Status: PLANNED
Date: 2026-07-21

---

## 実装順序

### Step 1 — `LineageEntry` に `schema` フィールド追加（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- `LineageEntry` struct の末尾に追加:
  ```rust
  pub schema: Option<String>,  // v52.3.0: first assert_schema<T> type name in body
  ```
- `cargo build` → コンパイルエラーで `LineageEntry { ... }` 構築箇所を洗い出す
  - 期待される構築箇所: `lineage_analysis` 内の TrfDef ループ、FnDef ループの 2 箇所
  - 両方に `schema: None,` を追加（実際のスキーマ収集は Step 3 で行う）

### Step 2 — `collect_assert_schema_name` 関数追加（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- 既存の `collect_sql_literals_inner` などの collect 系関数の直後に追加
- 3 つの関数を実装:
  - `pub fn collect_assert_schema_name(expr: &ast::Expr) -> Option<String>`
  - `fn collect_assert_schema_name_stmt(s: &ast::Stmt) -> Option<String>`
  - `fn collect_assert_schema_name_block(b: &ast::Block) -> Option<String>`
- `Expr::AssertSchema { ty_name, .. }` アームを最初に処理し、残りは再帰
- `Expr::Block` / `Pipeline` / `Apply` / `If` / `Match` の各アームを実装
  - 他のアームは `_ => None` でまとめる

**注意**: `collect_sql_literals_inner` を参考にするが、SQL 収集と違い「最初の 1 件だけ返す」設計。
`find_map` を活用する。

### Step 3 — `lineage_analysis` 更新（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- TrfDef ループ内の `LineageEntry` 構築直前にスキーマ収集を追加:
  ```rust
  let schema = collect_assert_schema_name_block(&trf.body);
  ```
  Note: `collect_sql_literals` のように `Expr::Block(Box::new(trf.body.clone()))` でラップ**しない**こと。
  `collect_assert_schema_name_block` を直接呼べばクローン不要で、Clippy の不要アロケーション警告も出ない。
- `LineageEntry { ..., schema }` として設定（Step 1 で追加した `schema: None` を `schema` に変更）
- FnDef ループの `LineageEntry { ..., schema: None }` は変更しない（fn は対象外）

### Step 4 — `render_lineage_mermaid_with_schema` 追加（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- 既存の `render_lineage_mermaid_with_opts` の直後に新関数を追加:
  ```rust
  pub fn render_lineage_mermaid_with_schema(
      report: &LineageReport,
      show_dead: bool,
      with_schema: bool,
  ) -> String
  ```
- 既存 `render_lineage_mermaid_with_opts` の内部ロジックをコピーし、ノードラベル部分のみ変更:
  ```rust
  let schema_label = if with_schema {
      entry.schema.as_ref()
          .map(|s| format!("<br/>schema:{}", s))
          .unwrap_or_default()
  } else {
      String::new()
  };
  out.push_str(&format!(
      "  {}[\"{}<br/>{}{}\"]\n",
      id, entry.name, effects, schema_label
  ));
  ```
- 既存の `render_lineage_mermaid_with_opts` / `render_lineage_mermaid` は変更しない

### Step 5 — `render_lineage_dot_with_schema` 追加（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- 既存の `render_lineage_dot` の直後に新関数を追加:
  ```rust
  pub fn render_lineage_dot_with_schema(report: &LineageReport, with_schema: bool) -> String
  ```
- 既存 `render_lineage_dot` のロジックをコピーし、ラベル部分のみ変更:
  ```rust
  let schema_part = if with_schema {
      entry.schema.as_ref()
          .map(|s| format!("\\nschema:{}", s))
          .unwrap_or_default()
  } else {
      String::new()
  };
  let label = format!("{}\\n{}{}", entry.name, entry.kind, schema_part);
  ```
- 既存の `render_lineage_dot` は変更しない

### Step 6 — `cmd_explain_lineage` 更新（driver.rs）

- ファイル: `fav/src/driver.rs`
- 関数シグネチャ変更（line ~23382）:
  ```rust
  pub fn cmd_explain_lineage(file: Option<&str>, format: &str, show_dead: bool, with_schema: bool)
  ```
- `match format` の各アームを更新:
  - `"mermaid"` → `render_lineage_mermaid_with_schema(&report, show_dead, with_schema)`
  - `"dot"` → `render_lineage_dot_with_schema(&report, with_schema)`
  - 他のアーム（json / d2 / svg / text）は変更なし
- `use` インポートに `render_lineage_mermaid_with_schema` / `render_lineage_dot_with_schema` を追加
  （既存の `use crate::lineage::*` があれば不要）

**シグネチャ変更の影響範囲確認**:
- `main.rs` の `cmd_explain_lineage(file, &format, show_dead)` 呼び出し → Step 7 で更新
- driver.rs 内の他の呼び出し箇所を `rg "cmd_explain_lineage"` で洗い出す

### Step 7 — `main.rs` — `--with-schema` フラグ解析

- ファイル: `fav/src/main.rs`
- `--lineage` ブロックの変数宣言に追加（`show_dead` の宣言の隣）:
  ```rust
  let mut with_schema = false;
  ```
- `while i < args.len()` ループの `match args[i].as_str()` に追加（`"--show-dead"` の直後）:
  ```rust
  "--with-schema" => { with_schema = true; i += 1; }
  ```
- `cmd_explain_lineage` 呼び出しを更新:
  ```rust
  cmd_explain_lineage(file, &format, show_dead, with_schema);
  ```

### Step 8 — `driver.rs` にテスト追加 + バージョン更新

- `v52300_tests` モジュールを `v52200_tests` の直前に追加（2 件）
- `fav/Cargo.toml` version → `"52.3.0"`
- `cargo test` → 3140 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

### Step 9 — 後処理

- `CHANGELOG.md` に v52.3.0 エントリ追加
- `versions/current.md` を v52.3.0（3140 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.3.0 実績欄を更新（テスト数を 3141 → 3140 に訂正）
- `tasks.md` を COMPLETE に更新（T0〜T9 全 `[x]`）

---

## 注意事項

- `render_lineage_mermaid_with_opts` / `render_lineage_dot` は変更しない（後方互換維持）
  — テスト済みの既存関数を壊さないために新関数を追加する方針
- `collect_assert_schema_name` は「最初に見つかった 1 件だけ」返す設計（stage に複数の `assert_schema` があっても先頭を採用）
- `schema: Option<String>` への `#[serde(skip_serializing_if = "Option::is_none")]` は付与しない（JSON 出力の一貫性のため `null` を出力）
- `trf.body.clone()` は既存の `collect_sql_literals` / `collect_snowflake_call_kinds` と同パターンのため問題なし
- `cmd_explain_lineage` のシグネチャ変更に伴い、driver.rs 内の他の呼び出し箇所も必ず更新すること
