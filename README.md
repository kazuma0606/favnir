# Favnir

**Favnir** はデータパイプラインの構築・解析に特化した、型安全なパイプラインファースト言語です。

企業のデータはサイロ化しています。SAP・DB・CSV・API——それぞれ「接続」はできても、
型がなく、境界が見えず、スキーマ変更が静かに下流を壊す。
そこに型とエフェクトで境界を引き、パイプラインを設計図として表現できる言語を作りたかった。
Favnir はその答えです。

---

## なぜ Favnir を作ったのか

Favnir が生まれるまでには、3つの試みがありました。

**1. RINQ — Rust 版 LINQ クエリビルダ**

C# の LINQ のように Rust でコレクション操作を書きたいと考え、クレートとして開発しました。
しかし Reddit でのフィードバックは「なぜ標準ライブラリの拡張ではなく新規クレートなのか」でした。
この問いに答えるためには、ライブラリではなく言語レベルの解決が必要だと気づきました。

**2. ForgeScript — Rust のラッパー言語**

実行とビルドの両方に対応した Rust ラッパー言語を開発しました。
しかし Rust を完全に置き換えるには、セキュリティや低レイヤー領域に精通したエンジニアが不可欠で、
個人プロジェクトとして維持するには範囲が広すぎました。

**3. Favnir — スコープを絞った専用言語**

「データ基盤とデータパイプラインの構築・解析」に特化し、
重い部分（VM・バイトコード実行）は Rust に委ね、
言語ロジック（コンパイラ・型チェッカー）は Favnir 自身で書く
**ハイブリッドセルフホスト**戦略を採用しました。

> 失敗から学んだ核心：「スコープを絞ることが言語の強さになる」

v9.0.0（2026-05-30）で、セルフホスト完成を宣言しました。
`fav check` も `fav run` も、すべての経路が Favnir 自身の型チェッカー・コンパイラを経由して動きます。
v10.0.0（2026-06-03）で、OSS 公開準備が完了しました。

---

## 言語の思想

Favnir は **Convention over Configuration** をパイプライン構造に適用した言語です。

通常の言語では、関数の合成は「ライブラリの慣習」に過ぎず、ツールからは「ただの関数呼び出し」にしか見えません。
Favnir では `stage`（変換）と `seq`（パイプライン）が言語プリミティブです。

```favnir
// stage: 型契約とエフェクトを持つ変換の単位
stage ParseCsv: String -> List<Row> !Io = |s| { /* ... */ }

stage ValidateRow: Row -> Row = |row| { /* ... */ }

stage SaveToDb: Row -> Int !Db = |row| { /* ... */ }

// seq: 名前を持つデータフローの構造
seq UserImport = ParseCsv |> ValidateRow |> SaveToDb
```

`seq UserImport` は関数合成の結果ではなく、**名前を持つアーキテクチャの単位**です。
これにより、コンパイラがパイプライン構造を理解し、以下が実現できます:

- **エフェクトの静的追跡** — どの段階で I/O・DB・イベント発行が起きるか
- **`fav explain` による可視化** — パイプライン構造をそのまま設計図として出力
- **`abstract seq` による依存注入** — 型安全なスロット差し替え

---

## 現在の状態

**v10.0.0（2026-06-03）— OSS 公開準備完了**

テスト: **1260 件すべて通過**

| 機能カテゴリ | 機能 | 状態 |
|---|---|---|
| **言語コア** | 型チェッカー（ジェネリクス・HM 型推論） | ✓ |
| | パターンマッチ（ネスト・ガード・バリアント） | ✓ |
| | エフェクト型（`!Db` `!IO` `!Http` `!Llm` 等） | ✓ |
| | 名目型ラッパー（`type UserId(Int)` + `where` バリデーター） | ✓ |
| | `interface` / `impl ... for` / `type T with Iface` | ✓ |
| | `par [A, B] \|> Merge` 並列 stage 実行 | ✓ |
| | `collect` / `yield` / クロージャ / `expr?` | ✓ |
| **パイプライン** | `stage` / `seq` / `\|>` | ✓ |
| | `abstract stage` / `abstract seq`（依存注入） | ✓ |
| | `fav explain --lineage`（静的リネージ解析） | ✓ |
| **CLI ツール** | `fav run` / `fav check` / `fav test` / `fav bench` | ✓ |
| | `fav fmt`（冪等コードフォーマッタ） | ✓ |
| | `fav lint`（W001〜W005 静的解析） | ✓ |
| | `fav doc`（`///` コメント → Markdown 生成） | ✓ |
| | `fav profile`（stage 別実行時間計測） | ✓ |
| | `fav watch`（ファイル監視 + 自動再実行） | ✓ |
| | `fav repl`（インタラクティブ REPL） | ✓ |
| | `fav new <name>`（プロジェクトスキャフォールディング） | ✓ |
| **Rune エコシステム** | AWS / DuckDB / SQL / DB / fs / Parquet | ✓ |
| | http / grpc / graphql（`!Http` エフェクト） | ✓ |
| | llm（`!Llm` エフェクト、Claude / OpenAI） | ✓ |
| | json / csv / gen（uuid / uuid_v7 / nano_id） | ✓ |
| | slack / queue / cache / email / auth / log | ✓ |
| **開発体験** | LSP（hover・diagnostics・補完・go-to-definition） | ✓ |
| | Schema Authority（fav infer → T.validate） | ✓ |
| | WASM バックエンド（Playground 向け） | ✓ |
| | `rvm` 独立実行バイナリ | ✓ |
| **セルフホスト** | コンパイラ（`fav/self/compiler.fav`） | ✓ |
| | 型チェッカー（`fav/self/checker.fav`） | ✓ |
| | CLI（`fav/self/cli.fav`） | ✓ |
| | Bootstrap 検証（`bytecode_A == bytecode_B`） | ✓ |

### セルフホスト経路（v9.0.0 以降）

| 経路 | 実装 |
|---|---|
| `fav check` | checker.fav（v8.1.0〜） |
| `fav run` 単一ファイル | compiler.fav（v8.5.0〜） |
| `fav run` rune import あり | compiler.fav + ソース結合（v8.6.0〜） |
| `fav run` fav.toml プロジェクト | compiler.fav + プロジェクト収集（v8.11.0〜） |
| VM・ファイル I/O | Rust（恒久・設計上） |

Bootstrap 検証（v6.2.0 で確立・維持中）:
```
Stage 1: Rust VM で compiler.fav → hello.fav → bytecode_A
Stage 2: Rust VM で compiler.fav → compiler.fav → compiler_artifact
Stage 3: Rust VM で compiler_artifact → hello.fav → bytecode_B
検証: bytecode_A == bytecode_B ✓
```

---

## コード例

### 基本パイプライン

```favnir
import rune "duckdb"
import rune "csv"

type Order   = { customer: String  amount: Float }
type Summary = { customer: String  total: Float }

stage LoadOrders: String -> List<Order> !Io = |path| {
  csv.read<Order>(path)
}

stage Summarize: List<Order> -> List<Summary> = |orders| {
  List.map(orders, |o| Summary { customer: o.customer  total: o.amount })
}

// seq: 名前を持つパイプラインの構造
seq OrderReport = LoadOrders |> Summarize

// fav explain --lineage で構造を可視化:
// NAME          TYPE                         EFFECTS
// OrderReport   String -> List<Summary>      !Io
```

### 並列実行（v9.13.0〜）

```favnir
import rune "http"

stage FetchOrders: String -> List<Order> !Db  = |conn| { /* DB から取得 */ }
stage FetchPrices: String -> List<Price> !Http = |url|  { /* API から取得 */ }
stage Merge:       (List<Order>, List<Price>) -> Report = |pair| { /* マージ */ }

// par: 複数 stage を並列実行し、結果をタプルで次 stage に渡す
seq FullReport = par [FetchOrders, FetchPrices] |> Merge

// fav explain で:
// par[FetchOrders(!Db), FetchPrices(!Http)] → Merge
// → DB と HTTP API を並列で読む — が静的に保証される
```

### 型バリデーション（v9.7.0〜）

```favnir
// 名目型ラッパー + where バリデーター
type Email(String)   where |v| String.contains(v, "@")
type Percent(Float)  where |v| v >= 0.0 && v <= 100.0

stage ParseInput: String -> Email !Io = |s| {
  Email(s)  // Result<Email, String> を返す
}
```

### LLM 統合（v9.6.0〜）

```favnir
import rune "llm"

stage Summarize: String -> String !Llm = |text| {
  llm.complete("3行で要約してください:\n" + text)
}

// fav explain --lineage で:
// Effects: !Db(read), !Llm, !AWS(S3 write) — AI依存度が静的に可視化される
```

---

## クイックスタート

```bash
git clone https://github.com/kazuma0606/favnir
cd favnir/fav
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

```bash
# 新規プロジェクト作成
fav new myproject
cd myproject
fav run src/main.fav

# 既存ファイルの操作
fav check pipeline.fav          # 型チェック
fav run pipeline.fav            # 実行
fav fmt pipeline.fav            # フォーマット
fav lint pipeline.fav           # 静的解析
fav doc src/ --out docs/        # ドキュメント生成
fav explain --lineage pipeline.fav  # リネージ可視化
```

---

## ロードマップ

| バージョン | テーマ | 状態 |
|---|---|---|
| v4.1〜v4.12 | Rune エコシステム（DB・HTTP・AWS・LSP・MCP） | 完了 |
| v5.0.0 | AWS 本番稼働・CI/CD・リファレンスサイト | 完了 |
| v6.0.0〜v6.6.0 | セルフホスト + Bootstrap 検証 + T.validate | 完了 |
| v7.1.0〜v7.9.0 | fav explain リネージ・Rune 拡充・checker.fav HM 型推論 | 完了 |
| v8.0.0〜v8.11.0 | checker.fav/compiler.fav セルフホスト完成・全経路 Favnir pipeline 化 | 完了 |
| v9.0.0 | **セルフホスト完成宣言**・`--legacy` 非推奨化 | 完了 |
| v9.1.0〜v9.4.0 | stdlib 拡充・`fav fmt`・`fav lint`・json/csv/gen Rune | 完了 |
| v9.5.0〜v9.6.0 | http/grpc/graphql Rune（`!Http`）・llm Rune（`!Llm`） | 完了 |
| v9.7.0〜v9.8.0 | 名目型ラッパー・`where` バリデーター・`fav doc` | 完了 |
| v9.9.0〜v9.11.0 | `fav profile`・`fav watch`・`fav repl`・LSP 補完強化 | 完了 |
| v9.12.0〜v9.13.0 | `interface`/`impl` セルフホスト・`par` 並列実行 | 完了 |
| **v10.0.0** | **OSS 公開準備完了**（`fav new`・CI self-check・CONTRIBUTING/CHANGELOG） | **完了** |
| v10.x | サイトドキュメント更新・Playground 改善・macOS CI | 予定 |

---

## リポジトリ構成

```
favnir/
  fav/          コンパイラ・VM・CLIツールチェーン（Rust）
  fav/self/     Favnir 製セルフホストコンパイラ・型チェッカー
  runes/        標準ルーンライブラリ（Favnir）
  site/         リファレンスサイト（Next.js）
  infra/        インフラ（Terraform / AWS）
  versions/     バージョン履歴・ロードマップ・言語仕様
```

### infra/e2e-demo — バイトコードポータビリティ証明デモ

セルフホストコンパイラが生成する `.fvc` バイトコードが、
**ソースコードなしで** 異なる実行環境上で動作することを
4つのシナリオで証明したデモ。すべての証跡は `s3://favnir-e2e-demo/proof/` に保存。

| デモ | 環境 | アーキテクチャ | 結果 |
|---|---|---|---|
| ECS | EC2 × 2 + Fargate | Machine A（コンパイル）→ S3 → Machine B（実行）→ ECS ETL | **PASS=8 / FAIL=0** |
| EKS | EKS Fargate | compiler Pod（`.fav`→`.fvc`）→ executor Pod（VM のみ）| **PASS=6 / FAIL=0** |
| Lambda | Lambda + SQS + Aurora | S3 イベント → compiler Lambda → SQS → executor Lambda → RDS | **PASS=6 / FAIL=0** |

**共通の証跡ポイント（EKS / Lambda）:**

| チェック | compiler | executor |
|---|---|---|
| `.fav` ソースの有無 | あり（toolchain イメージ） | なし（runtime イメージ） |
| `.fvc` 生成 | `fav build` で生成・S3 保存 | S3 からダウンロードして実行 |
| DB 書き込み | — | Aurora PostgreSQL → S3 サマリー |

詳細:
- [`infra/e2e-demo/ecs/README.md`](infra/e2e-demo/ecs/README.md)
- [`infra/e2e-demo/eks/README.md`](infra/e2e-demo/eks/README.md)
- [`infra/e2e-demo/lambda/README.md`](infra/e2e-demo/lambda/README.md)

---

## 対応プラットフォーム

| OS | 状態 | 備考 |
|----|------|------|
| Windows (MSVC) | サポート | 日本語環境は追加設定が必要（下記参照） |
| Linux / WSL | サポート | 追加設定不要 |
| macOS | 非対応 | 開発者が Mac を持っていないため未対応。将来対応予定 |

### Windows 日本語環境（CP932 ロケール）

`.cargo/config.toml` に `CXXFLAGS = "/EHsc /utf-8"` が設定済みです（`force = false`）。
PowerShell・Git Bash いずれからビルドしても自動的に適用されます。

### Linux / WSL

`~/.bashrc` に以下を追加してください:

```bash
export CXXFLAGS=
```

---

## ライセンス

MIT
