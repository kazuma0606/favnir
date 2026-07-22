# Tasks: v51.2.0 — `par` Phase 2（Merge.ordered / Merge.any）

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3115 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `ast.rs` に `FlwStep::Merge` が**存在しない**ことを確認
- [x] `ast.rs` に `MergeMode` enum が**存在しない**ことを確認
- [x] `parser.rs` が `Merge.ordered` / `Merge.any` をパースしないことを確認
- [x] `v51100_tests::cargo_toml_version_is_51_1_0` が存在することを確認（削除対象）
- [x] `middle/ast_lower_checker.rs` の `lower_flw_step` 関数を確認（`SStage` パターン使用 — `SMergeOrdered` / `SMergeAny` タグで追加）
- [x] `middle/compiler.rs` の実際の関数名確認: `build_step_call` / `build_step_call_ctx` / `flw_step_name`（`IO_GLOBAL_IDX` 定数は存在せず `ctx.resolve_global("IO")` パターンを使用）
- [x] `middle/checker.rs` の FlwStep match 箇所を確認（5箇所）

## T1 — `ast.rs` — MergeMode enum + FlwStep::Merge variant 追加

- [x] `MergeMode { Ordered, Any }` enum を追加
- [x] `FlwStep::Merge(MergeMode)` variant を追加（unit variant 2 つではなく tuple variant 1 つ）
- [x] `stage_names()` に `Merge(_) => vec![]` arm 追加
- [x] `display_str()` に `Merge(MergeMode::Ordered) => "Merge.ordered"` / `Merge(MergeMode::Any) => "Merge.any"` arm 追加
- [x] `cargo build` が通ることを確認

## T2 — `frontend/parser.rs` — Merge.ordered / Merge.any パース

- [x] `parse_flw_step` 内の `"Merge"` 分岐を特定
- [x] peek2 による先読みで `"Merge" + "." + "ordered"` → `FlwStep::Merge(MergeMode::Ordered)` 分岐追加
- [x] peek2 による先読みで `"Merge" + "." + "any"` → `FlwStep::Merge(MergeMode::Any)` 分岐追加
- [x] `"Merge"` のみ（フォールバック）→ `FlwStep::Stage("Merge")` 継続（既存テスト互換）

## T3 — `middle/compiler.rs` — build_step_call / build_step_call_ctx / flw_step_name 更新

- [x] `build_step_call` に `FlwStep::Merge(mode)` arm 追加（`ctx.resolve_global("IO")` + FieldAccess + Call）
- [x] `build_step_call_ctx` に `FlwStep::Merge(mode)` arm 追加
- [x] `flw_step_name` に `Merge(Ordered) => "merge.ordered"` / `Merge(Any) => "merge.any"` arm 追加

## T4 — `backend/vm.rs` — merge_ordered_raw / merge_any_raw ハンドラ追加

- [x] `IO` builtin dispatch に `"merge_ordered_raw"` ハンドラ追加
  - [x] 各要素: `Variant("ok" | "some", Some(payload))` → payload を unwrap して results に追加
  - [x] 各要素: `Variant("ok" | "some", None)` → `VMValue::Unit` を results に追加
  - [x] 各要素: `Variant("err", payload)` → fail-fast で Err を返す
  - [x] 各要素: その他 → そのまま results に追加（非 Result stages 互換）
  - [x] `Ok(VMValue::List(FavList::new(results)))` を返す
- [x] `IO` builtin dispatch に `"merge_any_raw"` ハンドラ追加
  - [x] std::thread 実装では `merge_ordered_raw` と同一動作
  - [x] コメントで「将来 tokio 実装時に FuturesUnordered 相当の再実装を予定（v51.3+）」と明記

## T5 — match 網羅性更新

- [x] `middle/checker.rs` — メイン step match に `Merge(_)` arm 追加（`current_output = Some(Type::Unknown)`）
- [x] `middle/checker.rs` — `step_first_stage` match に `Merge(_) => None` 追加
- [x] `middle/checker.rs` — `step_last_stage` match に `Merge(_) => None` 追加
- [x] `emit_python.rs` — `build_chain_expr` の FlwStep match に `Merge(_) => { /* skip */ }` arm 追加
- [x] `emit_python.rs` — `emit_flw_with_par` の FlwStep match に `Merge(_) => { /* skip */ }` arm 追加
- [x] `emit_python.rs` — `has_par` チェック（行 1185 付近）は**変更なし**
- [x] `middle/ast_lower_checker.rs` — `lower_flw_step` に `Merge(Ordered) => v1("SMergeOrdered", ...)` / `Merge(Any) => v1("SMergeAny", ...)` arm 追加
- [x] `cargo build` が通ることを確認

## T6 — `driver.rs` — v51200_tests 追加

- [x] `v51200_tests` モジュールを `v51100_tests` の直前に追加（3 件）:
  - [x] `cargo_toml_version_is_51_2_0`: version が `"51.2.0"` を含むことを assert
  - [x] `par_stage_merge_ordered`: `Merge.ordered` が `VMValue::List` かつ `len() == 2` を返すことを assert
  - [x] `par_stage_merge_unordered`: `Merge.any` が `VMValue::List` かつ `len() == 2` を返すことを assert
- [x] `v51100_tests::cargo_toml_version_is_51_1_0` を削除（他テストは保持）

## T7 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"51.2.0"`
- [x] `cargo test` 3117 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.2.0 エントリ追加
- [x] `versions/current.md` を v51.2.0（3117 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.2.0 実績欄を更新（Stream<T> スコープ外も明記）
- [x] tasks.md を COMPLETE に更新（T0〜T7 全 `[x]`）
