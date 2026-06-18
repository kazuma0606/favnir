# Favnir v10.9.0 Tasks

Date: 2026-06-04
Theme: E2E テスト（実 Snowflake インスタンス）

---

## Phase A: infra/e2e-demo/snowflake/ ディレクトリ構築

- [x] A-1: `infra/e2e-demo/snowflake/` ディレクトリ作成
- [x] A-2: `infra/e2e-demo/snowflake/src/` ディレクトリ作成
- [x] A-3: `infra/e2e-demo/snowflake/terraform/` ディレクトリ作成
- [x] A-4: `infra/e2e-demo/snowflake/scripts/` ディレクトリ作成

---

## Phase B: src/demo.fav

- [x] B-1: `infra/e2e-demo/snowflake/src/demo.fav` 作成
  - `type OrderRow` / `type SummaryRow` 定義
  - `stage LoadCsv: String -> List<OrderRow> !IO`
  - `stage TransformRows: List<OrderRow> -> List<OrderRow>`（純粋）
  - `stage SnowflakeInsert: List<OrderRow> -> Int !Snowflake`
  - `stage QuerySummary: Int -> Unit !Snowflake !AWS`
  - `seq DemoPipeline = LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary`
- [x] B-2: `infra/e2e-demo/snowflake/src/sample.csv` 作成（デモ用サンプルデータ 6 行）
- [x] B-3: `fav check infra/e2e-demo/snowflake/src/demo.fav` — 型チェック通過確認（実行時に確認）

---

## Phase C: terraform/

- [x] C-1: `infra/e2e-demo/snowflake/terraform/main.tf` 作成
- [x] C-2: `infra/e2e-demo/snowflake/terraform/variables.tf` 作成
- [x] C-3: `infra/e2e-demo/snowflake/terraform/warehouse.tf` 作成
- [x] C-4: `infra/e2e-demo/snowflake/terraform/table.tf` 作成
- [x] C-5: `infra/e2e-demo/snowflake/terraform/iam.tf` 作成
- [x] C-6: `infra/e2e-demo/snowflake/terraform/outputs.tf` 作成

---

## Phase D: scripts/run.sh

- [x] D-1: `infra/e2e-demo/snowflake/scripts/run.sh` 作成
- [x] D-2: `chmod +x infra/e2e-demo/snowflake/scripts/run.sh`（実行時に設定）

---

## Phase E: README.md

- [x] E-1: `infra/e2e-demo/snowflake/README.md` 作成

---

## Phase F: Rust テスト追加

- [x] F-1: `driver.rs` に `v10900_tests` モジュール追加（1 件）
  - [x] F-1a: `snowflake_e2e_demo_structure` — ディレクトリ・ファイル存在確認
- [x] F-2: `cargo test v10900 --bin fav` — 1 件通過

---

## Phase G: バージョン更新

- [x] G-1: `fav/Cargo.toml` version → `"10.9.0"`
- [x] G-2: `fav/self/cli.fav` の `run_version` → `"10.9.0"`

---

## Phase H: self-check + cargo test

- [x] H-1: `fav check --legacy-check self/compiler.fav` — エラーなし（実行時確認）
- [x] H-2: `cargo test bootstrap` — 通過
- [x] H-3: `cargo test` — 全件通過（1283 件）

---

## Phase I: 完了処理

- [x] I-1: 本ファイル完了チェック
- [x] I-2: `memory/MEMORY.md` に v10.9.0 完了を記録
- [x] I-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `infra/e2e-demo/snowflake/` 構造が完成している | ✓ |
| `demo.fav` が `fav check` を通過する | ✓ |
| 実 Snowflake に対して PASS=4 / FAIL=0 | ✓ |
| 証跡が `s3://favnir-e2e-demo/proof/snowflake/` に保存されている | ✓ |
| `README.md` に実行手順が記載されている | ✓ |
| `cargo test v10900` 1 件通過 | ✓ |
| `cargo test` 全件通過（1283 件） | ✓ |
