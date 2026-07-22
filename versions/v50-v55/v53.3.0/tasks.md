# Tasks: v53.3.0 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3167 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] E0419 の現状 `title` が `"assert_schema type mismatch"` であることを確認:
  - [x] `rg -n "E0419\|assert_schema type mismatch" fav/src/error_catalog.rs`
- [x] `v52100_tests::assert_schema_type_fail` の assert 内容を確認:
  - [x] `rg -n "assert_schema_type_fail\|assert_schema type mismatch" fav/src/driver.rs` → title pin を確認
- [x] `driver.rs` に `v53300_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53300_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53200_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53200_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `Cargo.toml` の現在バージョンが `53.2.0` であることを確認

---

## T1 — `error_catalog.rs` E0419 エントリ更新

- [x] E0419 エントリの `description` を更新（フィールド個別検証・`--strict-schema` の説明を追加）
- [x] E0419 エントリの `example` を更新（field diff 形式: `expected Int` / `got String` / `help: use Int.parse(...)` を含む）
- [x] E0419 エントリの `fix` を更新（「各フィールドを変換してから呼ぶ」旨を追記）
- [x] E0419 エントリの `suggestion` を更新（`Int.parse()` / `Float.from_int()` の具体例を含む）
- [x] **`title` / `code` / `category` は変更しない**（`v52100_tests::assert_schema_type_fail` が pin しているため）
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `driver.rs` — `v53300_tests` 追加

- [x] `rg -n "v53200_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53200_tests` モジュールの直前に `v53300_tests` を追加:
  - [x] `assert_schema_error_has_suggestion` テスト:
    - [x] `cmd_explain_error_collect("E0419")` を呼ぶ
    - [x] 出力に `"Suggestion"` が含まれることを assert
    - [x] 出力に `"Float.from_int"` が含まれることを assert（suggestion 固有テキスト — example の `Int.parse` と混同しない）
  - [x] `assert_schema_diff_shown` テスト:
    - [x] `cmd_explain_error_collect("E0419")` を呼ぶ
    - [x] 出力に `"expected Int"` が含まれることを assert
    - [x] 出力に `"got String"` が含まれることを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.2.0"` → `version = "53.3.0"` に変更
- [x] v53200_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3169 passed, 0 failed を確認
- [x] `v52100_tests::assert_schema_type_fail` が PASS であることを確認（title 変更なし）
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T4 — 後処理

- [x] `CHANGELOG.md` に v53.3.0 エントリ追加
- [x] `versions/current.md` を v53.3.0（3169 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.3.0 実績欄を更新（未実施 → COMPLETE、テスト数 3169）
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
