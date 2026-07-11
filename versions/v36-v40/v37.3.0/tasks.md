# v37.3.0 タスクリスト — `join` ステージ演算子（関数形式）

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.3.0（「`join` ステージ演算子」）に沿ったバージョン。
> スコープ注記: `join ... on ...` キーワード構文は v37.4.0 以降。今バージョンは `List.join_on` 関数として実装。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2712（v37.2.0 完了時点の実績値））し、実測値をここに記録: 2712
- [x] Cargo.toml バージョンが `37.2.0` であることを確認
- [x] `v37200_tests::cargo_toml_version_is_37_2_0` がライブアサーション（`assert!(cargo.contains("37.2.0"), ...)`）であることを確認し、行番号を記録: 43222
- [x] `driver.rs` に `v37300_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.3.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37200_tests` の閉じ `}` の行番号を確認し、ここに記録: 43277
- [x] `versions/current.md` の最新安定版が `v37.2.0`・次バージョンが `v37.3.0` であることを確認
- [x] `fav/src/backend/vm.rs` の `"List.join"` ケース終端行番号を確認し、記録: 11084
- [x] `fav/src/middle/checker.rs` の `("List", "filter")` ケース行番号を確認し、記録: 5726
- [x] vm.rs で 2 引数 `call_value`（`vec![x, y]`）の使用例を grep 確認し、行番号を記録: 3354, 3373, 3466（List.fold / List.sort_by 等）
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.3.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.3.0] エントリを追加

- [x] `## [v37.2.0]` の `---` セパレータ直後に `## [v37.3.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: `vm.rs` — `"List.join_on"` ビルトイン追加

- [x] T0 で確認した `"List.join"` 終端行番号の付近を Read で確認
- [x] `"List.filter"` ケースの直後（self スコープ内）に `"List.join_on"` ケースを追加（spec.md §1）
  - 注: `"List.join"` の直後（静的関数スコープ）ではなく `"List.filter"` 直後（メソッドスコープ）に配置（self.call_value / self.error 使用のため）
- [x] `cargo build` でコンパイルエラーがないことを確認

## T3: `checker.rs` — `("List", "join_on")` 戻り型定義追加

- [x] T0 で確認した `("List", "filter")` 行番号付近を Read で確認
- [x] `("List", "filter")` ケースの直後に `("List", "join_on")` ケースを追加（spec.md §2）
- [x] `cargo build` でコンパイルエラーがないことを確認

## T4: driver.rs — `v37200_tests::cargo_toml_version_is_37_2_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.3.0` に変更
- [x] **注意:** `changelog_has_v37_2_0` はスタブ化しない（CHANGELOG に `[v37.2.0]` エントリが残るため生きたアサーションのまま）

## T5: driver.rs — `v37300_tests` モジュールを新規追加

- [x] `v37200_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（行番号: 43277）
- [x] `v37200_tests` の閉じ `}` の後に `v37300_tests` モジュールを追加（spec.md §3）
  - [x] `use super::{build_artifact, exec_artifact_main}` / `use crate::frontend::parser::Parser` / `use crate::value::Value`
  - [x] ローカル `run()` ヘルパー定義
  - [x] `cargo_toml_version_is_37_3_0`
  - [x] `changelog_has_v37_3_0`
  - [x] `list_join_on_basic`（`List.join_on` で length 2 を確認）
    - 注: リストは `List.singleton + List.push` で構築（Favnir に `[x,y,z]` リテラル構文なし）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.3.0` に更新

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥ 2715 passed; 0 failed — 実測: 2715 passed
- [x] `v37300_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_37_3_0` が pass
- [x] `changelog_has_v37_3_0` が pass
- [x] `list_join_on_basic` が pass

## T8: ドキュメント更新

- [x] `versions/v36-v40/v37.3.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.3.0（最新安定版）・v37.4.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.3.0 を完了済みにマーク（✅）かつ完了条件をスコープ縮小後に更新（`join ... on ...` 構文は v37.4.0 以降と明記）
- [x] （任意）`site/content/docs/` への追記はスキップ（サイトビルド確認コスト大のため）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.3.0` | `cargo_toml_version_is_37_3_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.3.0]` が含まれる | `changelog_has_v37_3_0` テスト |
| 3 | `List.join_on` が正しく semi-join を実行する | `list_join_on_basic` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2715） | 実測: 2715 passed, 0 failed ✅ |
