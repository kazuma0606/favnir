# Tasks: v80.3.0 — `TestFixture` / `DataFactory` モックデータ生成

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3815）と実際のベース（3814）が 1 件ずれているが、
> v80.2.0 コードレビュー対応で追加されたテストが原因。完了時の目標は **3816**。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3814 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.2.0` であることを確認する
- [x] `fav/src/test_framework.rs` に v80.2.0 の `GoldenDataset` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `FieldSpec` enum（`#[derive(Debug, Clone)]`、`Str(String)` / `Int(i64)` / `Float(f64)` / `Bool(bool)` / `Null`）を追加する
- [x] `RowSpec` 型エイリアス（`pub type RowSpec = Vec<(String, FieldSpec)>;`）を追加する
- [x] `TestFixture` 構造体（`#[derive(Debug)]`、`name: String`, `schema: Vec<String>`, `rows: Vec<RowSpec>`）を追加する
- [x] `DataFactory` 構造体（`#[derive(Debug)]`、`seed: u64`）を追加する
- [x] `DataFactory::from_seed(seed: u64) -> DataFactory` を実装する
- [x] `DataFactory::generate_rows(&self, spec: &TestFixture, count: usize) -> Vec<Vec<String>>` を実装する
  - `spec.rows` が空の場合は空の Vec を返す
  - `stride = seed.max(1)` で循環インデックスを計算する
  - 各行を `spec.schema` の列順でソートして文字列変換する
  - `FieldSpec` 変換: `Str` → そのまま / `Int` → `to_string()` / `Float` → `to_string()` / `Bool` → `"true"`/`"false"` / `Null` → `""`

## T2: `fav/src/driver.rs` に `mod v80300_tests` を追加

- [x] `mod v80200_tests { ... }` の直後に `#[cfg(test)] mod v80300_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `data_factory_generates_rows` テストを実装する
  - `DataFactory::from_seed(1)` で 2 行生成
  - `rows.len() == 2`、各行の `len() == schema.len()`（2）
  - `rows[0] == ["alice", "30"]`、`rows[1] == ["alice", "30"]`（seed=1 の循環パターン）
- [x] `test_fixture_schema_matches_rows` テストを実装する
  - `DataFactory::from_seed(0)` で 3 行生成
  - 全行の `len() == schema.len()`（2）を確認
  - `rows[0]` が `["alice", "30"]` であることも確認する（値アサーション）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3816 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v80.3.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
