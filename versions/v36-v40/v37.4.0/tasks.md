# v37.4.0 タスクリスト — `fan_out` / `fan_in` リスト演算子

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.4.0（「`fan_out` / `fan_in`」）に沿ったバージョン。
> スコープ注記: `fan_out ... | ...` キーワード構文は v37.5.0 以降。今バージョンは `List.fan_out` / `List.fan_in` 関数として実装。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2715（v37.3.0 完了時点の実績値））し、実測値をここに記録: 2715
- [x] Cargo.toml バージョンが `37.3.0` であることを確認
- [x] `v37300_tests::cargo_toml_version_is_37_3_0` がライブアサーション（`assert!(cargo.contains("37.3.0"), ...)`）であることを確認し、行番号を記録: 43292
- [x] `driver.rs` に `v37400_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.4.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37300_tests` の閉じ `}` の行番号を確認し、ここに記録: 43315
- [x] `versions/current.md` の最新安定版が `v37.3.0`・次バージョンが `v37.4.0` であることを確認
- [x] `fav/src/backend/vm.rs` の `"List.join_on"` ケース終端行番号を確認し、記録: 3324
- [x] `fav/src/middle/checker.rs` の `("List", "join_on")` ケース行番号を確認し、記録: 5730
- [x] vm.rs の `"List.fan_out"` が存在しないことを確認（今回新規追加）
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.4.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.4.0] エントリを追加

- [x] `## [v37.3.0]` の `---` セパレータ直後に `## [v37.4.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: `vm.rs` — `"List.fan_out"` / `"List.fan_in"` ビルトイン追加

- [x] T0 で確認した `"List.join_on"` 終端行番号付近を Read で確認
- [x] `"List.join_on"` ケースの閉じ `}` の直後に `"List.fan_out"` ケースを追加（spec.md §1）
  - [x] 2 引数ガード (`args.len() != 2`)
  - [x] `VMValue::Int(n)` でチャンク数取得・`n <= 0` チェック
  - [x] `into_iter().collect()` → `chunks(chunk_size)` → `FavList::new`
- [x] `"List.fan_out"` の直後に `"List.fan_in"` ケースを追加（spec.md §1）
  - [x] 1 引数ガード (`args.len() != 1`)
  - [x] 外側 `VMValue::List(outer)` マッチ → 内側ループで各要素を `VMValue::List(inner)` にマッチ
  - [x] 非 List 内側要素に対して `vmvalue_type_name` エラー
- [x] `cargo build` でコンパイルエラーがないことを確認

## T3: `checker.rs` — 戻り型定義追加

- [x] T0 で確認した `("List", "join_on")` 行番号付近を Read で確認
- [x] `("List", "join_on")` ケースの直後に `("List", "fan_out")` ケースを追加（spec.md §2）
  - [x] `expect_list_arg(&arg_tys, 0, span)` で elem 取得
  - [x] `Some(Type::List(Box::new(Type::List(Box::new(elem)))))` を返す
- [x] `("List", "fan_out")` の直後に `("List", "fan_in")` ケースを追加（spec.md §2）
  - [x] `Some(Type::List(Box::new(Type::Unknown)))` を返す
- [x] `cargo build` でコンパイルエラーがないことを確認

## T4: driver.rs — `v37300_tests::cargo_toml_version_is_37_3_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.4.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_3_0` はスタブ化しない（CHANGELOG に `[v37.3.0]` エントリが残るため）

## T5: driver.rs — `v37400_tests` モジュールを新規追加

- [x] `v37300_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v37300_tests` の閉じ `}` の後に `v37400_tests` モジュールを追加（spec.md §3）
  - [x] imports: `use super::{build_artifact, exec_artifact_main}` / `use crate::frontend::parser::Parser` / `use crate::value::Value`
  - [x] ローカル `run()` ヘルパー定義
  - [x] `cargo_toml_version_is_37_4_0`（`include_str!("../Cargo.toml")`）
  - [x] `changelog_has_v37_4_0`（`include_str!("../../CHANGELOG.md")`）
  - [x] `list_fan_out_basic`（`List.fan_out(4 要素リスト, 2)` → length 2）
  - [x] `list_fan_in_basic`（`List.fan_in(fan_out 結果)` → length 4）
  - [x] 各テストで `public fn main()` を使用（`fn main()` では `exec_artifact_main` が見つからない）
  - [x] リストは `List.singleton(x)` + `List.push(list, x)` で構築（`[x,y,z]` リテラルなし）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.4.0` に更新

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥ 2719 passed; 0 failed — 実測: 2719 passed
- [x] `v37400_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_4_0` が pass
- [x] `changelog_has_v37_4_0` が pass
- [x] `list_fan_out_basic` が pass
- [x] `list_fan_in_basic` が pass

## T8: ドキュメント更新

- [x] `versions/v36-v40/v37.4.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.4.0（最新安定版）・v37.5.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.4.0 を完了済みにマーク（✅）かつ完了条件をスコープ縮小後に更新
  - 「`fan_out` キーワード構文は v37.5.0 以降に持ち越し」と明記
  - ロードマップの「Rust テスト 2 件」を「Rust テスト 4 件（meta 2 件 + 機能 2 件）」に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.4.0` | `cargo_toml_version_is_37_4_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.4.0]` が含まれる | `changelog_has_v37_4_0` テスト |
| 3 | `List.fan_out` が正しくチャンク分割を実行する | `list_fan_out_basic` テスト |
| 4 | `List.fan_in` が正しくフラット化を実行する | `list_fan_in_basic` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2719） | 実測: 2719 passed, 0 failed ✅ |
