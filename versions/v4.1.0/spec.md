# Favnir v4.1.0 Language Specification

## Theme: Rune マルチファイル対応 — ディレクトリ単位の rune モジュール

v4.0.0 まで rune は単一ファイル（例: `runes/db/db.fav` 1枚）に限られていた。
v4.1.0 ではディレクトリ全体を 1 つの rune モジュールとして扱えるようにし、
以後の rune 充実化（v4.2.0〜）の前提インフラを整える。

---

## 変更サマリー

| 分類 | 機能 | 由来 |
|------|------|------|
| rune システム | ディレクトリ rune ロード（`runes/db/` + `db.fav` エントリポイント） | ロードマップ v4.1.0 |
| 言語構文 | `use X.{ a, b }` — rune 内部ファイル間のインポート | ロードマップ v4.1.0 |
| 言語構文 | `use X.*` — rune 内部ファイルの全公開名をインポート | ロードマップ v4.1.0 |
| rune システム | rune から別 rune を `import "json"` できるようにする | ロードマップ v4.1.0 |
| 後方互換 | 既存単一ファイル rune は変更なしで動作する | ロードマップ v4.1.0 |

---

## 1. ディレクトリ rune のロード

### 背景

v4.0.0 まで `import "db"` は `runes/db.fav` か `runes/db/db.fav`（単一ファイル）しか
解釈できなかった。このため rune の責務分割が不可能で、全ロジックを 1 ファイルに詰め込む
必要があった。

v4.1.0 からは `runes/db/` ディレクトリ全体を 1 つの rune モジュールとして扱い、
ファイル分割による責務分割が可能になる。

### ディレクトリ構成

```
runes/
  db/
    db.fav          ← public API エントリポイント（外部から見えるのはここの public のみ）
    connection.fav  ← 内部モジュール（外部不可視）
    query.fav       ← 内部モジュール（外部不可視）
    migration.fav   ← 内部モジュール（外部不可視）
```

### ロード手順

1. `import "db"` を検出
2. `runes/db/` ディレクトリが存在するか確認
3. `runes/db/db.fav` をエントリポイントとしてパース
4. `db.fav` 内の `use X.{ ... }` を解析し、参照されている `X.fav` を順次ロード・パース
5. 全ファイルの型定義・関数定義を内部スコープにマージ
6. `db.fav` で `public` として宣言された関数・型のみを rune の外部 API として公開

### 解決優先順位

```
import "db" の解決順（先にマッチしたものを使用）:
  1. runes/db/       ← ディレクトリ rune（v4.1.0 新規）
  2. runes/db.fav    ← 単一ファイル rune（後方互換）
  3. コンパイルエラー（E0xxx: rune not found）
```

---

## 2. `use` 文 — rune 内部インポート

### 構文

```
UseDecl:
  "use" Ident "." "{" NameList "}"   // 選択インポート
  "use" Ident "." "*"                // 全インポート
```

```favnir
// db.fav 内から
use connection.{ connect, close }
use query.{ run, paginate, run_raw }
use migration.*
```

### セマンティクス

- `use X.{ a, b }` は `runes/<rune>/X.fav` から名前 `a`, `b` を
  現在のファイルのスコープに取り込む。
- `use X.*` は `X.fav` 内の `public` 宣言を全て取り込む。
- `use` は **rune 内部専用**。通常の `.fav` ファイルから `use` は使えない
  （コンパイルエラー: E04xx — use outside rune）。
- 参照された `X.fav` が存在しない場合はコンパイルエラー（E04xx）。
- 循環参照（`A.fav` → `B.fav` → `A.fav`）はコンパイルエラー（E04xx）。

### AST

```rust
// ast.rs に追加
pub struct UseDecl {
    pub module: String,          // "connection"
    pub names: UseNames,         // Specific(vec!["connect", "close"]) | Wildcard
    pub span: Span,
}

pub enum UseNames {
    Specific(Vec<String>),
    Wildcard,
}
```

### 公開ルール

```
connection.fav の public fn connect → db.fav が use connection.{ connect } した場合:
  - db.fav のスコープ内で connect を呼び出せる
  - db.fav が public fn として再エクスポートしない限り、rune 外部からは不可視

connection.fav の public fn connect → db.fav が再エクスポートする場合:
  public fn connect(cfg: DbConfig) -> Result<DbHandle, DbError> !Db {
      connection.connect(cfg)   // ← connection.fav の実装に委譲
  }
```

---

## 3. Rune 内部からの rune 間インポート

rune ファイル（`runes/<name>/*.fav`）から他の rune を `import` できる。

```favnir
// runes/grpc/grpc.fav 内
import "json"   // ← json rune を grpc rune の内部で使う

public fn call_json(host: String, method: String, payload: Map<String, String>) -> Result<String, RpcError> !Rpc {
    bind body <- json.encode(payload)
    Grpc.call_raw(host, method, body)
}
```

セマンティクス:
- rune ファイル内の `import "X"` は通常ファイルと同じロジックで解決する
  （ベア名 → rune 自動検出、v4.0.0 の import 構文変更を踏襲）。
- rune 間で循環 import はコンパイルエラー（E04xx）。
- rune 内部から import した rune の public API は、そのファイルのスコープでのみ有効。

---

## 4. 後方互換性

| 構成 | v4.0.0 | v4.1.0 |
|------|--------|--------|
| `runes/db.fav`（単一ファイル） | 動作 | 引き続き動作（優先度 2） |
| `runes/db/db.fav`（ディレクトリ内単一ファイル） | 動作 | 引き続き動作（`use` なければ等価） |
| `runes/db/` + `use` | 不可 | v4.1.0 新機能 |
| `import rune "db"` 構文 | 動作（v4.0.0 以前） | 引き続き動作（backward compat） |
| `import "db"` 構文 | v4.0.0 で追加 | 動作 |

既存の全 rune（`http`, `parquet`, `db`, `grpc`, `incremental`, `gen`）は
単一ファイル rune のまま動作し、変更不要。

---

## 5. エラーコード

| コード | 状況 |
|--------|------|
| E04x0 | `use` が rune ファイル外で使われた |
| E04x1 | `use X.{ a }` の `X.fav` が rune ディレクトリに存在しない |
| E04x2 | `use X.{ a }` の `a` が `X.fav` に存在しない |
| E04x3 | `use` の循環参照 |
| E04x4 | rune ディレクトリにエントリポイント `<name>.fav` が存在しない |
| E04x5 | rune 間 import の循環参照 |

---

## 6. 典型ワークフロー

### 既存 db rune のマルチファイル化

```
# 既存
runes/db/db.fav   (1ファイル、300行)

# v4.1.0 以降
runes/db/
  db.fav           (public API、50行)
  connection.fav   (接続管理、80行)
  query.fav        (クエリ実行、120行)
  migration.fav    (マイグレーション、80行)
```

```favnir
// runes/db/db.fav
use connection.{ connect, close }
use query.{ run, paginate }
use migration.{ up, down, status }

public fn connect(cfg: DbConfig) -> Result<DbHandle, DbError> !Db {
    connection.connect(cfg)
}

public fn query<T>(conn: DbHandle, sql: String) -> Result<List<T>, DbError> !Db {
    query.run(conn, sql)
}
```

```favnir
// runes/db/connection.fav
public fn connect(cfg: DbConfig) -> Result<DbHandle, DbError> !Db {
    DB.connect_raw(cfg.url)
}

public fn close(conn: DbHandle) -> Unit !Db {
    DB.close_raw(conn)
}
```

### fav test での動作確認

```
fav test runes/db/db.test.fav
```

`db.test.fav` は `import "db"` を使い、マルチファイル化後の rune をテストできる。

---

## 新規 Cargo 依存

なし（既存の依存関係で実装可能）。

---

## テスト目標

- `use` 構文のパーサーテスト（選択 / ワイルドカード）
- ディレクトリ rune のロードテスト（driver 統合テスト）
- 内部モジュールから `use` した関数が呼び出せることの確認
- rune 外部から内部モジュールの非 public 関数が見えないことの確認
- 後方互換テスト（既存単一ファイル rune が引き続き動作する）
- エラーケーステスト（循環参照、存在しないモジュール、rune 外 `use`）
- 全既存テストがパスすること（リグレッションなし）
