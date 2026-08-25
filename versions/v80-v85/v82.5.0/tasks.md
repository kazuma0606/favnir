# v82.5.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,873 tests pass、0 failures であることを確認する（前提: v82.4.0 完了済み）

## T1: `infer_field_type_from_str` 関数追加

- [x] `fav/src/test_framework.rs` に `infer_field_type_from_str(type_name: &str) -> ContractFieldType` を追加する
  - `"Int"` → `Int`、`"Float"` → `Float`、`"Bool"` → `Bool`、その他 → `Str`

## T2: `infer_contract_from_schema` 関数追加

- [x] `fav/src/test_framework.rs` に `infer_contract_from_schema(schema, name, version) -> IoContract` を追加する
  - nullable 列 → `Nullable(base_type)` / `required: false`
  - non-nullable 列 → base_type / `required: true`
  - input = 変換フィールド群、output = 空

## T3: `merge_contracts` 関数追加

- [x] `fav/src/test_framework.rs` に `merge_contracts(base, override_) -> IoContract` を追加する
  - 同名フィールドは `override_` が優先
  - `name` / `version` は `override_` の値を使用

## T4: `format_contract_as_toml` 関数追加

- [x] `fav/src/test_framework.rs` に `format_contract_as_toml(contract) -> String` を追加する
  - TOML ライクな文字列を生成（toml クレート不使用）
  - `Nullable(inner)` / `List(inner)` は再帰的に型名を生成する

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.5.0 エントリを追加する

## T6: `v82500_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82500_tests` を追加する
  - `contract_inferred_from_schema`: SchemaSnapshot → IoContract の型変換・required が正しいことを確認（`merge_contracts` の動作確認もこのテスト内で行う）
  - `contract_formatted_as_toml`: TOML 文字列にコントラクト名・フィールド名・型名が含まれることを確認

## T7: テスト通過確認

- [x] `cargo test` が 3,875 tests pass（+2）、0 failures であることを確認する

## T8: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
