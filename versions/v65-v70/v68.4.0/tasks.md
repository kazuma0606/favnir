# v68.4.0 タスクリスト

Status: COMPLETE
Version: 68.4.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3525
Target tests: 3527

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3525 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/retry.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod k8s;` が存在することを確認（`mod retry;` の挿入位置）
- [x] `driver.rs` に `v68300_tests` が存在することを確認（`v68400_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68400_tests` を `v68300_tests` の直前に挿入する
- [x] `driver.rs` に `v68400_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68300_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `k8s_pipeline_manifest_gen`, `k8s_stage_replicas`
- [x] `driver.rs` のテストブロックが降順配置（`v68300_tests` が `v68200_tests` より上）であることを確認
- [x] `versions/current.md` の「進行中バージョン」が `v68.3.0` であることを確認

---

## T1: `fav/src/retry.rs` 新規作成

- [x] `fav/src/retry.rs` を新規作成
  - [x] `pub fn cmd_retry_policy(src: &str) -> String` を追加
    - [x] `"ExponentialBackoff"` / `"LinearBackoff"` / `"timeout_ms"` を含む出力（`retry_exponential_backoff` テスト要件）
    - [x] `"Fallback"` / `"DeadLetterQueue"` / `"circuit_breaker"` を含む出力（`retry_fallback_stage` テスト要件）
    - [x] 出力末尾は `[stub] Would apply retry policies at runtime`（実際のリトライ実行なし）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod retry;` を mod 宣言部（`mod k8s;` の直後）に追加
- [x] `Some("run")` アームの `--checkpoint`/`--resume` ブランチの直後に `--retry-policy` ブランチを追加
  - [x] `args.iter().any(|a| a == "--retry-policy")` で分岐
  - [x] `src` 検出: `args.iter().skip(2).find(|a| !a.starts_with('-'))` — `--retry-policy` は値なしフラグのため除外不要
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", retry::cmd_retry_policy(src))` + `return;`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68400_tests` 追加

- [x] `// -- v68300_tests (v68.3.0) -- Kubernetes-Native Orchestration --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `retry_exponential_backoff`: 各キーワードを個別 `assert!` で検証（`"ExponentialBackoff"` / `"LinearBackoff"` / `"timeout_ms"`）
  - [x] `retry_fallback_stage`: 各キーワードを個別 `assert!` で検証（`"Fallback"` / `"DeadLetterQueue"` / `"circuit_breaker"`）
- [x] `use super::*` は不要（`crate::retry::` で直接参照）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68400_tests` で 2 件 PASS
  - [x] `retry_exponential_backoff` PASS
  - [x] `retry_fallback_stage` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3527 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3527 tests passed）を確認してから実施すること。

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.4.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.4.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- `ExponentialBackoff` / `LinearBackoff` / `FixedDelay` の実際のリトライ実行: 将来フェーズ
- `Fallback(stage)` / `Skip` / `DeadLetterQueue` の実際のフォールバック処理: 将来フェーズ
- `timeout_ms` によるステージレベルのタイムアウト制御: 将来フェーズ
- `circuit_breaker` の連続失敗カウント・状態遷移: 将来フェーズ
- `with { ... }` 構文の parser / checker への正式組み込み: 将来フェーズ

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|---|---|---|
| [MED] | `--retry-policy` + `--checkpoint ./dir` 同時指定時に `./dir` が `src` として誤検出されるリスク（スタブ段階では単独使用を前提） | `--retry-policy` ブランチに制限事項をコメントで明記 |
| [LOW] | テスト関数名 `retry_exponential_backoff` が `LinearBackoff`/`timeout_ms` も検証しており名称が実態を正確に反映しない | ロードマップ指定のテスト名（`fn retry_exponential_backoff`）と一致させる必要があるため変更しない |
| [LOW] | `retry.rs` の `format!` で `src` が 2 回展開（Clippy `uninlined_format_args` 警告の可能性） | スタブ実装段階では許容（将来の Clippy 強化時に対処） |
