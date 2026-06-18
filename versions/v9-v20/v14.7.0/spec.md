# v14.7.0 Spec — site/ ドキュメント更新 + rune ファイル精査

Date: 2026-06-12

---

## 目的

v15.0.0 CrossCloud E2E デモの前に、外部から見えるドキュメントと rune ライブラリの
整合性を修正する。

1. `site/content/docs/` の主要 MDX ファイルを v14.0.0 Capability Context 体系に合わせる
2. `runes/` の各 .fav ファイルが E0025（bang notation 禁止）に違反していないか確認し、
   問題があれば修正または v14.8.0 送りとして分類する

コードの追加・変更は rune ファイルの修正のみ。VM / checker / driver には手を加えない。

---

## 現状（v14.6.0 時点）

### site/content/docs/

| ファイル | 問題 |
|---|---|
| `introduction.mdx` | 旧エフェクト表で説明。`fav deploy` / `MCP` / `Notebook` という存在しない機能が記載 |
| `language/effects.mdx` | 旧エフェクトシステムのみを説明。エラーコード `E0370`（実在しない）。v14.0.0 Capability Context への言及ゼロ |
| `quickstart.mdx` | `!Io`, `!Db`, `!AWS` 旧スタイル全面使用 |
| `installation.mdx` | バージョン表示が `v5.0.0` のまま |
| `language/pipeline.mdx` | `stage` の `!Io`/`!Db` 例示は現状でも --legacy 用途として有効だが、v14.0.0 記述がない |
| `runes/aws.mdx` 等 | ambient API（ctx なし）を例示。ctx-aware API への言及なし |

### runes/ ファイル

| 分類 | ファイル | 状況 |
|---|---|---|
| **ambient !Effect（ctx なし）** | `cache/cache.fav`, `fs/fs.fav`, `log/emitter.fav`, `log/metric.fav`, `queue/queue.fav`, `gen/output.fav`, `grpc/server.fav` | VM primitive を `ctx:` なしで呼ぶ。E0025 の可能性あり |
| **ctx 経由の !Effect** | `aws/secrets.fav`, `azure-blob/azure_blob.fav`, `azure-postgres/azure_postgres.fav` | `ctx: String` + `!Effect` — v14.x 正しいパターン |
| **旧 ambient API（ctx なし）** | `aws/dynamodb.fav`, `aws/s3.fav`（旧関数群）, `aws/sqs.fav` | v14.4.0 で ctx-aware 版を追加済み。旧関数は --legacy 専用として残存 |
| **DB ハンドル渡し** | `duckdb/query.fav`, `duckdb/io.fav`, `db/connection.fav` | `conn: DbHandle` を引数に取る正しいパターン（ctx とは異なるが副作用を制御している） |
| **精査不要** | `csv/csv.fav`, `json/json.fav`, `toml/toml.fav` 等 | 純粋関数または明確に正しい |

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `introduction.mdx` 書き直し | 存在しない機能削除、Capability Context 体系で説明 |
| `effects.mdx` 書き直し | v14.0.0 新体系を主体に。旧 `!Effect` は `--legacy` として付記。E0370 → E0023/E0025 |
| `quickstart.mdx` 更新 | 主要サンプルを Capability Context スタイルに |
| `installation.mdx` 更新 | バージョン表示を現行（v14.x）に修正 |
| rune ファイル E0025 精査 | `fav check` で各 rune import を試験。違反ファイルを特定 |
| ambient rune ファイルの修正 | E0025 違反が確認された場合に修正（軽微なもの） |
| `v147000_tests`（3 件） | バージョン・サイトドキュメント検証 |
| Cargo.toml バージョン `14.7.0` | |

### Out of Scope

- rune docs（`runes/aws.mdx` 等）の全面書き直し — v14.8.0 以降（量が多いため）
- `language/pipeline.mdx` 等の --legacy スタイルサンプル — 現状でも有効なため保留
- rune ファイルの ctx-aware 全面移行 — 影響範囲が大きいため v14.8.0 以降
- 新機能追加

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `introduction.mdx` に `fav deploy` / `MCP` / `Notebook` が含まれない | ✅ |
| `effects.mdx` に `E0370` が含まれない | ✅ |
| `effects.mdx` に `E0025` または `E0023` への言及がある | ✅ |
| `quickstart.mdx` に Capability Context サンプルが存在する | ✅ |
| `installation.mdx` のバージョン表示が `v5.0.0` でない | ✅ |
| rune ファイル E0025 精査完了・結果が tasks.md に記録 | ✅ |
| `cargo test v147000` 全 3 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.7.0"` | ✅ |
