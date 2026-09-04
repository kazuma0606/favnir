# Favnir

**Favnir** はデータパイプラインの構築・解析に特化した、型安全なパイプラインファースト言語です。

SAP・DB・CSV・API——それぞれ「接続」はできても、型がなく、境界が見えず、スキーマ変更が静かに下流を壊す。
Favnir は型とエフェクトで境界を引き、パイプラインを設計図として表現できる言語です。

---

## クイックスタート

```bash
git clone https://github.com/kazuma0606/favnir
cd favnir/fav
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

```bash
fav new myproject       # プロジェクト作成
fav run pipeline.fav    # 実行
fav check pipeline.fav  # 型チェック
fav test pipeline.fav   # テスト
fav fmt pipeline.fav    # フォーマット
fav lint pipeline.fav   # 静的解析
```

---

## コード例

SAP BusinessPartner を取得し、GDPR マスキングして Snowflake に保存する pipeline:

```favnir
pipeline sap_partner_sync !SapOData !Snowflake {
    stage Fetch {
        bind client <- ctx.sap.connect("BusinessPartner")
        bind result <- client.query_builder<BusinessPartner>()
            |> QueryBuilder.filter("Country eq 'JP'")
            |> QueryBuilder.page(1000, 0)
            |> client.execute
    }
    |> stage Mask {
        bind masked <- List.map(result.value, fn(p) -> mask(p.Email))
    }
    |> stage Save {
        bind _ <- Snowflake.insert("partner_masked", masked)
    }
}
```

テスト時は `Ctx.mock()` で本番接続なしにパイプライン全体を検証できます:

```favnir
fn test_sync(ctx: AppCtx) -> Result<Bool, String> {
    bind mock <- Ctx.mock(MockSapClient.empty(), MockSnowflake.empty())
    bind _    <- sap_partner_sync(mock)
    Result.ok(true)
}
```

---

## 主な機能

**言語コア**
- 型チェッカー（ジェネリクス・HM 型推論・境界付きジェネリクス `T with Ord`）
- パターンマッチ（ネスト・ガード・or-pattern・list-pattern）
- 名目型ラッパー（`type Email(String) where |v| String.contains(v, "@")`）
- `interface` / `impl` — 型クラス的な多態性
- `par [A, B] |> Merge` — 並列 stage 実行

**Capability Context**
- 副作用は `ctx: AppCtx` 引数で表現。`capability 引数がなければ純粋` が言語レベルで保証される
- `Ctx.build()` で本番設定を注入、`Ctx.mock()` でテスト用スタブに差し替え

**CLI ツール**
- `fav run` / `fav check` / `fav test` / `fav fmt` / `fav lint`
- `fav explain --lineage` — パイプライン依存関係の静的可視化
- `fav infer --from sap --metadata <url>` — SAP `$metadata` から型を自動生成
- `fav doc` — `///` コメントから Markdown ドキュメント生成
- `fav bench` / `fav profile` / `fav watch` / `fav repl`

**Rune エコシステム**
- SAP OData（BusinessPartner / SalesOrder / Material / PurchaseOrder / JournalEntry）
- Snowflake / PostgreSQL / DuckDB / Redis / DynamoDB
- AWS（S3 / SQS / Secrets Manager）/ Azure Blob / GCP BigQuery
- Kafka / HTTP / gRPC / GraphQL / LLM（Claude / OpenAI）

**セルフホスト**
- コンパイラ・型チェッカー・CLI を Favnir 自身で実装（`fav/self/`）
- Bootstrap 検証: `bytecode_A == bytecode_B` を CI で毎回確認

---

## なぜ Favnir を作ったのか

Favnir が生まれるまでに 2 つの試みがありました。

**RINQ**（Rust 版 LINQ クレート）では「なぜライブラリではなく言語が必要なのか」という問いに行き当たりました。
**ForgeScript**（Rust ラッパー言語）では、汎用言語として維持するには範囲が広すぎました。

「データパイプライン」に絞ることで、コンパイラが `stage` と `seq` の構造を理解し、
エフェクトの静的追跡・リネージ可視化・型安全な依存注入が実現できました。

> 失敗から学んだ核心：「スコープを絞ることが言語の強さになる」

---

## 現在の状態

**v100.0.0 — Favnir SAP Platform 1.0 宣言（2026-09-04）**

- テスト数: **4,279 件**（0 failures）
- `cargo install fav --version "100.0.0"`

バージョン履歴は [CHANGELOG.md](./CHANGELOG.md)、マイルストーン一覧は [MILESTONE.md](./MILESTONE.md) を参照。

---

## リポジトリ構成

```
favnir/
  fav/          コンパイラ・VM・CLI ツールチェーン（Rust）
  fav/self/     Favnir 製セルフホストコンパイラ・型チェッカー
  runes/        標準 Rune ライブラリ（Favnir）
  site/         リファレンスサイト（Next.js）
  infra/        インフラ（Terraform / AWS）
  versions/     バージョン履歴・ロードマップ・言語仕様
```

---

## 対応プラットフォーム

| OS | 状態 | 備考 |
|----|------|------|
| Windows (MSVC) | サポート | `.cargo/config.toml` に `/utf-8` フラグ設定済み |
| Linux / WSL | サポート | `export CXXFLAGS=` を `~/.bashrc` に追加 |
| macOS | 非対応 | 将来対応予定 |

---

## ライセンス

MIT
