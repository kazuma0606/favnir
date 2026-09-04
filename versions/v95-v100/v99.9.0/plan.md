# Plan: v99.9.0 — コードフリーズ・最終確認

## 実装順序

### Step 1: 全 SAP ガイドドキュメントのリンク切れ確認（手動）

以下の 3 ファイルが存在し、相互参照リンクが正しいことを確認する:
- `site/content/docs/guides/sap-platform.mdx` → `sap-migration` / `sap-enterprise-checklist` を参照
- `site/content/docs/guides/sap-migration.mdx` → `sap-platform` / `sap-enterprise-checklist` を参照
- `site/content/docs/guides/sap-enterprise-checklist.mdx` → `sap-platform` / `sap-migration` を参照

### Step 2: versions/current.md の「次に切る版」欄を v100.0.0 に更新

「次に切る版」セクションを以下に更新する:

```
**v100.0.0** — Favnir SAP Platform 1.0 宣言（v99.9.0 コードフリーズ完了後に実施）
```

`v100` キーワードが含まれること（Step 5 の `current_md_next_version_is_v100` テストが通るための前提）。

### Step 3: driver.rs に mod v99900_tests を追加

`fav/src/driver.rs` の `mod v99800_tests` 直後に `mod v99900_tests`（2 テスト）を追加する。

- `sap_guide_docs_all_exist`: 3 ガイド MDX ファイルの存在を `expect()` で確認
- `current_md_next_version_is_v100`: `../versions/current.md` に `"v100"` が含まれることを確認

ブロック先頭に `// use super::* は不要（std::fs のみ使用）` コメントを記載する。

### Step 4: cargo test で全 pass 確認

`cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,275 tests, 0 failures を確認する。

### Step 5: CHANGELOG.md に v99.9.0 エントリ追加

### Step 6: versions/current.md の最新安定版を v99.9.0 に更新

`最終更新:` と「最新安定版」を `v99.9.0` に更新する。

## 依存関係

- Step 1（手動確認）→ Step 2 → Step 3
- Step 3 → Step 4
- Step 4 → Step 5, Step 6（並列可）
