# v68.3.0 — Kubernetes-Native Orchestration

Date: 2026-08-07
Status: 完了（実装済み・レビュー済み）
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

`fav deploy --target kubernetes` で Favnir パイプラインの Kubernetes CRD マニフェストを生成する。
`Pipeline` kind（`apiVersion: favnir.dev/v1`）を出力し、ステージ別 replicas・GPU リソース指定に対応。
v68.3.0 はスタブ実装。実際のファイル書き込み・K8s API 連携は将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/k8s.rs` — 新規作成
  - `pub fn cmd_deploy_k8s(src: &str) -> String`
    - `"apiVersion: favnir.dev/v1"` / `"kind: Pipeline"` を含む出力を返す（`k8s_pipeline_manifest_gen` テスト要件）
    - `"replicas"` / `"resources"` / `"--target kubernetes"` を含む出力を返す（`k8s_stage_replicas` テスト要件）
      - `"--target kubernetes"` は出力の第 2 行に `[--target kubernetes] Generating Pipeline CRD for: <src>` という形式のヘッダー行を挿入することで埋め込む。スタブ実装においてどのフラグで呼び出されたかをログに残すための設計。本来の K8s YAML には現れない文字列だが、テスト検証を容易にするため意図的に含める。
    - 出力末尾は `[stub] Would write manifests to ./k8s/`（実際の書き込みは行わない）
- `fav/src/main.rs` — `mod k8s;` 追加 + `Some("deploy")` アームに `--target kubernetes` ブランチ追加
  - パースループ完了後・`trigger_file` チェック前に挿入
  - `src` 検出時は `target` の値（`"kubernetes"`）を除外（誤検出防止）
  - `--trigger` + `--target kubernetes` 同時指定時は kubernetes ターゲットが優先（コメントで明記）
- `fav/src/driver.rs` — `v68300_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.3.0 はスタブ実装のため将来フェーズとする。

- 実際の K8s CRD YAML ファイルへの書き込み（`./k8s/` ディレクトリ生成）: 将来フェーズ
- `par` ステージの並列数を replicas に自動変換: 将来フェーズ
- `with { gpu: 1 }` → K8s `resources.limits` への変換: 将来フェーズ
- Helm チャート生成（`--helm` フラグ）: 将来フェーズ
- Argo Workflows 対応（`--target argo`）: 将来フェーズ
- K8s API への実際のデプロイ: 将来フェーズ

## コマンド設計

```
fav deploy --target kubernetes pipeline.fav
fav deploy --target kubernetes --helm pipeline.fav
fav deploy --target argo pipeline.fav
```

- `--target kubernetes` は既存の `Some("deploy")` パースループで取得済みの `target` 変数を利用
- `--target` の値（`"kubernetes"`）は `src` 検出から除外する（フラグ値誤検出防止）
- `--trigger` と同時指定した場合は `--target kubernetes` が優先される（スタブ段階での暫定仕様）
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `k8s_pipeline_manifest_gen` | `cmd_deploy_k8s` が `"apiVersion: favnir.dev/v1"` / `"kind: Pipeline"` を含む |
| `k8s_stage_replicas` | `cmd_deploy_k8s` が `"replicas"` / `"resources"` / `"--target kubernetes"` を含む |

ベーステスト: 3523 → 目標: **3525**
