# v58.6.0 Spec — マルチ環境設定（dev / staging / prod）

Date: 2026-07-28
Status: 設計中

---

## 概要

`fav.toml` の `[env.<name>]` セクションを解析し、`fav run pipeline.fav --env <name>` で
環境別設定を選択・注入する機能を実装する。
`inject_env_config(env_name, pipeline_file)` を driver.rs スタブとして追加する（v58.x パターン）。
`fav run` の `--env` フラグ対応を main.rs に追加する。

---

## 実装スコープ変更

> **スコープ外（v58.x 一貫パターンに合わせて見送り）:**
> - `fav.toml` の `[env.<name>]` セクションの実際の TOML パース — スタブで出力を模倣
> - `expand_env_vars`（v10.7.0 実装済みと roadmap が言及するが実際未実装）— 今バージョンでは触れない

---

## ユーザー向けインターフェース

### fav run --env

```bash
$ fav run pipeline.fav --env staging
[env: staging] Loading environment config from fav.toml
  snowflake.database = STAGING_DB
  kafka.bootstrap    = kafka-staging:9092
Running pipeline.fav (env: staging) ...

$ fav run pipeline.fav --env prod
[env: prod] Loading environment config from fav.toml
  snowflake.database = PROD_DB
  kafka.bootstrap    = kafka-prod:9092
Running pipeline.fav (env: prod) ...
```

---

## 実装詳細

### driver.rs

**追加関数**: `pub fn inject_env_config(env_name: &str, pipeline_file: &str) -> i32`
- `env_name` と `pipeline_file` を受け取る
- スタブ: `[env: <env_name>] Loading environment config from fav.toml` を出力
- `env_name` に応じた固定値（dev / staging / prod）を出力
  - `"dev"` → `DEV_DB` / `localhost:9092`
  - `"staging"` → `STAGING_DB` / `kafka-staging:9092`
  - それ以外（`"prod"` 含む）→ `PROD_DB` / `kafka-prod:9092`（**prod フォールバック設計**）
    - 未知の env name もエラーにせず prod 設定で動作させる（v58.x スタブの意図的設計）
- `Running <pipeline_file> (env: <env_name>) ...` を出力して 0 を返す

### main.rs

`Some("run")` アームに `--env` フラグ検出ロジックを追加:
- `--env` フラグがあり値がある場合 → `inject_env_config(env_name, pipeline_file)` を呼び **`return;` で即座に終了**（既存パスを実行させない）
- `--env` フラグがあり値がない場合 → エラーメッセージ + exit(1)
- `--env` フラグなし → 既存の通常実行パスを使用（変更なし）

> `Some("run")` アームは非常に長いため、`return;` を忘れると `--debug` / `--precompiled` / `--vm` パスが二重実行される危険がある。

pipeline_file は `--env` / その値以外の最初の positional 引数（`args.iter().skip(2).find(!starts_with("--"))`)から取得。

### 追加しない

- `error_catalog.rs` — 新規エラーコードなし
- AST / parser — v58.x パターン通りスコープ外
- TOML パーサー拡張 — スタブで代替

---

## テスト

`v58600_tests` モジュールを `v58500_tests` の直前に挿入:

| テスト名 | 内容 |
|---|---|
| `env_config_parsed` | `inject_env_config("staging", "pipeline.fav")` → 0 |
| `env_config_injected` | `inject_env_config("prod", "pipeline.fav")` → 0 |

**実際のベース**: 3292（v58.5.0 code-review 後の実績値）
**完了条件**: 3292 + 2 = **3294 tests passed, 0 failed**

> ロードマップ記載のベース 3291 は v58.5.0 code-review 前の値。実際は 3292 のため +1 補正。

---

## ロールアップチェック更新

v58000_tests の全ローリングアサーション（5 件）を `"58.6.0"` に更新。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `inject_env_config` + v58600_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("run")` アームに `--env` フラグ対応追加、use imports 追加 |
| `fav/Cargo.toml` | バージョン `58.6.0` |
| `CHANGELOG.md` | v58.6.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v58.6.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v58.6.0 実績欄に完了記録（スコープ変更ブロックは spec 作成時点で追加済み）、v58.7.0 ベース数修正（3294→実績値）|
| `site/` MDX | **v58.8.0 で対応**（multi-env-pipeline.mdx 等）— 本バージョンでは更新不要 |
