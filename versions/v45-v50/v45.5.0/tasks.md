# Tasks: v45.5.0 — 型エイリアス完全化（E0413 opaque alias coerce）

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2977 passed, 0 failed を確認
- [x] `grep -r "opaque type" fav/src/` で既存テストに `opaque type` 使用が0件であることを確認

## T1 — `checker.rs`: `opaque_alias_inner` フィールド追加

- [x] `Checker` struct に `opaque_alias_inner: HashMap<String, Type>` を追加
- [x] `Checker::new()` の初期化リストに `opaque_alias_inner: HashMap::new()` を追加
- [x] `Checker::new_with_resolver()` の初期化リストにも `opaque_alias_inner: HashMap::new()` を追加

## T2 — `checker.rs`: `register_item_signatures` 修正

- [x] `TypeBody::Alias` の処理を `td.is_opaque` で分岐
  - [x] `is_opaque = false` → 従来通り `type_aliases` に登録
  - [x] `is_opaque = true` → `type_aliases` に登録せず、`resolve_type_expr(inner_te)` の結果を `opaque_alias_inner` に登録

## T3 — `checker.rs`: `check_fn_def` / `check_trf_def` に E0413 追加

- [x] `check_fn_def` の戻り型チェック箇所（E0101 発行前）に E0413 チェックを追加
  - [x] 期待型が `Type::Named(n, _)` で `opaque_alias_inner` に `n` が存在する場合
  - [x] 実際の型が inner type と一致する（`body_ty.is_compatible(inner_ty)`）場合 → E0413 発行して return
- [x] `check_trf_def` の戻り型チェック箇所にも同様の E0413 チェックを追加
  - [x] 期待型が `Type::Named(n, _)` で `opaque_alias_inner` に `n` が存在する場合
  - [x] 実際の型が inner type と一致する場合 → E0413 発行して return

## T4 — `driver.rs`: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.5.0`
- [x] `v455000_tests` モジュール追加（2件）
  - [x] `check_src` ヘルパー定義（`.code.to_string()` で `Vec<String>` に変換）
  - [x] `transparent_alias_compatible` — `type UserId = Int`、`fn get_id() -> UserId { 42 }` → エラーなし
  - [x] `opaque_alias_incompatible` — `opaque type Token = String`、`fn make_token() -> Token { "abc" }` → E0413

## T5 — テスト＆完了

- [x] `cargo test` 2979 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.5.0 エントリ追加
- [x] `versions/current.md` を v45.5.0（2979 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T5 全チェック）

## コードレビュー指摘と対応

- [HIGH] code-reviewer: `check_fn_def` の `return;` が `env.pop()` / `linear_env` 復元等をスキップしスコープ崩壊 → `e0413_fired` フラグに変更し E0101 のみ抑制（クリーンアップはそのまま実行）
- [HIGH] code-reviewer: `check_trf_def` で E0413 + E0101 がダブル発火 → 同様に `e0413_fired` フラグで E0101 を抑制
- [HIGH] code-reviewer: エラーメッセージ「use an explicit constructor」がコンストラクタ未実装と矛盾 → 「declare a dedicated constructor function for this opaque type」に変更
- [LOW] code-reviewer: `check_fn_def` コメントに矛盾（「E0101 も emit」と「return early」が逆） → コメント削除・フラグ説明に置き換え
- [LOW] code-reviewer: `check_trf_def` パスのテストなし → `opaque_alias_trf_incompatible_no_double_error` テスト追加（stage 構文使用）
- [HIGH] spec-reviewer: `collect_transparent_alias_chain` ヘルパー未実装 → `resolve_type_expr` の再帰処理で代替済みと明記（新関数不要）
- [HIGH] spec-reviewer: `new_with_resolver()` への `opaque_alias_inner` 追加漏れ → plan.md / tasks.md T1 両方に追加・実装済み
- [HIGH] spec-reviewer: チェーン解決根拠なし → spec.md §1 に `resolve_type_expr` 再帰動作を説明追記
- [HIGH] spec-reviewer: `Type::Named` 記述と `resolve_type_expr` 戻り値型の矛盾 → spec.md §2.2 / plan.md Step 2 を `Type::String` 等の実際の型に統一
