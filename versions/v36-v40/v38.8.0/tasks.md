# v38.8.0 タスクリスト — AI 支援 cookbook 3 本

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.8.0（「AI 支援 cookbook 3 本」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2774（v38.7.0 完了時点の実績値））し、実測値をここに記録: 2774
- [x] Cargo.toml バージョンが `38.7.0` であることを確認
- [x] `v38700_tests::cargo_toml_version_is_38_7_0` がライブアサーション（`assert!(cargo.contains("38.7.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 43915
- [x] `v38700_tests` の他テスト（`changelog_has_v38_7_0` / `llm_rune_enhanced_primitives_exist` / `llm_test_fav_has_new_functions`）はバージョン変更後も pass することを確認（CHANGELOG に v38.7.0 エントリが残るため pass / vm.rs 内容は削除しない限り pass）
- [x] `driver.rs` に `v38800_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38700_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 43943
- [x] `CHANGELOG.md` に `[v38.8.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `site/content/cookbook/sql-to-favnir.mdx` が存在しないことを確認（今回作成）
- [x] `site/content/cookbook/rag-pipeline.mdx` が存在しないことを確認（今回作成）
- [x] `site/content/cookbook/llm-streaming.mdx` が存在しないことを確認（今回作成）
- [x] `versions/current.md` の最新安定版が `v38.7.0`・次バージョンが `v38.8.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.8.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `site/content/cookbook/pinecone-rag.mdx` を Read で確認し `rag-pipeline.mdx` との差別化ポイントを記録: rag-pipeline は Favnir テンプレートシステム（`fav new --template`）と `llm.embed` 新 API に焦点、pinecone は Pinecone 専用統合に焦点

## T1: CHANGELOG.md に [v38.8.0] エントリを追加

- [x] `## [v38.7.0]` の直前に `## [v38.8.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `site/content/cookbook/sql-to-favnir.mdx` 新規作成

- [x] frontmatter（title / description）が正しく設定されていることを確認
- [x] `fav generate --from sql` の使用例コードブロックが含まれていることを確認
- [x] `ai_cookbook_files_exist` テストが検証するキーワード `"fav generate"` が含まれていることを確認

## T3: `site/content/cookbook/rag-pipeline.mdx` 新規作成

- [x] frontmatter（title / description）が正しく設定されていることを確認
- [x] `llm.embed` の使用例コードブロックが含まれていることを確認
- [x] `ai_cookbook_files_exist` テストが検証するキーワード `"llm.embed"` が含まれていることを確認
- [x] 既存の `pinecone-rag.mdx` と内容が重複しないことを確認（テンプレート `fav new --template rag-pipeline` に焦点を当てること）

## T4: `site/content/cookbook/llm-streaming.mdx` 新規作成

- [x] frontmatter（title / description）が正しく設定されていることを確認
- [x] `llm.stream` の使用例コードブロックが含まれていることを確認（`BuildPrompt |> AskLlm |> FormatOutput` pipeline 形式）
- [x] `BuildPrompt` ステージが定義されており、pipeline の入力源が明示されていることを確認
- [x] v38.7.0 の collect-all 制約（true SSE streaming は v39.x）が説明されていることを確認
- [x] `ai_cookbook_files_exist` テストが検証するキーワード `"llm.stream"` が含まれていることを確認

## T5: `driver.rs` — `v38700_tests::cargo_toml_version_is_38_7_0` をスタブ化

- [x] Grep で `cargo_toml_version_is_38_7_0` の行番号を確認（43913〜43915）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.8.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_7_0` / `llm_rune_enhanced_primitives_exist` / `llm_test_fav_has_new_functions` テストはスタブ化しない

## T6: `driver.rs` — `v38800_tests` モジュールを新規追加（T1〜T4 完了後に実施）

- [x] T1（CHANGELOG）・T2〜T4（MDX ファイル作成）が完了していることを確認してから着手
- [x] `v38700_tests` の閉じ `}` の行番号（T0 で記録済み: 43943）を Read で特定してから Edit を実行
- [x] `v38700_tests` の閉じ `}` の後に `v38800_tests` モジュールを追加（3 テスト）
  - [x] `cargo_toml_version_is_38_8_0`
  - [x] `changelog_has_v38_8_0`
  - [x] `ai_cookbook_files_exist`（3 MDX ファイルが存在し適切なキーワードを含む）
- [x] `include_str!` パスが `../../site/content/cookbook/<file>.mdx` 形式であることを確認

## T7: Cargo.toml バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.8.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2777 passed; 0 failed — 実測: 2777 passed, 0 failed
- [x] `v38800_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_38_8_0` が pass
- [x] `changelog_has_v38_8_0` が pass
- [x] `ai_cookbook_files_exist` が pass（3 MDX ファイルすべて、キーワードアサーションも通過）

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.8.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.8.0（最新安定版）・v38.9.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.8.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新
- [x] roadmap の v38.8.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ✅ v38.8.0 — AI 支援 cookbook 3 本
- [x] roadmap の v38.8.0 行を Read で確認し「3 件」が含まれることをここに記録: テスト件数 3 件確認: Rust テスト 3 件（2777 tests passed, 0 failed）
- [x] `versions/current.md` を Read で確認し「v38.8.0」が最新安定版として含まれることをここに記録: 確認: **v38.8.0** — AI 支援 cookbook 3 本（2026-07-10）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | 3 つの MDX ファイルが作成され適切なコード例を含む | `ai_cookbook_files_exist` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v38.8.0]` が含まれる | `changelog_has_v38_8_0` テスト ✅ |
| 3 | `Cargo.toml` バージョンが `38.8.0` | `cargo_toml_version_is_38_8_0` テスト ✅ |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2777） | 実測: 2777 passed, 0 failed ✅ |
| 5 | `roadmap-v38.1-v39.0.md` の v38.8.0 が ✅ かつテスト件数が 3 件 | 確認済み ✅ |
| 6 | `versions/current.md` が v38.8.0（最新安定版）に更新されている | 確認済み ✅ |
