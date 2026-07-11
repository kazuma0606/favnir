# v38.9.0 タスクリスト — v39.0 前調整・安定化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.9.0（「v39.0 前調整・安定化」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2777（v38.8.0 完了時点の実績値））し、実測値をここに記録: 2777
- [x] Cargo.toml バージョンが `38.8.0` であることを確認
- [x] `v38800_tests::cargo_toml_version_is_38_8_0` がライブアサーション（`assert!(cargo.contains("38.8.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 43950
- [x] `v38800_tests` の他テスト（`changelog_has_v38_8_0` / `ai_cookbook_files_exist`）はバージョン変更後も pass することを確認（CHANGELOG に v38.8.0 エントリが残るため pass / MDX ファイルは削除しない限り pass）
- [x] `driver.rs` に `v38900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38800_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 43968
- [x] `CHANGELOG.md` に `[v38.9.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `site/content/docs/ai-overview.mdx` が存在しないことを確認（今回作成）
- [x] `fav/src/suggest.rs` に `llm_suggest` 関数が含まれることを Grep で確認し、行番号を記録: 38
- [x] `versions/current.md` の最新安定版が `v38.8.0`・次バージョンが `v38.9.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.9.0 が未完了（✅ なし）であることを確認（T7 で更新）

## T1: CHANGELOG.md に [v38.9.0] エントリを追加

- [x] `## [v38.8.0]` の直前に `## [v38.9.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `site/content/docs/ai-overview.mdx` 新規作成

- [x] frontmatter（title / description）が正しく設定されていることを確認
- [x] `fav suggest` のコード例コードブロックが含まれていることを確認
- [x] `fav generate --from sql` の説明が含まれていることを確認
- [x] `llm.stream` / `llm.embed` の説明が含まれていることを確認
- [x] 環境変数まとめテーブルが含まれていることを確認
- [x] `ai_overview_doc_exists` テストが検証するキーワード `"fav suggest"` が含まれていることを確認

## T3: `driver.rs` — `v38800_tests::cargo_toml_version_is_38_8_0` をスタブ化

- [x] Grep で `cargo_toml_version_is_38_8_0` の行番号を確認（43948〜43950）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.9.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_8_0` / `ai_cookbook_files_exist` テストはスタブ化しない

## T4: `driver.rs` — `v38900_tests` モジュールを新規追加（T1〜T2 完了後に実施）

- [x] T1（CHANGELOG）・T2（MDX ファイル作成）が完了していることを確認してから着手
- [x] `v38800_tests` の閉じ `}` の行番号（T0 で記録済み: 43968）を Read で特定してから Edit を実行
- [x] `v38800_tests` の閉じ `}` の後に `v38900_tests` モジュールを追加（4 テスト）
  - [x] `cargo_toml_version_is_38_9_0`
  - [x] `changelog_has_v38_9_0`
  - [x] `ai_overview_doc_exists`（`ai-overview.mdx` が存在し `fav suggest` を含む）
  - [x] `suggest_rs_has_llm_suggest`（`suggest.rs` に `llm_suggest` が含まれる）
- [x] `include_str!` パスが正しい形式であることを確認
  - `"../../site/content/docs/ai-overview.mdx"` — docs MDX
  - `"suggest.rs"` — 同ディレクトリ

## T5: Cargo.toml バージョン更新（T1〜T4 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.9.0` に更新

## T6: テスト実行

- [x] T5（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2781 passed; 0 failed — 実測: 2781 passed, 0 failed
- [x] `v38900_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_38_9_0` が pass
- [x] `changelog_has_v38_9_0` が pass
- [x] `ai_overview_doc_exists` が pass
- [x] `suggest_rs_has_llm_suggest` が pass

## T7: ドキュメント更新（T6 完了後）

- [x] `versions/v36-v40/v38.9.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.9.0（最新安定版）・v39.0.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.9.0 を完了済みにマーク（✅）し、テスト件数を 4 件・成果物（`ai-overview.mdx` 新規作成）を追記
- [x] roadmap の v38.9.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ✅ v38.9.0 — v39.0 前調整・安定化
- [x] roadmap の v38.9.0 行を Read で確認し「4 件」が含まれることをここに記録: テスト件数 4 件確認: Rust テスト 4 件（2781 tests passed, 0 failed）
- [x] `versions/current.md` を Read で確認し「v38.9.0」が最新安定版として含まれることをここに記録: 確認: **v38.9.0** — v39.0 前調整・安定化（2026-07-10）
- [x] roadmap の v39.0.0 テスト数目標が `2785+` に更新済みであることを確認（v38.9.0 実績ベース、「4800+」から修正済み）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `38.9.0` | `cargo_toml_version_is_38_9_0` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v38.9.0]` が含まれる | `changelog_has_v38_9_0` テスト ✅ |
| 3 | `site/content/docs/ai-overview.mdx` が作成され `fav suggest` を含む | `ai_overview_doc_exists` テスト ✅ |
| 4 | `suggest.rs` に `llm_suggest` が含まれる | `suggest_rs_has_llm_suggest` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2781） | 実測: 2781 passed, 0 failed ✅ |
| 6 | `roadmap-v38.1-v39.0.md` の v38.9.0 が ✅ かつテスト件数が 4 件 | 確認済み ✅ |
| 7 | `versions/current.md` が v38.9.0（最新安定版）に更新されている | 確認済み ✅ |
