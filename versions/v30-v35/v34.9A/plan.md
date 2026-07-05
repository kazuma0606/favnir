# v34.9A plan

## 実装順序

v34.9A は影響範囲が広い（14 ファイル）。
コンパイルエラーを追いながら段階的に削除する。

### フェーズ A — ast.rs から削除（コンパイルエラーを起点に使う）

1. `ast.rs` から `Effect` enum・`EffectDef`・各構造体の `effects` フィールドを削除
2. `cargo build 2>&1 | grep "^error"` でコンパイルエラー一覧を取得
3. エラーを起点に関連ファイルを修正（エラー数が多い順に処理）

### フェーズ B — parser.rs

- `parse_effects_acc` 関数を削除
- `parse_fn_def_after_ret` / `parse_stage_def` の effects 関連コードを削除
- E0374 エラーチェック（v34.8A で追加）も削除（Effects フィールドなしでは不要）
  → ただし E0374 エントリ自体は error_catalog.rs に残す（エラーコードの欠番は避ける）

### フェーズ C — checker.rs（最多）

```bash
grep -n "\.effects\|Effect::" fav/src/middle/checker.rs
```
で全箇所を確認し、削除 or 空 vec! 相当のコードパスに統一する。

主要パターン:
```rust
// パターン1: 削除（常に false だったチェック）
if fd.effects.contains(&Effect::Http) { ... }    // → 削除（ブロック全体）

// パターン2: 削除（空ループ）
for e in &fd.effects { ... }                      // → 削除

// パターン3: 変更（空 vec を前提に）
let effects: Vec<Effect> = vec![];                // → 削除（宣言ごと）
```

### フェーズ D — lineage.rs

Effect ベースの lineage トラッキング（`collect_postgres_call_kinds` 等）は
Rune 呼び出しベース（関数名ベース）への移行が既に行われている可能性あり。
削除前に `lineage.rs` を通読して影響を確認する。

### フェーズ E — wasm_codegen.rs / wasm_exec.rs

FnDef 構造体リテラルの `effects` フィールドを削除する。
コンパイルエラーを起点にすれば机上確認不要。

### フェーズ F — fmt.rs / emit_python.rs / codegen.rs

Effect を出力するコードを削除。
`fmt.rs` の `fn format_effects` 等を削除。
`emit_python.rs` は Python 出力時に effects を `# @effect:Io` 等と出力している可能性あり → 削除。

### フェーズ G — driver.rs

Effect 関連テストを多数スタブ化する。
`grep -n "Effect\|effects" driver.rs | grep -v "//"` で抽出してから処理。

## v35500_tests の内容（5 件）

1. `cargo_toml_version_is_35_5_0` — バージョン確認
2. `ast_has_no_effect_enum` — `ast.rs` に `Effect` が含まれないこと
3. `parser_has_no_parse_effects_fn` — `parser.rs` に `parse_effects_acc` が含まれないこと
4. `lint_has_no_w022` — `lint.rs` に `W022` が含まれないこと
5. `no_effects_field_in_structs` — `ast.rs` に `effects:` フィールドが含まれないこと

## 作業量見積もり

| ファイル | 削除行数（概算） |
|---|---|
| ast.rs | ~60 |
| parser.rs | ~80 |
| checker.rs | ~80 |
| lineage.rs | ~50 |
| fmt.rs | ~35 |
| emit_python.rs | ~25 |
| wasm_codegen.rs | ~15 |
| compiler.rs | ~15 |
| その他 | ~20 |
| **合計** | **~380 行削除** |
