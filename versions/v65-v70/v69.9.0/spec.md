# v69.9.0 仕様書

## 概要

**バージョン**: v69.9.0
**テーマ**: コードフリーズ・最終 lint / チェック
**ベーステスト数**: 3553
**目標テスト数**: 3555（+2）

---

## Background

v69.5.0〜v69.8.0 の安定化スプリントを経て、v70.0.0「Intelligent ETL 1.0 宣言」に向けた最終コードフリーズを行う。
v69.9.0 は v70.0.0 宣言直前の検証バージョンであり、以下を確認する:

1. v69.x スプリント全体の成果物（ドキュメント・デモ・ベンチマーク・Playground）が正しく揃っているか
2. v70.0 ロードマップの完了条件（テスト数 3559 目標）が文書化されているか

---

## Goals

1. v70.0 ロードマップの「Intelligent ETL 1.0 宣言」エントリが `roadmap-v69.1-v70.0.md` に存在することを Rust テストで保証する
2. v69.6 で追加した Playground ETL サンプル（`etl-samples.mdx`）の `bind` 構文と `schema Order` が揃っていることを Rust テストで保証する
3. `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルに v69.9.0 行を追加する
4. `versions/current.md` の進行中バージョンを v69.9.0 に更新する

---

## 追加テスト（driver.rs）

### テスト 1: `code_freeze_v699_v70_roadmap_has_milestone_declaration`

```rust
let src = include_str!("../../versions/roadmap/roadmap-v69.1-v70.0.md");
assert!(src.contains("Intelligent ETL 1.0 宣言"), ...);
assert!(src.contains("3559"), ...);
```

v70.0.0 の宣言文と目標テスト数 3559 が roadmap に文書化されていることを確認する。

### テスト 2: `code_freeze_v699_playground_etl_samples_complete`

```rust
let src = include_str!("../../site/content/playground/etl-samples.mdx");
assert!(src.contains("schema Order"), ...);
assert!(src.contains("bind"), ...);
```

v69.6.0 で追加した ETL Playground サンプルの重要要素が揃っていることを確認する。

---

## 参照ファイル

- `versions/roadmap/roadmap-v69.1-v70.0.md` — v70.0 宣言文・テスト数確認
- `site/content/playground/etl-samples.mdx` — Playground ETL サンプル
- `fav/src/driver.rs` — テスト追加先（`v69900_tests` を `v69800_tests` の直前に挿入）

---

## Success Criteria

1. `cargo test --bin fav -- --test-threads=8` で **3555 tests passed, 0 failed**
2. `v69900_tests` モジュールが `v69800_tests` の直前に存在する（降順ルール準拠）
3. `roadmap-v69.1-v70.0.md` の v69.9.0 行が 3555 で確定
4. `versions/current.md` の進行中バージョンが v69.9.0

---

## Error codes

新規エラーコードなし。

---

## sub-version ポリシー

v69.x では `fav/Cargo.toml` および `CHANGELOG.md` は変更しない。
これらは v70.0.0 宣言時に一括更新する。
