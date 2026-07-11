# v35.5.0 plan — `!Effect` 廃止 Phase 2

## 実装ステップ

### Step 1: ast.rs — Effect enum と effects フィールドの削除（既存確認）

`fav/src/ast.rs` から以下が削除済みであることを確認する:
- `pub enum Effect { ... }` — Effect 型定義
- `effects: Vec<Effect>` — FnDef 等の effects フィールド

削除済みであることはコンパイル成功によっても証明される（Effect を参照するコードが残存していればコンパイルエラーになる）。

### Step 2: parser.rs — parse_effects_acc の削除（既存確認）

`fav/src/frontend/parser.rs` から `fn parse_effects_acc` が削除済みであることを確認する。

v35.4.0 で `!Effect` 構文が E0374 エラーになったため、エフェクト列をパースする関数は不要。

### Step 3: checker.rs — effect 宣言の no-op 化（既存確認）

`effect Foo` のような effect 宣言文がパーサーで解析され、チェッカーが何もエラーを返さないことを確認する。

`effect_registry` フィールドは残存するが、`EffectDef` 処理時に登録処理を行わない。

### Step 4: driver.rs — v35500_tests モジュール追加

```
v35500_tests
├── cargo_toml_version_is_35_5_0     (stub: 35.6.0 で bump)
├── effect_enum_removed_from_ast
├── effects_field_removed_from_fn_def
├── parse_effects_acc_removed_from_parser
├── effect_def_no_longer_registers_in_checker
└── changelog_has_v35_5_0
```

### Step 5: Cargo.toml バージョン bump

**前提**: `v35400_tests::cargo_toml_version_is_35_4_0` は現在 **生きたアサーション**
（`assert!(cargo.contains("35.4.0"), ...)`）のため、Cargo.toml を 35.5.0 に bump する前に
スタブ化する。

スタブ化後、`fav/Cargo.toml` を `35.4.0` → `35.5.0` に更新する。

### Step 6: CHANGELOG.md 更新

`## [35.5.0]` エントリを追加する（既存の場合は確認のみ）。

## ファイル変更サマリ

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追加 | `v35500_tests` モジュール（6 件）、`v35400_tests::cargo_toml_version_is_35_4_0` スタブ化 |
| `fav/Cargo.toml` | 変更 | `version = "35.5.0"` |
| `CHANGELOG.md` | 追加/確認 | `## [35.5.0]` エントリ |

## 注意事項

- `ast.rs` / `parser.rs` への変更は不要（スプリント中に削除済み — 本バージョンでテストにより確認）
- `checker.rs` への変更も不要だが、行 8232 のコメント `// v35.5.0: effect_registry removed; ...` が実態と乖離している
  - `effect_registry` フィールドは実際には残存しているため、コメントを「effect declarations are no-ops — registration stubbed」に修正することを推奨（LOW 優先）
- `v35400_tests::cargo_toml_version_is_35_4_0` のスタブ化を Cargo.toml bump **前**に必ず実施する
- v35500_tests の `cargo_toml_version_is_35_5_0` はスタブとして追加する（v35.6.0 で bump）
