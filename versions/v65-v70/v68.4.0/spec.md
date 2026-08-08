# v68.4.0 — Stage Retry Policies（型安全エラー回復）

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

ステージレベルで型安全なリトライ・フォールバックポリシーを設定する。
LLM 呼び出しのタイムアウト・レート制限への対処を宣言的に記述できる。
v68.4.0 はスタブ実装。実際のリトライ実行・サーキットブレーカー動作は将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/retry.rs` — 新規作成
  - `pub fn cmd_retry_policy(src: &str) -> String`
    - `"ExponentialBackoff"` / `"LinearBackoff"` / `"timeout_ms"` を含む出力を返す（`retry_exponential_backoff` テスト要件）
    - `"Fallback"` / `"DeadLetterQueue"` / `"circuit_breaker"` を含む出力を返す（`retry_fallback_stage` テスト要件）
    - 出力はスタブとしてポリシー情報のサマリーを返す（実際のリトライ実行は行わない）
- `fav/src/main.rs` — `mod retry;` 追加 + `Some("run")` アームに `--retry-policy` ブランチ追加
  - `--retry-policy` フラグが存在する場合に `cmd_retry_policy(src)` を呼び出して `return`
  - `src` 検出時はフラグ値を除外（誤検出防止）
  - `src` 省略時デフォルト: `"pipeline.fav"`
- `fav/src/driver.rs` — `v68400_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.4.0 はスタブ実装のため将来フェーズとする。

- `ExponentialBackoff` / `LinearBackoff` / `FixedDelay` の実際のリトライ実行: 将来フェーズ
- `Fallback(stage)` / `Skip` / `DeadLetterQueue(queue_name)` の実際のフォールバック処理: 将来フェーズ
- `timeout_ms` によるステージレベルのタイムアウト制御: 将来フェーズ
- `circuit_breaker` の連続失敗カウント・オープン/クローズ遷移: 将来フェーズ
- `with { ... }` 構文の parser / checker への正式組み込み: 将来フェーズ
  ※ ロードマップ「実装内容」の `with { ... } 構文を parser / checker で正式サポート` に対応。現状 `with` はパース済みだが retry ポリシー専用フィールドは未解釈

## コマンド設計

```
fav run pipeline.fav --retry-policy
```

- `--retry-policy` フラグは `Some("run")` の既存フラグ群と干渉しない（`--checkpoint` ブランチの後、`--env` ブランチの前に挿入）
- `--checkpoint` / `--resume` と同時指定した場合は `--checkpoint` ブランチが優先される（ブランチ順による暫定仕様）
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `retry_exponential_backoff` | `cmd_retry_policy` が `"ExponentialBackoff"` / `"LinearBackoff"` / `"timeout_ms"` を含む |
| `retry_fallback_stage` | `cmd_retry_policy` が `"Fallback"` / `"DeadLetterQueue"` / `"circuit_breaker"` を含む |

ベーステスト: 3525 → 目標: **3527**

> 各テストは `use super::*` 不要。`crate::retry::cmd_retry_policy("pipeline.fav")` を直接呼び出す。各キーワードは個別の `assert!` で検証する（失敗時の診断性確保）。
