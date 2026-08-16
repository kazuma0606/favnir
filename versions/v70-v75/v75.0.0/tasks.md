# v75.0.0 タスクリスト — Favnir 2.0 宣言 ★クリーンアップ

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.9.0` であることを確認
- [x] `cargo test` が 3688 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v749000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v75000_tests` が未存在であることを確認
- [x] `CHANGELOG.md` に `[v74.9.0]` エントリが存在することを確認

---

## T1: MILESTONE.md / README.md 更新（テストより先に実施）

- [x] `MILESTONE.md` に「Favnir 2.0 宣言」セクションを追記した
  - 「Favnir 2.0」という文字列が含まれることを確認
- [x] `README.md` に v75.0 または「Favnir 2.0」の記載を追記した

---

## T2: `v75000_tests` モジュールを追加

- [x] `v749000_tests` の直後に `// --- v75.0.0: Favnir 2.0 宣言 ★クリーンアップ ---` セクションコメントを追加した
- [x] `v75000_tests` モジュールを追加した（`use super::*` 不要）
- [x] `cargo_toml_version_is_75_0_0` テストを実装した
  - `include_str!("../Cargo.toml")` で `version = "75.0.0"` を assert
- [x] `changelog_has_v75_0_0` テストを実装した
  - `include_str!("../../CHANGELOG.md")` で `[v75.0.0]` を assert
- [x] `milestone_has_favnir_2` テストを実装した
  - `include_str!("../../MILESTONE.md")` で `Favnir 2.0` を assert
- [x] `readme_mentions_favnir_2` テストを実装した
  - `include_str!("../../README.md")` で `v75.0` または `Favnir 2.0` を assert
- [x] `cargo build` でエラーがないことを確認

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.9.0"` → `version = "75.0.0"` に変更した
- [x] `driver.rs` 内の `version = \"74.9.0\"` を `version = \"75.0.0\"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.9.0` を `version should be 75.0.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.9.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "75.0.0"` を含むことを確認

---

## T4: CHANGELOG.md 更新（T3.5 より前に実施）

- [x] `## [v75.0.0]` エントリを先頭に追加した
  - Tests: 4 件、合計テスト数 3692（+4）
  - MILESTONE.md / README.md 更新の記載を含む

---

## T3.5: 部分テスト確認（T4 後に実施）

- [x] `cargo test v75000` で 4 件 pass することを確認（`changelog_has_v75_0_0` も通ること）

---

## T5: cargo clean クリーンアップ

- [x] `cargo clean` を実施した（target/ 削除）
- [x] `fav/tmp/hello.fav` の存在を確認した
  - 消えていた場合: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` で復元した

---

## T6: 全体テスト確認（cargo clean 後）

- [x] `cargo test` 全体で 3692 tests pass（0 failures）であることを確認

---

## T7: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v75.0.0)` に更新した
- [x] 「最新安定版」を `v75.0.0` に更新した
- [x] 「進行中バージョン」を更新した（次フェーズ未計画）

---

## T8: ロードマップ更新

- [x] `versions/roadmap/roadmap-v74.1-v75.0.md` の Status を「完了」に更新した

---

## T9: 最終確認（全タスク完了後）

- [x] `cargo test v75000` で 4 件 pass することを確認
- [x] `cargo test` 全体で 3692 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `75.0.0` であることを確認
- [x] `CHANGELOG.md` に `[v75.0.0]` エントリが存在することを確認
- [x] `MILESTONE.md` に「Favnir 2.0」が記載されていることを確認
- [x] `README.md` に「Favnir 2.0」または「v75.0」が記載されていることを確認
- [x] `versions/current.md` が更新されていることを確認

---

## スコープ外（明示的除外）

- 新規機能・新規構造体・新規関数の追加
- CI パイプラインの変更
- 次フェーズ（v75.1.0〜）のロードマップ策定
- `site/` MDX 追加
