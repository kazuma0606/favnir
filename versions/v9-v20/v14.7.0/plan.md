# v14.7.0 Plan — site/ ドキュメント更新 + rune ファイル精査

Date: 2026-06-12

---

## Phase A — `site/content/docs/introduction.mdx` 書き直し

### A-1: 旧エフェクト表・存在しない機能を削除し、Capability Context で説明

**削除対象:**
- `| エフェクト型 | !Io / !Db / !AWS / !Auth / !Env を関数シグネチャに明示 |` — 旧仕様
- `| MCP | ... |` — 未実装
- `| Notebook | ... |` — 未実装
- `| fav deploy | ... |` — 未実装

**置き換え後の設計思想セクション:**

```markdown
**Capability Context** — 副作用は `ctx` 引数（capability）で表現します。
`ctx` を受け取らない関数は純粋関数としてコンパイル時に保証されます。

**Rune エコシステム** — AWS・Azure・DuckDB・HTTP・LLM など 20+ の標準 Rune を内蔵。
`import rune "aws"` の 1 行で AWS S3/SQS/DynamoDB/Secrets Manager を型安全に操作できます。

**AWS / Azure ネイティブ** — LocalStack / Azurite で開発し、環境変数の切り替えだけで本番クラウドに接続します。
```

**置き換え後の特徴テーブル:**

```markdown
| 機能 | 説明 |
|------|------|
| 静的型チェック | `fav check` でコンパイル時に全エラーを検出 |
| Capability Context | `ctx: LoadCtx` 等の引数が副作用の境界。ctx なし = 純粋関数 |
| Rune | 外部モジュールシステム（AWS / Azure / DuckDB / HTTP / LLM 等） |
| LSP | VS Code でリアルタイム補完・型チェック |
| Python トランスパイラ | `fav transpile --target python` で boto3 / psycopg2 コードを自動生成 |
```

**コード例（Capability Context スタイル）:**

```markdown
```favnir
// ctx がなければ純粋関数
fn double(n: Int) -> Int { n * 2 }

// ctx があれば副作用あり（コンパイル時に保証）
public fn main(ctx: AppCtx) -> Unit {
  ctx.io.println(double(21))
}
```
```

---

## Phase B — `site/content/docs/language/effects.mdx` 書き直し

### B-1: v14.0.0 Capability Context を主体に全面書き直し

**新構成:**

```markdown
---
title: "副作用とCapability"
order: 2
category: "言語仕様"
description: "Favnir の Capability Context — 副作用を型で表現する（v14.0.0〜）"
---

# 副作用と Capability（v14.0.0〜）

Favnir では副作用を **Capability Context**（`ctx` 引数）で表現します。
`ctx` を受け取らない関数は **純粋関数** としてコンパイル時に保証されます。

## Capability Context の基本

```favnir
// 純粋関数: ctx なし
fn add(a: Int, b: Int) -> Int { a + b }

// 副作用あり: ctx を受け取る
public fn main(ctx: AppCtx) -> Unit {
  ctx.io.println(add(1, 2))
}
```

`ctx.io.println` のように、副作用は `ctx.capability.method()` の形式で呼び出します。
`ctx` がなければ `println` は呼び出せません。コンパイラが静的に保証します。

## 組み込み Capability Interface

| Interface | 用途 |
|-----------|------|
| `Io` | 標準入出力（`ctx.io.println` 等） |
| `DbRead` | データベース読み取り |
| `DbWrite` | データベース書き込み |
| `StorageRead` | オブジェクトストレージ読み取り |
| `StorageWrite` | オブジェクトストレージ書き込み |
| `HttpClient` | HTTP リクエスト |
| `Env` | 環境変数アクセス |

## コンテキスト型

```favnir
// LoadCtx: DbRead + Io を持つコンテキスト
fn load_users(ctx: LoadCtx, page: Int) -> Result<List<User>, String> {
  ctx.db.query($"SELECT * FROM users LIMIT 10 OFFSET {page * 10}")
}

// WriteCtx: DbWrite + StorageWrite を持つコンテキスト
fn save_result(ctx: WriteCtx, data: String) -> Result<Unit, String> {
  ctx.db.execute("INSERT INTO results VALUES (?)", [data])
}

// AppCtx: すべての Capability を持つ汎用コンテキスト
public fn main(ctx: AppCtx) -> Unit {
  match load_users(ctx, 0) {
    Ok(users) => ctx.io.println($"{List.length(users)} 件取得")
    Err(e)    => ctx.io.println($"エラー: {e}")
  }
}
```

## テストでのモック

```favnir
fn run_test() -> Bool {
  let ctx = Ctx.mock(MockDb.empty(), MockStorage.empty())
  let result = load_users(ctx, 0)
  Result.is_ok(result)
}
```

## --legacy モードでの旧 !Effect 記法

v14.0.0 以前の `!Effect` 記法は `--legacy` フラグ使用時のみ有効です:

```favnir
// --legacy モードでのみ動作（非推奨）
public fn main() -> Unit !Io {
  IO.println("Hello")
}
```

`fav migrate --from-effects` で自動移行できます。

## エラーコード

| コード | 説明 |
|--------|------|
| E0023 | ambient effect call — `ctx` なしで副作用を呼び出した |
| E0025 | bang notation removed — 非 legacy モードで `!Effect` を使用した |
| E0021 | capability not in context — コンテキストに必要な capability がない |
```

---

## Phase C — `site/content/docs/quickstart.mdx` 更新

### C-1: Hello World を Capability Context スタイルに

```markdown
## Hello World

```favnir
public fn main(ctx: AppCtx) -> Unit {
  ctx.io.println("Hello, Favnir!")
}
```

```bash
fav run src/main.fav
# Hello, Favnir!
```
```

### C-2: DuckDB / AWS サンプルを ctx-aware スタイルに

```markdown
## DuckDB でデータ分析

```favnir
import rune "duckdb"

type Summary = { customer: String  total: Float }

public fn main(ctx: AppCtx) -> Unit {
  bind conn   <- duckdb.open(":memory:")
  bind result <- duckdb.query<Summary>(conn,
    "SELECT customer, SUM(amount) AS total
     FROM 'data/orders.parquet'
     GROUP BY customer ORDER BY total DESC")
  ctx.io.println(result)
}
```

## AWS S3 からデータを読む

```favnir
import rune "aws"

public fn main(ctx: AppCtx) -> Unit {
  let aws_ctx = Ctx.build_aws_raw("ap-northeast-1", "my-bucket", "")
  bind body <- AWS.s3_get_object_raw("my-bucket", "data/input.csv")
  ctx.io.println($"取得完了: {String.length(body)} bytes")
}
```
```

---

## Phase D — `site/content/docs/installation.mdx` 更新

### D-1: バージョン表示を修正

```markdown
# Before
fav --version
# Favnir v5.0.0

# After
fav --version
# Favnir v14.7.0
```

---

## Phase E — rune ファイル E0025 精査

### E-1: 精査方法

各 rune を `import` するテスト用 .fav を書いて `fav check` し、E0025 の有無を確認する。

```fav
// /tmp/check_rune.fav
import rune "cache"
public fn main(ctx: AppCtx) -> Unit { ctx.io.println("ok") }
```

```bash
fav check /tmp/check_rune.fav
```

### E-2: 精査対象ファイル（優先順）

| rune | ファイル | 疑惑理由 |
|---|---|---|
| cache | `cache/cache.fav` | `fn get(key) !Cache` — ctx なし ambient |
| fs | `fs/fs.fav` | `fn read(path) !IO` — ctx なし ambient |
| log | `log/emitter.fav`, `log/metric.fav` | `fn info(...) !Io` — ctx なし ambient |
| queue | `queue/queue.fav` | `fn send(url, body) !Queue` — ctx なし ambient |
| gen | `gen/output.fav` | `fn to_csv(...) !Io`, `fn load_into(...) !Db` — ambient |
| http | `http/request.fav` | `fn get(url) !Network` / `!Http` — ambient |
| graphql | `graphql/client.fav` | `fn gql_post(url, query) !Http` — ambient |
| grpc | `grpc/server.fav` | `fn serve(port, ...) !Io !Rpc` — ambient |
| duckdb | `duckdb/query.fav`, `duckdb/io.fav` | `!Db` — DbHandle 渡しは別扱い |
| db | `db/connection.fav` | `!Db` — DbHandle パターン |
| aws | `aws/dynamodb.fav`, `aws/sqs.fav` | `!AWS` ctx なし旧 API |

### E-3: 修正方針

精査結果に応じて以下のいずれかに分類:

| 分類 | 対処 |
|---|---|
| **E0025 が出る + 修正容易** | 今バージョンで修正（`--legacy` コメント追加 or ctx 追加） |
| **E0025 が出る + 修正複雑** | v14.8.0 送り。tasks.md に記録 |
| **E0025 が出ない** | 現状維持。tasks.md に「OK」記録 |
| **`--legacy` モードで動作することが意図的** | コメントを追加して明示 |

**注意**: rune ファイルは VM プリミティブのラッパーであるため、`!Effect` アノテーションが `BUILTIN_EFFECTS` を介して許可されている可能性がある。精査結果を見てから判断する。

---

## Phase F — `fav/src/driver.rs`: v147000_tests + バージョンバンプ

### F-1: `v147000_tests` モジュールを追加（`v146000_tests` の直前）

```rust
// ── v147000_tests (v14.7.0) — site/ ドキュメント更新 + rune 精査 ───────────────
#[cfg(test)]
mod v147000_tests {
    #[test]
    fn version_is_14_7_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.7.0");
    }

    #[test]
    fn site_effects_doc_no_e0370() {
        // effects.mdx に存在しないエラーコード E0370 が含まれないことを確認
        let effects = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("site/content/docs/language/effects.mdx")
        ).expect("effects.mdx should exist");
        assert!(!effects.contains("E0370"),
            "effects.mdx should not contain nonexistent error code E0370");
    }

    #[test]
    fn site_introduction_no_fav_deploy() {
        // introduction.mdx に存在しない機能 "fav deploy" が含まれないことを確認
        let intro = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("site/content/docs/introduction.mdx")
        ).expect("introduction.mdx should exist");
        assert!(!intro.contains("fav deploy"),
            "introduction.mdx should not contain nonexistent feature 'fav deploy'");
    }
}
```

### F-2: `v146000_tests` の `version_is_14_6_0` を `>=` 比較に修正

```rust
assert!(env!("CARGO_PKG_VERSION") >= "14.6.0",
    "expected >= 14.6.0, got {}", env!("CARGO_PKG_VERSION"));
```

### F-3: `fav/Cargo.toml` バージョンを `"14.7.0"` にバンプ

---

## Phase G — 確認 + コミット

```bash
cargo test v147000  # 3 件全パス
cargo test          # 全件パス
git commit -m "feat: v14.7.0 — site/ ドキュメント更新 + rune ファイル精査"
```
