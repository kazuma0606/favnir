# v25.3.0 仕様書 — redis Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.3.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | redis Rune の「動く Rune」5 条件達成 |
| 依存関係 | v25.1.0（examples/ ディレクトリが v25.1.0 で作成済み。ロードマップの「v25.1 と並行可能」に準拠） |
| 目標テスト数 | 1993 件（+7 件 ≥ ロードマップ最小 5 件） |

---

## 背景と目的

v25.2.0 で s3 Rune を実質化した。次は「キャッシュ・セッション・レート制限・Pub/Sub のハブ」として
あらゆるデータパイプラインで使われる Redis を「動く Rune」の 5 条件を満たすよう実質化する。

既存の `runes/redis/redis.fav` は v24.5.0 で追加されたスタブのみ（関数定義なし）。
また `!Cache` エフェクトは既存（v7.3.0）だが、これは**インメモリキャッシュ用**であり
Redis（外部サービス）とは別物。本バージョンでは新しい `!Redis` エフェクトを追加する。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `REDIS_URL` 環境変数（例: `redis://localhost:6379/0`）経由で接続確立 |
| 2 | read | `Redis.get` / `Redis.rpop` / `Redis.subscribe_once` |
| 3 | write | `Redis.set` / `Redis.del` / `Redis.incr` / `Redis.lpush` / `Redis.publish` |
| 4 | error | `Result<T, String>` 統一、エラーメッセージに key / channel を含む |
| 5 | test | `v253000_tests` 7 件 PASS + `examples/redis_rate_limiter.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/redis/redis.fav` | スタブのみ（関数なし） | v24.5.0 で追加 |
| `Effect::Redis` | **未定義** | v25.3.0 で追加（`ast.rs`） |
| `Redis.*_raw` primitives | **なし** | v25.3.0 で追加（`vm.rs`） |
| `Cache.*_raw` primitives | 実装済み（別物） | インメモリ用、Redis とは独立 |
| `redis` crate | **未追加** | v25.3.0 で `Cargo.toml` に追加 |

---

## 機能仕様

### 型定義

```favnir
// 接続 URL ラッパー型（"redis://host:port/db" 形式）
// runes/redis/redis.fav に直接定義（単一ファイル Rune）
// 将来ディレクトリ構成に移行する場合は runes/redis/connection.fav に移動予定
type RedisConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Redis.connect` | `(url: String) -> Result<RedisConn, String> !Redis` | 接続確立 |
| `Redis.get` | `(conn: RedisConn, key: String) -> Result<String, String> !Redis` | 値取得（存在しない場合 `Result.err("nil")`） |
| `Redis.set` | `(conn: RedisConn, key: String, value: String, ttl_secs: Int) -> Result<Unit, String> !Redis` | 値設定（ttl_secs = 0 で無期限） |
| `Redis.del` | `(conn: RedisConn, key: String) -> Result<Int, String> !Redis` | 削除（削除件数を返す） |
| `Redis.incr` | `(conn: RedisConn, key: String) -> Result<Int, String> !Redis` | INCR（カウンタ・レート制限用） |
| `Redis.lpush` | `(conn: RedisConn, key: String, value: String) -> Result<Int, String> !Redis` | リスト先頭追加（リスト長を返す） |
| `Redis.rpop` | `(conn: RedisConn, key: String) -> Result<String, String> !Redis` | リスト末尾取得（存在しない場合 `Result.err("nil")`） |
| `Redis.publish` | `(conn: RedisConn, channel: String, msg: String) -> Result<Int, String> !Redis` | Pub/Sub 送信（受信者数を返す） |
| `Redis.subscribe_once` | `(conn: RedisConn, channel: String) -> Result<String, String> !Redis` | Pub/Sub 1 件受信（ブロッキング、タイムアウト 30 秒） |

> **注意（ロードマップからの意図的逸脱）**: ロードマップ v25.3 の `Redis.subscribe(conn, channel, fn)` は
> VM からクロージャを呼び出せない制約により **v25.3.0 では `subscribe_once`（1 件受信）として実装する**。
> ループ型 `subscribe` は将来バージョンで別関数として追加する（破壊的変更なし）。
>
> **`subscribe_once` タイムアウト仕様**:
> - タイムアウト: 30 秒（`set_read_timeout(Some(Duration::from_secs(30)))`）
> - タイムアウト時の戻り値: `Result.err("timeout: no message received within 30s")`

---

## エフェクト追加仕様（`!Redis`）

v25.3.0 で `Effect::Redis` を新たに追加する。これは `!Cache`（インメモリ）と別物。

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `Effect` enum に `Redis` バリアント追加 |
| `fav/src/middle/checker.rs` | `ns_to_effect` / `require_redis_effect` / `redis_builtin_fns` 追加 |
| `fav/src/middle/reachability.rs` | `Effect::*` 網羅的 match に `Redis` 追加（漏れるとコンパイルエラー） |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::*` 網羅的 match に `Redis` 追加（漏れるとコンパイルエラー） |
| `fav/src/emit_python.rs` | `Effect::Redis => Some("@redis")` を追加（既存の非網羅的 match に明示アームとして追加） |
| `fav/src/lineage.rs` | `Effect::Redis` のリネージ追跡追加 |
| `fav/src/lint.rs` | `effect_to_str` 網羅的 match に `Effect::Redis` 追加（漏れるとコンパイルエラー） |
| `fav/src/error_catalog.rs` | E0320「undeclared !Redis effect」追加 |
| `fav/src/fmt.rs` | `Effect::Redis` の表示文字列追加 |

> **注意**: `emit_python.rs` の `effect_to_python_annotation` は既に一部のエフェクト（`Gcp` / `Stream` 等）が
> `Unknown` キャッチオールで処理されており非網羅的な状態。`Effect::Redis` はキャッチオールに任せず
> 明示的なアーム（`Effect::Redis => Some("@redis".to_string())`）として追加する。

---

## Redis クライアント実装方針

- `redis = { version = "0.25", default-features = false, features = ["tcp_nodelay"] }` を
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加
- 同期 API（`redis::Client::open` → `client.get_connection()`）を使用
- `REDIS_URL` 環境変数を優先、未設定時は `redis://127.0.0.1:6379/0` をフォールバック
- `connect_raw` は URL を `RedisConn` にラップ（実際の接続確立は各 raw primitive 内で実施）
- `cfg(not(target_arch = "wasm32"))` ガードを全 Redis primitive に付与

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0320 | UndeclaredRedisEffect | `!Redis` エフェクトなしで Redis 系 Rune を呼び出した場合 |

---

## `examples/redis_rate_limiter.fav`

```favnir
import rune "redis"

// ── Redis を使ったレート制限デモ (v25.3.0) ──────────────────────────────────
// 前提: docker run -p 6379:6379 redis:7
// 実行: fav run examples/redis_rate_limiter.fav

stage CheckRateLimit: String -> Bool !Redis = |user_id| {
    bind conn    <- Redis.connect("redis://localhost:6379/0")
    bind key     <- Result.ok("rate:" + user_id)
    bind count   <- Redis.incr(conn, key)
    bind _       <- Redis.set(conn, key, String.from_int(count), 60)
    Result.ok(count <= 10)
}

stage RecordEvent: String -> Unit !Redis = |event| {
    bind conn <- Redis.connect("redis://localhost:6379/0")
    Redis.lpush(conn, "events", event)
}

pipeline RateLimitPipeline = CheckRateLimit |> RecordEvent
```

---

## やらないこと（スコープ外）

- Redis Cluster / Sentinel 対応
- コネクションプール（`Pool.create` は v25.x 以降）
- Pipeline（MULTI/EXEC トランザクション）
- `subscribe` コールバックループ（`subscribe_once` のみ実装）
- TLS 接続（`redis+tls://` スキーム）
- `Redis.get<T>` ジェネリクス（`String` 返却のみ。JSON デシリアライズは呼び出し元で実施）

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `Redis.connect` が `runes/redis/redis.fav` に実装済み |
| 2 | `Redis.get` / `Redis.set` / `Redis.del` / `Redis.incr` が `runes/redis/redis.fav` に実装済み |
| 3 | `Redis.lpush` / `Redis.rpop` / `Redis.publish` / `Redis.subscribe_once` が実装済み |
| 4 | `Redis.*_raw` VM primitives（9 件）が `fav/src/backend/vm.rs` に存在する |
| 5 | `Effect::Redis` が `fav/src/ast.rs` に存在し、E0320 が `error_catalog.rs` に存在する（`cargo build` で exhaustive match を確認） |
| 6 | `examples/redis_rate_limiter.fav` が存在し `import rune "redis"` + `incr` + `lpush` を含む |
| 7 | `CHANGELOG.md` に `[v25.3.0]` エントリが存在する |
| 8 | `site/content/docs/runes/redis.mdx` に新規 API（connect / get / set / incr / lpush / rpop / publish / subscribe_once）が記載済み |
| 9 | `cargo test v253000` で 7 件すべて PASS（Effect::Redis テスト含む） |
| 10 | 総テスト数 ≥ 1993 件 |

---

## 検証コマンド

```bash
cd fav && cargo test v253000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
