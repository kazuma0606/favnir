# v35.4.0 plan — `!Effect` アノテーション廃止 Phase 1

## 実装ステップ

### Step 1: parser.rs — E0374 パースエラー化（既存確認）

`fav/src/frontend/parser.rs` の `parse_effect_ann` で `!` トークンを検出した際に
`[E0374] !Effect annotation syntax was removed in v35.4.0` エラーを返すことを確認する。

既存の実装（v34.8A で追加済み）が正しく動作していることをテストで保証する。

### Step 2: lint.rs — W022 削除（既存確認）

`fav/src/lint.rs` に `check_w022_deprecated_effect_annotation` が
存在しないことを確認する。

v34.5.0 で W022 は no-op 化され、v34.8A で削除されたため、
v35.4.0 での削除済みを正式にテストで保証する。

### Step 3: error_catalog.rs — E0374 登録（既存確認）

`fav/src/error_catalog.rs` に `code: "E0374"` エントリが存在することを確認する。

### Step 4: ctx:AppCtx effect bypass（既存確認）

`ctx: AppCtx` 引数を持つ関数内で `Db.execute(...)` を呼び出しても
E0107（effectful call in pure function）が発生しないことを確認する。

### Step 5: driver.rs — v35400_tests モジュール追加

上記 4 件 + `cargo_toml_version_is_35_4_0` + `changelog_has_v35_4_0` を含む `v35400_tests` モジュールを追加する。

```
v35400_tests
├── cargo_toml_version_is_35_4_0
├── effect_annotation_is_parse_error_e0374
├── ctx_appctx_bypasses_effect_check
├── w022_lint_removed
├── e0374_in_error_catalog
└── changelog_has_v35_4_0
```

### Step 6: Cargo.toml バージョン bump

**前提**: `v35300_ci_tests::cargo_toml_version_is_35_3_0` をスタブ化してから更新する。
現在このテストは生きたアサーション（`assert!(cargo.contains("35.3.0"), ...)`）のため、
スタブ化なしで Cargo.toml を 35.4.0 に変更すると即座に FAIL する。

スタブ化後、`fav/Cargo.toml` を `35.3.0` → `35.4.0` に更新する（テスト実行前に実施）。

### Step 7: CHANGELOG.md 更新

`## [35.4.0]` エントリを追加する。

## ファイル変更サマリ

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追加 | `v35400_tests` モジュール（5 件） |
| `fav/Cargo.toml` | 変更 | `version = "35.4.0"` |
| `CHANGELOG.md` | 追加 | `## [35.4.0]` エントリ |

## 注意事項

- `v35300_ci_tests::cargo_toml_version_is_35_3_0` は Cargo.toml を 35.4.0 に bump した後、スタブ化する（既にスタブ済みの場合はスキップ）
- `parser.rs` / `lint.rs` / `error_catalog.rs` への変更は不要（既存実装の検証のみ）
