# v35.6.0 plan — ctx 構文統一 + Production Ready 宣言

## 実装ステップ

### Step 1: 既存実装の確認

以下がスプリント中に実施済みであることを確認する:

| 項目 | 確認方法 |
|---|---|
| `ctx-syntax-guide.mdx` に E0374 と `ctx: AppCtx` が含まれる | grep / ファイル確認 |
| `README.md` に `ctx: AppCtx` または `AppCtx` が含まれる | grep / ファイル確認 |
| `MILESTONE.md` に `Production Ready` が含まれる | grep / ファイル確認 |
| `CHANGELOG.md` に `[v35.6.0]` が含まれる | grep / ファイル確認 |

### Step 2: `v35500_tests::cargo_toml_version_is_35_5_0` の確認

`v35500_tests::cargo_toml_version_is_35_5_0` が既にスタブ化済みであることを確認する
（v35.5.0 実装時に `// stubbed: version bumped to 35.6.0` とスタブ化済み）。

### Step 3: `cargo_toml_version_is_35_6_0` を生きたアサーションに修正

現在の実装は以下の「半スタブ」状態:

```rust
// Stubbed: version bumped to 35.7.0 in v35.0B
let cargo = include_str!("../Cargo.toml");
assert!(cargo.contains("35."), "Cargo.toml must contain a 35.x version");
```

Cargo.toml の bump（Step 4）より**前**に以下の生きたアサーションに修正する:

```rust
let cargo = include_str!("../Cargo.toml");
assert!(cargo.contains("35.6.0"), "Cargo.toml must contain version 35.6.0");
```

（v35.7.0 の Cargo.toml bump 時にスタブ化する）

### Step 4: Cargo.toml バージョン bump

`fav/Cargo.toml` を `35.5.0` → `35.6.0` に更新する。

**前提**: `v35500_tests::cargo_toml_version_is_35_5_0` が既にスタブ済みであること（Step 2 で確認済み）。

### Step 5: テスト実行

```
v35600_tests
├── cargo_toml_version_is_35_6_0     (生きたアサーション — 35.6.0)
├── milestone_has_production_ready
├── ctx_syntax_guide_has_e0374_section
├── readme_ctx_syntax_documented
└── changelog_has_v35_6_0
```

## ファイル変更サマリ

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 修正 | `cargo_toml_version_is_35_6_0` を生きたアサーション（35.6.0）に修正 |
| `fav/Cargo.toml` | 変更 | `version = "35.6.0"` |

## 注意事項

- `v35500_tests::cargo_toml_version_is_35_5_0` は v35.5.0 実装時にスタブ化済み — 追加のスタブ化は不要
- `v35600_tests` の 5 件は全て実装済み — 新規テスト追加は不要
- `site/content/` の MDX 大量更新はスプリント中に実施済み — 本バージョンでの追加作業なし
