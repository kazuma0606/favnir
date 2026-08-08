# v69.5.0 Spec — E2E デモの動作確認

## Background

v69.1.0 で `infra/e2e-demo/ai-etl/` に AI-ETL E2E デモを追加した。
v69.5.0 では、このデモの構造的な整合性を Rust テストで保証する。

対象: `infra/e2e-demo/ai-etl/src/pipeline.fav` と `infra/e2e-demo/ai-etl/workers.yaml`

## Goals

1. `pipeline.fav` が正しい `bind x <- expr` 構文を使用していることをテストで確認する
   （`bind x = expr` は不正構文 — パーサーが拒否するため）
2. `workers.yaml` がロードマップ記載どおり 4 ワーカーエントリを持つことをテストで確認する
   - workers.yaml の構造: `workers:` リスト配下に `- host: / port: / cores:` フィールドを持つ 4 エントリ
   - `name:` フィールドは存在しない。`- host:` で始まる行（trim_start 後）でエントリ数をカウントする

## Out of Scope

- 実際のパイプライン実行（`cargo run`、`fav run` 等）
- CI 環境での E2E 実行（Docker / Lambda 起動）
- ネットワーク疎通確認
- パフォーマンス計測

## Success Criteria

- `cargo test --bin fav v695` で **2 tests passed, 0 failed**
- ベース (3545) + 2 = **3547 tests**
- `pipeline.fav` に `bind` かつ `<-` が含まれる（bind arrow 構文チェック）
- `workers.yaml` に `- host:` 行が 4 件含まれる（ワーカー数チェック）

## Files to Modify

- `fav/src/driver.rs` — `#[cfg(test)]` モジュール末尾に 2 テスト追加
- `versions/roadmap/roadmap-v69.1-v70.0.md` — v69.5.0 テスト数（3547）を記入・状態を「完了」に更新

## Files NOT to Modify

- `infra/e2e-demo/ai-etl/` 以下のファイル（既存のまま）
- `Cargo.toml`（sub-version ポリシー）
- `CHANGELOG.md`（sub-version ポリシー）

## Error Codes

なし（テスト追加のみ）

## 既存テストとの差別化

v69.1.0 で追加済みの `v69100_tests` は:
- デモ構造の存在確認（ファイルが存在するか）
- パイプラインの主要ステージ名
- スクリプト名

v69.5.0 で追加するテストは:
- **`bind x <- expr` 構文の使用確認**（`=` ではなく `<-` を使っているか）
- **ワーカー数の正確な確認**（`- host:` エントリが 4 件であることを数値で検証）
