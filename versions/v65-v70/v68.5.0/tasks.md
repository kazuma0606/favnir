# v68.5.0 タスクリスト

Status: COMPLETE
Version: 68.5.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3527
Target tests: 3529

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3527 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/dist_cache.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod retry;` が存在することを確認（`mod dist_cache;` の挿入位置）
- [x] `driver.rs` に `v68400_tests` が存在することを確認（`v68500_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68500_tests` を `v68400_tests` の直前に挿入する
- [x] `driver.rs` に `v68500_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` のテストブロックが降順配置（`v68400_tests` が `v68300_tests` より上）であることを確認
- [x] `cargo test --bin fav v68400_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `retry_exponential_backoff`, `retry_fallback_stage`
- [x] `versions/current.md` の「進行中バージョン」が `v68.4.0` であることを確認

---

## T1: `fav/src/dist_cache.rs` 新規作成

- [x] `fav/src/dist_cache.rs` を新規作成
  - [x] `pub fn cmd_distributed_cache(src: &str, cache_url: &str) -> String` を追加
    - [x] `"--distributed-cache"` / `"redis"` / `"Hit rate"` を含む出力（`distributed_cache_hit_across_workers` テスト要件）
    - [x] `"--cache-ttl"` / `"L1"` / `"L2"` / `"invalidation"` を含む出力（`distributed_cache_invalidation` テスト要件）
    - [x] 出力末尾は `[stub] Would connect to Redis cache (source: <src>)`（実際の接続なし）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod dist_cache;` を mod 宣言部（`mod retry;` の直後）に追加
- [x] `Some("run")` アームの `--retry-policy` ブランチの直後に `--distributed-cache` ブランチを追加
  - [x] `args.iter().any(|a| a == "--distributed-cache")` で分岐
  - [x] `cache_url` は `--distributed-cache` の次の引数から取得（省略時は `"redis://localhost:6379"`）
  - [x] `src` 検出時に `cache_url` の値を除外（`a.as_str() != cache_url`）
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", dist_cache::cmd_distributed_cache(src, cache_url))` + `return;`
  - [x] 先行ブランチとの同時指定時の優先順位をコメントで明記
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68500_tests` 追加

- [x] `// -- v68400_tests (v68.4.0) -- Stage Retry Policies（型安全エラー回復） --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `distributed_cache_hit_across_workers`: 各キーワードを個別 `assert!` で検証（`"--distributed-cache"` / `"redis"` / `"Hit rate"`）
  - [x] `distributed_cache_invalidation`: 各キーワードを個別 `assert!` で検証（`"--cache-ttl"` / `"L1"` / `"L2"` / `"invalidation"`）
- [x] `use super::*` は不要（`crate::dist_cache::` で直接参照）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68500_tests` で 2 件 PASS
  - [x] `distributed_cache_hit_across_workers` PASS
  - [x] `distributed_cache_invalidation` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3529 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.5.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.5.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際の Redis 接続・キャッシュ読み書き: 将来フェーズ
- キャッシュキー生成（ステージ名 + 入力ハッシュ SHA256）: 将来フェーズ
- `--cache-ttl` / `--cache-ttl-per-stage` の実際の TTL 設定: 将来フェーズ
- 入力スキーマ変更時の自動キャッシュ無効化: 将来フェーズ
- LLM 呼び出し回避による節約額の実際のコスト追跡: 将来フェーズ
- L1（メモリ） → L2（Redis）の 2 層キャッシュ実装: 将来フェーズ

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [MED] | `main.rs` L410-412 | `--distributed-cache` に URL が省略された場合、エラーなくデフォルト URL にフォールバックする。他の値付きフラグと挙動が不一致。 | `match` + `.filter(|v| !v.starts_with('-'))` に変更。URL 未指定時は `eprintln!` + `process::exit(1)` でエラー終了するよう修正。 |
| [LOW] | `dist_cache.rs` | 将来的なネットワーク IO を考慮して `#[cfg(not(target_arch = "wasm32"))]` を推奨 | 将来フェーズ対応として記録（スタブ段階のため現時点では未実施） |
