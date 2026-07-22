# Tasks: v52.2.0 — `assert_schema` Phase 2（nullable・追加フィールド対応）

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3136 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `middle/ir.rs` の `FieldMeta` に `optional` フィールドが**存在しない**ことを確認（新規追加対象）
- [x] `backend/vm.rs` の `Vm` struct に `strict_schema` が**存在しない**ことを確認（新規追加対象）
- [x] `lint.rs` に `W036` が**存在しない**ことを確認（新規追加対象）
- [x] `v52100_tests` に `cargo_toml_version_is_52_1_0` が**存在しない**ことを確認（削除対象なし）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("middle/ir.rs")` → `fav/src/middle/ir.rs` ✓
  - [x] `include_str!("backend/vm.rs")` → `fav/src/backend/vm.rs` ✓
  - [x] `include_str!("lint.rs")` → `fav/src/lint.rs` ✓

## T1 — `FieldMeta` に `optional` 追加（ir.rs）

- [x] `FieldMeta` 構造体に `#[serde(default)] pub optional: bool,` を `col_index` の後に追加
  - `#[serde(default)]` は JSON serde 用後方互換。TMET バイナリ後方互換は T3 で別途確保。
- [x] `cargo build` でコンパイルエラーを確認し、`FieldMeta` 構築箇所を全部対応:
  - [x] `middle/compiler.rs` の `build_type_meta` 内 → T2 で対応
  - [x] `backend/artifact.rs` の `read_type_meta_section` 内（line 379）→ T3 で対応
  - [x] `backend/artifact.rs` のテスト内 `FieldMeta` リテラル（line 514, 519）→ T3 で対応

## T2 — `build_type_meta` 更新（compiler.rs）

- [x] `build_type_meta` の `FieldMeta` 構築部分を更新
  - [x] `TypeExpr::Optional(inner, _)` → `ty = lower_type_expr(inner).display()`, `optional = true`
  - [x] その他 → `ty = lower_type_expr(&field.ty).display()`, `optional = false`
  - [x] `col_index` ロジックは既存のまま保持

## T3 — `artifact.rs` バイナリ直列化更新

- [x] `write_type_meta_section` のフィールドフラグ書き込みを bit-flag 方式に更新:
  - [x] `flag |= 0x01` for `col_index`（既存）
  - [x] `flag |= 0x02` for `optional`（新規）
  - [x] 1 バイトでフラグを書き込んだ後、`col_index` がある場合のみ u32 を書き込む
- [x] `read_type_meta_section` の読み込みを bit-flag 方式に更新:
  - [x] `col_index = if flag[0] & 0x01 != 0 { ... }`（bit0）
  - [x] `optional  = flag[0] & 0x02 != 0`（bit1）
  - [x] `FieldMeta { name, ty, col_index, optional }` を構築
- [x] テスト内 `FieldMeta` リテラル（line 514, 519）に `optional: false` を追加

## T4 — `Vm` struct に `strict_schema` 追加（vm.rs）

- [x] `Vm` struct に `pub strict_schema: bool,` を追加
- [x] thread-local `STRICT_SCHEMA` + `pub fn set_strict_schema(bool)` を追加
- [x] `new_with_db_path` の初期化に `strict_schema: STRICT_SCHEMA.with(|s| s.get())` を追加

## T5 — VM `AssertSchema` ハンドラ更新（vm.rs）

- [x] missing フィールド処理に `optional` 判定を追加:
  - [x] `field.optional == true` → `continue`（スキップ）
  - [x] `field.optional == false` → 既存エラー処理（変更なし）
- [x] 追加フィールド収集ロジックを追加（mismatch チェック後）
- [x] `strict_schema == true` → `err_vm("E0419: ... unexpected fields: ...")`
- [x] `strict_schema == false` → `vm.emit_log.push(W036 警告)` + `ok_vm(val)` を返す
- [x] Clippy 指摘（`is_some() + unwrap()`）→ `if let Some(msg) = mismatch` に修正

## T6 — `main.rs` に `--strict-schema` フラグ追加

- [x] `--strict-schema` 検出アームを追加、`strict_schema: bool` 変数に記録
- [x] `cmd_run` 呼び出しに `strict_schema` 引数を追加

## T7 — `driver.rs` の `cmd_run` シグネチャ更新 + lint.rs W036 スタブ追加

- [x] `driver.rs` の `cmd_run` に `strict_schema: bool` 引数追加
- [x] `set_strict_schema(strict_schema)` 呼び出しを追加
- [x] self-host の `cmd_run(...)` 呼び出し箇所（line 1564）に `false` を追加
- [x] lint.rs: W036 スタブ関数 + `run_lint` 呼び出し登録
- [x] `cargo clippy -- -D warnings` でスタブ追加後に警告なし確認

## T8 — `driver.rs` にテスト追加 + バージョン更新

- [x] `v52200_tests` モジュールを `v52100_tests` の直前に追加（2 件）:
  - [x] `assert_schema_nullable_field`
  - [x] `assert_schema_extra_field_warn`
- [x] `v52100_tests` に version テストなし → 削除対象なし（確認済み）
- [x] `fav/Cargo.toml` version → `"52.2.0"`
- [x] `cargo test` 実行 → 3138 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T9 — 後処理

- [x] `CHANGELOG.md` に v52.2.0 エントリ追加
- [x] `versions/current.md` を v52.2.0（3138 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.2.0 実績欄を更新 + テスト数を 3138 に訂正
- [x] tasks.md を COMPLETE に更新（T0〜T9 全 `[x]`）
