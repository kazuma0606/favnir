# Tasks: v49.1.0 — 全機能統合テスト + E2E デモ更新

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3069 passed, 0 failed を確認（ベース確認）
- [x] `examples/v50-demo/` が存在しないことを確認（新規作成対象）

## T1 — E2E デモ作成

- [x] `examples/v50-demo/fav.toml` 新規作成
  - [x] `[package]` セクション（name / version）
  - [x] `[runes]` セクション（`kafka = "2.1.0"`）
- [x] `examples/v50-demo/stages/` ディレクトリ作成
- [x] `examples/v50-demo/stages/validate.fav` 新規作成
  - [x] `RawOrder` / `Order` 型定義
  - [x] `run(raw: RawOrder) -> Result<Order, String>` — `return` ガード節使用
- [x] `examples/v50-demo/pipeline.fav` 新規作成
  - [x] `import "./stages/validate" as validate`（ローカル import）
  - [x] `import kafka`（パッケージ import・新構文）
  - [x] `pipeline OrderIngestion` ブロック
  - [x] `#[test]` インラインテスト 2 件
  - [x] `import rune` を使わないこと（W035 回避）

## T2 — `driver.rs` テスト追加

- [x] `v491000_tests` モジュールを `v49000_tests` の直前に追加（2テスト）
  - [x] `e2e_demo_v50_structure`: `examples/v50-demo/` + `fav.toml` + `pipeline.fav` + `stages/validate.fav` 存在確認
  - [x] `e2e_demo_uses_new_import`: `import kafka` 含む・`import "./stages/validate"` 含む・`import rune` を含まないことを確認
- [x] `cargo_toml_version_is_49_0_0` をスタブ化（バージョンバンプ 49.0.0→49.1.0 による）

## T3 — バージョン更新・完了

- [x] site/ MDX 更新: 不要（デモ追加のみ・新構文なし）
- [x] `fav/Cargo.toml` version → `"49.1.0"`
- [x] `CHANGELOG.md` に v49.1.0 エントリ追加
- [x] `cargo test` 3071 passed, 0 failed（3069 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v49.1.0（3071 tests）に更新、進行中バージョンを `v49.2.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.1.0 実績を 3071 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: `examples/v50-demo/` の配置先は `favnir/examples/`（`fav/examples/` ではない）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
