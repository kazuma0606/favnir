# Tasks: v93.1.0 — EDMX XML パーサー基盤（`parse_edmx`）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,120 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93000_tests` が存在することを確認する（v93.0.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `fav/src/sap_metadata.rs` を新規作成する

- [x] `EdmxProperty` 構造体を定義する（`name: String`, `edm_type: String`）
- [x] `EdmxEntityType` 構造体を定義する（`name: String`, `properties: Vec<EdmxProperty>`）
- [x] `EdmxSchema` 構造体を定義する（`namespace: String`, `entity_types: Vec<EdmxEntityType>`）
- [x] `pub fn parse_edmx(_xml: &str) -> Vec<EdmxSchema>` スタブを実装する（`Vec::new()` を返す）

## T1.5: ロードマップ実測値の反映

- [x] `roadmap-v93.1-v94.0.md` のテスト数推移表を実測値（ベース 4,120、+13 オフセット）に更新する（spec-reviewer 修正時に実施済み）

## T2: `fav/src/main.rs` に `mod sap_metadata;` を追加する

- [x] `rune_cmd;` の直後に `mod sap_metadata;` を追加する

## T3: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4: `driver.rs` に `mod v93100_tests` を追加する

- [x] ファイル末尾の `mod v93000_tests { ... }` の直後に `#[cfg(test)] mod v93100_tests { ... }` を追加する
- [x] `sap_metadata_file_exists` テストを実装する（`src/sap_metadata.rs` が存在する）
- [x] `parse_edmx_function_defined` テストを実装する（`sap_metadata.rs` に `parse_edmx` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,122 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
