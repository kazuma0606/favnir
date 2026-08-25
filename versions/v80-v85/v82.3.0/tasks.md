# v82.3.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,869 tests pass、0 failures であることを確認する（前提: v82.2.0 完了済み）

## T1: `ContractDependency` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractDependency` 構造体を追加する
  - `upstream: String` / `downstream: String` / `output_contract: String`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T2: `DependencyGraph` 構造体追加

- [x] `fav/src/test_framework.rs` に `DependencyGraph` 構造体を追加する
  - `dependencies: Vec<ContractDependency>`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T3: `build_dependency_graph` 関数追加

- [x] `build_dependency_graph(contracts: &[IoContract]) -> DependencyGraph` を実装する
  - 全ペア (i, j) を走査し、`contracts[i].output` の field name と `contracts[j].input` の field name に共通要素があればエッジを追加する

## T4: `detect_circular_dependencies` 関数追加

- [x] `detect_circular_dependencies(graph: &DependencyGraph) -> Vec<Vec<String>>` を実装する
  - 隣接リストを構築し、A→B かつ B→A のサイクルを検出する

## T5: `format_dependency_graph` 関数追加

- [x] `format_dependency_graph(graph: &DependencyGraph) -> String` を実装する
  - 各エッジを `"{upstream} -> {downstream} ({output_contract})"` 形式で改行結合する

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.3.0 エントリを追加する

## T7: `v82300_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82300_tests` を追加する
  - `dependency_graph_built_from_contracts`: output/input フィールド名一致 → エッジが生成されることを確認
  - `circular_dependency_detected`: A→B, B→A → サイクルが検出されることを確認

## T8: テスト通過確認

- [x] `cargo test` が 3,871 tests pass（+2）、0 failures であることを確認する

## T9: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
