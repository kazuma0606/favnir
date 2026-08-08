# v61.9.0 タスクリスト

Status: COMPLETE
Version: 61.9.0
Base tests: 3376
Target tests: 3378

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3376 tests passed, 0 failed を確認
- [x] `v61800_tests` が `driver.rs` に存在することを grep で確認
- [x] v61.1〜v61.4 の既存テスト（`v61100_tests`〜`v61400_tests`）の構文を確認
- [x] 安定化チェックリスト（OR パターン・as-pattern・ガード・record update + bind）の確認

---

## T1: driver.rs — `v61900_tests` 追加

- [x] `v61800_tests` 直前に `v61900_tests` モジュールを挿入
- [x] `pattern_all_forms_coexist` テスト追加
  - `Point` 型を定義し、`score_label`（OR パターン + per-arm ガード）と `origin_check`（as-pattern）の 2 関数が同一プログラムで型チェックを通過することを確認
- [x] `record_update_bind_mixed` テスト追加
  - `Row` 型で `tag` 関数（ガード付き match + record update）が型チェックを通過することを確認
- [x] `cargo test v61900` で 2 件 PASS

---

## T2: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v61900` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3378 tests passed, 0 failed を確認

---

## T3: ドキュメント更新

- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` v61.9.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v61.9.0（3378 tests）に更新、「次」を v62.0.0 に
- [x] `CHANGELOG.md` に v61.9.0 エントリを追加
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3378 passed, 0 failed（ベース 3376 + 2）
- 完了日: 2026-08-01
