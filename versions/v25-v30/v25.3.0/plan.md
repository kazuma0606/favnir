# v25.3.0 実装計画 — redis Rune 実質化

## 実装ステップ

### Step 0: Cargo.toml bump + redis crate 追加

`fav/Cargo.toml`:
- `version = "25.2.0"` → `version = "25.3.0"`
- `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに以下を追加:

```toml
redis = { version = "0.25", default-features = false, features = ["tcp_nodelay"] }
```

> **検証**: 追加後に `cargo build` を実行し、`get_connection()` / Pub/Sub が `tcp_nodelay` のみで
> ビルドできることを確認する。非同期サポートが必要になった場合は `features = ["tokio-comp"]` を追加。

---

### Step 1: `Effect::Redis` 追加（ast.rs）

**ファイル**: `fav/src/ast.rs`

`Effect` enum の `Postgres` の直後（または末尾、`Unknown` の前）に追加:

```rust
/// v25.3.0: Redis Rune effect
Redis,
```

---

### Step 2: checker.rs 更新

**ファイル**: `fav/src/middle/checker.rs`

#### 2-1: `ns_to_effect` に Redis 追加

既存の `"Postgres" => Effect::Postgres` パターンと同様に追加:

```rust
"Redis" => Effect::Redis,
```

#### 2-2: Redis 組み込み関数スキームを checker に登録

既存の postgres_fn / snowflake_fn パターンと同様に `redis_builtin_fns` を追加。
返す型スキームは `Result<String, String>` / `Result<Int, String>` / `Result<Unit, String>` のいずれか。

#### 2-3: `require_redis_effect` 追加

```rust
fn require_redis_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Redis)) {
        self.type_error(
            "E0320",
            "Redis.* call requires `!Redis` effect on enclosing fn/stage",
            span,
        );
    }
}
```

`"Redis"` namespace の呼び出し検出時（`check_call` / `check_prim_call` 等）に `require_redis_effect` を呼ぶ。

---

### Step 3: emit_python.rs / lineage.rs / fmt.rs / reachability.rs / ast_lower_checker.rs / lint.rs 更新

`Effect::Redis` を追加すると以下のファイルで網羅的 match が存在するためすべて更新が必要。
更新漏れはコンパイルエラーで検出される。

**ファイル**: `fav/src/emit_python.rs`

```rust
// NOTE: effect_to_python_annotation は既に非網羅的（Gcp/Stream 等が Unknown キャッチオールで処理済み）
// Redis は明示的アームとして追加する（キャッチオールに任せない）
Effect::Redis => Some("@redis".to_string()),
```

**ファイル**: `fav/src/lineage.rs`

```rust
Effect::Redis => "Redis",
```

**ファイル**: `fav/src/fmt.rs`

```rust
Effect::Redis => write!(f, "!Redis"),
```

**ファイル**: `fav/src/middle/reachability.rs`

`Effect::*` の網羅的 match（行 57-124 付近）に `Effect::Redis` アームを追加。
既存の `Effect::Postgres` アームと同じパターンで対応する。

**ファイル**: `fav/src/middle/ast_lower_checker.rs`

`ast::Effect::*` の網羅的 match（行 427-448 付近）に `Effect::Redis` アームを追加。

**ファイル**: `fav/src/lint.rs`

`effect_to_str` 関数の網羅的 match に `Effect::Redis => "Redis"` を追加。

---

### Step 4: error_catalog.rs 更新

**ファイル**: `fav/src/error_catalog.rs`

E0315（Postgres）の後に E0320 を追加:

```rust
ErrorEntry {
    code: "E0320",
    title: "undeclared !Redis effect",
    category: "effects",
    description: "A Redis operation was used in a function that does not declare `!Redis`.",
    example: "fn run(key: String) -> Result<String, String> {\n    Redis.get_raw(conn, key)  // E0320: !Redis not declared\n}",
    fix: "Add `!Redis` to the function signature: `fn run(key: String) -> Result<String, String> !Redis`.",
},
```

---

### Step 5: `runes/redis/redis.fav` 更新

**ファイル**: `runes/redis/redis.fav`（既存スタブを全面更新）

```favnir
// runes/redis/redis.fav — Redis Rune (v25.3.0)
// 使い方: import rune "redis"
// 前提: docker run -p 6379:6379 redis:7

// 接続 URL ラッパー型（"redis://host:port/db" 形式）
type RedisConn(String)

// ── 接続 ─────────────────────────────────────────────────────────────────────

public fn connect(url: String) -> Result<RedisConn, String> !Redis {
    Redis.connect_raw(url)
}

// ── 基本操作 ─────────────────────────────────────────────────────────────────

// get — 値取得（存在しない場合は Result.err("nil")）
public fn get(conn: RedisConn, key: String) -> Result<String, String> !Redis {
    Redis.get_raw(conn, key)
}

// set — 値設定（ttl_secs = 0 で無期限）
public fn set(conn: RedisConn, key: String, value: String, ttl_secs: Int) -> Result<Unit, String> !Redis {
    Redis.set_raw(conn, key, value, ttl_secs)
}

// del — 削除（削除件数を返す）
public fn del(conn: RedisConn, key: String) -> Result<Int, String> !Redis {
    Redis.del_raw(conn, key)
}

// incr — インクリメント（カウンタ・レート制限用）
public fn incr(conn: RedisConn, key: String) -> Result<Int, String> !Redis {
    Redis.incr_raw(conn, key)
}

// ── リスト操作 ────────────────────────────────────────────────────────────────

// lpush — リスト先頭追加（リスト長を返す）
public fn lpush(conn: RedisConn, key: String, value: String) -> Result<Int, String> !Redis {
    Redis.lpush_raw(conn, key, value)
}

// rpop — リスト末尾取得（存在しない場合は Result.err("nil")）
public fn rpop(conn: RedisConn, key: String) -> Result<String, String> !Redis {
    Redis.rpop_raw(conn, key)
}

// ── Pub/Sub ──────────────────────────────────────────────────────────────────

// publish — チャンネルにメッセージ送信（受信者数を返す）
public fn publish(conn: RedisConn, channel: String, msg: String) -> Result<Int, String> !Redis {
    Redis.publish_raw(conn, channel, msg)
}

// subscribe_once — チャンネルからメッセージを 1 件受信（ブロッキング）
// NOTE: ループ型 subscribe(conn, channel, fn) は将来バージョンで別関数として追加予定
public fn subscribe_once(conn: RedisConn, channel: String) -> Result<String, String> !Redis {
    Redis.subscribe_once_raw(conn, channel)
}
```

---

### Step 6: `fav/src/backend/vm.rs` 更新（Redis primitives 9 件追加）

**挿入位置**: 既存 `Cache.del_prefix_raw` ブロック末尾（または Cache セクションの後）の次に
`// ── Redis (v25.3.0)` セクションとして追加。

実装は `#[cfg(not(target_arch = "wasm32"))]` の vm.rs 内の他の外部サービス primitive と同様。

#### ヘルパー

```rust
fn redis_conn(url: &str) -> Result<redis::Connection, String> {
    let client = redis::Client::open(url)
        .map_err(|e| format!("Redis connect error: {}", e))?;
    client.get_connection()
        .map_err(|e| format!("Redis connection error: {}", e))
}

fn redis_url_from_conn(v: VMValue) -> Result<String, String> {
    match v {
        VMValue::Str(s) => Ok(s),
        _ => Err("Redis: conn must be a String (RedisConn)".to_string()),
    }
}
```

#### 各 primitive の概要

| primitive | Redis コマンド | 返り値 VMValue |
|---|---|---|
| `"Redis.connect_raw"` | 接続確認（PING） | `Str(url)` をそのまま wrap |
| `"Redis.get_raw"` | GET key | `Str(value)` or err("nil") |
| `"Redis.set_raw"` | SET key value [EX ttl] | `Unit` |
| `"Redis.del_raw"` | DEL key | `Int(count)` |
| `"Redis.incr_raw"` | INCR key | `Int(new_value)` |
| `"Redis.lpush_raw"` | LPUSH key value | `Int(list_len)` |
| `"Redis.rpop_raw"` | RPOP key | `Str(value)` or err("nil") |
| `"Redis.publish_raw"` | PUBLISH channel msg | `Int(receivers)` |
| `"Redis.subscribe_once_raw"` | SUBSCRIBE channel (1 recv) | `Str(message)` |

**注意**:
- `connect_raw` は URL の PING 確認のみ行い、接続文字列を `VMValue::Str` として返す
  （`PgConn` パターンと同様、実接続は各 raw primitive 内で都度確立）
- `get_raw` / `rpop_raw`: nil (key not found) は `err_vm(VMValue::Str("nil".to_string()))` で返す
- `set_raw`: `ttl_secs == 0` の場合は `SET key value`（EX なし）、`> 0` の場合は `SET key value EX ttl`
- `subscribe_once_raw`: `redis::PubSub` を使って 1 件だけ受信してから drop

#### subscribe_once の実装方針

```rust
"Redis.subscribe_once_raw" => {
    // ... url と channel を取得
    let client = redis::Client::open(url.as_str())
        .map_err(|e| format!("Redis subscribe error: {}", e))?;
    // NOTE: redis crate v0.25 では `get_connection().as_pubsub()` は存在しない。
    //       正しくは `client.get_pubsub()` を使用する。
    let mut pubsub = client.get_pubsub()
        .map_err(|e| format!("Redis subscribe connection error: {}", e))?;
    // タイムアウト 30 秒（ブロッキング無限待機を防ぐ）
    pubsub.set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .map_err(|e| format!("Redis set_read_timeout error: {}", e))?;
    pubsub.subscribe(&channel)
        .map_err(|e| format!("Redis subscribe error: {}", e))?;
    let msg = pubsub.get_message()
        .map_err(|e| {
            let s = e.to_string();
            if s.contains("timed out") || s.contains("timeout") {
                "timeout: no message received within 30s".to_string()
            } else {
                format!("Redis get_message error: {}", s)
            }
        })?;
    let payload: String = msg.get_payload()
        .map_err(|e| format!("Redis payload error: {}", e))?;
    Ok(ok_vm(VMValue::Str(payload)))
}
```

> `get_pubsub()` が v0.25 で利用できない場合は `get_connection()` の戻り値に `.into_pubsub()` を
> チェーンする（`Connection` が `PubSub` に変換される）。実装時に redis crate v0.25 の API を確認すること。

---

### Step 7: `examples/redis_rate_limiter.fav` 作成

**ファイル**: `examples/redis_rate_limiter.fav`（新規作成）

```favnir
// examples/redis_rate_limiter.fav — Redis レート制限デモ (v25.3.0)
// 前提: docker run -p 6379:6379 redis:7
// 実行: fav run examples/redis_rate_limiter.fav

import rune "redis"

type RateResult = { allowed: Bool, count: Int }

// ── Stage 1: リクエスト数をインクリメントしてレート確認 ──────────────────────
stage CheckRateLimit: String -> RateResult !Redis = |user_id| {
    bind conn  <- Redis.connect("redis://localhost:6379/0")
    bind key   <- Result.ok("rate:" + user_id)
    bind count <- Redis.incr(conn, key)
    // count は Int、set の value は String のため String.from_int で変換
    bind _     <- Redis.set(conn, key, String.from_int(count), 60)
    Result.ok({ allowed: count <= 10, count: count })
}

// ── Stage 2: 結果をイベントキューに記録 ──────────────────────────────────────
stage RecordEvent: RateResult -> Unit !Redis = |result| {
    bind conn <- Redis.connect("redis://localhost:6379/0")
    bind msg  <- Result.ok("rate_check:" + result.allowed)
    Redis.lpush(conn, "events", msg)
}

// ── パイプライン ──────────────────────────────────────────────────────────────
pipeline RateLimitPipeline = CheckRateLimit |> RecordEvent
```

---

### Step 8: `CHANGELOG.md` 更新

```
## [v25.3.0] — 2026-06-25

### Added
- `Effect::Redis` — 新エフェクト追加（`!Cache` とは独立した外部 Redis 専用）
- E0320「undeclared !Redis effect」エラーコード追加
- `Redis.connect(url)` — RedisConn（接続 URL ラッパー）を返す
- `Redis.get / set / del / incr` — 基本 KV 操作
- `Redis.lpush / rpop` — リスト操作（キュー用途）
- `Redis.publish / subscribe_once` — Pub/Sub（1 件受信）
- `examples/redis_rate_limiter.fav` — Redis を使ったレート制限 E2E デモ
- `v253000_tests`（6 件）: connect / get / set / incr primitive 存在確認 + example + changelog
```

---

### Step 9: `site/content/docs/runes/redis.mdx` 作成・更新

`site/content/docs/runes/redis.mdx` を作成し、以下の API を記載する:
- `Redis.connect` / `RedisConn` 型
- `Redis.get` / `Redis.set` / `Redis.del` / `Redis.incr`
- `Redis.lpush` / `Redis.rpop`
- `Redis.publish` / `Redis.subscribe_once`（タイムアウト 30 秒の注記を含む）

---

### Step 10: `fav/src/driver.rs` 更新（v253000_tests 7 件追加）

```rust
// ── v253000_tests (v25.3.0) — redis Rune 実質化 ──────────────────────────────
#[cfg(test)]
mod v253000_tests {
    #[test]
    fn redis_rune_has_connect_fn() {
        let src = include_str!("../../runes/redis/redis.fav");
        assert!(src.contains("fn connect"), "redis.fav must contain 'fn connect'");
        assert!(src.contains("Redis.connect_raw"), "connect must call Redis.connect_raw");
    }

    #[test]
    fn redis_rune_has_get_set_del_incr() {
        let src = include_str!("../../runes/redis/redis.fav");
        assert!(src.contains("fn get"), "redis.fav must contain 'fn get'");
        assert!(src.contains("fn set"), "redis.fav must contain 'fn set'");
        assert!(src.contains("fn del"), "redis.fav must contain 'fn del'");
        assert!(src.contains("fn incr"), "redis.fav must contain 'fn incr'");
    }

    #[test]
    fn redis_rune_has_list_pubsub_fns() {
        let src = include_str!("../../runes/redis/redis.fav");
        assert!(src.contains("fn lpush"), "redis.fav must contain 'fn lpush'");
        assert!(src.contains("fn rpop"), "redis.fav must contain 'fn rpop'");
        assert!(src.contains("fn publish"), "redis.fav must contain 'fn publish'");
        assert!(src.contains("fn subscribe_once"), "redis.fav must contain 'fn subscribe_once'");
    }

    #[test]
    fn redis_primitives_exist_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"Redis.connect_raw\""), "vm.rs must have Redis.connect_raw");
        assert!(src.contains("\"Redis.get_raw\""), "vm.rs must have Redis.get_raw");
        assert!(src.contains("\"Redis.set_raw\""), "vm.rs must have Redis.set_raw");
        assert!(src.contains("\"Redis.incr_raw\""), "vm.rs must have Redis.incr_raw");
        assert!(src.contains("\"Redis.subscribe_once_raw\""), "vm.rs must have Redis.subscribe_once_raw");
    }

    #[test]
    fn redis_rate_limiter_example_exists() {
        let src = include_str!("../../examples/redis_rate_limiter.fav");
        assert!(src.contains("import rune \"redis\""), "example must import redis rune");
        assert!(src.contains("incr"), "example must use Redis.incr");
        assert!(src.contains("lpush"), "example must use Redis.lpush");
    }

    #[test]
    fn changelog_has_v25_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("v25.3.0"), "CHANGELOG.md must contain 'v25.3.0'");
        assert!(src.contains("Redis.connect"), "CHANGELOG must mention Redis.connect");
        assert!(src.contains("subscribe_once"), "CHANGELOG must mention subscribe_once");
    }

    #[test]
    fn effect_redis_exists_in_ast() {
        let src = include_str!("ast.rs");
        assert!(src.contains("Redis,"), "ast.rs must contain Effect::Redis variant");
    }
}
```

---

### Step 11: `benchmarks/v25.3.0.json` 作成

> **注意**: Step 10（テスト追加）完了後に作成し、`test_count` に実測値（1993）を記載する。

```json
{
  "version": "25.3.0",
  "timestamp": "2026-06-25T00:00:00Z",
  "metrics": {
    "test_count": 1993,
    "compile_hello_ms": 12,
    "compile_etl_ms": 45
  }
}
```

---

### Step 12: テスト実行

```bash
cd fav && cargo test v253000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## 実装順序まとめ

```
Step 0:  Cargo.toml（25.2.0 → 25.3.0、redis crate 追加。cargo build で features 確認）
Step 1:  ast.rs（Effect::Redis 追加）
Step 2:  checker.rs（ns_to_effect / require_redis_effect / redis_builtin_fns）
Step 3:  emit_python.rs / lineage.rs / fmt.rs / reachability.rs / ast_lower_checker.rs / lint.rs（Effect::Redis 対応）
Step 4:  error_catalog.rs（E0320 追加）
Step 5:  runes/redis/redis.fav（全面更新）
Step 6:  fav/src/backend/vm.rs（Redis primitives 9 件追加、subscribe_once はタイムアウト 30 秒付き）
Step 7:  examples/redis_rate_limiter.fav（新規作成、String.from_int 使用）
Step 8:  CHANGELOG.md（v25.3.0 エントリ追加）
Step 9:  site/content/docs/runes/redis.mdx（新規作成）
Step 10: fav/src/driver.rs（v253000_tests 7 件追加）
Step 11: benchmarks/v25.3.0.json（テスト実行後に実測値で作成）
Step 12: テスト実行・確認
```

---

## リスクと対応

| リスク | 対応 |
|---|---|
| `redis` crate v0.25 で `get_pubsub()` が利用できない場合 | `get_connection()?.into_pubsub()` に切り替える。Step 0 の `cargo build` で確認必須 |
| `subscribe_once` がメッセージなしの場合にブロック永続 | `set_read_timeout(Some(Duration::from_secs(30)))` で 30 秒タイムアウト。`Result.err("timeout: ...")` を返す |
| `Effect::Redis` 追加で網羅的 match がコンパイルエラー | reachability.rs / ast_lower_checker.rs / lint.rs の 3 ファイルが特に漏れやすい（Step 3 で明示的に対応） |
| `redis` crate が wasm32 でビルドエラー | `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加済み。primitive は既存パターンでガード |
| `Cache.*_raw` との混同 | `Cache.*_raw`（インメモリ）と `Redis.*_raw`（外部 Redis）は完全独立セクション。コメントで明記 |
| benchmarks の test_count が不正確 | Step 11（benchmark 作成）を Step 10（テスト追加）の後に配置。テスト実行後に実測値で記入 |
