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

**v9.0.0（2026-05-30）— セルフホスト完成宣言**

| 機能 | 状態 |
|---|---|
| 型チェッカー（ジェネリクス・インターフェース・エフェクト） | 完了 |
| バイトコードコンパイラ + VM | 完了 |
| パターンマッチ（ネスト・ガード・バリアント） | 完了 |
| `collect` / `yield` / クロージャ | 完了 |
| `fav test` / `fav bench` / `fav check` / `fav run` | 完了 |
| Rune システム（AWS / DuckDB / Auth / Log / Env / Gen / SQL） | 完了 |
| セルフホストコンパイラ（`fav/self/compiler.fav`） | 完了 |
| セルフホスト型チェッカー（`fav/self/checker.fav`） | 完了 |
| Bootstrap 検証（compiler.fav が自分自身をコンパイル） | 完了 |
| `fav run` 全経路が Favnir pipeline 経由 | 完了（v9.0.0） |
| WASM バックエンド | 完了 |
| LSP（hover・diagnostics） | 完了 |
| `fav explain` / `fav bundle` / `fav graph` | 完了 |
| `stage` / `seq` / `\|>` パイプライン構文 | 完了 |
| `abstract stage` / `abstract seq`（依存注入） | 完了 |
| Schema Authority（fav infer → schemas → T.validate） | 完了 |

テスト: **1136 件すべて通過**

### セルフホスト完成状態（v9.0.0）

| 経路 | 実装 |
|---|---|
| `fav check` | checker.fav（v8.1.0〜） |
| `fav run` 単一ファイル | compiler.fav（v8.5.0〜） |
| `fav run` rune import あり | compiler.fav + ソース結合（v8.6.0〜） |
| `fav run` fav.toml プロジェクト | compiler.fav + プロジェクト収集（v8.11.0〜） |
| VM・ファイルI/O | Rust（恒久・設計上） |

Bootstrap 検証（v6.2.0 で確立・v9.0.0 まで維持）:
```
Stage 1: Rust VM で compiler.fav → hello.fav → bytecode_A
Stage 2: Rust VM で compiler.fav → compiler.fav → compiler_artifact
Stage 3: Rust VM で compiler_artifact → hello.fav → bytecode_B
検証: bytecode_A == bytecode_B ✓
```

---

## コード例

```favnir
import rune "duckdb"

type Order   = { customer: String  amount: Float }
type Summary = { customer: String  total: Float }

stage LoadOrders: String -> List<Order> !Db = |path| {
  bind conn <- duckdb.open(":memory:")
  duckdb.query<Order>(conn, $"SELECT * FROM '{path}'")
}

stage Summarize: List<Order> -> List<Summary> = |orders| {
  List.map(orders, |o| Summary { customer: o.customer  total: o.amount })
}

// seq: 名前を持つパイプラインの構造
seq OrderReport = LoadOrders |> Summarize

// fav explain で構造を可視化:
// NAME          TYPE                         EFFECTS
// OrderReport   String -> List<Summary>      !Db
```

---

## クイックスタート

```bash
git clone https://github.com/yourname/favnir
cd favnir/fav
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

```bash
fav run examples/basic/hello.fav
fav check examples/pipeline/pipeline.fav
fav test examples/testing/math.fav
```

---

## ロードマップ

| バージョン | テーマ | 状態 |
|---|---|---|
| v4.1〜v4.12 | Rune エコシステム（DB・HTTP・AWS・LSP・MCP） | 完了 |
| v5.0.0 | AWS 本番稼働・CI/CD・リファレンスサイト | 完了 |
| v6.0.0〜v6.3.0 | セルフホスト + Bootstrap 検証 + stage/seq 対応 | 完了 |
| v6.4.0〜v6.9.0 | Playground・サイト・T.validate・OSS 準備 | 完了 |
| v7.1.0〜v7.9.0 | fav explain リネージ・Rune 拡充・checker.fav HM 型推論 | 完了 |
| v8.0.0〜v8.4.0 | checker.fav セルフホスト完成・fav check/run 切替 | 完了 |
| v8.5.0〜v8.11.0 | fav run 全経路 Favnir pipeline 化 | 完了 |
| **v9.0.0** | **セルフホスト完成宣言・--legacy 非推奨化** | **完了** |
| v9.x | Rune 拡充・HTTP serve・OSS 公開準備 | 計画中 |

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
