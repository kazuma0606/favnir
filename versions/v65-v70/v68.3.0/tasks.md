# v68.3.0 タスクリスト

Status: COMPLETE
Version: 68.3.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3523
Target tests: 3525

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3523 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/k8s.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod checkpoint;` が存在することを確認（`mod k8s;` の挿入位置）
- [x] `driver.rs` に `v68200_tests` が存在することを確認（`v68300_tests` の挿入位置）
- [x] `driver.rs` に `v68300_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68200_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `checkpoint_save_restore`, `checkpoint_resume_mid_pipeline`
- [x] `versions/current.md` の「進行中バージョン」が `v68.2.0` であることを確認

---

## T1: `fav/src/k8s.rs` 新規作成

- [x] `fav/src/k8s.rs` を新規作成
  - [x] `pub fn cmd_deploy_k8s(src: &str) -> String` を追加
    - [x] `"apiVersion: favnir.dev/v1"` / `"kind: Pipeline"` を含む出力（`k8s_pipeline_manifest_gen` テスト要件）
    - [x] `"replicas"` / `"resources"` / `"--target kubernetes"` を含む出力（`k8s_stage_replicas` テスト要件）
    - [x] 出力末尾は `[stub] Would write manifests to ./k8s/`（実際の書き込みなし）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod k8s;` を mod 宣言部（`mod checkpoint;` の直後）に追加
- [x] `Some("deploy")` アームにパースループ完了後・`trigger_file` チェック前に `--target kubernetes` ブランチを追加
  - [x] `target.as_deref() == Some("kubernetes")` で分岐
  - [x] `src` 検出時に `target.as_deref()` 値（`"kubernetes"`）を除外（誤検出防止）
  - [x] `--trigger` + `--target kubernetes` 同時指定時の優先順位をコメントで明記
  - [x] `println!("{}", k8s::cmd_deploy_k8s(src))` + `return;`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68300_tests` 追加

- [x] `// -- v68200_tests (v68.2.0) -- Pipeline Checkpointing（耐障害性・再開） --` の直前に挿入
  - [x] `k8s_pipeline_manifest_gen`: `cmd_deploy_k8s` の戻り値に `"apiVersion: favnir.dev/v1"` / `"kind: Pipeline"` を含む
  - [x] `k8s_stage_replicas`: `cmd_deploy_k8s` の戻り値に `"replicas"` / `"resources"` / `"--target kubernetes"` を含む
- [x] `use super::*` は不要（`crate::k8s::` で直接参照）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68300_tests` で 2 件 PASS
  - [x] `k8s_pipeline_manifest_gen` PASS
  - [x] `k8s_stage_replicas` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3525 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3525 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.3.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v68.3.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際の K8s CRD YAML ファイルへの書き込み（`./k8s/` ディレクトリ生成）: 将来フェーズ
- `par` ステージの並列数を replicas に自動変換: 将来フェーズ
- `with { gpu: 1 }` → K8s `resources.limits` への変換: 将来フェーズ
- Helm チャート生成（`--helm` フラグ）: 将来フェーズ
- Argo Workflows 対応（`--target argo`）: 将来フェーズ

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `src` 検出で `"kubernetes"`（`--target` の値）を誤検出 | `Some(a.as_str()) != target.as_deref()` フィルターを追加 |
| [MED] | `--trigger` + `--target kubernetes` 同時指定時の優先順位が未文書 | コメントに「kubernetes が優先、--trigger は無視」を明記 |
| [LOW] | `[done] Manifests written to ./k8s/` はスタブで実際の書き込みなし | `[stub] Would write manifests to ./k8s/` に変更 |
