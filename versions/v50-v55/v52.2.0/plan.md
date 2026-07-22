# Plan: v52.2.0 — `assert_schema` Phase 2（nullable・追加フィールド対応）

Status: PLANNED
Date: 2026-07-20

---

## 実装順序

### Step 1 — `FieldMeta` に `optional` 追加（ir.rs）

- ファイル: `fav/src/middle/ir.rs`
- `FieldMeta` に `#[serde(default)] pub optional: bool,` を `col_index` の後に追加
  - `#[serde(default)]` は JSON serde 用後方互換。バイナリ直列化（TMET）は Step 3 で別途対応。
- `cargo build` → コンパイルエラーで `FieldMeta { ... }` 構築箇所を洗い出す

### Step 2 — `build_type_meta` 更新（compiler.rs）

- ファイル: `fav/src/middle/compiler.rs`
- `build_type_meta` の `FieldMeta { ... }` 構築部分を更新:
  ```rust
  let (ty_str, optional) = match &field.ty {
      TypeExpr::Optional(inner, _) => (lower_type_expr(inner).display(), true),
      other => (lower_type_expr(other).display(), false),
  };
  FieldMeta { name: field.name.clone(), ty: ty_str, col_index: ..., optional }
  ```
- `col_index` ロジックは既存のまま保持

### Step 3 — `artifact.rs` バイナリ直列化更新

- ファイル: `fav/src/backend/artifact.rs`

**重要**: `write/read_type_meta_section` はカスタムバイナリ形式（serde 非使用）。
`#[serde(default)]` は TMET セクション後方互換に無効。bit-flag 方式で更新する。

**`write_type_meta_section` 変更** (line 345-356):
```rust
let mut flag: u8 = 0;
if field.col_index.is_some() { flag |= 0x01; }
if field.optional             { flag |= 0x02; }
w.write_all(&[flag])?;
if let Some(idx) = field.col_index {
    write_u32(w, idx as u32)?;
}
```

**`read_type_meta_section` 変更** (line 372-383):
```rust
let mut flag = [0u8; 1];
r.read_exact(&mut flag)?;
let col_index = if flag[0] & 0x01 != 0 { Some(read_u32(r)? as usize) } else { None };
let optional  = flag[0] & 0x02 != 0;
fields.push(FieldMeta { name, ty, col_index, optional });
```

旧バイナリはフラグが `0x00` / `0x01` のみ → bit1 は常に 0 → `optional = false` で後方互換。

**artifact.rs テスト内 `FieldMeta` リテラル** (line 514, 519): `optional: false` を追加。

### Step 4 — `Vm` struct に `strict_schema` 追加（vm.rs）

- ファイル: `fav/src/backend/vm.rs`
- `Vm` struct に `pub strict_schema: bool,` を追加
- 唯一の Vm 構築関数 `new_with_db_path`（line ~1673）の初期化に `strict_schema: false` を追加
  - Note: `Vm::new` / `Vm::new_for_test` は存在しない（`new_with_db_path` のみ）
- WASM ビルド非対応フィールドは `#[cfg]` 不要（`strict_schema` は bool のため）

### Step 5 — VM `AssertSchema` ハンドラ更新（vm.rs）

既存の `Opcode::AssertSchema as u8` ハンドラを更新:

1. **missing フィールド処理**（既存ループ内）:
   - `field.optional == true` → `continue`（スキップ、エラーにしない）
   - `field.optional == false` → 既存の `mismatch` セット処理（変更なし）

2. **追加フィールド収集**（mismatch チェック後）:
   ```rust
   let schema_field_names: std::collections::HashSet<&str> =
       meta.fields.iter().map(|f| f.name.as_str()).collect();
   let extra: Vec<String> = map.keys()
       .filter(|k| !schema_field_names.contains(k.as_str()))
       .cloned().collect();
   ```

3. **extra フィールド処理**:
   ```rust
   if !extra.is_empty() {
       if vm.strict_schema {
           result = err_vm(VMValue::Str(format!(
               "E0419: assert_schema — unexpected fields: {}",
               extra.join(", ")
           )));
       } else {
           vm.emit_log.push(NanVal::from_vmvalue(VMValue::Str(format!(
               "W036: assert_schema — unexpected fields (use --strict-schema to error): {}",
               extra.join(", ")
           ))));
       }
   }
   ```

### Step 6 — `main.rs` に `--strict-schema` フラグ追加

- ファイル: `fav/src/main.rs`
- `fav run` コマンドライン解析で `"--strict-schema"` を検出するアームを追加
- `vm.strict_schema = true` をセット（`--verbose` / `--trace` の追加パターンと同様）

### Step 7 — lint.rs に W036 スタブ追加

- ファイル: `fav/src/lint.rs`
- W035 関数（`check_w035_legacy_import_rune`）の後に追加:
  ```rust
  // ── W036: extra_schema_fields (v52.2.0) ────────────────────────────────────────
  // Runtime-only warning: W036 is emitted in the VM AssertSchema handler.
  // This stub reserves the lint entry for future static analysis.
  fn check_w036_extra_schema_fields(_program: &Program, _errors: &mut Vec<LintError>) {}
  ```
- `run_lint` の W035 呼び出しの直後に追加:
  ```rust
  check_w036_extra_schema_fields(program, &mut errors);
  ```
- `cargo clippy -- -D warnings` でスタブ追加後に警告が出ないことを確認
  （`_program` / `_errors` とアンダースコアを付けているため Clippy 警告は出ないはず）

### Step 8 — `driver.rs` にテスト追加 + バージョン更新

- `v52200_tests` モジュールを `v52100_tests` の直前に追加（2 件）
- `fav/Cargo.toml` version → `"52.2.0"`
- `cargo test` → 3138 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

### Step 9 — 後処理

- `CHANGELOG.md` に v52.2.0 エントリ追加
- `versions/current.md` を v52.2.0（3138 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.2.0 実績欄を更新 + テスト数を 3139 → 3138 に訂正
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `FieldMeta.optional` の後方互換は「bit-flag 方式」で確保（`serde(default)` は JSON のみ有効）
- `Vm` に `strict_schema` を追加したら `new_with_db_path` の構造体初期化を必ず更新すること
- W036 スタブは `_` プレフィックスにより Clippy の `dead_code` / `unused_variables` 警告を回避
- `note: String?` 構文（`TypeExpr::Optional`）が対象。`note?: String` はパーサー未対応
