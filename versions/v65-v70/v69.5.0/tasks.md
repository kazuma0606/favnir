# v69.5.0 タスクリスト

Status: COMPLETE
Version: 69.5.0
Note: E2E デモの動作確認 — driver.rs に 2 テスト追加
Base tests: 3545
Target tests: 3547（+2）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3545 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.4.0` であることを確認（もし v69.3.0 等であれば前バージョンの tasks.md が COMPLETE になっているか確認してから続行）
- [x] `infra/e2e-demo/ai-etl/src/pipeline.fav` に `bind` と `<-` が含まれることを確認
- [x] `infra/e2e-demo/ai-etl/workers.yaml` に `- host:` 行が 4 件あることを確認（`name:` フィールドは存在しない）
- [x] `driver.rs` の既存 `v69100_tests` に `ai_etl_demo_pipeline_uses_bind_arrow_syntax` / `ai_etl_demo_workers_yaml_has_four_workers` が存在しないことを確認（重複防止）

---

## T1: `driver.rs` — テスト追加

- [x] `#[cfg(test)]` モジュール末尾に `ai_etl_demo_pipeline_uses_bind_arrow_syntax` テストを追加
  - [x] `include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav")` でファイルを読み込む
  - [x] `src.contains("bind")` アサート
  - [x] `src.contains("<-")` アサート
- [x] `#[cfg(test)]` モジュール末尾に `ai_etl_demo_workers_yaml_has_four_workers` テストを追加
  - [x] `include_str!("../../infra/e2e-demo/ai-etl/workers.yaml")` でファイルを読み込む
  - [x] `l.trim_start().starts_with("- host:")` でフィルタして行数をカウント（`name:` は存在しないため使わない）
  - [x] `count == 4` アサート（エラーメッセージに実際の数を含める）

---

## T2: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3547 tests passed, 0 failed** を確認

---

## T3: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルに v69.5.0 の新規行を追加して 3547（+2）を記入
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.5.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.1.0 セクションのサンプルコード中 `bind x = expr` を `bind x <- expr` に修正
- [x] `versions/current.md` の「進行中バージョン」を `v69.4.0` から `v69.5.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

---

## 設計上の意図的省略

- 実際の E2E 実行（`fav run pipeline.fav` 等）: 将来フェーズ（本 sub-version は構造確認のみ）
- CI 環境での Docker / Lambda 起動: 将来フェーズ
