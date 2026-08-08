# v68.9.0 — 安定化・コードフリーズ

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

v68.1〜v68.8 の全分散機能が正常動作することを確認する安定化バージョン。
新規スタブモジュールは作成しない。既存モジュールを統合確認するテストと MDX ドキュメントを追加する。

## スコープ

### IN スコープ

- `site/content/docs/runtime/distributed.mdx` — 新規作成
  - `"--cluster"` を含む（`distributed_docs_complete` テスト要件）
  - Distributed Favnir スプリント（v68.1〜v68.8）の概要ドキュメント
- `fav/src/driver.rs` — `v68900_tests` 追加（2 件）
  - `distributed_all_stable`: 既存 v68.x モジュールを直接呼び出して主要フラグの出力を確認
    - `crate::cluster::cmd_cluster_run("pipeline.fav", "workers.yaml", "row_id % 4")` → `"--cluster"` を含む（`partition_by` は `"row_id % 4"` を使用、これはロードマップ例示の標準値）
    - `crate::checkpoint::cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "")` → `"--checkpoint"` を含む（`resume_file` 空 = 初回実行モード）
    - `crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379")` → `"--distributed-cache"` を含む
    - `crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317")` → `"--otel-endpoint"` を含む
  - `distributed_docs_complete`: `include_str!("../../site/content/docs/runtime/distributed.mdx")` → `"--cluster"` を含む

### OUT スコープ

- 新規 `.rs` スタブモジュールの作成: 本バージョンでは不要（安定化フェーズのため）
- `fav/src/main.rs` の変更: 本バージョンでは不要
- v68.1〜v68.8 の機能実装の追加・変更: 将来フェーズ（スタブのまま）
- K8s マニフェスト生成の正しさ検証: v68.3.0 の `v68300_tests` で既に確認済みのため本バージョンでは省略
- `fav cost-estimate` 出力検証: v68.6.0 の `v68600_tests` で既に確認済みのため本バージョンでは省略

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `distributed_all_stable` | 既存 v68.x モジュール（cluster / checkpoint / dist_cache / dist_otel）が `"--cluster"` / `"--checkpoint"` / `"--distributed-cache"` / `"--otel-endpoint"` を含む出力を返す |
| `distributed_docs_complete` | `site/content/docs/runtime/distributed.mdx` が存在し `"--cluster"` を含む |

ベーステスト: 3535 → 目標: **3537**

> `distributed_all_stable` は 3 件の個別 `assert!`（各モジュール 1 件ずつ）。`distributed_docs_complete` は `include_str!` で MDX を読み込み 1 件の `assert!` で検証。

## `distributed.mdx` 内容要件

- `"--cluster"` キーワードを含む（テスト要件）
- v68.1〜v68.8 の各機能（cluster / checkpoint / k8s / retry / dist_cache / cost_estimate / ai_routing / dist_otel）の概要を記載
- MDX として valid な構文（先頭 import 行なし）
