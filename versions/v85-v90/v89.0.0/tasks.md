# Tasks: v89.0.0 — SAP Procurement 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,015 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88900_tests` が存在することを確認する（v88.9.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する（`cargo clean` 後も残るが事前確認）

## T1: CHANGELOG.md 更新（include_str! のためテスト前に実施）

- [x] CHANGELOG.md の先頭（`## [v88.0.0]` の前）に v89.0.0 エントリを追加する
- [x] エントリに宣言文・Added・Changed セクションを含める（テスト数: 4,019）

## T2: MILESTONE.md 更新

- [x] MILESTONE.md の先頭（v88.0.0 エントリの前）に SAP Procurement 1.0 マイルストーンを追加する
- [x] `"SAP Procurement"` という文字列が含まれていることを確認する

## T3: `cargo clean` 実施

- [x] `cargo clean` を実行する
- [x] `fav/tmp/hello.fav` が残っていることを確認する（消えた場合は復元する）— 残存確認済み

## T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "88.0.0"` → `version = "89.0.0"` に変更する

## T5: `driver.rs` 内の `"88.0.0"` を `"89.0.0"` に一括更新

- [x] `sed -i 's/88\.0\.0/89.0.0/g' /c/Users/yoshi/favnir/fav/src/driver.rs` を実行する（文字列約40箇所）
- [x] `grep '88\.0\.0' /c/Users/yoshi/favnir/fav/src/driver.rs || echo "OK: no 88.0.0 found"` — 残存なし確認済み

## T6: `driver.rs` に `mod v89000_tests` を追加

- [x] `mod v88900_tests { ... }` の直後に `#[cfg(test)] mod v89000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_89_0_0` テストを追加する
- [x] `changelog_has_v89_0_0` テストを追加する
- [x] `milestone_has_sap_procurement` テストを追加する
- [x] `sap_odata_rune_has_material_type` テストを追加する

## T7: `versions/current.md` 更新

- [x] `versions/current.md` を v89.0.0 に更新する

## T8: `README.md` 更新

- [x] `README.md` に v89.0 SAP Procurement 1.0 セクションを追加する

## T9: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,019 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応

- [MED] `sed` コマンドのパスを絶対パス `/c/Users/yoshi/favnir/fav/src/driver.rs` に修正
