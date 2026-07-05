# v34.6.0 — 実装プラン

## 方針

3 つの ctx インターフェース（DbCtx / HttpClient / StreamClient）を `runes/ctx/` に追加し、
移行ステータスページを整備する。
既存 rune ファイルの書き換えはスコープ外。
`cargo clean` は x.6.0 のため不要。

**前提**: `fav migrate --from-effects` は v13.10.0 で実装済み。
W022 lint は v34.5.0 で追加済み（既存 rune ファイルは W022 対象だが本バージョンでは修正しない）。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.5.0` → `34.6.0` に変更。

---

### Step 2: runes/ctx/db.fav 作成

```favnir
// runes/ctx/db.fav — DbCtx interface（v34.6.0）
// !DbRead / !DbWrite エフェクトの Capability Context 移行用インターフェース。
// `!DbRead` / `!DbWrite` を使用している関数は ctx.db を通じて DB 操作を行う。

interface DbCtx {
    fn execute(ctx: DbCtx, sql: String, params: String) -> Result<Unit, String>
    fn query_raw(ctx: DbCtx, sql: String, params: String) -> Result<String, String>
    fn transaction(ctx: DbCtx, action: (DbCtx) -> Result<Unit, String>) -> Result<Unit, String>
}
```

---

### Step 3: runes/ctx/http.fav 作成

```favnir
// runes/ctx/http.fav — HttpClient interface（v34.6.0）
// !Http エフェクトの Capability Context 移行用インターフェース。
// `!Http` を使用している関数は ctx.http を通じて HTTP 操作を行う。

interface HttpClient {
    fn get(ctx: HttpClient, url: String) -> Result<String, String>
    fn post(ctx: HttpClient, url: String, body: String) -> Result<String, String>
    fn put(ctx: HttpClient, url: String, body: String) -> Result<String, String>
    fn delete(ctx: HttpClient, url: String) -> Result<String, String>
}
```

---

### Step 4: runes/ctx/stream.fav 作成

```favnir
// runes/ctx/stream.fav — StreamClient interface（v34.6.0）
// !Stream エフェクトの Capability Context 移行用インターフェース。
// `!Stream` を使用している関数は ctx.stream を通じてストリーム操作を行う。

interface StreamClient {
    fn produce(ctx: StreamClient, topic: String, key: String, value: String) -> Result<Unit, String>
    fn consume(ctx: StreamClient, topic: String, group: String) -> Result<List<String>, String>
    fn commit(ctx: StreamClient, topic: String, group: String) -> Result<Unit, String>
}
```

---

### Step 5: site/content/docs/runes/ctx-migration-status.mdx 作成

```markdown
---
title: "Rune ctx 移行ステータス"
description: "Favnir v34.x における !Effect → Capability Context 移行の進捗状況"
---

# Rune ctx 移行ステータス

v34.5.0〜v34.7.0 シリーズで `!Effect` アノテーションを Capability Context（`ctx` パラメータ）に移行する。

## 追加済み ctx インターフェース

| インターフェース | ファイル | 対応 `!Effect` | 追加バージョン |
|---|---|---|---|
| `IoCtx` | `runes/ctx/io.fav` | `!Io` | v34.5.0 |
| `DbCtx` | `runes/ctx/db.fav` | `!DbRead` / `!DbWrite` | v34.6.0 |
| `HttpClient` | `runes/ctx/http.fav` | `!Http` | v34.6.0 |
| `StreamClient` | `runes/ctx/stream.fav` | `!Stream` | v34.6.0 |

## 移行対象 Rune ファイル（v34.7 以降）

| Rune ファイル | 使用エフェクト | 状態 |
|---|---|---|
| `runes/postgres/client.fav` | `!Postgres` | 未移行（v34.7 予定） |
| `runes/redis/redis.fav` | `!Redis` | 未移行（v34.7 予定） |
| `runes/kafka/kafka.fav` | `!Stream` | 未移行（v34.7 予定） |
| その他 25+ ファイル | 各種 | 未移行（v34.7 予定） |

## 移行手順

W022 警告が表示されたファイルを移行するには:

```bash
# ドライラン（変更内容を確認）
fav migrate --from-effects --dry-run src/pipeline.fav

# インプレース変換
fav migrate --from-effects --in-place src/pipeline.fav

# ディレクトリ全体
fav migrate --from-effects --dir runes/
```

詳細は [migration-effects](./tools/migration-effects) ガイドを参照。

## AppCtx フィールド一覧

```favnir
// AppCtx は全 ctx フィールドを持つ汎用コンテキスト
// ctx.io     : IoCtx        — 標準入出力・ファイル操作
// ctx.db     : DbCtx        — DB 読み書き（Postgres / MySQL 等）
// ctx.http   : HttpClient   — HTTP 通信
// ctx.stream : StreamClient — Kafka / Kinesis 等ストリーム
```
```

---

### Step 6: driver.rs 更新

1. `cargo_toml_version_is_34_5_0` を空スタブ化
2. `v345000_tests` 直後・`// ── v31.7.0 tests` の前に `v346000_tests` を挿入

挿入位置の確認:

```bash
grep -n "v345000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
```

```rust
// ── v34.6.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v346000_tests {
    #[test]
    fn cargo_toml_version_is_34_6_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.6.0"), "Cargo.toml must contain '34.6.0'");
    }

    #[test]
    fn db_ctx_rune_exists() {
        let src = include_str!("../../runes/ctx/db.fav");
        assert!(
            src.contains("DbCtx"),
            "runes/ctx/db.fav must define DbCtx interface"
        );
    }

    #[test]
    fn http_ctx_rune_exists() {
        let src = include_str!("../../runes/ctx/http.fav");
        assert!(
            src.contains("HttpClient"),
            "runes/ctx/http.fav must define HttpClient interface"
        );
    }

    #[test]
    fn stream_ctx_rune_exists() {
        let src = include_str!("../../runes/ctx/stream.fav");
        assert!(
            src.contains("StreamClient"),
            "runes/ctx/stream.fav must define StreamClient interface"
        );
    }

    #[test]
    fn ctx_migration_status_page_exists() {
        let src = include_str!("../../site/content/docs/runes/ctx-migration-status.mdx");
        assert!(
            src.contains("DbCtx"),
            "ctx-migration-status.mdx must list DbCtx interface"
        );
    }
}
```

**注意**: `use super::*` は**不要**（`include_str!` のみ使用）。

---

### Step 7: CHANGELOG.md 更新

```markdown
## [v34.6.0] — 2026-07-04

### Added
- `runes/ctx/db.fav` — DbCtx interface 定義（`!DbRead`/`!DbWrite` → `ctx.db` 移行用）
- `runes/ctx/http.fav` — HttpClient interface 定義（`!Http` → `ctx.http` 移行用）
- `runes/ctx/stream.fav` — StreamClient interface 定義（`!Stream` → `ctx.stream` 移行用）
- `site/content/docs/runes/ctx-migration-status.mdx` — ctx 移行ステータスサマリーページ

### Changed
- `versions/current.md` — 最新安定版を v34.6.0 に更新
```

---

### Step 8: benchmarks/v34.6.0.json 作成

```json
{
  "version": "34.6.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2566,
  "tests_failed": 0,
  "notes": "runes/ctx/db.fav (DbCtx) 追加。runes/ctx/http.fav (HttpClient) 追加。runes/ctx/stream.fav (StreamClient) 追加。ctx-migration-status.mdx 追加。v346000_tests 5 件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

---

### Step 9: versions/current.md 更新

- `最新安定版` 行: `**v34.5.0** — !Effect 廃止・コンテキスト構文統一` → `**v34.6.0** — Rune ファイル ctx 移行`
- `cargo install` 行: `"34.5.0"` → `"34.6.0"`
- `進行中バージョン`: `なし（v34.5.0 完了直後）` → `なし（v34.6.0 完了直後）`
- `次に切る版`: `**v34.6.0**` → `**v34.7.0** — ドキュメント・examples ctx 移行`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v346000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.6.0.json` の `tests_passed` を実測値で確定
  （実測値が想定 2566 と異なる場合は spec.md の完了条件も更新する）
- `tasks.md` を COMPLETE に更新
