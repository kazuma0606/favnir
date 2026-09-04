# Tasks: v93.3.0 — `NavigationProperty` → `ExpandClause` フィールド生成

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,124 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93200_tests` が存在することを確認する（v93.2.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在し `entity_type_to_favnir` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `sap_metadata.rs` に `nav_property_to_favnir_comment` を追加する

- [x] `nav_names` が空のとき空文字列を返すロジックを実装する
- [x] ヘッダー行 `-- Navigation properties (use with ExpandClause):` + 各 nav_name を `-- "name"` 形式で出力するロジックを実装する

## T2: `sap_metadata.rs` に `to_snake_case` と `nav_to_expand_helper_fn` を追加する

- [x] PascalCase → snake_case 変換の内部ヘルパー `to_snake_case` を実装する
- [x] `nav_name` の `to_` プレフィックス除去ロジックを実装する
- [x] `{snake_entity}_expand_{snake_nav}()` 形式の関数名生成ロジックを実装する
- [x] `fn {fn_name}() -> ExpandClause<{entity_name}> { expand_nav<{entity_name}>(["nav_name"]) }` 文字列生成を実装する

## T3: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4: `driver.rs` に `mod v93300_tests` を追加する

- [x] ファイル末尾の `mod v93200_tests { ... }` の直後に `#[cfg(test)] mod v93300_tests { ... }` を追加する
- [x] `nav_property_parser_defined` テストを実装する
- [x] `nav_property_generates_expand_helper` テストを実装する（`nav_to_expand_helper_fn` の存在確認）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,126 tests, 0 failures であることを確認する

## T5a: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.3.0 のエントリを追加する

## T5b: ロードマップのテスト数を実測値に更新する

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.3.0 完了条件テスト数を 4,113 → 4,126 に更新する
- [x] ロードマップ行 150 の確認関数名を `expand_nav` → `nav_to_expand_helper_fn` に更新する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
