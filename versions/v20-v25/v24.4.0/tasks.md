# v24.4.0 — v1.0 後方互換性ポリシー確定タスク

## ステータス: COMPLETE（2026-06-23）— コードレビュー指摘修正済み

---

## タスク一覧

### T0: 事前確認 + `ast.rs` FnDef 拡張

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.3.0"` であること
- [x] `grep -n "mod v244000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "deprecated: bool\|W020\|check_w020" fav/src/ast.rs fav/src/lint.rs | head -5` — 全 0 件
- [x] **T0-1**: `fav/src/ast.rs` の `FnDef` 構造体に `pub deprecated: bool,` フィールドを追加
- [x] **T0-2**: `FnDef` を構築している全箇所に `deprecated: false,` を追加（`grep -rn "FnDef {" fav/src/` で全ファイルを対象に検索し、`cargo check` でエラーになる箇所を全修正）
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T1: `fav/src/frontend/parser.rs` — `#[deprecated]` パース追加

- [x] **T1-1**: `parse_deprecated_annotation() -> Result<bool, ParseError>` を追加
  - `#`、`[`、`deprecated`（Ident）、`]` の 4 トークンを先読みして `true` を返す
  - 非一致の場合は `false` を返す（エラーにしない）
- [x] **T1-2**: `parse_item()` の先頭（既存アノテーション解析の前）に呼び出しを追加
  - `let deprecated_ann = self.parse_deprecated_annotation()?;`
- [x] **T1-3**: `parse_item()` 内の `fn` アームと `async fn` アームで `fd.deprecated = deprecated_ann;` を設定
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: `fav/src/lint.rs` — W020 追加

- [x] **事前確認**: `grep -n "Expr::Apply\|Expr::Ident\|MatchArm" fav/src/ast.rs | head -10` — `Apply(Box<Expr>, Vec<Expr>, Span)` / `Ident(String, Span)` / `MatchArm.body: Expr` を確認（FnCall ではない）
- [x] **T2-1**: `collect_deprecated_calls_in_expr` / `collect_deprecated_calls_in_block` を追加
- [x] **T2-2**: `pub fn check_w020_deprecated_call(program: &Program, errors: &mut Vec<LintError>)` を追加（既存 `check_w01x_*` と同じ 2 引数シグネチャ）
- [x] **T2-3**: `lint_program()` の `check_w019_string_concat_chain` 直後に追加:
  ```rust
  check_w020_deprecated_call(program, &mut errors);
  ```
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T3: `fav/src/driver.rs` — v244000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_3_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T5-1 より前に必須）**: `v243000_tests::version_is_24_3_0` テスト関数を**削除**
- [x] **T3-2**: `v244000_tests` モジュールを `v243000_tests` の直後に追加（5 件）
  - `version_is_24_4_0`
  - `deprecated_fn_annotation_parsed`
  - `deprecated_call_emits_w020`
  - `stability_md_has_policy`
  - `changelog_has_v24_4_0`
- [x] `cargo test v244000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1948 件合格）を確認

---

### T4: `STABILITY.md` 作成 + ドキュメントサイト更新

- [x] `STABILITY.md` をリポジトリルート（`C:\Users\yoshi\favnir\STABILITY.md`）に作成
  - セクション: v1.x 後方互換性保証 / v2.0 破壊的変更ポリシー / SemVer 準拠 / `--legacy` フラグ
  - `"v1.x"` と `"v2.0"` の両文字列を含む（`stability_md_has_policy` テスト要件）
  - **パス確認**: `driver.rs` から `include_str!("../../STABILITY.md")` は `fav/src/` → `fav/` → リポジトリルートを指す。`C:\Users\yoshi\favnir\STABILITY.md` で正しい
- [x] `site/content/docs/tools/lint.mdx` に W020 セクションを追記（`#[deprecated]` アノテーションの使い方と W020 警告の説明）

---

### T5: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.3.0"` → `"24.4.0"` に変更（T3-1 完了後）
- [x] `CHANGELOG.md` 先頭に v24.4.0 エントリを追加
- [x] `benchmarks/v24.4.0.json` を新規作成（test_count: 1948、duration_ms は実測値に更新）
- [x] `cargo test v244000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1948 件合格）

---

## テスト一覧（v244000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_4_0` | Cargo.toml に `version = "24.4.0"` | — |
| `deprecated_fn_annotation_parsed` | `#[deprecated] fn old_func(...)` を parse して `fn.deprecated == true` | `true` |
| `deprecated_call_emits_w020` | deprecated fn を別 fn から呼び出すと W020 が 1 件（`check_w020_deprecated_call(&prog, &mut warnings)` で検証） | `warnings.len() == 1` |
| `stability_md_has_policy` | `STABILITY.md` に `"v1.x"` と `"v2.0"` が含まれる | — |
| `changelog_has_v24_4_0` | `CHANGELOG.md` に `[v24.4.0]` | — |

---

## 完了条件チェックリスト

- [x] `FnDef.deprecated: bool` フィールド追加済み（全構築箇所に `false` 追加）
- [x] `parse_deprecated_annotation()` 実装済み（`#[deprecated]` を認識）
- [x] `check_w020_deprecated_call()` 実装済み（`lint_program()` に組み込み）
- [x] `v243000_tests::version_is_24_3_0` が削除済み（T5-1 より前）
- [x] `cargo test v244000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1948 件合格）
- [x] `STABILITY.md` 作成済み（v1.x / v2.0 / SemVer / --legacy の 4 セクション）
- [x] `CHANGELOG.md` に v24.4.0 エントリ
- [x] `benchmarks/v24.4.0.json` 作成済み（test_count: 1948）
- [x] `site/content/docs/tools/lint.mdx` に W020 セクション追記済み

---

## コードレビュー指摘と対応（2026-06-23）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| MED-1 | `check_w020_deprecated_call` が deprecated fn 自身の body もスキャンし再帰呼び出しで誤検出 | `Item::FnDef(fd) if !fd.deprecated` ガード追加 |
| MED-2 | deprecated fn の再帰呼び出しが W020 を発火しないことのテストなし | `deprecated_fn_self_call_no_w020` テスト追加（driver.rs）|
| LOW-3 | `check_w020_expr` に `Expr::Pipeline` アームが欠落（pipeline 内の deprecated 呼び出しを見逃す） | `Expr::Pipeline` アーム追加（lint.rs）|
| LOW-4 | `parse_item()` でアノテーション順序の説明なし | 順序説明コメント追加（parser.rs）|

修正後テスト: v244000 6/6 PASS、全体 1949 件合格（0 failures）
