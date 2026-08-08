# v68.9.0 タスクリスト

Status: COMPLETE
Version: 68.9.0
Note: 安定化バージョン — 新規モジュールなし、MDX 作成 + 統合テスト追加のみ
Base tests: 3535
Target tests: 3537

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3535 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `site/content/docs/runtime/distributed.mdx` が存在しないことを確認（新規作成）
- [x] `site/content/docs/runtime/` ディレクトリが存在することを確認（他の MDX ファイルと同階層）
- [x] `driver.rs` に `v68800_tests` が存在することを確認（`v68900_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68900_tests` を `v68800_tests` の直前に挿入する
- [x] `driver.rs` に `v68900_tests` が存在しないことを確認（新規追加）
- [x] `versions/current.md` の「進行中バージョン」が `v68.8.0` であることを確認
- [x] `cargo test --bin fav v68800_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `distributed_otel_trace`, `distributed_latency_breakdown`
- [x] `crate::cluster::cmd_cluster_run` が存在することを確認（`distributed_all_stable` で使用 / `partition_by = "row_id % 4"` を渡す）
- [x] `crate::checkpoint::cmd_checkpoint_run` が存在することを確認（`distributed_all_stable` で使用 / `resume_file = ""` で初回実行モード）
- [x] `crate::dist_cache::cmd_distributed_cache` が存在することを確認（`distributed_all_stable` で使用）
- [x] `crate::dist_otel::cmd_dist_otel` が存在することを確認（`distributed_all_stable` で使用）

---

## T1: `site/content/docs/runtime/distributed.mdx` 新規作成

- [x] `site/content/docs/runtime/` ディレクトリに `distributed.mdx` を新規作成
  - [x] `"--cluster"` キーワードを含む（`distributed_docs_complete` テスト要件）
  - [x] v68.1〜v68.8 の各機能（cluster / checkpoint / k8s / retry / dist_cache / cost_estimate / ai_routing / dist_otel）の概要を記載
  - [x] MDX として valid（先頭に ESM import 行なし）
- [x] `include_str!("../../site/content/docs/runtime/distributed.mdx")` でアクセスできることを確認

---

## T2: `driver.rs` — `v68900_tests` 追加

- [x] `// -- v68800_tests (v68.8.0) -- Distributed Observability --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `distributed_all_stable`:
    - [x] `crate::cluster::cmd_cluster_run("pipeline.fav", "workers.yaml", "row_id % 4")` → `"--cluster"` を個別 `assert!` で検証
    - [x] `crate::checkpoint::cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "")` → `"--checkpoint"` を個別 `assert!` で検証
    - [x] `crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379")` → `"--distributed-cache"` を個別 `assert!` で検証
    - [x] `crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317")` → `"--otel-endpoint"` を個別 `assert!` で検証
  - [x] `distributed_docs_complete`:
    - [x] `include_str!("../../site/content/docs/runtime/distributed.mdx")` でファイルを読み込む
    - [x] `"--cluster"` を `assert!` で検証
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v68900_tests` で 2 件 PASS
  - [x] `distributed_all_stable` PASS
  - [x] `distributed_docs_complete` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3537 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.9.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.9.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- v68.1〜v68.8 の機能実装の追加・変更: 将来フェーズ（スタブのまま v69.0.0 宣言へ）
- 新規 `.rs` スタブモジュール: 本バージョンでは不要

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [HIGH] | `main.rs`（git diff に混入） | v68.1〜v68.8 の実装変更が v68.9.0 の diff に混在。native-only モジュール（cluster 等）に `#[cfg(not(target_arch = "wasm32"))]` ガード欠落の可能性 | v68.9.0 の変更ではないことを確認。スタブ実装は `format!` のみで WASM 非互換 API 未使用のため現時点で実害なし。WASM ガード追加は将来フェーズに記録 |
| [MED] | `driver.rs` `distributed_all_stable` | v68.3/v68.4/v68.6/v68.7 の安定性確認が欠落（8 機能中 4 機能のみ） | 残り 4 モジュール（k8s / retry / cost_estimate / ai_routing）の呼び出しと assert を追加し全 8 機能をカバー |
| [LOW] | `distributed.mdx` | コードブロックがインデント記法でフェンス記法と不統一 | 4 スペースインデント → ` ```sh ``` ` フェンス記法に変更 |
