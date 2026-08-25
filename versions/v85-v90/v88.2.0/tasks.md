# Tasks: v88.2.0 — `material_by_id()` + `MaterialType` enum 完全化

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,999 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88100_tests` が存在することを確認する（v88.1.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/material.fav` に `material_by_id` を追加

- [x] `materials()` 関数の直後に `public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String>` スタブを追加する（`Result.err("not implemented")` 返し）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `materials()` ラッパーの直後に `public fn material_by_id(...)` ラッパーを追加する
- Note: T2 は手作業確認（Rust テストの対象外）

## T3: `driver.rs` に `mod v88200_tests` を追加

- [x] `mod v88100_tests { ... }` の直後に `#[cfg(test)] mod v88200_tests { ... }` を追加する
- [x] `material_by_id_function_exists` テストを実装する（`material.fav` で `"public fn material_by_id("` を確認）
- [x] `material_type_enum_has_all_variants` テストを実装する（`material.fav` で 5 バリアント全て `FinishedProduct` / `RawMaterial` / `SemiFinished` / `Trading` / `Service` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,001 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] テスト名を `material_type_enum_has_finished_product` → `material_type_enum_has_all_variants` に変更し、5 バリアント全てを assert で確認
- [MED] spec.md の「手作業確認項目」に re-export が Rust テスト対象外である理由を追記
- [LOW] spec.md の Files to Modify 表に「Rust テストなし（手作業確認）」注釈を追加
