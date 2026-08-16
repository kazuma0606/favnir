# v72.0.0 タスクリスト — Type System 2.0 宣言 ★クリーンアップ

Date: 2026-08-11
Status: 完了

---

## T0: 事前確認

- [x]`fav/Cargo.toml` のバージョンが `71.9.0` であることを確認
- [x]`cargo test` が 3608 tests pass（0 failures）であることを確認
- [x]`driver.rs` に `v719000_tests` モジュールが存在することを確認
- [x]`driver.rs` に `v72000_tests` が未存在であることを確認
- [x]`MILESTONE.md` に「Type System 2.0」がまだ含まれていないことを確認
- [x]`README.md` に「Type System 2.0」または「v72.0」がまだ含まれていないことを確認

---

## T1: `MILESTONE.md` 追記

- [x]v72.0.0 エントリを先頭に追加した（宣言文 + v71.1〜v71.9 達成内容）
- [x]`Type System 2.0` という文字列が含まれていることを確認

---

## T2: `README.md` 追記

- [x]`## v72.0 — Type System 2.0 宣言（2026-08-11）` セクションを `## v71.0` の直前に追加した
- [x]`Type System 2.0` または `v72.0` という文字列が含まれていることを確認

---

## T3: `CHANGELOG.md` に v72.0.0 エントリ追加

- [x]`## [v72.0.0]` エントリを先頭に追加した

---

## T4: `v72000_tests` 追加（`driver.rs`）

- [x]`v719000_tests` モジュールの直後に `v72000_tests` モジュールを追加した
- [x]`#[cfg(test)]` のみ（`use` 不要 — `include_str!` のみ使用）
- [x]`cargo_toml_version_is_72_0_0` テストを実装した（`version = "72.0.0"` を assert）
- [x]`changelog_has_v72_0_0` テストを実装した（`[v72.0.0]` を assert）
- [x]`milestone_has_type_system_2` テストを実装した（`Type System 2.0` を assert）
- [x]`readme_mentions_type_system_2` テストを実装した（`Type System 2.0` or `v72.0` を assert）
- [x]`cargo build` でエラーがないことを確認

---

## T5: Cargo.toml バージョン更新 + driver.rs version アサーション更新

- [x]`fav/Cargo.toml` の `version = "71.9.0"` → `version = "72.0.0"` に変更した
- [x]`driver.rs` 内の `"71.9.0"` バージョンアサーション文字列を `"72.0.0"` に replace_all した

---

## T6: `cargo clean`

- [x]`cargo clean` を実施した（ビルドアーティファクト削除）
- [x]`fav/tmp/hello.fav` が存在することを確認（cargo clean で消える可能性）
  - 内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
  - 消えていた場合は復元する

---

## T7: 部分テスト確認

> **前提**: T6（`cargo clean`）完了後に実施。`hello.fav` が消えていた場合は T6 で復元済みであることを確認してから実行する。

- [x]`cargo test v72000` で 4 件 pass することを確認

---

## T8: 全体テスト確認

- [x]`cargo test` 全体で 3612 tests pass（0 failures）であることを確認

---

## T9: versions/current.md 更新

- [x]「進行中バージョン」を `v72.0.0`（Type System 2.0 宣言）に更新した
- [x]「次に切る版」を `v72.1.0` に更新した

---

## T10: 最終確認

- [x]`cargo test v72000` で 4 件 pass することを確認
- [x]`cargo test` 全体で 3612 tests pass（0 failures）であることを確認
- [x]`fav/Cargo.toml` のバージョンが `72.0.0` であることを確認
- [x]`MILESTONE.md` に「Type System 2.0」が含まれていることを確認
- [x]`README.md` に「Type System 2.0」または「v72.0」が含まれていることを確認
- [x]`versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- `site/` MDX 更新: 別タスク（v72.1.0 以降）
- v72.1.0 以降の機能実装: 次スプリント
- `roadmap-v72.1-v73.0.md` 作成: v72.0.0 完了後に別途実施

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [MED] | v71000_tests 内の関数名 `cargo_toml_version_is_71_2_0` とエラーメッセージ `"should declare version 71.4.0"` が不整合 | エラーメッセージを `"72.0.0"` に修正（replace_all） |
| [MED] | 旧バージョンモジュール 9 箇所（行 53057 等）の assert エラーメッセージが `"71.4.0"` のまま | `"Cargo.toml version should be 71.4.0"` → `"72.0.0"` に replace_all で修正 |
| [LOW] | v70000_tests が `"72.0.0"` を assert（replace_all の構造的問題） | 仕様通り（現行バージョン追従）。次回以降の carry-over として記録 |
| [LOW] | tasks.md コードレビュー指摘対応テーブルが未記入 | 本テーブルに記録 |

---

## 完了チェックリスト

- [x]全タスク（T0〜T10）が完了している
- [x]`cargo_toml_version_is_72_0_0` が pass
- [x]`changelog_has_v72_0_0` が pass
- [x]`milestone_has_type_system_2` が pass
- [x]`readme_mentions_type_system_2` が pass
- [x]テスト総数: 3612（+4）
