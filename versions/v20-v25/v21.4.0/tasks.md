# v21.4.0 — `fav lint` 強化（W010〜W019）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/lint.rs` — W010〜W019 実装

- [x] **事前確認**: `grep -n "^pub fn lint_program\|^fn check_" fav/src/lint.rs | tail -10` で追加箇所を確認
- [x] **事前確認**: `grep -n "AMBIENT_NAMESPACES" fav/src/lint.rs | head -5` で既存定数の行番号を確認
- [x] `use std::collections::{HashMap, HashSet};` を lint.rs の use 宣言に追加（未追加なら）
- [x] **重要**: `LintError::new` の第1引数は `&'static str`。コードは必ず `"W010"` 等の文字列リテラルで渡す（変数は使わない）
- [x] W010 — `check_w010_stage_too_large` を実装
  - [x] `Item::TrfDef(td)` を走査し `td.body.stmts.len() > 30` を検査
  - [x] エラーコード `"W010"`、span は `td.span.clone()`
- [x] W011 — `check_w011_effectless_io_call` を実装
  - [x] `const W011_AMBIENT: &[&str]` を新規追加（lint.rs に AMBIENT_NAMESPACES という既存定数はない）
  - [x] `td.effects.is_empty()` かつ body 内に `W011_AMBIENT` namespace の FieldAccess 呼び出しがある場合
  - [x] `find_ambient_call_in_block` / `find_ambient_call_in_expr` ヘルパーを新規実装
  - [x] エラーコード `"W011"`
- [x] W012 — `check_w012_unused_type` を実装
  - [x] `Item::TypeDef(td)` で `td.visibility.is_none()` の名前を収集
  - [x] `collect_used_type_names_program` → TypeExpr::Named を再帰収集（型引数含む）
  - [x] 使用されていない TypeDef 名 → W012
- [x] W013 — `check_w013_map_filter_chain` を実装
  - [x] `check_w013_expr(expr)` で `Expr::Pipeline(steps, _)` を走査
  - [x] `is_list_map(step)` / `is_list_filter(step)` ヘルパー（FieldAccess + Ident("List") チェック）
  - [x] 連続する `map → filter` ペアで W013
  - [x] 全 Item の body を再帰的に走査
- [x] W014 — `check_w014_redundant_result_ok` を実装
  - [x] `check_w014_block(block)` で `Stmt::Bind(b)` を走査
  - [x] `b.pattern` が `Pattern::Bind(name, _)`（Wildcard でない）かつ `b.expr` が `Apply(FieldAccess(Ident("Result"), "ok"), [_])` → W014
  - [x] **注意**: `BindStmt` に `.name` フィールドはない。`.pattern: Pattern` を使う
- [x] W015 — `check_w015_rebind_in_block` を実装
  - [x] `check_w015_block(block)` でブロック内の `Stmt::Bind(b)` を走査
  - [x] `b.pattern` が `Pattern::Bind(name, _)` のとき `HashMap<name, first_span>` に記録
  - [x] `Pattern::Wildcard(_)` はスキップ
  - [x] 同名が既に登録済み → W015（最初の Span を記録）
  - [x] ネストした block（ForIn の body 等）は独立してチェック
  - [x] **注意**: `BindStmt` に `.name` フィールドはない。`.pattern: Pattern` を使う
- [x] W016 — `check_w016_wildcard_only_match` を実装
  - [x] `check_w016_expr(expr)` で `Expr::Match(_, arms, span)` を走査
  - [x] `arms.len() == 1 && matches!(arms[0].pattern, Pattern::Wildcard(_))` → W016
  - [x] 全 Item の body を再帰的に走査
- [x] W017 — `check_w017_deep_nesting` を実装
  - [x] `nesting_depth(expr) -> usize`: Match/If に遭遇するたびに +1、子の最大深さを返す
  - [x] 全 Item の body の expr を走査し `depth > 4` → W017
  - [x] span: 最も外側の Match/If の span
- [x] W018 — `check_w018_magic_number` を実装
  - [x] `check_w018_expr(expr)` で `Expr::Lit(Lit::Int(n), span)` / `Lit::Float(f)` を走査
  - [x] `n.unsigned_abs() > 100` または `f.abs() > 100.0` → W018
  - [x] 全 Item の body（FnDef, TrfDef, TestDef 等）を対象
- [x] W019 — `check_w019_string_concat_chain` を実装
  - [x] `is_string_concat(expr)` ヘルパー
  - [x] `check_w019_expr(expr)`: `is_string_concat(expr)` かつその引数に `is_string_concat` → W019
  - [x] 全 Item の body を再帰的に走査
- [x] `lint_program` の末尾に 10 関数を追加
  ```rust
  check_w010_stage_too_large(program, &mut errors);
  // ... W011-W019
  ```
- [x] `cargo check` でコンパイルエラー 0
- [x] `cargo test lint` — 既存 lint_tests がリグレッションしていないことを確認

---

### T2: `fav/src/driver.rs` — `cmd_explain_hint` 更新

- [x] **事前確認**: `grep -n "cmd_explain_hint\|\"W009\"" fav/src/driver.rs | head -5`
  - `cmd_explain_hint` が存在しない場合は新規 `pub fn cmd_explain_hint(code: &str)` として追加
  - `"W009"` が存在しない場合は W010〜W019 エントリのみ追加
- [x] `cmd_explain_hint` の match ブロックに W010〜W019 を追加
  ```rust
  "W010" => &["split the stage into smaller, focused stages"],
  "W011" => &["add `!Io` to the stage signature, or pass capability via ctx"],
  "W012" => &["remove the unused type, or make it `pub` if used externally"],
  "W013" => &["use `List.filter_map(|x| { ... })` instead"],
  "W014" => &["bind directly from the inner expression without Result.ok"],
  "W015" => &["rename the second binding, or use `bind _` to discard"],
  "W016" => &["add specific patterns before `_`; if intentional, this is fine"],
  "W017" => &["extract the inner logic to a separate helper function"],
  "W018" => &["extract to a named constant"],
  "W019" => &["use an f-string: f\"{a}{b}{c}\""],
  ```
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/Cargo.toml` バージョン更新

- [x] **事前確認**: `grep -n "version_is_21_3_0" fav/src/driver.rs | head -3` で行番号を確認
- [x] `version = "21.3.0"` → `"21.4.0"` に変更
- [x] `v213000_tests::version_is_21_3_0` に `#[ignore]` を追加
- [x] `cargo test v213000` — `version_is_21_3_0` が ignore されること

---

### T4: `CHANGELOG.md` + `site/content/docs/tools/lint.mdx`

- [x] `CHANGELOG.md` の先頭に v21.4.0 エントリを追加（plan.md T4 の内容に従う）
- [x] **事前確認**: `ls site/content/docs/tools/lint.mdx 2>/dev/null || echo "not exists"` で存在確認
- [x] `site/content/docs/tools/lint.mdx` を新規作成または更新
  - [x] W010 セクション（stage_too_large 説明・悪い例・良い例）
  - [x] W011 セクション（effectless_io_call 説明・悪い例・良い例）
  - [x] W012 セクション（unused_type 説明・悪い例・良い例）
  - [x] W013 セクション（map_filter_chain 説明・悪い例・良い例）
  - [x] W014 セクション（redundant_result_ok 説明・悪い例・良い例）
  - [x] W015 セクション（rebind_in_block 説明・悪い例・良い例）
  - [x] W016 セクション（wildcard_only_match 説明・悪い例・良い例）
  - [x] W017 セクション（deep_nesting 説明・悪い例・良い例）
  - [x] W018 セクション（magic_number 説明・悪い例・良い例）
  - [x] W019 セクション（string_concat_chain 説明・悪い例・良い例）

---

### T5: `fav/src/driver.rs` — `v214000_tests` 追加

- [x] **事前確認**: `grep -n "mod v213000_tests" fav/src/driver.rs | head -3` で追加位置を確認
- [x] `v213000_tests` モジュールの後に `v214000_tests` モジュールを追加（plan.md T5 の内容に従う）
  - [x] `version_is_21_4_0`
  - [x] `lint_w010_stage_too_large`
  - [x] `lint_w011_effectless_io_call`
  - [x] `lint_w012_unused_type`
  - [x] `lint_w013_map_filter_chain`
  - [x] `lint_w014_redundant_result_ok`
  - [x] `lint_w015_rebind_in_block`
  - [x] `lint_w016_wildcard_only_match`
  - [x] `lint_w017_deep_nesting`
  - [x] `lint_w017_no_w017_at_4_levels`（4重ネストでは W017 が出ないことを確認）
  - [x] `lint_w018_magic_number`
  - [x] `lint_w019_string_concat_chain`
- [x] `cargo test v214000` — 12/12 PASS を確認
- [x] `cargo test` — リグレッションなし（exit 0）を確認

---

## テスト（v214000_tests、12件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_4_0` | Cargo.toml に `"21.4.0"` が含まれる |
| `lint_w010_stage_too_large` | 31 stmt の stage → W010 |
| `lint_w011_effectless_io_call` | エフェクトなし stage が IO.println → W011 |
| `lint_w012_unused_type` | 参照されない TypeDef → W012 |
| `lint_w013_map_filter_chain` | List.map |> List.filter パイプライン → W013 |
| `lint_w014_redundant_result_ok` | bind x <- Result.ok(expr) → W014 |
| `lint_w015_rebind_in_block` | 同名 bind が同一ブロックに 2 回 → W015 |
| `lint_w016_wildcard_only_match` | match が `_ =>` のみ → W016 |
| `lint_w017_deep_nesting` | ネスト 5 段 → W017 |
| `lint_w017_no_w017_at_4_levels` | ネスト 4 段 → W017 が出ない（ネガティブ） |
| `lint_w018_magic_number` | リテラル 9999 → W018 |
| `lint_w019_string_concat_chain` | String.concat(String.concat(...)) → W019 |

---

## 完了条件チェックリスト

- [x] `fav lint` で W010〜W019 が出力される
- [x] 各ルールが独立して動作する（他ルールを誤検知しない）
- [x] `cargo test v214000` — 12/12 PASS
- [x] `cargo test` — リグレッションなし（exit 0）
- [x] `CHANGELOG.md` に v21.4.0 エントリが追加されている
- [x] `fav/Cargo.toml` version が `21.4.0`
- [x] `site/content/docs/tools/lint.mdx` に W010〜W019 が記載されている

---

## 優先度

```
T1（lint.rs — W010〜W019）       ← 最初（最大タスク）
T2（driver.rs — explain_hint）   ← T1 完了後
T3（Cargo.toml バージョン）       ← T1 と並列可
T4（CHANGELOG + MDX）            ← T3 完了後
T5（driver.rs テスト）            ← T1 完了後
```

---

## 実装リスクと対策

| リスク | 対策 |
|--------|------|
| W011 が W008/E0023 と重複誤検知 | W011 は `TrfDef` 限定。`FnDef` は対象外 |
| W012 が型引数（`List<Ghost>`）を見落とす | `collect_used_type_names` は TypeExpr::Named の型引数を再帰収集 |
| W015 が `bind _ <- ...` を誤検知 | 名前が `"_"` の場合はスキップ |
| W018 が型注釈リテラルを誤検知 | `check_w018_expr` は `Expr` コンテキストのみ（TypeExpr は別系統） |
| 既存 lint_tests がリグレッション | W010〜W019 は新規コードのみ。L001〜L008 は変更しない |
