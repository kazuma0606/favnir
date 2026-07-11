# v39.9.0 タスクリスト — v40.0 前調整・安定化 + 全スプリント振り返り

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.9.0（「v40.0 前調整・安定化 + 全スプリント振り返り」）に沿ったバージョン。
> ロードマップにテスト数指定なし。標準パターン（meta 2 件: version + changelog）を採用。

## T0: 事前確認

- [x]`cargo test` の実測通過数を確認（目安: 2808（v39.8.0 完了時点の実績値））し、実測値をここに記録: 2808
- [x]Cargo.toml バージョンが `39.8.0` であることを確認
- [x]`v39800_tests::cargo_toml_version_is_39_8_0` がライブアサーション（`assert!(cargo.contains("39.8.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 44226
- [x]`cargo_toml_version_is_39_8_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` が付いていることを確認（T3 のスタブ化範囲に含まれる）。NOTE コメントが欠落している場合は実装を中断し報告すること
- [x]`v39800_tests` の `changelog_has_v39_8_0` および `site_has_governance_docs` はバージョン変更後も pass することを確認
- [x]`driver.rs` に `v39900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x]`v39800_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44249
- [x]`CHANGELOG.md` に `[v39.9.0]` エントリが存在しないことを確認（今回新規作成）
- [x]`site/content/docs/enterprise-governance.mdx` が存在しないことを確認（今回新規作成）
- [x]`MILESTONE.md` に Enterprise Governance エントリが存在しないことを確認（v40.0.0 スコープのため本バージョンでは更新しない）
- [x]`versions/current.md` の最新安定版が `v39.8.0`・「次に切る版」が `v39.9.0` であることを確認
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.9.0 が未完了（✅ なし）であることを確認（T6 で更新）

## T1: CHANGELOG.md に [v39.9.0] エントリを追加

- [x]`## [v39.8.0]` ヘッダ行の直前に `## [v39.9.0]` エントリを挿入
- [x]日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x]セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x]`### Added` セクションを使用していることを確認
- [x]`enterprise-governance.mdx` への言及が記載されていることを確認

## T2: `site/content/docs/enterprise-governance.mdx` を新規作成

- [x]frontmatter に `title: "Enterprise Governance — v39 スプリント完了"` と `description` を含む
- [x]v39 スプリント達成機能一覧テーブル（v39.1〜v39.8、8 行）を含む（spec.md §1 の表を参照）
- [x]v40.0 宣言文（暫定）をロードマップから引用して含む
- [x]docs/governance/ 3 件（rbac / audit-log / policy）への参照を含む
- [x]cookbook 3 件（multi-tenant-etl / secret-manager-vault / ci-policy-gate）への参照を含む

## T3: `driver.rs` — `v39800_tests::cargo_toml_version_is_39_8_0` をスタブ化

- [x]Grep で `cargo_toml_version_is_39_8_0` の行番号を確認（T0 で記録済み）
- [x]NOTE コメントとライブアサーション全体 → `// Stubbed: version bumped to 39.9.0 — assertion intentionally removed` に変更
- [x]**注意:** `changelog_has_v39_8_0` および `site_has_governance_docs` はスタブ化しない
- [x]スタブ形式が前バージョンのスタブと一致していることを確認

## T4: `driver.rs` — `v39900_tests` モジュールを新規追加（T1 完了後に実施）

- [x]T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x]T3（スタブ化）が完了していることを確認してから着手
- [x]`v39800_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x]`v39800_tests` の閉じ `}` の後に `v39900_tests` モジュールを追加
  - [x]imports 不要（`include_str!` のみ）
  - [x]`cargo_toml_version_is_39_9_0`（NOTE コメント付き）
  - [x]`changelog_has_v39_9_0`
- [x]テスト数が 2 件であることを確認

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x]`fav/Cargo.toml` バージョンを `39.9.0` に更新

## T6: テスト実行 + ドキュメント更新

- [x]`cargo test` 全通過 — ≥ 2810 passed; 0 failed — 実測: 2810 passed, 0 failed
- [x]`v39900_tests` の 2 テストがすべて pass
- [x]`cargo_toml_version_is_39_9_0` が pass
- [x]`changelog_has_v39_9_0` が pass
- [x]`enterprise-governance.mdx` が存在することを目視確認
- [x]`MILESTONE.md` が変更されていないことを確認（v40.0.0 スコープのため本バージョンでは更新しない）
- [x]`versions/v36-v40/v39.9.0/tasks.md` を COMPLETE ステータスに更新（T0〜T6 全チェックボックスを `[x]` に）
- [x]`versions/current.md` を v39.9.0（最新安定版）・v40.0.0（次に切る版）に更新
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.9.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.9.0]` が含まれる | `changelog_has_v39_9_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.9.0` | `cargo_toml_version_is_39_9_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2810） | `cargo test` 実行結果（2808 + 2 = 2810） |
| 4 | `enterprise-governance.mdx` が存在する | T6 目視確認 |
| 5 | `roadmap-v39.1-v40.0.md` の v39.9.0 が ✅ | T6 後に目視確認 |
