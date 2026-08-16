# v76.0.0 タスクリスト — Temporal Data Native 宣言 ★クリーンアップ

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.9.0` であることを確認
- [x] `cargo test` が全 pass（3710 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.0.0 エントリを追加する
- [x] Milestone セクション（Temporal Data Native スプリント完成）と Tests セクション（4 件）を含める

---

## T2: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭に v76.0.0 Temporal Data Native 宣言エントリを追加する
- [x] 宣言文・v75.1〜v75.9 達成内容リストを含める

---

## T3: README.md 更新

- [x] `README.md` の `## v75.0 — Favnir 2.0 宣言` の前に `## v76.0 — Temporal Data Native 宣言` セクションを追加する
- [x] "Temporal" キーワードが含まれていることを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.9.0"` → `"76.0.0"` に変更する
- [x] `driver.rs` 内の `75.9.0` バージョン文字列アサーションを `76.0.0` に一括更新（replace_all）

---

## T5: driver.rs — テストモジュール追加（Cargo.toml 更新後）

- [x] `fav/src/driver.rs` の末尾に `v76000_tests` モジュールを追加する（`use super::*` 不要 — 外部ファイル読み込みのみ）
- [x] `cargo_toml_version_is_76_0_0` テストを実装する（`cargo_toml_version_is_76_0_0` が先に Cargo.toml に `76.0.0` が必要なため T4 より後）
- [x] `changelog_has_v76_0_0` テストを実装する
- [x] `milestone_has_temporal_data_native` テストを実装する
- [x] `readme_mentions_temporal` テストを実装する
- [x] `cargo test v76000` で 4 件が pass することを確認する

---

## T6: versions/current.md 更新（旧 T5 → T6 に繰り下げ）

- [x] 「進行中バージョン」を v76.0.0 に更新する
- [x] 「次に切る版」を v76.1.0 に更新する

---

## T7: ★cargo clean + hello.fav 復元

- [x] `cargo clean` を実行してビルドキャッシュをリセットする
- [x] `fav/tmp/hello.fav` を復元する（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）

---

## T8: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3714 tests）
- [x] `cargo test v76000` で 4 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.0.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.0.0]` であることを確認する
- [x] `MILESTONE.md` の先頭が `v76.0.0` であることを確認する
- [x] `README.md` に "Temporal" が含まれていることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T8）が完了している
- [x] `cargo_toml_version_is_76_0_0` が pass
- [x] `changelog_has_v76_0_0` が pass
- [x] `milestone_has_temporal_data_native` が pass
- [x] `readme_mentions_temporal` が pass
- [x] テスト総数: 3714（+4）
- [x] `cargo clean` 実施済み
