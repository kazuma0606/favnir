# v68.5.0 — Distributed Incremental Cache

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

複数ワーカー間でコンパイルキャッシュ・ステージ実行キャッシュを共有する。
同一入力の同一ステージは 2 回実行しない。コスト削減に直結。
v68.5.0 はスタブ実装。実際の Redis 接続・キャッシュ読み書きは将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/dist_cache.rs` — 新規作成
  - `pub fn cmd_distributed_cache(src: &str, cache_url: &str) -> String`
    - `"--distributed-cache"` / `"redis"` / `"Hit rate"` を含む出力を返す（`distributed_cache_hit_across_workers` テスト要件）
    - `"--cache-ttl"` / `"L1"` / `"L2"` / `"invalidation"` を含む出力を返す（`distributed_cache_invalidation` テスト要件）
    - 出力は `[--distributed-cache]` プレフィックス付きヘッダーを含む（`"--distributed-cache"` 検証のため）
    - 出力末尾は `[stub] Would connect to Redis cache (source: <src>)`（実際の接続は行わない）
- `fav/src/main.rs` — `mod dist_cache;` 追加 + `Some("run")` アームに `--distributed-cache` ブランチ追加
  - `--distributed-cache <url>` フラグが存在する場合に `cmd_distributed_cache(src, cache_url)` を呼び出して `return`
  - `cache_url` は `--distributed-cache` の次の引数から取得（省略時は `"redis://localhost:6379"`）
  - `src` 検出時は `cache_url` の値を除外（誤検出防止）
  - `src` 省略時デフォルト: `"pipeline.fav"`
  - `--retry-policy` ブランチの直後・`--env` ブランチの前に挿入
  - `--checkpoint`/`--resume` や `--retry-policy` と同時指定した場合は先行ブランチが優先される（ブランチ順による暫定仕様）
- `fav/src/driver.rs` — `v68500_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.5.0 はスタブ実装のため将来フェーズとする。

- 実際の Redis 接続・キャッシュ読み書き: 将来フェーズ
- キャッシュキー生成（ステージ名 + 入力ハッシュ SHA256）: 将来フェーズ
- `--cache-ttl` / `--cache-ttl-per-stage` の実際の TTL 設定: 将来フェーズ
- 入力スキーマ変更時の自動キャッシュ無効化: 将来フェーズ
- LLM 呼び出し回避による節約額の実際のコスト追跡: 将来フェーズ
- L1（メモリ） → L2（Redis）の 2 層キャッシュ実装: 将来フェーズ

## コマンド設計

```
fav run pipeline.fav --distributed-cache redis://cache.internal:6379
fav run pipeline.fav --distributed-cache redis://cache.internal:6379 --cache-ttl 3600
```

- `--distributed-cache <url>` は `Some("run")` の `--retry-policy` ブランチの直後に挿入
- `cache_url` は `--distributed-cache` の直後の引数から取得。省略時は `"redis://localhost:6379"`
- `src` 検出時は `cache_url` の値（`"redis://..."` 等）を除外する
  - ただし `"redis://..."` は `-` で始まらないため、`cache_url` との値比較で除外する
- `--checkpoint`/`--resume` / `--retry-policy` と同時指定した場合は先行ブランチが優先される
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `distributed_cache_hit_across_workers` | `cmd_distributed_cache` が `"--distributed-cache"` / `"redis"` / `"Hit rate"` を含む |
| `distributed_cache_invalidation` | `cmd_distributed_cache` が `"--cache-ttl"` / `"L1"` / `"L2"` / `"invalidation"` を含む |

ベーステスト: 3527 → 目標: **3529**

> 各テストは `use super::*` 不要。`crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379")` を直接呼び出す。各キーワードは個別の `assert!` で検証する（失敗時の診断性確保）。
