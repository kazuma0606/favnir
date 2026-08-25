# Tasks: v88.1.0 — `Material` 型定義 + `MaterialFilter` + `materials()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,997 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88000_tests` が存在することを確認する（v88.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/material.fav` を新規作成

- [x] ファイル先頭に `use sap_odata.types` を追加する
- [x] `public type MaterialType = FinishedProduct | RawMaterial | SemiFinished | Trading | Service` を定義する
- [x] `public type Material = { ... }` を定義する（7 フィールド: `material_id` / `description` / `material_type` / `base_unit` / `weight` / `weight_unit` / `plant`）
- [x] `public type MaterialFilter = { ... }` を定義する（3 フィールド: `material_type` / `plant` / `top`）
- [x] `public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String>` スタブを追加する（`Result.err("not implemented")` 返し）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `use sap_odata.sales_report` の直後に `use sap_odata.material` を追加する
- [x] `format_sales_report` ラッパーの直後に以下を追加する:
  - `public type MaterialType = material.MaterialType`
  - `public type Material = material.Material`
  - `public type MaterialFilter = material.MaterialFilter`
  - `public fn materials(...)` ラッパー関数
- Note: T2 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは material.fav を参照）

## T3: `driver.rs` に `mod v88100_tests` を追加

- [x] `mod v88000_tests { ... }` の直後に `#[cfg(test)] mod v88100_tests { ... }` を追加する
- [x] `material_type_defined_in_rune` テストを実装する（`material.fav` で `"MaterialType"` を確認）
- [x] `materials_function_exists` テストを実装する（`material.fav` で `"public fn materials("` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,999 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
