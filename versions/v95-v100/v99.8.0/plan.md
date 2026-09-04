# Plan: v99.8.0 — 総合ドキュメント

## 実装順序

### Step 1: site/content/docs/guides/ ディレクトリ確認

`site/content/docs/guides/` が存在するか確認する。存在しなければ作成する。

### Step 2: sap-platform.mdx 作成

`site/content/docs/guides/sap-platform.mdx` を新規作成する。

内容:
- frontmatter（title, description）
- SAP Platform 1.0 概要セクション
- 主要コンポーネント（sap-odata Rune・ctx 統合・Workflow・ガードレール）
- コードサンプル（`bind` 構文使用、`let` 禁止）
- キーワード `SAP Platform` が含まれること

### Step 3: sap-migration.mdx 作成

`site/content/docs/guides/sap-migration.mdx` を新規作成する。

内容:
- frontmatter（title, description）
- v95.0 → v99.x 移行の変更概要セクション
- 移行手順（SapClient・CircuitBreaker・TenantContext・Masked<T>・fav sla-check）
- キーワード `migration` または `移行` が含まれること

### Step 4: sap-enterprise-checklist.mdx 作成

`site/content/docs/guides/sap-enterprise-checklist.mdx` を新規作成する。

内容:
- frontmatter（title, description）
- 本番投入前チェックリスト（認証・SLA・CB・マルチテナント・GDPR・E2E・モニタリング）
- キーワード `checklist` または `チェック` が含まれること

### Step 5: driver.rs に mod v99800_tests を追加

`fav/src/driver.rs` の `mod v99700_tests` 直後に `mod v99800_tests`（2 テスト）を追加する。

- `sap_platform_mdx_exists`: `../site/content/docs/guides/sap-platform.mdx` の存在確認
- `sap_platform_docs_have_keywords`: 3 ファイルのキーワード存在確認（3 アサート）

ブロック先頭に `// use super::* は不要（std::fs のみ使用）` コメントを記載する。

### Step 6: cargo test で全 pass 確認

`cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,273 tests, 0 failures を確認する。

### Step 7: CHANGELOG.md に v99.8.0 エントリ追加

### Step 8: versions/current.md 更新

`最終更新:` を `v99.8.0` に、最新安定版を `v99.8.0 — 総合ドキュメント — 4,273 tests` に更新する。

## 依存関係

- Step 1 → Step 2, 3, 4（並列可）
- Step 2, 3, 4 → Step 5
- Step 5 → Step 6
- Step 6 → Step 7, 8
