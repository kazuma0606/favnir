# Favnir Migration Guide

`!Effect` アノテーションから Capability Context（`ctx` パラメータ）への移行ガイド。

---

## 背景

v34.5.0 で `!Effect` アノテーションを deprecated 化し、Capability Context（`AppCtx`）に一本化した。

**理由**:
- エフェクトが型シグネチャだけに存在し、コンテキスト（どこに接続するか）は実行時に決まっていた
- `AppCtx` に統一することで、テスト時に `Ctx.mock()` でモック差し替えが容易になる
- `bind { http } <- ctx` 構文で依存関係が明示的になる

---

## `fav upgrade` を使った自動移行

### 基本コマンド

```bash
# ドライラン（変更内容のプレビュー）
fav upgrade --from-effects --dry-run my_pipeline.fav

# インプレース書き換え
fav upgrade --from-effects --in-place my_pipeline.fav

# ディレクトリ一括移行
fav upgrade --from-effects --in-place --dir src/
```

### フラグ一覧

| フラグ | 説明 |
|---|---|
| `--from-effects` | `!Effect` → ctx 移行モード（必須）|
| `--dry-run` | 変更内容をプレビュー表示（ファイルは書き換えない）|
| `--in-place` | ファイルを直接書き換え |
| `--dir <path>` | ディレクトリ内の全 `.fav` ファイルを対象にする |

### `fav migrate` との使い分け

| コマンド | 用途 |
|---|---|
| `fav upgrade --from-effects` | `!Effect` → ctx 移行（v34.5 破壊的変更対応）|
| `fav migrate --from-effects` | 同上（`upgrade` と `migrate --from-effects` は同じ移行処理を行う独立コマンド）|
| `fav migrate --config fav.toml` | `fav.toml` フォーマット移行 |
| `fav migrate --from 33.0 --to 34.0` | バージョン間の構文差分を自動適用 |

---

## `!Effect` → ctx 対応表

| 廃止する `!Effect` | 代替 ctx フィールド | 型 |
|---|---|---|
| `!Io` | `ctx.io` | `IoCtx` |
| `!DbRead` / `!DbWrite` | `ctx.db` | `DbRead` / `DbWrite` |
| `!Http` | `ctx.http` | `HttpClient` |
| `!Postgres` / `!MySQL` | `ctx.db` | `PgConn` / `MySqlConn` |
| `!Redis` | `ctx.redis` | `RedisClient` |
| `!Llm` | `ctx.llm` | `LlmClient` |
| `!Stream` | `ctx.stream` | `StreamClient` |
| `!Trace` | `ctx.tracer` | `Tracer` |
| `!Snowflake` | `ctx.warehouse` | `SnowflakeConn` |
| `!Emit<T>` | `ctx.emitter` | `Emitter<T>` |

---

## 手動移行手順

1. `fav lint` で W022 警告（`deprecated_effect_annotation`）を確認する
2. 関数シグネチャに `ctx: AppCtx` パラメータを追加する
3. `!Effect` アノテーションを削除する
4. 関数本体の先頭に `bind { field } <- ctx` で必要なフィールドを取り出す
5. `fav check` で型エラーがないことを確認する

---

## Before / After

### Before（v34.4 以前）

```favnir
fn fetch_orders(url: String) -> Result<List<Order>, String> !Http {
    HTTP.get(url)
}
```

### After（v34.5 以降）

```favnir
fn fetch_orders(ctx: AppCtx, url: String) -> Result<List<Order>, String> {
    bind { http } <- ctx
    http.get(url)
}
```

---

## FAQ

**Q: `--legacy` フラグと `!Effect` の違いは？**

`--legacy` は v9.0 の Rust コンパイラパスへのフォールバック。`!Effect` は Favnir の型アノテーションで別の概念。

**Q: `AppCtx` を使わず `!Effect` だけ残すことはできる？**

v34.5 以降、`!Effect` は W022 警告対象。v35.x で削除予定のため、`fav upgrade --from-effects` で移行を推奨する。

**Q: テストコードも移行が必要？**

`Ctx.mock()` を使えばテスト用モックコンテキストを簡単に作れる。移行後のテストは読みやすくなる。

```favnir
fn test_fetch_orders() -> Bool {
    bind ctx <- Ctx.mock({ http: MockHttp.ok("[]") })
    bind result <- fetch_orders(ctx, "https://api.example.com/orders")
    result == Ok([])
}
```
