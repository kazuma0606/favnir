# Plan: v94.6.0 — OSS 整備（SAP コミュニティ向けドキュメント）

## 実装ステップ

### Step 1: `runes/sap-odata/README.md` を新規作成する

以下のセクションを含む README を作成する:
- タイトル + 概要（SAP S/4HANA OData 統合 Rune）
- **Setup** セクション（必須 — テスト要件）
  - `fav.toml` の `[sap]` 設定例
  - 環境変数（`SAP_USER` / `SAP_PASS`）の設定方法
  - SSM SecureString パス（本番環境向け）の案内
- **Usage** セクション
  - `query<BusinessPartner>()` / `batch()` / `infer --from sap` の使い方例
- **License** セクション（MIT）

### Step 2: `CONTRIBUTING.md` に SAP セットアップ手順を追記する

既存の `CONTRIBUTING.md` に `## SAP テスト環境のセットアップ` セクションを追加する。
- SAP Gateway デモシステムへのアクセス方法
- `fav.toml` の `[sap]` 設定
- `SAP_USER` / `SAP_PASS` 環境変数の設定

### Step 3: `.github/ISSUE_TEMPLATE/sap-bug.md` を新規作成する

GitHub Issues テンプレートを作成する:
- YAML front matter（name / about / labels）
- 環境（Favnir バージョン / SAP バージョン）
- 再現手順
- 期待する動作 / 実際の動作
- ログ出力貼り付け欄

### Step 4: `fav/src/driver.rs` に `mod v94600_tests` を追加する

`mod v94500_tests { ... }` の直後に追加。

テスト 2 件:
- `sap_odata_rune_readme_exists`: `std::path::Path::new("../runes/sap-odata/README.md").exists()` で存在確認
- `sap_odata_rune_readme_has_setup`: `std::fs::read_to_string("../runes/sap-odata/README.md")` で
  `"Setup"` または `"setup"` が含まれることを確認

### Step 5: `CHANGELOG.md` に v94.6.0 エントリを追記する

### Step 6: `cargo build` でコンパイル確認

### Step 7: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,154 tests, 0 failures を確認する。

### Step 8: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
