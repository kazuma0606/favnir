# Tasks: v52.7.0 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

Status: COMPLETE
Date: 2026-07-21

---

## T0 — 事前確認

- [x] `cargo test` 3149 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `otel.rs` に `attrs` フィールドが**存在しない**ことを確認（新規追加対象）:
  - [x] `rg -n "pub attrs" fav/src/otel.rs` → 0 件
- [x] `otel.rs` に `otel_add_attr` が**存在しない**ことを確認:
  - [x] `rg -n "otel_add_attr" fav/src/otel.rs` → 0 件
- [x] `vm.rs` に `OTEL_PREV_STAGE` が**存在しない**ことを確認:
  - [x] `rg -n "OTEL_PREV_STAGE" fav/src/backend/vm.rs` → 0 件
- [x] `vm.rs` に `reset_stage_lineage` が**存在しない**ことを確認:
  - [x] `rg -n "reset_stage_lineage" fav/src/backend/vm.rs` → 0 件
- [x] `otel.rs` の `OtelSpan` 構造体の行番号を確認（`pub status:` の直後が挿入位置）:
  - [x] `rg -n "pub status" fav/src/otel.rs` → 行番号を特定
- [x] `otel.rs` の `otel_span_start` 初期化ブロックの行番号を確認（`attrs:` 追加位置）:
  - [x] `rg -n "status.*OtelStatus" fav/src/otel.rs` → OtelSpan 初期化内 status 行を特定
- [x] `otel.rs` の `otel_span_end` と `otel_collected_spans` の間の行番号を確認（追加関数の挿入位置）:
  - [x] `rg -n "fn otel_collected_spans" fav/src/otel.rs` → 行番号を特定
- [x] `vm.rs` の `AUDIT_LOG_PATH` thread-local ブロックの末尾を確認（`OTEL_PREV_STAGE` 挿入位置）:
  - [x] `rg -n "AUDIT_LOG_PATH" fav/src/backend/vm.rs` → ブロック末尾行を特定
- [x] `vm.rs` の `append_audit_event` 関数末尾を確認（`reset_stage_lineage` 挿入位置）:
  - [x] `rg -n "append_audit_event" fav/src/backend/vm.rs` → 関数末尾行を特定
- [x] `vm.rs` の `SeqStageEnter` 内 `otel_span_start` 呼び出し行を確認（lineage フック挿入位置）:
  - [x] `rg -n "otel_span_start" fav/src/backend/vm.rs` → 行番号を特定
- [x] `vm.rs` の `AssertSchema` opcode 内 `vm.stack.push(NanVal::from_vmvalue(result))` 行を確認:
  - [x] `rg -n "from_vmvalue.result" fav/src/backend/vm.rs` → 行番号を特定
- [x] `driver.rs` の `otel_init` 呼び出し行を確認（`reset_stage_lineage` 挿入位置）:
  - [x] `rg -n "otel_init" fav/src/driver.rs` → 行番号を特定
- [x] `vm.rs` の `SeqStageCheck` ハンドラ内に `otel_span_end` 呼び出しがあることを確認（`otel_patch_attr_on_last` の安全前提）:
  - [x] `rg -n "otel_span_end" fav/src/backend/vm.rs` → SeqStageCheck 行を確認
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("otel.rs")` → `fav/src/otel.rs` ✓
- [x] `v52600_tests` に version テストなし → 削除対象なし（確認済み）

## T1 — `otel.rs` 更新

- [x] `OtelSpan` 構造体に `pub attrs: Vec<(String, String)>` フィールド追加（`pub status:` の直後）
- [x] `otel_span_start` 内の `OtelSpan { ... }` 初期化に `attrs: Vec::new()` 追加
- [x] `otel_add_attr(key: &str, val: &str)` 関数を追加（`otel_span_end` と `otel_collected_spans` の間）:
  - [x] PARENT_STACK の先頭 span_id を取得
  - [x] PENDING_SPANS から該当 span を取得し `attrs.push((key, val))` する
- [x] `otel_patch_attr_on_last(key: &str, val: &str)` 関数を追加（`otel_add_attr` の直後）:
  - [x] OTEL_SPANS の最後エントリを取得し `attrs.push((key, val))` する
- [x] `build_otlp_json` 更新:
  - [x] `extra_attrs` / `extra_part` を生成する変数ブロックを追加
  - [x] `let attrs = format!(...)` の末尾 `]` を `{}]` に変更し `extra_part` を引数に追加
  - [x] `"schema.name"` / `"schema.fields"` / `"lineage.upstream"` / `"lineage.downstream"` が JSON attrs キーとして出現することを確認
- [x] `otel_export_stdout` 更新:
  - [x] `eprintln!("[OTEL] span ...")` の直後に `for (k, v) in &span.attrs { eprintln!(...); }` を追加
- [x] `cargo build` → コンパイルエラーなし確認

## T2 — `vm.rs` 更新

- [x] `OTEL_PREV_STAGE` thread-local を追加（`AUDIT_LOG_PATH` ブロックの直後）:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` で保護
  - [x] `std::cell::RefCell<Option<String>>` を使用
  - [x] `const { RefCell::new(None) }` で初期化
- [x] `reset_stage_lineage()` 関数を追加（`append_audit_event` の直後）:
  - [x] `pub` かつ `#[cfg(not(target_arch = "wasm32"))]`
  - [x] `OTEL_PREV_STAGE.with(|p| *p.borrow_mut() = None)` の実装
- [x] `SeqStageEnter` opcode の `if crate::otel::otel_is_enabled() { ... }` 内に lineage フックを追加:
  - [x] `otel_span_start(...)` 呼び出しの直後に追加
  - [x] `OTEL_PREV_STAGE` から prev_name を取得
  - [x] prev_name がある場合: `otel_patch_attr_on_last("lineage.downstream", stage_name)` + `otel_add_attr("lineage.upstream", prev_name)` を呼ぶ
  - [x] `OTEL_PREV_STAGE` を stage_name に更新
  - [x] `vm.current_otel_span_id = Some(span_id)` の前に挿入（順序を守る）
- [x] `AssertSchema` opcode の `vm.stack.push(NanVal::from_vmvalue(result));` 直前に schema フックを追加:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` で保護
  - [x] `crate::otel::otel_is_enabled()` でガード
  - [x] `result` が `VMValue::Variant("ok", _)` の場合のみ実行
  - [x] `vm.type_metas.get(&ty_name)` からフィールド名列挙 → `join(",")`
  - [x] `otel_add_attr("schema.name", &ty_name)` + `otel_add_attr("schema.fields", &fields_str)` を呼ぶ
- [x] `cargo build` → コンパイルエラーなし確認

## T3 — `driver.rs` 更新

- [x] `rg -n "otel_init" fav/src/driver.rs` で `if trace { otel_init(); }` ブロックの位置を確認
- [x] `if trace { crate::otel::otel_init(); }` ブロックの直後に追加:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` で保護
  - [x] `crate::backend::vm::reset_stage_lineage();`
- [x] `v52700_tests` モジュールを `v52600_tests` の直前に追加（2 件）:
  - [x] `otel_span_has_schema_attr`:
    - [x] `include_str!("otel.rs")` に `"schema.name"` が含まれることを assert
    - [x] `include_str!("otel.rs")` に `"schema.fields"` が含まれることを assert
  - [x] `otel_span_has_lineage_attr`:
    - [x] `include_str!("otel.rs")` に `"lineage.upstream"` が含まれることを assert
    - [x] `include_str!("otel.rs")` に `"lineage.downstream"` が含まれることを assert
- [x] `fav/Cargo.toml` version → `"52.7.0"`
- [x] `cargo test` 実行 → 3151 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T4 — 後処理

- [x] `CHANGELOG.md` に v52.7.0 エントリ追加
- [x] `versions/current.md` を v52.7.0（3151 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.7.0 実績欄を更新:
  - [x] ロードマップ推定値 3149 → 実績 3151 に修正（v52.6.0 実績 3149 + 追加 2 件）
  - [x] v52.8.0 の推定値を確認し、現在の推定 3151 を 3153（v52.7.0 実績 3151 + 追加 2 件）に修正する
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
