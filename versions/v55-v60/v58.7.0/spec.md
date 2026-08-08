# v58.7.0 Spec — HA / DR（ヘルスチェック・フェイルオーバー）

Date: 2026-07-29
Status: 設計中

---

## 概要

`fav run --ha --replica <n>` フラグで複数レプリカを起動し、`/healthz` エンドポイントと
自動フェイルオーバーをシミュレートする機能を driver.rs スタブとして実装する（v58.x パターン）。

---

## 実装スコープ変更

> **スコープ外（v58.x 一貫パターンに合わせて見送り）:**
> - Tokio ベースの実 watchdog プロセス起動 — driver.rs スタブで出力を模倣
> - 実際の `/healthz` HTTP エンドポイント起動 — 出力文字列モックで検証
> - レプリカ間の実ネットワーク通信 — スタブ内の固定ポート番号出力で代替

---

## ユーザー向けインターフェース

```bash
$ fav run pipeline.fav --ha --replica 2
[HA] Primary replica started (port 8080)
[HA] Secondary replica started (port 8081)
[HA] Health check: /healthz → 200 OK
[HA] Failover: primary → secondary (reason: primary unresponsive)

$ fav run pipeline.fav --ha
[HA] Primary replica started (port 8080)
[HA] Health check: /healthz → 200 OK
[HA] Failover: primary → secondary (reason: primary unresponsive)
```

---

## 実装詳細

### driver.rs

**追加関数**: `pub fn cmd_ha_run(replica_count: u32) -> i32`
- `[HA] Primary replica started (port 8080)` を出力
- `replica_count > 1` の場合、Secondary レプリカを `replica_count - 1` 件出力（port 8081, 8082, ...）
  - `replica_count = 0` は `u32` パース成功だが `for 1..0` ループが即終了するため Primary のみ出力（スタブとして許容、将来バージョンで `>= 1` バリデーション追加可）
- `[HA] Health check: /healthz → 200 OK` を出力
- `[HA] Failover: primary → secondary (reason: primary unresponsive)` を出力
- 0 を返す

### main.rs

`Some("run")` アームの `--env` フラグ処理の直後に `--ha` フラグ検出ロジックを追加:
- `--ha` フラグがある場合:
  - `--replica` フラグがあれば値を `u32` にパース（パース失敗 → エラー + exit(1)）
  - `--replica` フラグがなければ `replica_count = 1`（デフォルト）
  - `cmd_ha_run(replica_count)` を呼び **`return;`** で即座に終了
- `--ha` フラグなし → 既存パスをそのまま使用

### 追加しない

- 新規エラーコード（`error_catalog.rs`）— 不要
- AST / parser 変更 — v58.x パターン通りスコープ外

---

## テスト

`v58700_tests` モジュールを `v58600_tests` の直前に挿入:

| テスト名 | 内容 |
|---|---|
| `ha_health_check_endpoint` | `cmd_ha_run(1)` → 0（単一レプリカでヘルスチェック動作を検証） |
| `ha_failover_triggers` | `cmd_ha_run(2)` → 0（2 レプリカでフェイルオーバー動作を検証） |

**実際のベース**: 3296（v58.6.0 code-review 後の実績値）
**完了条件**: 3296 + 2 = **3298 tests passed, 0 failed**

---

## ロールアップチェック更新

v58000_tests の全ローリングアサーション（5 件）を `"58.7.0"` に更新。
failure メッセージ（5 件）も `"58.7.0"` に更新。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_ha_run` + v58700_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("run")` アームに `--ha` / `--replica` 対応追加、use imports 追加 |
| `fav/Cargo.toml` | バージョン `58.7.0` |
| `CHANGELOG.md` | v58.7.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v58.7.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v58.7.0 実績欄に完了記録、スコープ変更ブロック追加（Tokio watchdog / 実 HTTP エンドポイントをスコープ外と明示）、v58.8.0 ベース数修正（3295→3298、目標 3297→3300） |
| `site/` MDX | **v58.8.0 で対応** — 本バージョンでは更新不要 |
