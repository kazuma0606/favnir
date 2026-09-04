# Tasks: v93.4.0 — `EnumType` → Favnir `type E = | A | B` 変換

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,126 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93300_tests` が存在することを確認する（v93.3.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在し `nav_property_to_favnir_comment` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `sap_metadata.rs` に `EdmxEnumMember` / `EdmxEnumType` 構造体を追加する

- [x] `EdmxEnumMember { name: String }` 構造体を `#[derive(Debug)]` 付きで追加する
- [x] `EdmxEnumType { name: String, members: Vec<EdmxEnumMember> }` 構造体を `#[derive(Debug)]` 付きで追加する

## T2: `sap_metadata.rs` に `screaming_snake_to_pascal` と `enum_type_to_favnir` を追加する

- [x] `screaming_snake_to_pascal` 内部ヘルパーを実装する（`_` 分割 → 各単語先頭大文字・残り小文字 → 連結）
- [x] `enum_type_to_favnir` 関数を実装する
  - [x] EnumType 名を `screaming_snake_to_pascal` で変換する
  - [x] 先頭が数字のメンバー名に `Val` プレフィックスを付与する
  - [x] `type {Name} =\n    | Val1\n    | Val2` 形式で出力する

## T3: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4: `driver.rs` に `mod v93400_tests` を追加する

- [x] ファイル末尾の `mod v93300_tests { ... }` の直後に `#[cfg(test)] mod v93400_tests { ... }` を追加する
- [x] `enum_type_to_favnir_defined` テストを実装する
- [x] `edmx_enum_type_struct_defined` テストを実装する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,128 tests, 0 failures であることを確認する

## T5a: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.4.0 のエントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
