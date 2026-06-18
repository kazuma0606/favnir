# Favnir v9.0.0 言語仕様書

作成日: 2026-05-31
対象バージョン: v9.0.0（セルフホスト完成宣言）

---

## 1. 概要

v9.0.0 は **Favnir のセルフホスト完成宣言バージョン**。
`fav check` / `fav run`（全経路）が Favnir 自身の型チェッカー・コンパイラ経由で動作し、
Rust への機能依存を「VM・OS インターフェース層」のみに限定した。

テスト: **1136 件すべて通過**

---

## 2. 実行パイプラインと Rust 依存マップ

### 2.1 コマンド別パイプライン

| コマンド | 型チェック | コンパイル | 実行 |
|---|---|---|---|
| `fav check <file>` | **checker.fav**（Favnir） | — | — |
| `fav run <file>` | **checker.fav**（Favnir） | **compiler.fav**（Favnir） | Rust VM |
| `fav run --legacy <file>` | checker.rs（Rust, 非推奨） | compiler.rs（Rust, 非推奨） | Rust VM |
| `fav test <file>` | checker.fav | compiler.fav | Rust VM |
| `fav bench <file>` | checker.fav | compiler.fav | Rust VM |
| `fav build <file>` | checker.fav | compiler.fav | —（.fvc 出力） |
| `fav exec <file.fvc>` | — | — | Rust VM（バイトコード直接実行） |

### 2.2 Rust 担当コンポーネント（恒久・設計上変更しない）

| コンポーネント | 理由 |
|---|---|
| VM（バイトコード実行エンジン） | メモリ安全・性能・設計上の決定 |
| ファイル I/O primitive（`IO.read_file_raw` 等） | OS インターフェース層 |
| ネットワーク primitive（`IO.http_get_raw` 等） | OS インターフェース層 |
| Rune ローダー（`rune_modules/` のパス解決） | ファイルシステム依存 |
| バイトコードシリアライズ / デシリアライズ | バイナリ互換性保証 |

### 2.3 Favnir 担当コンポーネント（v9.0.0 時点）

| コンポーネント | ファイル | 導入バージョン |
|---|---|---|
| 型チェッカー | `fav/self/checker.fav` | v8.0.0（完成）, v8.1.0（fav check 切替） |
| コンパイラ（lexer / parser / codegen） | `fav/self/compiler.fav` | v8.3.0〜v8.11.0 |
| CLI サブコマンド | `fav/self/cli.fav` | v7.6.0 |
| lexer | `fav/self/lexer.fav` | v6.0.0〜 |
| parser | `fav/self/parser.fav` | v6.0.0〜 |
| codegen | `fav/self/codegen.fav` | v6.0.0〜 |
| stdlib（一部） | `fav/self/stdlib/*.fav` | v8.2.0（3 関数） |

### 2.4 Rust パーサーの役割（v9.0.0 時点）

Rust パーサー（`fav/src/frontend/parser.rs`）は以下を担う：

- `stage` / `seq` / `abstract` 構文の解析 → `fn` に脱糖してから Favnir 側に渡す
- `type T?` / `type T!` の解析（`TypeExpr::Optional` / `TypeExpr::Fallible`）
- `??` 演算子の解析
- `fav check` 時の AST 生成（`ast_lower_checker.rs` で checker.fav 入力形式に変換）

**既知の実装漏れ（v9.7.0 で修正予定）**:
`compiler.fav` の自前 lexer/parser は `?`/`??` トークンを持たないため、
`fav run`（Favnir pipeline）では `T?` / `T!` / `??` / `expr?` が未対応。
`fav check` は Rust パーサーを経由するため正常に動作する。

---

## 3. 型システム

### 3.1 プリミティブ型

| 型 | リテラル例 | 説明 |
|---|---|---|
| `Int` | `42`, `-1` | 64 ビット整数 |
| `Float` | `3.14`, `-0.5` | 64 ビット浮動小数点 |
| `String` | `"hello"` | UTF-8 文字列 |
| `Bool` | `true`, `false` | 真偽値 |
| `Unit` | `()` | 値なし（副作用のみの戻り型） |

### 3.2 複合型

| 型 | 構文 | 説明 |
|---|---|---|
| リスト | `List<T>` | 順序付きコレクション |
| マップ | `Map<K, V>` | キー・値のマッピング |
| オプション | `Option<T>` / `T?` | Some(v) または None |
| 結果 | `Result<T, E>` / `T!` | Ok(v) または Err(e)、`T!` は E = String |
| 関数型 | `A -> B` | 1 引数関数の型 |

**注意**: `T?` / `T!` は Rust パイプラインでのみ完全動作。self-hosted pipeline での対応は v9.7.0 予定。

### 3.3 型エイリアス

```favnir
type UserId = Int       // エイリアス: UserId と Int は同じ型
type Name   = String
```

### 3.4 レコード型

```favnir
type Order = {
  id:       Int
  item:     String
  amount:   Float
}

// 構築
let o = Order { id: 1  item: "widget"  amount: 9.99 }

// フィールドアクセス
o.item
```

### 3.5 Sum 型（variant）

```favnir
type Status = Active | Inactive | Pending

// ペイロードあり
type Shape =
  | Circle(Float)
  | Rect(Float, Float)
  | Point

// パターンマッチ
match shape {
  Circle(r)   -> Float.pi * r * r
  Rect(w, h)  -> w * h
  Point       -> 0.0
}
```

### 3.6 ジェネリクス

```favnir
// ジェネリック関数
fn identity<A>(x: A) -> A = x

fn map_option<A, B>(f: A -> B, opt: Option<A>) -> Option<B> = {
  match opt {
    None    -> None
    Some(v) -> Some(f(v))
  }
}
```

型パラメータは大文字 1 文字（`A`, `B`, `T`, `K`, `V` 等）が慣例。

### 3.7 エフェクト型

関数・stage の副作用を型レベルで宣言する。

```favnir
stage Fetch: String -> String !Io = |url| { ... }
stage Save:  Row    -> Unit   !Db = |row| { ... }
fn   Log:    String -> Unit   !Log = |msg| { ... }
```

| エフェクト | 意味 |
|---|---|
| `!Io` | ファイル・標準 I/O・HTTP（汎用） |
| `!Db` | データベースアクセス |
| `!AWS` | AWS サービス（S3, SQS 等） |
| `!Http` | HTTP クライアント（v9.5.0 で `!Io` から分離予定） |
| `!Llm` | LLM API 呼び出し（v9.6.0 予定） |
| `!Gen` | 乱数・UUID 生成 |
| `!Queue` | メッセージキュー（SQS, Redis 等） |
| `!Cache` | キャッシュ（Redis 等） |
| `!Log` | ログ出力 |
| `!Auth` | 認証・認可 |
| `!Env` | 環境変数アクセス |
| `!Rpc` | gRPC 通信 |

エフェクトを宣言せず副作用を使うと `E0003` 未宣言エフェクトエラー。

---

## 4. 文法

### 4.1 変数束縛

```favnir
// モナド束縛（Result / Option を unwrap、失敗時は即 return）
bind x <- some_result_expr

// 例: ファイル読み込み
bind raw <- IO.read_file_raw("data.csv")
bind rows <- csv.parse<Order>(raw)
```

`let` キーワードは存在しない。純粋な値の代入も `bind` を使う。
`bind x <- expr` は `expr` が `Result.err` / `Option.none` のとき即座に呼び出し元へ伝播する。

### 4.2 関数定義

```favnir
// 基本形
fn add(x: Int, y: Int) -> Int = x + y

// エフェクトあり
fn read_file(path: String) -> String !Io = IO.read_file_raw(path)

// ブロックボディ
fn process(s: String) -> Int !Io = {
  bind raw <- IO.read_file_raw(s)
  String.length(raw)
}

// public（rune からエクスポート）
public fn entry(x: String) -> String = x
```

### 4.3 stage / seq / パイプライン演算子

```favnir
// stage: 型契約とエフェクトを持つ変換の単位
stage ParseCsv:  String     -> List<Row>     !Io = |s| { ... }
stage Validate:  Row        -> Row               = |row| { ... }
stage SaveToDb:  Row        -> Int           !Db = |row| { ... }

// seq: 名前を持つパイプラインの構造
seq UserImport = ParseCsv |> Validate |> SaveToDb

// |> 演算子（左結合）
"data.csv" |> ParseCsv |> Validate |> SaveToDb

// pipe match（パターンマッチへの直接パイプ）
result |> match {
  Ok(v)  -> v
  Err(e) -> default
}
```

`seq` は単なる関数合成ではなく「名前を持つアーキテクチャの単位」。
`fav explain` でパイプライン構造を可視化できる。

### 4.4 abstract stage / abstract seq（依存注入）

```favnir
// 抽象 stage: 実装をスロットとして定義
abstract stage Notify: String -> Unit !Io

// 具体実装を差し替え
seq AlertPipeline = Fetch |> Notify[SlackNotify]
seq TestPipeline  = Fetch |> Notify[MockNotify]
```

型安全なスロット差し替えにより、テスト・本番の切り替えをコンパイル時に保証。

### 4.5 if / else if / else

```favnir
if x > 0 {
  "positive"
} else if x == 0 {
  "zero"
} else {
  "negative"
}
```

`if` は式。ブロック全体が値を返す。
`else if` は v8.10.0 で追加。

### 4.6 match とパターンマッチ

```favnir
match value {
  // ワイルドカード
  _ -> "default"

  // 変数束縛
  n -> Int.to_string(n)

  // リテラル
  0    -> "zero"
  "ok" -> "good"
  true -> "yes"

  // variant（ペイロードなし）
  None -> "empty"

  // variant（ペイロードあり）
  Some(v) -> v
  Ok(v)   -> v
  Err(e)  -> e

  // ガード（where）
  n where n > 100 -> "large"
  n where n > 0   -> "small positive"
}
```

**網羅性チェック**: `Option` / `Result` のパターンマッチは全腕を網羅しないと `E0004`。

**既知の制限（v9.0.0 時点）**:
- 多値バリアント `Variant(a, b)` のパターンは単値（`PVariantP`）のみ対応
- タプルパターンは未対応

### 4.7 クロージャ

```favnir
// 単引数
|x| x + 1

// ブロックボディ
|row| {
  bind cleaned <- clean(row)
  cleaned
}

// 高階関数との組み合わせ
List.map(orders, |o| o.amount * 1.1)
List.filter(items, |i| i.active)
```

クロージャはスコープの変数をキャプチャできる。

### 4.8 ブロック式

```favnir
{
  bind x <- step_one()
  bind y <- step_two(x)
  y
}
```

ブロックの最後の式が戻り値。`bind` のみで構成する場合は明示的な最終値が必要。
**既知の制限**: lambda ボディに `bind` / `if` を直接書けないケースがある → 別関数に切り出す。

### 4.9 文字列補間

```favnir
$"Hello, {name}!"
$"Result: {Int.to_string(x)}"
$"Query: SELECT * FROM '{table}'"
```

`{}` 内は文字列式。非文字列値は `Int.to_string` 等で変換が必要。

### 4.10 import

```favnir
// rune のインポート
import rune "duckdb"
import rune "aws"
import rune "http"

// ローカルファイルのインポート（スラッシュ含む）
import "utils/helpers"

// 裸名インポートは rune として解決される
import "name"    // → rune_modules/name/ を探索
```

---

## 5. エラーコード一覧（v9.0.0 時点）

| コード | 名前 | 検出器 | 説明 |
|---|---|---|---|
| E0001 | UndefinedVariable | checker.fav | 未定義変数の参照 |
| E0002 | TypeMismatch | checker.fav | 型の不一致（一般） |
| E0003 | UndeclaredEffect | checker.fav | エフェクト未宣言の副作用使用 |
| E0004 | NonExhaustiveMatch | checker.fav | Option/Result のパターンマッチが網羅的でない |
| E0005 | UnifyFailed | checker.fav | 型変数の単一化失敗（ジェネリクス） |
| E0006 | — | — | （未使用） |
| E0007 | UndefinedFunction | checker.fav | 未定義のユーザー定義関数呼び出し |
| E0008 | ArityMismatch | checker.fav | ジェネリック関数の引数数不一致 |
| E0009 | ReturnTypeMismatch | checker.fav | 宣言戻り型と推論戻り型の不一致 |
| E0010 | — | — | 名目型の型不一致（v9.7.0 予定） |
| E0011 | — | — | 未定義インターフェース（v9.7.0 予定） |
| E0012 | — | — | 非ジェネリック関数の引数数不一致（v9.1.0 予定） |
| E0013 | — | — | `expr?` の誤用（非 Result 関数内）（v9.7.0 予定） |

---

## 6. Rune システム

Rune は外部サービス・副作用をカプセル化するモジュール単位。`runes/<name>/` に配置。

| Rune | import | 主なエフェクト | 説明 |
|---|---|---|---|
| `duckdb` | `import rune "duckdb"` | `!Db` | DuckDB（Parquet/CSV/SQL 対応） |
| `aws` | `import rune "aws"` | `!AWS` | S3, SQS, Lambda 等 |
| `sql` | `import rune "sql"` | `!Db` | PostgreSQL / MySQL 汎用 SQL |
| `http` | `import rune "http"` | `!Io` | HTTP クライアント |
| `grpc` | `import rune "grpc"` | `!Rpc` | gRPC クライアント・サーバー |
| `fs` | `import rune "fs"` | `!Io` | ファイルシステム操作 |
| `queue` | `import rune "queue"` | `!Queue` | メッセージキュー |
| `cache` | `import rune "cache"` | `!Cache` | キャッシュ（Redis 等） |
| `slack` | `import rune "slack"` | `!Io` | Slack 通知 |
| `email` | `import rune "email"` | `!Io` | メール送信 |
| `auth` | `import rune "auth"` | `!Auth` | 認証・JWT |
| `log` | `import rune "log"` | `!Log` | 構造化ログ |
| `env` | `import rune "env"` | `!Env` | 環境変数 |
| `gen` | `import rune "gen"` | `!Gen` | 乱数生成（UUID は v9.4.0 予定） |
| `llm` | `import rune "llm"` | `!Llm` | LLM API（v9.6.0 予定） |
| `json` | `import rune "json"` | — | JSON エンコード/デコード（v9.4.0 予定） |
| `csv` | `import rune "csv"` | `!Io` | CSV 読み書き（v9.4.0 予定） |

Rune の作成方法:
```
runes/<name>/
  rune.toml    # エントリポイント宣言
  <name>.fav   # 公開 API（public fn）
  ...
```

---

## 7. 標準ライブラリ（stdlib, v9.0.0 時点）

### Favnir 実装済み（`fav/self/stdlib/`）

| 関数 | モジュール | 導入 |
|---|---|---|
| `List.intersperse` | list_stdlib.fav | v8.2.0 |
| `String.capitalize` | string_stdlib.fav | v8.2.0 |
| `String.indent` | string_stdlib.fav | v8.2.0 |

### Rust 実装（主要なもの）

**List**: `push`, `length`, `first`, `concat`, `drop`, `take`, `map`, `filter`, `fold`, `fold_left`, `find`, `any`, `all`, `sort_by`, `unique`, `scan`, `partition`, `empty`, `singleton`, `zip`

**String**: `length`, `concat`, `split`, `trim`, `to_upper`, `to_lower`, `contains`, `slice`, `to_int`, `to_float`, `chars`, `from_chars`, `starts_with`, `ends_with`, `lines`, `words`

**Map**: `get`, `set`, `keys`, `values`, `remove`, `contains_key`, `empty`, `from_pairs`, `to_pairs`, `size`

**Int**: `to_string`, `abs`, `min`, `max`, `parse`

**Float**: `to_string`, `parse`, `floor`, `ceil`, `round`, `abs`, `sqrt`, `pi`

**Option**: `some`, `none`, `is_some`, `is_none`, `and_then`, `unwrap_or`

**Result**: `ok`, `err`, `is_ok`, `is_err`, `and_then`, `map`, `unwrap_or`

**IO**: `read_file_raw`, `write_file_raw`, `println`, `http_get_raw`, `http_post_raw`

**Env**: `get_raw`

---

## 8. CLI コマンド

| コマンド | 説明 |
|---|---|
| `fav run <file>` | ファイルを実行（Favnir pipeline デフォルト） |
| `fav run --legacy <file>` | Rust pipeline で実行（非推奨） |
| `fav check <file>` | 型チェックのみ（checker.fav 経由） |
| `fav build <file>` | `.fvc` バイトコードを生成 |
| `fav exec <file.fvc>` | バイトコードを直接実行（ソース不要） |
| `rvm <file.fvc>` | VM 単体バイナリで実行（v9.1.0 追加予定、executor イメージ向け） |
| `rvm --version` | VM バージョンを表示（言語バージョンとは独立して採番） |
| `fav test <file>` | `test "name" { expr }` ブロックを実行 |
| `fav bench <file>` | ベンチマーク実行 |
| `fav explain <file>` | パイプライン構造を表示 |
| `fav explain --lineage <file>` | データリネージを静的解析・表示 |
| `fav graph <file>` | パイプライングラフを出力 |
| `fav bundle <file>` | 依存を含むバンドルを生成 |
| `fav infer <csv_or_db>` | スキーマを型定義として推論 |
| `fav install` | `fav.toml` の依存 rune をインストール |
| `fav publish` | rune をローカルレジストリに公開 |

---

## 9. Bootstrap 検証（v6.2.0〜 維持）

```
Stage 1: Rust VM で compiler.fav → hello.fav → bytecode_A
Stage 2: Rust VM で compiler.fav → compiler.fav → compiler_artifact
Stage 3: Rust VM で compiler_artifact → hello.fav → bytecode_B
検証:    bytecode_A == bytecode_B  ✓
```

`compiler.fav` が自分自身をコンパイルして同一バイトコードを生成することを確認。
v9.0.0 まで継続して維持。

---

## 10. 既知の実装漏れ（v9.0.0 時点）

以下は仕様として意図されていたが実装が漏れているもの。
v9.x ロードマップで順次修正予定。

| 項目 | 影響範囲 | 修正予定 |
|---|---|---|
| `T?` / `T!` / `??` が self-hosted pipeline（`fav run`）で未対応 | compiler.fav の lexer/parser に `?` トークンなし | v9.7.0 |
| `expr?` エラー伝播演算子が未実装 | どのパイプラインでも未実装 | v9.7.0 |
| 非ジェネリック関数の引数数チェック（E0012）が未実装 | checker.fav が env に戻り型のみ保存のため | v9.1.0 |
| `type T(Inner)` 名目型ラッパー未実装 | 新構文（v9.7.0 で設計・実装） | v9.7.0 |
| `where \|v\| pred` バリデーション未実装 | 新機能（v9.7.0 で設計・実装） | v9.7.0 |
| `with Eq, Show` インターフェース自動合成未実装 | 新機能（v9.7.0 で設計・実装） | v9.7.0 |
| stdlib Favnir 化が 3 関数のみ | Rust 実装が大半 | v9.1.0〜 |
| `fav fmt` / `fav lint` / `fav doc` / `fav profile` 未実装 | 新 CLI コマンド | v9.2.0〜v9.9.0 |

---

## 11. セルフホストコンパイラ構成

```
fav/self/
  lexer.fav      字句解析（Token 型、scan 関数）
  parser.fav     構文解析（AST 型、parse 関数）
  codegen.fav    コード生成（バイトコード命令列生成）
  compiler.fav   エントリポイント（lexer → parser → codegen → serialize）
  checker.fav    型チェッカー（HM 型推論、エフェクト追跡、E0001〜E0009）
  cli.fav        CLI サブコマンド実装
  stdlib/
    list_stdlib.fav    List 関数（Favnir 実装）
    string_stdlib.fav  String 関数（Favnir 実装）
```

### self-hosted parser の TypeExpr（v9.0.0 時点）

```favnir
type TypeExpr =
  | TeSimple(String)             // Int, String, Bool, Unit, 型名
  | TeList(TypeExpr)             // List<T>
  | TeOption(TypeExpr)           // Option<T>（T? は v9.7.0 で追加予定）
  | TeResult(TypeExpr, TypeExpr) // Result<T, E>
  | TeMap(TypeExpr, TypeExpr)    // Map<K, V>
  | TeFn(TypeExpr, TypeExpr)     // A -> B
```

### self-hosted parser の Item（v9.0.0 時点）

```favnir
type Item =
  | IFn(FnDef)     // fn / stage（Rust が脱糖してから渡す）
  | IType(TypeDef) // type（レコード型 / sum 型）
  | ITest(TestDef) // test "name" { ... }
```

`stage` / `seq` / `abstract` は Rust パーサーが解析・脱糖し、`IFn` として渡される。
self-hosted parser は `stage` キーワードを直接扱わない（設計上の分担）。

---

## 12. fav.toml プロジェクト設定

```toml
[project]
name    = "my-pipeline"
version = "1.0.0"
entry   = "src/main.fav"

[dependencies]
duckdb = { path = "runes/duckdb" }
aws    = { path = "runes/aws" }

[database]
url = "${DB_URL}"

[aws]
region = "ap-northeast-1"
```

`fav run` はプロジェクトモードでも Favnir pipeline で動作（v8.11.0〜）。

---

## 付録: バージョン別主要変更

| バージョン | 主な変更 |
|---|---|
| v6.0.0 | セルフホスト Phase A〜H、compiler.fav / checker.fav 基盤 |
| v6.2.0 | Bootstrap 検証確立（bytecode_A == bytecode_B） |
| v6.3.0 | compiler.fav に stage/seq/\|> 対応 |
| v6.6.0 | T.validate 完成（Schema Authority） |
| v7.1.0 | fav explain --lineage（静的リネージ解析） |
| v7.7.0 | checker.fav エフェクト追跡・match 網羅性チェック |
| v7.9.0 | checker.fav HM 型推論（unify_deep / fresh_var） |
| v8.0.0 | checker.fav セルフホスト完成（1103 tests） |
| v8.1.0 | fav check → checker.fav 経由に切替 |
| v8.5.0 | fav run デフォルト Favnir pipeline 化（--legacy 追加） |
| v8.6.0 | fav run の rune import 対応（Favnir pipeline） |
| v8.10.0 | else if 構文サポート、E0009 戻り型不一致チェック |
| v8.11.0 | fav.toml プロジェクトモード Favnir pipeline 化 |
| v9.0.0 | セルフホスト完成宣言、--legacy 非推奨化（1136 tests） |
