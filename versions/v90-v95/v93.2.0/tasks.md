# Tasks: v93.2.0 — `EntityType` → Favnir `type` 変換

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,122 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93100_tests` が存在することを確認する（v93.1.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在し `parse_edmx` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `sap_metadata.rs` に `edm_type_to_favnir` を追加する

- [x] EDM 型名 → Favnir 型名の `match` 変換を実装する
- [x] 対応表: `Edm.String`/`Edm.DateTimeOffset`/`Edm.Guid` → `String`、`Edm.Int32`/`Edm.Int64` → `Int`、`Edm.Decimal` → `Float`、`Edm.Boolean` → `Bool`、その他 → `String`

## T2: `sap_metadata.rs` に `entity_type_to_favnir` を追加する

- [x] 先頭の `X_` プレフィックス除去ロジックを実装する
- [x] 末尾の `Type` サフィックス除去ロジックを実装する（`strip_suffix`）
- [x] プロパティリストを `    FieldName: FavnirType,` 形式で出力するロジックを実装する

## T3: `cargo build` でコンパイル確認（テストモジュール追加前の中間確認）

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4: `driver.rs` に `mod v93200_tests` を追加する

- [x] ファイル末尾の `mod v93100_tests { ... }` の直後に `#[cfg(test)] mod v93200_tests { ... }` を追加する
- [x] `entity_type_to_favnir_defined` テストを実装する
- [x] `edm_type_to_favnir_defined` テストを実装する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,124 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
