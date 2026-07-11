# v39.7.0 タスクリスト — CI/CD ポリシーゲート

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.7.0（「CI/CD ポリシーゲート」）に沿ったバージョン。
> ロードマップ「Rust テスト 2 件」= meta 2 件（version + changelog）のみ。
> `generate_ci_yaml` の変更は既存 `generate_ci_yaml_has_*` 3 テストの継続 pass で暗黙的に検証。

## T0: 事前確認

- [x]`cargo test` の実測通過数を確認（目安: 2803（v39.6.0 完了時点の実績値））し、実測値をここに記録: 2803
- [x]Cargo.toml バージョンが `39.6.0` であることを確認
- [x]`v39600_tests::cargo_toml_version_is_39_6_0` がライブアサーション（`assert!(cargo.contains("39.6.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 44190
- [x]`cargo_toml_version_is_39_6_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` が付いていることを確認（T3 のスタブ化範囲に含まれる）。NOTE コメントが欠落している場合は実装を中断し報告すること
- [x]`v39600_tests` の `changelog_has_v39_6_0` はバージョン変更後も pass することを確認
- [x]`driver.rs` に `v39700_tests` モジュールが存在しないことを確認（今回新規作成）
- [x]`v39600_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44201
- [x]`CHANGELOG.md` に `[v39.7.0]` エントリが存在しないことを確認（今回新規作成）
- [x]`generate_ci_yaml` の現在の出力に `fav policy check --ci` が含まれないことを確認（今回追加）
- [x]`versions/current.md` の最新安定版が `v39.6.0`・「次に切る版」が `v39.7.0` であることを確認
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.7.0 が未完了（✅ なし）であることを確認（T6 で更新）
- [x]`roadmap-v39.1-v40.0.md` の v39.7.0 テスト件数欄が「Rust テスト 2 件」と記載されていることを確認

## T1: CHANGELOG.md に [v39.7.0] エントリを追加

- [x]`## [v39.6.0]` ヘッダ行の直前に `## [v39.7.0]` エントリを挿入
- [x]日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x]セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x]`### Changed` セクションを使用していることを確認（`### Added` ではない）
- [x]`generate_ci_yaml` + `fav policy check --ci` への言及が記載されていることを確認

## T2: `generate_ci_yaml` に Policy check ステップ追加

- [x]`driver.rs` の `pub fn generate_ci_yaml` 関数（行 15492 付近）を Read で確認
- [x]既存行のインデントパターン（スペース 5 個 + `- name:`）を確認
- [x]`fav test` ステップ行の直後に Policy check ステップを追加:
  ```
           - name: Policy check\n\
             run: fav policy check --ci\n
  ```
- [x]追加後も `fav check` / `fav lint` / `fav test` の 3 行が維持されていることを確認
- [x]インデントが既存行と一致していることを確認

## T3: `driver.rs` — `v39600_tests::cargo_toml_version_is_39_6_0` をスタブ化

- [x]Grep で `cargo_toml_version_is_39_6_0` の行番号を確認（T0 で記録済み）
- [x]NOTE コメントとライブアサーション全体 → `// Stubbed: version bumped to 39.7.0 — assertion intentionally removed` に変更
- [x]**注意:** `changelog_has_v39_6_0` はスタブ化しない
- [x]スタブ形式が前バージョンのスタブと一致していることを確認

## T4: `driver.rs` — `v39700_tests` モジュールを新規追加（T1 完了後に実施）

- [x]T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x]`v39600_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x]`v39600_tests` の閉じ `}` の後に `v39700_tests` モジュールを追加
  - [x]imports 不要（`include_str!` のみ）
  - [x]`cargo_toml_version_is_39_7_0`（NOTE コメント付き）
  - [x]`changelog_has_v39_7_0`
- [x]テスト数が 2 件であることを確認（ロードマップ「Rust テスト 2 件」に一致）

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x]`fav/Cargo.toml` バージョンを `39.7.0` に更新

## T6: テスト実行 + ドキュメント更新

- [x]`cargo test` 全通過 — ≥ 2805 passed; 0 failed — 実測: 2805 passed, 0 failed
- [x]`v39700_tests` の 2 テストがすべて pass
- [x]`cargo_toml_version_is_39_7_0` が pass
- [x]`changelog_has_v39_7_0` が pass
- [x]既存テストの regression なし（`generate_ci_yaml_has_check_step` / `_lint_step` / `_test_step` が pass）
- [x]`versions/v36-v40/v39.7.0/tasks.md` を COMPLETE ステータスに更新（T0〜T6 全チェックボックスを `[x]` に）
- [x]`versions/current.md` を v39.7.0（最新安定版）・v39.8.0（次に切る版）に更新
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.7.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.7.0]` が含まれる | `changelog_has_v39_7_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.7.0` | `cargo_toml_version_is_39_7_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2805） | `cargo test` 実行結果（2803 + 2 = 2805） |
| 4 | `generate_ci_yaml` 出力に `fav policy check --ci` が含まれる | T6 既存テスト pass + cargo コンパイル |
| 5 | 既存 `generate_ci_yaml_has_*` 3 テストが引き続き pass | T6 regression 確認 |
| 6 | `roadmap-v39.1-v40.0.md` の v39.7.0 が ✅ | T6 後に目視確認 |
