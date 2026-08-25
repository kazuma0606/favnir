# v82.8.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,879 tests pass、0 failures であることを確認する（前提: v82.7.0 完了済み）

## T1: `ContractRegistryEntry` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractRegistryEntry` 構造体を追加する
  - `name: String` / `version: ContractVersion` / `contract: IoContract` / `registered_at: String`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T2: `ContractRegistry` 構造体と `new()` 追加

- [x] `fav/src/test_framework.rs` に `ContractRegistry` 構造体を追加する
  - `entries: Vec<ContractRegistryEntry>`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する
  - `ContractRegistry::new() -> ContractRegistry` を実装する

## T3: `register` メソッド追加

- [x] `ContractRegistry::register(&self, entry: ContractRegistryEntry) -> ContractRegistry` を実装する
  - `self.entries` をクローンし末尾に `entry` を追加した新しい `ContractRegistry` を返す

## T4: `lookup` メソッド追加

- [x] `ContractRegistry::lookup(&self, name: &str, version: Option<&str>) -> Option<&ContractRegistryEntry>` を実装する
  - `version` が Some のとき: `ContractVersion::parse` でパースして完全一致検索
  - `version` が None のとき: 同名の最後（`rev().find`）のエントリを返す

## T5: `list_all` メソッド追加

- [x] `ContractRegistry::list_all(&self) -> Vec<&ContractRegistryEntry>` を実装する
  - `self.entries.iter().collect()` で全エントリへの参照を返す

## T6: `format_registry_listing` 関数追加

- [x] `format_registry_listing(registry: &ContractRegistry) -> String` を実装する
  - 1 行目: `"Registry ({n} entries):"`
  - 各行: `"  {name} v{major}.{minor}.{patch} — registered_at: {registered_at}"`
  - `lines.join("\n")` で結合

## T7: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.8.0 エントリを追加する

（site/ MDX / ドキュメント更新: 本バージョンは内部 API 追加のみのため対象外）

## T8: `v82800_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82800_tests` を追加する
  - `contract_registry_register_and_lookup`: `register` / `lookup`（バージョン指定あり・なし・存在しない名前）/ `format_registry_listing` を確認
  - `contract_registry_list_all`: 2 件登録して `list_all` が 2 件・順序が正しいことを確認

## T9: テスト通過確認

- [x] `cargo test` が 3,881 tests pass（+2）、0 failures であることを確認する

## T10: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [LOW] `impl Default for ContractRegistry` を追加（`new_without_default` 対応）
- [x] [LOW] `lookup` のドキュメントに不正バージョン文字列時も `None` を返す旨を追記
- [x] [LOW] `registered_at` フィールドに RFC 3339 / ISO 8601 形式を期待する旨をドキュメントに追記
