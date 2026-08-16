# v74.0.0 タスクリスト — Production Proven 宣言 ★クリーンアップ

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.9.0` であることを確認
- [x] `cargo test` が 3665 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v739000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v74000_tests` が未存在であることを確認

---

## T1: `CHANGELOG.md` に v74.0.0 エントリを追加

- [x] `## [v74.0.0]` エントリを先頭に追加した
  - Declared: Production Proven マイルストーン到達
  - Changed: cargo clean / Cargo.toml 更新 / MILESTONE.md 更新 / README.md 更新
  - Tests: 4 件、合計テスト数 3669（+4）

---

## T2: `MILESTONE.md` に「Production Proven」を追記

- [x] `MILESTONE.md` に `v74.0.0 — Production Proven` セクションを追加した
- [x] 「Production Proven」という文字列が含まれることを確認

---

## T3: `README.md` に v74.0 達成を追記

- [x] `README.md` に「Production Proven」を含む記述を追記した
- [x] v74.0 の達成内容を簡潔に記載した

---

## T4: `v74000_tests` モジュールを `driver.rs` に追加

- [x] `v739000_tests` の直後に `// --- v74.0.0: Production Proven 宣言 ---` セクションを追加した
- [x] `v74000_tests` モジュールを追加した（`use super::*` 不要 — 外部シンボル未使用）
- [x] `cargo_toml_version_is_74_0_0` テストを実装した
- [x] `changelog_has_v74_0_0` テストを実装した
- [x] `milestone_has_production_proven` テストを実装した
- [x] `readme_mentions_production_proven` テストを実装した
- [x] `cargo test v74000` で 4 件 pass することを確認

---

## T5: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.9.0"` → `version = "74.0.0"` に変更した
- [x] `driver.rs` 内の `version = "73.9.0"` 参照を `version = "74.0.0"` に replace_all した
- [x] 残存 `73.9.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.0.0"` を含むことを確認

---

## T5.5: バージョン更新後の部分テスト再確認

- [x] T5 のバージョン更新後も `cargo test v74000` で 4 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3669 tests pass（0 failures）であることを確認

---

## T7: `cargo clean` 実施

- [x] `cd /c/Users/yoshi/favnir/fav && cargo clean` を実施した（32.9 GiB 削除）
- [x] `target/` ディレクトリが削除されたことを確認
- [x] `fav/tmp/hello.fav` を復元した

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v74.0.0)` に更新した
- [x] 「最新安定版」を `v74.0.0 — Production Proven 宣言` に更新した
- [x] 「進行中バージョン」を `v74.0.0` に更新した
- [x] 「次に切る版」を `v74.1.0` に更新した
- [x] マイルストーン一覧の `v74.0 — Production Proven` を「完了」に更新した

---

## T9: 最終確認（T7・T8 完了後）

- [x] `cargo test v74000` で 4 件 pass することを確認
- [x] `cargo test` 全体で 3669 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.0.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.0.0]` エントリが存在することを確認
- [x] `MILESTONE.md` に「Production Proven」が存在することを確認
- [x] `README.md` に「Production Proven」が存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.0.0` であることを確認

---

## スコープ外（明示的除外）

- v74.1.0 以降の新機能実装
- GitHub Releases へのバイナリ公開
- ドキュメントサイトの全面更新
