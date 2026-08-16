# v77.0.0 タスクリスト — Data Provenance 1.0 宣言 ★クリーンアップ

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.9.0` であることを確認
- [x] `cargo test` が全 pass（3732 tests）であることを確認（★クリーンアップ前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（cargo clean で消えるため事前記録）

---

## T1: ★クリーンアップ

- [x] `cargo clean` を実行する（ビルド成果物の削除）
- [x] `fav/tmp/hello.fav` を復元する（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）

---

## T2: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭（`## v76.0.0` の直前）に v77.0.0 エントリを挿入する
- [x] 宣言文（> 「データの来歴が型となった...」）を含める
- [x] v76.1〜v76.9 達成内容の箇条書き（9件）を含める

---

## T3: README.md 更新

- [x] `README.md` の `## v76.0` セクションの直前に `## v77.0 — Data Provenance 1.0 宣言（2026-08-15）` セクションを挿入する
- [x] `Provenance`・`ProvenanceTag`・`validate_provenance_contract`・`format_openlineage_json`・`format_lineage_dot` の主要型・関数を言及する

---

## T4: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.0.0 エントリを追加する
- [x] Added セクション（宣言・cargo clean）と Tests セクション（4件）を含める

---

## T5: Cargo.toml バージョン更新（テスト追加より先）

- [x] `fav/Cargo.toml` の `version` を `"76.9.0"` → `"77.0.0"` に変更する
- [x] **注意**: `cargo_toml_version_is_77_0_0` テストが `include_str!("../Cargo.toml")` を参照するため、テストモジュール追加より必ず前に実施する

---

## T6: driver.rs — バージョン文字列一括更新

- [x] `driver.rs` 内の `76.9.0` を `77.0.0` へ一括置換する（`replace_all: true`）

---

## T7: driver.rs — v77000_tests モジュール追加

- [x] `fav/src/driver.rs` の末尾に `v77000_tests` モジュールを追加する
- [x] `use super::*` **不要**（外部ファイル参照のみ）
- [x] `cargo_toml_version_is_77_0_0` テストを実装する（`include_str!("../Cargo.toml")`）
- [x] `changelog_has_v77_0_0` テストを実装する（`include_str!("../../CHANGELOG.md")`）
- [x] `milestone_has_data_provenance` テストを実装する（`include_str!("../../MILESTONE.md")`）
- [x] `readme_mentions_provenance` テストを実装する（`include_str!("../../README.md")`）
- [x] `cargo test v77000` で 4 件が pass することを確認する

---

## T8: versions/current.md 更新

- [x] 「進行中バージョン」を v77.0.0 に更新する
- [x] 「次に切る版」を v77.1.0 に更新する

---

## T9: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3736 tests）
- [x] `cargo test v77000` で 4 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.0.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.0.0]` であることを確認する
- [x] `MILESTONE.md` の先頭が `## v77.0.0` から始まることを確認する
- [x] `README.md` が `Provenance` / `provenance` を含む（Data Provenance 1.0 宣言セクション挿入済み）
- [x] `versions/current.md` の「進行中バージョン」が v77.0.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `cargo_toml_version_is_77_0_0` が pass
- [x] `changelog_has_v77_0_0` が pass
- [x] `milestone_has_data_provenance` が pass
- [x] `readme_mentions_provenance` が pass
- [x] テスト総数: 3736（+4）
- [x] `cargo clean` 実施済み（★クリーンアップ完了）
- [x] `fav/tmp/hello.fav` 復元済み
