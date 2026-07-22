# Spec: v52.2.0 — `assert_schema` Phase 2（nullable・追加フィールド対応）

Status: PLANNED
Date: 2026-07-20

---

## 目的

v52.1.0 で `assert_schema<T>(value)` の Phase 1（フィールド名・型チェック）を実装した。
Phase 2 では以下を追加する：

1. **nullable フィールド対応**: 型定義で `note: String?`（`TypeExpr::Optional`）としたフィールドを、
   入力マップに存在しなくてもエラーとしない。
   - Favnir パーサーは `field: Type?` 構文（`TypeExpr::Optional`）をサポート。
     ロードマップ記載の `note?: String` 構文はパーサー未対応のため、`note: String?` に統一。
2. **W036 想定外フィールド警告**: 入力マップに型定義にないフィールドがある場合、
   runtime に W036 警告を発行する。`--strict-schema` フラグでエラー化。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/ir.rs` | `FieldMeta` に `#[serde(default)] pub optional: bool` を追加 |
| `fav/src/middle/compiler.rs` | `build_type_meta` で `TypeExpr::Optional` → `optional: true` |
| `fav/src/backend/artifact.rs` | `write/read_type_meta_section` でフィールドフラグバイトの bit1 に `optional` を追加、テスト内 `FieldMeta` リテラルに `optional: false` を追加 |
| `fav/src/backend/vm.rs` | `AssertSchema` ハンドラ更新（optional 許容 + 追加フィールド W036 警告）、`Vm` struct に `pub strict_schema: bool` を追加、`new_with_db_path` 初期化に `strict_schema: false` |
| `fav/src/main.rs` | `--strict-schema` CLI フラグ解析 + `vm.strict_schema = true` セット |
| `fav/src/lint.rs` | W036 スタブ追加（将来の静的解析予約）+ `run_lint` へ登録 |
| `fav/src/driver.rs` | `v52200_tests` モジュール追加（2 件） |
| `fav/Cargo.toml` | version → `"52.2.0"` |
| `CHANGELOG.md` | v52.2.0 エントリ追加 |
| `versions/current.md` | v52.2.0（3138 tests）に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.2.0 実績欄を更新、完了条件のテスト数を 3138 に訂正 |

---

## 詳細仕様

### 1. `FieldMeta.optional: bool`（ir.rs）

```rust
pub struct FieldMeta {
    pub name: String,
    pub ty: String,
    pub col_index: Option<usize>,
    #[serde(default)]
    pub optional: bool,  // v52.2.0: true if field type is Optional (e.g. String?)
}
```

`#[serde(default)]` は JSON serde（artifact.rs の FvcArtifact JSON 出力）の後方互換用。

### 2. `build_type_meta` 更新（compiler.rs）

`field.ty` が `TypeExpr::Optional(inner, _)` の場合 → `optional: true`、`ty` は `inner` を lower。
それ以外 → `optional: false`、`ty` は `field.ty` を lower。

```rust
let (ty_str, optional) = match &field.ty {
    TypeExpr::Optional(inner, _) => (lower_type_expr(inner).display(), true),
    other => (lower_type_expr(other).display(), false),
};
FieldMeta { name: field.name.clone(), ty: ty_str, col_index: ..., optional }
```

### 3. `artifact.rs` バイナリ直列化更新（artifact.rs）

`write/read_type_meta_section` はカスタムバイナリ形式（serde 非使用）のため、
`#[serde(default)]` は TMET セクションの後方互換には無効。

**実装方針（bit-flag 方式）**: 既存のフィールドごとの 1 バイトフラグ（`col_index` 有無）の
bit1（`0x02`）を `optional` に割り当てる。

```
フラグバイト構成（v52.2.0〜）:
  bit0 (0x01): col_index あり
  bit1 (0x02): optional = true
```

旧バイナリはフラグバイトが `0x00` または `0x01` のみなので bit1 は常に 0 →
読み込み時 `optional = false` となり後方互換が保たれる。

**write_type_meta_section 変更**:
```rust
let mut flag: u8 = 0;
if field.col_index.is_some() { flag |= 0x01; }
if field.optional           { flag |= 0x02; }
w.write_all(&[flag])?;
if let Some(idx) = field.col_index {
    write_u32(w, idx as u32)?;
}
```

**read_type_meta_section 変更**:
```rust
let mut flag = [0u8; 1];
r.read_exact(&mut flag)?;
let col_index = if flag[0] & 0x01 != 0 { Some(read_u32(r)? as usize) } else { None };
let optional  = flag[0] & 0x02 != 0;
fields.push(FieldMeta { name, ty, col_index, optional });
```

**artifact.rs テスト内 `FieldMeta` リテラル**: `optional: false` を追加。

### 4. VM 更新（vm.rs）

**`Vm` struct**:
```rust
pub strict_schema: bool,  // v52.2.0
```
`new_with_db_path`（唯一の `Vm` 構築関数）の初期化に `strict_schema: false` を追加。

**`AssertSchema` ハンドラ更新**:
1. missing フィールド処理:
   - `field.optional == true` → スキップ（OK）
   - `field.optional == false` → E0419 エラー（既存動作）
2. 追加フィールド収集:
   - `map.keys()` のうち `meta.fields` に名前が存在しないキーを収集
   - `extra_fields` が空でない場合:
     - `vm.strict_schema == true` → `err_vm(VMValue::Str("E0419: assert_schema — unexpected fields: ..."))`
     - `vm.strict_schema == false` → `vm.emit_log.push(...)` で W036 警告を記録 + `ok_vm(val)`

### 5. `--strict-schema` フラグ（main.rs）

`fav run pipeline.fav --strict-schema` → `vm.strict_schema = true`。
`--verbose` / `--trace` フラグの追加パターンを参照して実装。

### 6. W036 lint スタブ（lint.rs）

W036 は runtime 警告（VM が発行）のため静的解析ではフィールド過不足の追跡が困難。
スタブ関数を `run_lint` に登録するのは将来の静的解析拡張の予約（W033/W034 は未登録コメントのみだが
W036 は将来実装のため関数定義を予約する）。

```rust
// ── W036: extra_schema_fields (v52.2.0) ────────────────────────────────────────
// Runtime-only warning: W036 is emitted in the VM AssertSchema handler.
// This stub reserves the lint entry for future static analysis.
fn check_w036_extra_schema_fields(_program: &Program, _errors: &mut Vec<LintError>) {
    // Future: statically detect assert_schema calls with extra-field-prone types.
}
```

---

## テスト（2 件）

追加先: `driver.rs` の `v52200_tests` モジュール（`v52100_tests` の直前）

### `assert_schema_nullable_field`

```rust
fn assert_schema_nullable_field() {
    let ir = include_str!("middle/ir.rs");
    assert!(ir.contains("optional"), "FieldMeta must have optional field");
    let vm = include_str!("backend/vm.rs");
    assert!(vm.contains("optional"), "VM AssertSchema must handle optional fields");
}
```

### `assert_schema_extra_field_warn`

```rust
fn assert_schema_extra_field_warn() {
    let lint = include_str!("lint.rs");
    assert!(lint.contains("W036"), "lint.rs must document W036");
    let vm = include_str!("backend/vm.rs");
    assert!(vm.contains("strict_schema"), "VM must support strict_schema flag");
}
```

---

## テスト数

- ベース: **3136** tests（v52.1.0 完了時点）
- `v52100_tests` に version テストなし → 削除 0 件
- 追加: `v52200_tests` 2 件
- **合計: 3138 tests**

（ロードマップ記載の「3139」は誤記 → 実装完了時に roadmap の当該行を 3138 に訂正する）

---

## 完了条件

- `cargo test` 3138 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `note: String?` 型フィールドが `TypeMeta` に `optional: true` として記録される
- artifact バイナリ（TMET セクション）の bit-flag 方式で後方互換が保たれる
- `--strict-schema` フラグが main.rs で解析され `vm.strict_schema` に反映される
- W036 が lint.rs に登録されている
