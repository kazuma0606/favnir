# v34.6.0 — Spec

## 概要

**テーマ**: Rune ファイル ctx 移行（インターフェース定義フェーズ）

**方針**: `!DbRead`/`!DbWrite`/`!Http`/`!Stream` エフェクトの Capability Context 移行用インターフェース
（`DbCtx` / `HttpClient` / `StreamClient`）を `runes/ctx/` に追加し、
移行状況サマリーページを整備する。

既存の rune ファイル（postgres/client.fav 等）の `!Effect` 関数書き換えは **スコープ外**
（影響範囲が大きいため v34.7 以降で `fav migrate --from-effects --dir runes/` を使って対応）。
本バージョンは「移行先インターフェースの定義」に集中する。

---

## 背景

v34.5.0 で `runes/ctx/io.fav`（IoCtx）を追加し W022 警告を有効化した。
v34.6.0 では DB / HTTP / Stream の 3 ctx インターフェースを追加し、
Rune 開発者が ctx ベース実装を作成できる基盤を整える。

### 既存実装の確認

| 機能 | 実装バージョン | 状態 |
|---|---|---|
| `runes/ctx/mock_db.fav` | v13.2.0 | 実装済み |
| `runes/ctx/io.fav` (IoCtx) | v34.5.0 | 実装済み |
| `runes/ctx/db.fav` (DbCtx) | 未実装 | **本バージョンで新規作成** |
| `runes/ctx/http.fav` (HttpClient) | 未実装 | **本バージョンで新規作成** |
| `runes/ctx/stream.fav` (StreamClient) | 未実装 | **本バージョンで新規作成** |
| ctx 移行ステータスページ | 未実装 | **本バージョンで新規作成** |
| 既存 rune ファイルの `!Effect` 書き換え | 未実装 | **スコープ外（v34.7 以降）** |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| postgres/client.fav 書き換え | `!Postgres` → `ctx.db: PgConn` | **スコープ外** — リスクが大きいため v34.7 で対応 |
| redis/redis.fav 書き換え | `!Redis` → `ctx.redis: RedisClient` | **スコープ外** — 同上 |
| kafka/kafka.fav 書き換え | `!Stream` → `ctx.stream: StreamClient` | **スコープ外** — 同上 |
| 50+ rune ファイル一括移行 | `fav migrate --from-effects --dir runes/` | **スコープ外** — v34.7 以降で実施 |
| DbCtx interface 定義 | `runes/ctx/db.fav` | **本バージョンで追加** |
| HttpClient interface 定義 | `runes/ctx/http.fav` | **本バージョンで追加** |
| StreamClient interface 定義 | `runes/ctx/stream.fav` | **本バージョンで追加** |

---

## 実装スコープ

### 新規ファイル

```
runes/ctx/db.fav                                   DbCtx interface 定義
runes/ctx/http.fav                                 HttpClient interface 定義
runes/ctx/stream.fav                               StreamClient interface 定義
site/content/docs/runes/ctx-migration-status.mdx   ctx 移行ステータスサマリー
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.5.0` → `34.6.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_5_0` をスタブ化、`v346000_tests` 5 件追加
3. `benchmarks/v34.6.0.json` — 新規作成
4. `CHANGELOG.md` — `[v34.6.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v34.6.0 に更新

---

## runes/ctx/db.fav 仕様

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

**含むべきキーワード**: `"DbCtx"`（アサーション対象）

---

## runes/ctx/http.fav 仕様

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

**含むべきキーワード**: `"HttpClient"`（アサーション対象）

---

## runes/ctx/stream.fav 仕様

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

**含むべきキーワード**: `"StreamClient"`（アサーション対象）

---

## site/content/docs/runes/ctx-migration-status.mdx 仕様

タイトル: `Rune ctx 移行ステータス`

含むべき内容:
- ctx 移行の概要（v34.5〜v34.7 系列の一環）
- 追加済み ctx インターフェース一覧（IoCtx / DbCtx / HttpClient / StreamClient）
- 移行対象 Rune ファイルの一覧と状態
- `fav migrate --from-effects` を使った移行手順への参照

**含むべきキーワード**: `"DbCtx"`（アサーション対象）— インターフェース一覧テーブルが含まれていることを確認

---

## テスト仕様（v346000_tests）

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

### 設計注記

- `use super::*` は**不要**（`include_str!` のみ使用）
- WASM ゲートなし
- v346000_tests は v345000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.6.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.6.0"`
- [ ] `cargo_toml_version_is_34_5_0` が空スタブになっていること
- [ ] `cargo test --bin fav v346000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2566 件想定 = 2561 + 5、0 failures）
- [ ] `runes/ctx/db.fav` が存在し `"DbCtx"` を含むこと
- [ ] `runes/ctx/http.fav` が存在し `"HttpClient"` を含むこと
- [ ] `runes/ctx/stream.fav` が存在し `"StreamClient"` を含むこと
- [ ] `site/content/docs/runes/ctx-migration-status.mdx` が存在し `"DbCtx"` を含むこと
- [ ] `CHANGELOG.md` に `[v34.6.0]` セクション
- [ ] `benchmarks/v34.6.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.6.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
