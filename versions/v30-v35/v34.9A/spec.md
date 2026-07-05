# v34.9A spec — `Effect` enum / `effects` フィールドの完全削除

**バージョン**: v35.5.0
**日付**: 2026-07-05
**前提**: v34.8A (v35.4.0) COMPLETE

---

## 目的

v34.8A でパーサーが `!Effect` を拒否するようになったため、
`FnDef.effects` / `StageDef.effects` 等は常に空ベクタになった。
このバージョンでは「使われなくなった死んだコード」を AST から物理削除し、
コードベースを完全にクリーンにする。

---

## 削除対象の全体像

| ファイル | 削除内容 | 行数規模 |
|---|---|---|
| `ast.rs` | `Effect` enum（25 バリアント）+ `EffectDef` 構造体 + 4 構造体の `effects` フィールド | ~60 行 |
| `parser.rs` | `parse_effects_acc` / `parse_effects` 関数、E0374 エラー生成コード | ~80 行 |
| `checker.rs` | effects 関連チェック・W022 参照跡（13+54 箇所） | ~80 行 |
| `lint.rs` | `check_w022` は v34.8A で削除済み。残る Effect import など | ~5 行 |
| `lineage.rs` | `!Effect` 依存の lineage 追跡ロジック | ~50 行 |
| `fmt.rs` | Effect を含む fn シグネチャのフォーマット | ~35 行 |
| `emit_python.rs` | Effect アノテーション出力コード | ~25 行 |
| `wasm_codegen.rs` | `effects: vec![Effect::Io]` 等の構造体リテラル | ~15 行 |
| `codegen.rs` | Effect 参照 2 箇所 | ~5 行 |
| `compiler.rs` | effects フィールドアクセス 9 箇所 | ~15 行 |
| `reachability.rs` | effects 関連 | ~5 行 |
| `error_catalog.rs` | E0370（`!Io not declared`）/ E0371（pure fn calls effectful）— Effect システム前提のエラー | ~15 行 |

---

## 変更方針

### ast.rs

```rust
// 削除: Effect enum 全体
pub enum Effect { ... }           // 削除
impl Effect { is_deprecated }     // 削除

// 削除: EffectDef 構造体
pub struct EffectDef { ... }      // 削除
Item::EffectDef(EffectDef)        // Item enum から削除

// 変更: 各構造体から effects フィールドを削除
pub struct FnDef {
    // effects: Vec<Effect>,      // 削除
    ...
}
// StageDef, SeqDef, StageTypeDef も同様
```

### parser.rs

`parse_effects_acc` と E0374 エラー生成コード（v34.8A で追加）を共に削除。
`parse_fn_def_after_ret` の E0374 チェックは削除し、
`effects` フィールドへの代入コードも除去する。

### wasm_codegen.rs / wasm_exec.rs

```rust
// 変更前
FnDef { name: "...", effects: vec![Effect::Io], ... }
// 変更後
FnDef { name: "...", ... }  // effects フィールド自体が消えるため
```

### lineage.rs

Effect に基づく lineage 分析（`!Postgres(read)` / `!S3(write)` 等の区別）は削除。
lineage は関数名・Rune 呼び出しベースのトラッキングに統一。

### error_catalog.rs

- E0370「`!Io` not declared」— `!Effect` システム前提のエラー → 削除
- E0371「pure fn calls effectful fn」— W021 と紐づく → 削除
  - ただし W021 lint（`pure_fn_calls_effectful`）が E0371 を使っているか要確認

---

## 注意点

- `wasm_codegen.rs` は `FnDef` の `effects` フィールドを埋めて合成 FnDef を作成している。
  フィールド削除後、それらのコードを更新する（フィールドを渡さなくてよくなる）。
- `ast.rs` の `EffectDef`（`effect MyEffect { ... }` 宣言構文）は別概念だが、
  `!Effect` 廃止に伴い `effect` キーワード自体も使われていないため同時削除する。
- driver.rs に `Effect` を直接参照するテストが多数ある。全スタブ化対象。

---

## 完了条件

- `Effect` という識別子が `fav/src/` 全体から消えている（コメント除く）
- `effects:` フィールドが AST 構造体から消えている
- `cargo test` 全件 PASS
- `cargo clippy --locked -- -D warnings` PASS
- `grep -rn "Effect" fav/src/ --include="*.rs" | grep -v "//\|test"` が 0 件
