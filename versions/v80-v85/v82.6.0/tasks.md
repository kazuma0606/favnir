# v82.6.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,875 tests pass、0 failures であることを確認する（前提: v82.5.0 完了済み）

## T1: `ContractVersion` 構造体 + `parse` 追加

- [x] `fav/src/test_framework.rs` に `ContractVersion` 構造体を追加する
  - `major: u32` / `minor: u32` / `patch: u32`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する
- [x] `impl ContractVersion` ブロックに `parse(s: &str) -> Result<ContractVersion, String>` を実装する
  - `"1.2.3"` 形式をパースして Ok を返す
  - 要素数 ≠ 3 または数値変換失敗 → `Err("invalid version: {s}")`

## T2: `CompatibilityResult` enum 追加

- [x] `fav/src/test_framework.rs` に `CompatibilityResult` enum を追加する
  - variants: `Compatible` / `BackwardsCompatible(Vec<String>)` / `Breaking(Vec<String>)`
  - `#[derive(Debug, PartialEq)]` を付与する

## T3: `check_contract_compatibility` 関数追加

- [x] `check_contract_compatibility(old, new_) -> CompatibilityResult` を実装する
  - Breaking: old の required フィールドが new_ に存在しない
  - Breaking: 同名フィールドの型が変わった
  - BackwardsCompatible: new_ に old にないフィールドが追加された
  - Compatible: 変更なし

## T4: `format_compatibility_result` 関数追加

- [x] `format_compatibility_result(result: &CompatibilityResult) -> String` を実装する
  - `Compatible` → `"Compatible"`
  - `BackwardsCompatible(fields)` → `"BackwardsCompatible: added [...]"`
  - `Breaking(fields)` → `"Breaking: [...]"`

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.6.0 エントリを追加する

## T6: `v82600_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82600_tests` を追加する
  - `contract_version_parsed`: `"1.2.3"` のパース成功・`"bad"` と `"1.x.3"` の失敗を確認・`format_compatibility_result(&CompatibilityResult::Compatible) == "Compatible"` を確認
  - `breaking_change_detected_on_field_removal`: 必須フィールド削除で Breaking が返ること・`format_compatibility_result` の出力に削除フィールド名が含まれることを確認

## T7: テスト通過確認

- [x] `cargo test` が 3,877 tests pass（+2）、0 failures であることを確認する

## T8: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
