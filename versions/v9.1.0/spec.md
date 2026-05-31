# Favnir v9.1.0 仕様書

作成日: 2026-05-31

---

## 概要

v9.1.0 は4つの独立した改善を一括して実施する。

1. **stdlib 拡充** — 純粋 Favnir で実装できる標準ライブラリ関数を約 30 件追加
2. **E0012** — 非ジェネリック関数の引数数不一致を検出できない correctness fix
3. **マルチパラメータクロージャ** — `|x, y| x + y` が self-hosted pipeline で動作しない correctness fix
4. **`rvm` 独立バイナリ** — Rust VM を単体実行できる `rvm` コマンドを追加

---

## 1. stdlib 拡充

### 背景

v8.2.0 で `intersperse` / `capitalize` / `indent` の 3 関数を Favnir 化した。
それ以外の関数は Rust 実装のまま。`List.zip_with` / `Result.all` 等の実用的な
関数が不足しており、v9.2.0 以降のツール（fav fmt / fav lint 等）実装時に不便。

### 追加関数一覧

**List** (`fav/self/stdlib/list_stdlib.fav`):

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `List.chunk` | `List<A> -> Int -> List<List<A>>` | n 件ずつ分割 |
| `List.flat_map` | `(A -> List<B>) -> List<A> -> List<B>` | map + concat |
| `List.group_by` | `(A -> String) -> List<A> -> List<{key: String, values: List<A>}>` | キーで分類 |
| `List.zip_with` | `(A -> B -> C) -> List<A> -> List<B> -> List<C>` | 2リストを f で合成 |
| `List.take_while` | `(A -> Bool) -> List<A> -> List<A>` | 条件を満たす間 take |
| `List.drop_while` | `(A -> Bool) -> List<A> -> List<A>` | 条件を満たす間 drop |
| `List.unique` | `List<A> -> List<A>` | 順序保持で重複除去 |
| `List.count` | `(A -> Bool) -> List<A> -> Int` | 条件を満たす個数 |
| `List.sum` | `List<Int> -> Int` | 合計（Int版） |
| `List.min` | `List<Int> -> Option<Int>` | 最小値 |
| `List.max` | `List<Int> -> Option<Int>` | 最大値 |

**String** (`fav/self/stdlib/string_stdlib.fav` に追加):

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `String.pad_left` | `String -> Int -> String -> String` | 左パディング |
| `String.pad_right` | `String -> Int -> String -> String` | 右パディング |
| `String.truncate` | `String -> Int -> String -> String` | 末尾省略（suffix 付き） |
| `String.repeat` | `String -> Int -> String` | 文字列の繰り返し |
| `String.trim_start` | `String -> String` | 先頭の空白除去 |
| `String.trim_end` | `String -> String` | 末尾の空白除去 |
| `String.replace` | `String -> String -> String -> String` | 部分文字列置換 |

**Map** (`fav/self/stdlib/map_stdlib.fav` 新規):

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `Map.merge_with` | `(A -> A -> A) -> Map<String,A> -> Map<String,A> -> Map<String,A>` | 同一キーを f で解決 |
| `Map.filter` | `(String -> A -> Bool) -> Map<String,A> -> Map<String,A>` | エントリを絞り込む |
| `Map.map_values` | `(A -> B) -> Map<String,A> -> Map<String,B>` | 値を変換 |
| `Map.from_list` | `List<{key: String, value: A}> -> Map<String,A>` | リストから Map 構築 |
| `Map.to_list` | `Map<String,A> -> List<{key: String, value: A}>` | Map をリスト化 |

**Result / Option** (`fav/self/stdlib/result_stdlib.fav` 新規):

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `Result.map_err` | `(E -> F) -> Result<A,E> -> Result<A,F>` | エラー側を変換 |
| `Result.and_then` | `(A -> Result<B,E>) -> Result<A,E> -> Result<B,E>` | モナド bind |
| `Result.all` | `List<Result<A,E>> -> Result<List<A>,E>` | 全成功 or 最初のエラー |
| `Option.map` | `(A -> B) -> Option<A> -> Option<B>` | Some 内を変換 |
| `Option.and_then` | `(A -> Option<B>) -> Option<A> -> Option<B>` | モナド bind |
| `Option.unwrap_or` | `A -> Option<A> -> A` | デフォルト値付き unwrap |
| `Option.is_some` | `Option<A> -> Bool` | Some かどうか |
| `Option.is_none` | `Option<A> -> Bool` | None かどうか |

### 各関数の登録先

- `fav/self/stdlib/*.fav` — Favnir 実装本体
- `fav/src/middle/checker.rs` — Rust チェッカーの型シグネチャ登録
- `fav/self/checker.fav` — self-hosted チェッカーの型スキーム登録
- `fav/src/vm.rs` — Favnir stdlib へのディスパッチ追加

---

## 2. E0012 — 非ジェネリック関数引数数チェック

### 背景

v8.8.0 でジェネリック関数のアリティチェック（E0008）を実装したが、
非ジェネリック関数は `checker.fav` の `env` に戻り型文字列のみ保存しているため
引数数の不一致を検出できない。

```favnir
fn greet(name: String) -> String = "Hello, " + name

// 現状エラーにならない（引数数チェックなし）
greet("Alice", "Bob")  // E0012 が未検出
```

### 仕様

**エラーコード**: E0012 — ArgCountMismatch

**検出条件**: ユーザー定義の非ジェネリック関数に対して宣言と異なる引数数で呼び出した場合

**エラーメッセージ**: `"E0012: greet expects 1 argument(s), got 2"`

**実装方針**:
- `fn_to_scheme_str` で env に保存するスキーム文字列を
  `"ReturnType"` から `"ArgCount:ReturnType"` 形式に拡張
- `check_fn_call_arity(env, name, arg_count)` を新設
- `infer_call_user` 内でアリティチェックを追加

---

## 3. マルチパラメータクロージャ self-hosted 対応

### 背景

`fav/self/parser.fav` の `Expr` 型：

```favnir
type Expr =
  | ELambda(String, Expr)   // 引数が 1 つのみ
```

`|x, y| x + y` のような 2 引数以上のクロージャが self-hosted pipeline（`fav run`）で
パースエラーになる。`fav check` は Rust パーサーを経由するため問題なし。

`List.zip_with(|x, y| x + y, xs, ys)` など、v9.1.0 で追加する stdlib 関数を
self-hosted pipeline で使う際に必須。

### 仕様

`ELambda(String, Expr)` → `ELambda(List<String>, Expr)` に変更する。

```favnir
// 変更後
type Expr =
  ...
  | ELambda(List<String>, Expr)
```

**後方互換性**:
- 既存の単引数 `|x| body` は `ELambda(["x"], body)` として表現
- parser / checker / compiler 内の全 `ELambda` パターンマッチを更新

**コード生成**:
- 多引数クロージャは **カリー化** で実装
  - `|x, y| body` → `|x| |y| body` として展開（compiler.fav で脱糖）
  - 既存の単引数 Lambda の VM 命令をそのまま使用できる

**ast_lower_checker.rs**:
- Rust の `Lambda(params, body)` を `ELambda(param_names_list, lowered_body)` に変換
- 現在は単引数のみ渡している場合は複数引数対応に更新

---

## 4. `rvm` 独立バイナリ

### 背景

`fav exec file.fvc` として VM 実行できるが、`fav` はフルツールチェーンであり
本番 executor には過剰。独立した `rvm` バイナリを用意することで：

- executor イメージに `fav` を含める必要がなくなる
- VM だけのバグフィックスを `fav` 全体の再ビルドなしにリリースできる
- ECS / EKS / Lambda の runtime イメージを `rvm` のみで構成できる

### 仕様

```bash
rvm --version           # Favnir VM 1.0.0
rvm file.fvc            # バイトコードを実行
rvm --db <url> file.fvc # DB 接続付きで実行
rvm --help              # ヘルプ
```

**VM バージョン採番**: 言語バージョン（`fav --version` の `9.1.0`）とは独立。
バイトコード仕様の互換性を管理するバージョン。初期値 `1.0.0`。

**実装**:
- `fav/src/bin/rvm.rs` 新規作成
- `fav/Cargo.toml` に `[[bin]] name = "rvm"` を追加
- `VM_VERSION: &str = "1.0.0"` を `fav/src/vm.rs` に定数として定義
- ロジックは `fav exec` と同等（`driver::cmd_exec` を呼び出す）

---

## 完了条件

- stdlib 全関数が `fav/self/stdlib/*.fav` に実装され `fav test` で動作する
- 各関数の型シグネチャが `checker.fav` / `checker.rs` に登録されている
- `greet("Alice", "Bob")` が E0012 を出力する
- `|x, y| x + y` が `fav run`（Favnir pipeline）で動作する
- `List.zip_with(|x, y| x + y, [1,2,3], [4,5,6])` が `[5,7,9]` を返す
- `rvm --version` が `Favnir VM 1.0.0` を表示する
- `rvm file.fvc` が `fav exec file.fvc` と同じ結果を返す
- `cargo test` 全テスト通過（目標: 1160 件以上）
