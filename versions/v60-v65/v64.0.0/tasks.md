# v64.0.0 タスクリスト

Status: COMPLETE
Version: 64.0.0
Base tests: 3427
Target tests: 3431

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3427 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"63.0.0"` であることを確認
- [x] `MILESTONE.md` に `"Incremental & Scale"` が含まれないことを確認
- [x] `README.md` に `"Incremental & Scale"` が含まれないことを確認
- [x] `driver.rs` に `v63900_tests` が存在することを確認（`v64000_tests` の挿入位置）
- [x] `driver.rs` に `v64000_tests` が存在しないことを確認

---

## T1: ファイル更新（宣言用）

- [x] `fav/Cargo.toml` の version を `"64.0.0"` に更新
- [x] `MILESTONE.md` 先頭に v64.0.0 エントリを追加
  - 宣言文・達成内容（v63.1〜v63.9）・テスト数 3431
- [x] `README.md` に v64.0.0 宣言文を追加（`v63.0.0` エントリの前）
- [x] `CHANGELOG.md` に v64.0.0 エントリを追加

---

## T2: `driver.rs` — `v64000_tests` 追加

- [x] `v63900_tests` の直前に `v64000_tests` を挿入（4 テスト）
  - `cargo_toml_version_is_64_0_0`
  - `changelog_has_v64_0_0`
  - `milestone_has_incremental_scale`
  - `readme_mentions_incremental_scale`
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v64000_tests` で 4 件 PASS
  - `cargo_toml_version_is_64_0_0` PASS
  - `changelog_has_v64_0_0` PASS
  - `milestone_has_incremental_scale` PASS
  - `readme_mentions_incremental_scale` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3431 tests passed, 0 failed を確認

---

## T4: ★クリーンアップ

- [x] `cargo clean` 実行
- [x] `fav/tmp/hello.fav` が存在することを確認（消えていたら復元）
  - 内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
- [x] `cargo build` で再ビルド成功を確認

---

## T5: ドキュメント更新

- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v64.0 セクションに実績追記（行 263 の `テスト数 ≥ 3428` → `≥ 3431`、行 264 の `3424 + 4 = 3428` → `3427 + 4 = 3431` に修正）
- [x] `versions/current.md` を v64.0.0（3431 tests）に更新し、次バージョン欄を `roadmap-v64.1-v65.0.md` に従って更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
