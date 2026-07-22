# Tasks: v50.1.0 — エラー診断統一 Phase 1（全コード suggestion 補完）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3091 passed, 0 failed を確認（ベース確認）
- [x] `grep -c "suggestion: None" fav/src/error_catalog.rs` が 34 件であることを確認
- [x] `v50000_tests` モジュールが `driver.rs` に存在することを確認（挿入位置の前提）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `error_catalog.rs` suggestion 全件補完

- [x] `grep -n "suggestion: None" fav/src/error_catalog.rs` で全 34 件の行番号を確認
- [x] 各 `suggestion: None` を `suggestion: Some("...")` に置き換え（全 34 件）
  - [x] 行 50 付近（E0107）
  - [x] 行 59 付近（E0108）
  - [x] 行 68 付近（E0109）
  - [x] 行 77 付近（E0110）
  - [x] 行 86 付近（E0112）
  - [x] 行 95 付近（E0136）
  - [x] 行 105 付近（E0020）
  - [x] 行 114 付近（E0021）
  - [x] 行 123 付近（E0022）
  - [x] 行 132 付近（E0023）
  - [x] 行 141 付近（E0024）
  - [x] 行 150 付近（E0025）
  - [x] 行 663〜856 の残り 22 件（E0380〜E0384 / E0420 / E0500〜E0505 / E0580〜E0581 / E0601〜E0605 / E0901〜E0903 系）
- [x] 置き換え後 `grep -c "suggestion: None" fav/src/error_catalog.rs` が 0 件であることを確認

## T2 — `v501000_tests` モジュール追加

- [x] `v501000_tests` モジュールを `v50000_tests` の直前に追加（3 テスト）
- [x] 挿入後 `grep -n v501000_tests fav/src/driver.rs` で存在確認
  - [x] `cargo_toml_version_is_50_1_0`: version = "50.1.0" を assert
  - [x] `error_suggestion_all_covered`: `"suggestion: None"` が含まれないことを assert
  - [x] `error_suggestion_e0018_text`: `"no longer needed"` が含まれることを assert（E0107 に設定）
- [x] `v50000_tests::cargo_toml_version_is_50_0_0` を削除（バージョン進行で不要）

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.1.0"`（先に更新）
- [x] `cargo test` 3093 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.1.0 エントリ追加
- [x] `versions/current.md` を v50.1.0（3093 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.1.0 実績を 3093 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
