# Favnir

**Favnir** はデータパイプラインの構築・解析に特化した、型安全なパイプラインファースト言語です。

北欧神話の竜ファフニールにちなんで名付けられました。

---

## なぜ Favnir を作ったのか

Favnir が生まれるまでには、3つの試みがありました。

**1. RINQ — Rust 版 LINQ クエリビルダ**

C# の LINQ のように Rust でコレクション操作を書きたいと考え、クレートとして開発しました。
しかし Reddit でのフィードバックは「なぜ標準ライブラリの拡張ではなく新規クレートなのか」というものでした。
この問いに答えるためには、ライブラリではなく言語レベルの解決が必要だと気づきました。

**2. ForgeScript — Rust のラッパー言語**

実行とビルドの両方に対応した Rust ラッパー言語を開発しました。
しかし Rust を完全に置き換えるには、セキュリティや低レイヤー領域に精通したエンジニアが不可欠であり、
個人プロジェクトとして維持するには範囲が広すぎることがわかりました。

**3. Favnir — スコープを絞った専用言語**

「データ基盤とデータパイプラインの構築・解析」に特化し、
重い部分（VM・バイトコード実行・セキュリティ）は Rust に委ね、
言語ロジック（コンパイラ・型チェッカー・ライブラリ）は Favnir 自身で書く
**ハイブリッドセルフホスト**戦略を採用しました。

> 失敗から学んだ核心：「スコープを絞ることが言語の強さになる」

---

## 言語の思想

Favnir は **Convention over Configuration（CoC）** をパイプライン構造に適用した言語です。

通常の言語では、関数の合成は「ライブラリの慣習」に過ぎず、ツールからは「ただの関数呼び出し」にしか見えません。
Favnir では `stage`（変換）と `seq`（パイプライン）が言語プリミティブです。

```favnir
// stage: 型契約とエフェクトを持つ変換の単位
stage ParseCsv: String -> List<Row> !Io

stage ValidateRow: Row -> Row !Emit<ValidationError>

stage SaveToDb: Row -> Int !Db

// seq: 名前を持つデータフローの構造
seq UserImport = ParseCsv |> ValidateRow |> SaveToDb
```

`seq UserImport` は関数合成の結果ではなく、**名前を持つアーキテクチャの単位**です。
これにより、コンパイラがパイプライン構造を理解し、以下が実現できます：

- **エフェクトの静的追跡** — どの段階で I/O・DB・イベント発行が起きるか
- **`fav explain` による可視化** — パイプライン構造をそのまま設計図として出力
- **`abstract seq` による依存注入** — 型安全なスロット差し替え

---

## 現在の状態

**v2.0.0 リリース済み（2026-05-09）**

| 機能 | 状態 |
|---|---|
| 型チェッカー（ジェネリクス・インターフェース・エフェクト・invariant） | 完了 |
| バイトコードコンパイラ + VM | 完了 |
| WASM バックエンド | 完了 |
| LSP（hover・diagnostics） | 完了 |
| `fav test` / `fav bench` / `fav fmt` / `fav lint` | 完了 |
| `fav explain` / `fav bundle` / `fav graph` | 完了 |
| `fav migrate`（v1.x → v2.0.0 自動変換） | 完了 |
| セルフホスト字句解析器（Favnir 製）| 着手済み |

テスト: **538 件すべて通過**

---

## 対応プラットフォーム

| OS | 状態 | 備考 |
|----|------|------|
| Windows (MSVC) | ✓ サポート | 日本語環境は下記の追加設定が必要 |
| Linux / WSL | ✓ サポート | 追加設定不要 |
| macOS | 非対応 | 開発者が Mac を持っていないため未対応。将来対応予定 |

### Windows 日本語環境（CP932 ロケール）

`.cargo/config.toml` に `CXXFLAGS = "/EHsc /utf-8"` が設定済みです（`force = false`）。
PowerShell・Git Bash いずれからビルドしても自動的に適用されます。追加設定は不要です。

### Linux / WSL

`.cargo/config.toml` の `CXXFLAGS` は `force = false` のため、シェル側で空値を設定すれば無効化されます。
WSL の `~/.bashrc` に以下を追加してください：

```bash
export CXXFLAGS=
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
fav explain examples/pipeline/stage_seq_demo.fav
fav test examples/testing/math.test.fav
```

---

## コード例

```favnir
// エフェクト宣言
effect Payment

// 変換の定義（入力型・出力型・エフェクトが一目瞭然）
stage ParseOrder:   String -> Order          !Io
stage ValidateOrder: Order -> Order          !Emit<ValidationError>
stage ChargeCard:   Order -> Receipt         !Payment
stage SendReceipt:  Receipt -> Unit          !Io

// パイプラインの組み立て（名前を持つ構造）
seq OrderFlow = ParseOrder |> ValidateOrder |> ChargeCard |> SendReceipt

// パイプライン構造の可視化
// $ fav explain main.fav
// NAME         TYPE                          EFFECTS
// OrderFlow    String -> Unit                !Io !Emit<ValidationError> !Payment
```

---

## ロードマップ

| バージョン | テーマ |
|---|---|
| v2.1.0 | 標準ライブラリ補完（Math・IO.read_line）+ `fav new` + CLI ウェルカム |
| v2.2.0 | pipe match + pattern guard |
| v2.3.0 | 分割 bind + 戻り型推論 |
| v2.4.0 | スタックトレース + ランタイム品質改善 |
| v2.5.0 | LSP 補完・定義ジャンプ |
| v2.6.0 | モジュールシステム（import/export） |
| v2.7.0 | `validate` ルーン（Favnir 実装） |
| v2.8.0 | `stat` ルーン（Favnir 実装） |
| v3.0.0 | セルフホスト完成（Favnir 製パーサー） |

詳細: [`versions/roadmap-v3.md`](versions/roadmap-v3.md)

---

## リポジトリ構成

```
favnir/
  fav/          コンパイラ・VM・CLIツールチェーン（Rust）
  runes/        標準ルーンライブラリ（Favnir）
  selfhost/     Favnir 製コンパイラコンポーネント
  versions/     バージョン履歴・ロードマップ・言語仕様
  dev/          設計ドキュメント
```

---

## セルフホスト戦略

Favnir は **ハイブリッドセルフホスト** を目指しています。

- **Rust** — VM・バイトコード実行・セキュリティ（骨格）
- **Favnir** — コンパイラロジック・ルーンライブラリ（知能）

Rust に依存し続ける部分を意識的に絞ることで、
個人プロジェクトとして持続可能な範囲でセルフホストを実現します。

---

## ライセンス

MIT
