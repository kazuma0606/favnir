# v40.0.0 タスクリスト — Enterprise Governance マイルストーン宣言

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v40.0.0（「Enterprise Governance マイルストーン宣言 ★クリーンアップ」）に沿ったバージョン。
> ロードマップのテスト数指定「5000+」は現実離れしているため 2814 件（2810+4）を採用（v36.0〜v39.0 と同規約）。
> GitHub Issues P1/P2 条件は OSS 未公開のため対象外（v36.0〜v39.0 と同規約）。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2810（v39.9.0 完了時点の実績値））し、実測値をここに記録: 2810
- [x] Cargo.toml バージョンが `39.9.0` であることを確認
- [x] `v39900_tests::cargo_toml_version_is_39_9_0` がライブアサーション（`assert!(cargo.contains("39.9.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 44255
- [x] `cargo_toml_version_is_39_9_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` が付いていることを確認（T4 のスタブ化範囲に含まれる）。NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v39900_tests::changelog_has_v39_9_0` がスタブ化対象でないことを確認（T4 注意書き「changelog_has_v39_9_0 はスタブ化しない」と照合）
- [x] `driver.rs` に `v40000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v39900_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44266
- [x] `CHANGELOG.md` に `[v40.0.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `MILESTONE.md` に `Enterprise Governance` エントリが存在しないことを確認（今回新規作成）
- [x] `README.md` に `Enterprise Governance` への言及がないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v39.9.0`・「次に切る版」が `v40.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v39.1-v40.0.md` の v40.0.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v40.0.0] エントリを追加

- [x] `## [v39.9.0]` ヘッダ行の直前に `## [v40.0.0]` エントリを挿入
- [x] 日付を `2026-07-11` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x] `### Added` セクションを使用していることを確認
- [x] Enterprise Governance マイルストーン宣言への言及が記載されていることを確認

## T2: MILESTONE.md に v40.0.0 セクションを追加

- [x] `# Favnir Milestones` ヘッダの直後、`## v39.0.0` セクションの直前に挿入
- [x] spec.md §MILESTONE.md への追加内容 の内容と一致していることを確認
- [x] 宣言文（blockquote `>` 形式）が含まれていることを確認
- [x] 達成コンポーネント表（9 行: v39.1〜v39.9）が含まれていることを確認
- [x] `**宣言日**: 2026-07-11` が含まれていることを確認

## T3: README.md に v40.0 マイルストーン宣言行を追加

- [x] `**v39.0（2026-07-10）で、[Intelligence & Assistance]...` 行を Grep で確認し行番号を記録: 102
- [x] その直後の行に `**v40.0（2026-07-11）で、[Enterprise Governance](./MILESTONE.md) マイルストーンを宣言しました。**` を挿入
- [x] 挿入後の行が v39.0 行の直後であることを Read で確認

## T4: driver.rs — v39900_tests::cargo_toml_version_is_39_9_0 をスタブ化

- [x] T0 で記録した行番号を Read で特定してから Edit を実行
- [x] NOTE コメントとライブアサーション全体 → `// Stubbed: version bumped to 40.0.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v39_9_0` はスタブ化しない
- [x] スタブ形式が前バージョン（`v39800_tests::cargo_toml_version_is_39_8_0` 等）のスタブと一致していることを確認

## T5: driver.rs — v40000_tests モジュールを新規追加（T1 完了後に実施）

- [x] T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x] T4（スタブ化）が完了していることを確認してから着手
- [x] `v39900_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x] `v39900_tests` の閉じ `}` の後に `v40000_tests` モジュールを追加
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_40_0_0`（NOTE コメント付き）
  - [x] `changelog_has_v40_0_0`
  - [x] `milestone_has_enterprise_governance`
  - [x] `readme_mentions_enterprise_governance`
- [x] テスト数が 4 件であることを確認
- [x] spec.md §v40000_tests の設計 のコードブロックと一致していることを確認

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `40.0.0` に更新

## T7: ★クリーンアップ + テスト実行

- [x] `fav/tmp/hello.fav` が存在することを確認（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
- [x] `cargo clean` 実施（削除: 25417 files, 23.4GiB）
- [x] `fav/tmp/hello.fav` が存在することを確認（消失なし）
- [x] `cargo test` 全通過 — ≥ 2814 passed; 0 failed — 実測: 2814 passed, 0 failed
- [x] `v40000_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_40_0_0` が pass
- [x] `changelog_has_v40_0_0` が pass
- [x] `milestone_has_enterprise_governance` が pass
- [x] `readme_mentions_enterprise_governance` が pass

## T8: ドキュメント更新

- [x] `versions/v36-v40/v40.0.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v40.0.0（最新安定版）・v41.0.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v39.1-v40.0.md` の v40.0.0 を完了済みにマーク（✅）
- [x] `versions/roadmap/roadmap-v35.1-v40.0.md` の v40.0 完了確認（必要に応じて ✅ 追加）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Enterprise Governance"` が含まれる | `milestone_has_enterprise_governance` テスト ✅ |
| 2 | `README.md` に `"Enterprise Governance"` が含まれる | `readme_mentions_enterprise_governance` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v40.0.0]` が含まれる | `changelog_has_v40_0_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `40.0.0` | `cargo_toml_version_is_40_0_0` テスト ✅ |
| 5 | `cargo clean` 実施済み | T7 実行記録 ✅ |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2814） | 2814 passed, 0 failed ✅ |
