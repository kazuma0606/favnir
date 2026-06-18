# Favnir v9.2.0 仕様書 — fav fmt（コードフォーマッタ）

Date: 2026-06-01

---

## 概要

`compiler.fav` が生成する AST から整形済みテキストを出力する pretty-printer を Favnir で実装し、
`cli.fav` に `fav fmt` サブコマンドとして追加する。
Rust を一切触れずに開発できる最初の CLI 拡張。

---

## 背景

`compiler.fav` は既にソースコードを AST（`Program` 型）に変換する機能を持っている。
その AST から整形済みテキストを出力する関数を `compiler.fav` に追加し、
`fav fmt <file>` でファイルを上書きフォーマットできるようにする。

`--check` フラグを付けると差分があれば終了コード 1 を返すため、CI でのフォーマット強制に使える。
v10.0.0 の CI パイプラインでは `fav fmt --check fav/self/` が必須チェックになる予定。

---

## 設計

### `compiler.fav` に追加する関数

#### `fn pretty_expr(expr: Expr, indent: Int) -> String`

式を整形文字列に変換する。インデント幅 2 スペース。

| 式の種類 | 整形ルール |
|---|---|
| `EInt(n)` | `Int.to_string(n)` |
| `EFloat(f)` | `Float.to_string(f)` |
| `EStr(s)` | `"\"" + s + "\""` |
| `EBool(b)` | `"true"` / `"false"` |
| `EVar(name)` | `name` そのまま |
| `EBinOp(op, l, r)` | `pretty_expr(l) + " " + op + " " + pretty_expr(r)` — 演算子前後にスペース |
| `EIf(cond, then, else_)` | `if` / `else` をインデント付きで複数行に展開 |
| `ELet(name, val, body)` | `let name = val\nbody` の 2 行形式 |
| `ELambda(param, body)` | `\|param\| body`（1 行に収まる場合）|
| `ECall(fn, args)` | `fn(arg1, arg2)` — `List.map(xs, f)` 形式 |
| `EMatch(expr, arms)` | `match expr { ... }` — 各腕を別行 |
| `ERecord(fields)` | `{ field1: val1  field2: val2 }` |
| `EBind(name, val, body)` | `bind name <- val\nbody` の 2 行形式 |
| `EList(items)` | `[item1, item2, ...]` |

#### `fn pretty_stmt(stmt: Stmt, indent: Int) -> String`

トップレベル定義を整形する。

| 定義の種類 | 整形ルール |
|---|---|
| `SFn(name, params, ret_ty, body)` | `fn name(p: T) -> R = body` |
| `SStage(name, in_ty, out_ty, effects, body)` | `stage Name: In -> Out !Eff = body` |
| `SSeq(name, stages)` | `seq Name = Stage1 \|> Stage2 \|> Stage3` |
| `SType(name, fields)` | `type Name = { field: Type ... }` |
| `SImport(path)` | `import "path"` |

#### `fn pretty_program(prog: Program) -> String`

プログラム全体を整形する。

- トップレベル定義間は空行 2 行
- `import` 宣言はまとめて先頭に配置（他の定義の前）
- 末尾に改行を 1 つ

---

### `cli.fav` に追加する関数

#### `fn cmd_fmt(path: String, check: Bool) -> Unit !Io`

```
fav fmt <file>           # ファイルを読み込み → parse → pretty_print → 上書き保存
fav fmt --check <file>   # 差分があれば終了コード 1（上書きしない）
```

処理フロー:
1. `IO.read_file_raw(path)` でソースを読み込む
2. `lex(src)` → `parse(tokens)` で `Program` を得る
3. `pretty_program(prog)` で整形文字列を生成
4. `check` モード: 元のソースと比較、差分があれば `IO.exit(1)`
5. 通常モード: `IO.write_file_raw(path, formatted)` で上書き

---

## 完了条件

| 条件 | 詳細 |
|---|---|
| 冪等性 | `fav fmt` を 2 回通しても差分が出ない |
| 自己適用 | `fav fmt fav/self/compiler.fav` が `compiler.fav` に適用できる |
| `--check` フラグ | 差分あり → 終了コード 1、差分なし → 終了コード 0 |
| 統合テスト | 3 件以上 |
| self-check | `fav check fav/self/compiler.fav` が引き続き通る |
| Bootstrap 検証 | `bytecode_A == bytecode_B` を維持 |

---

## 整形ルール詳細

### インデント

- ベースインデント: 0
- ブロック内（`if`/`match`/`fn` ボディ）: +2 スペース
- インデント文字: スペース（タブ不使用）

### 演算子

- 二項演算子（`+`/`-`/`*`/`/`/`==`/`&&`/`||` 等）の前後にスペース 1 つ
- フィールドアクセス（`record.field`）にスペースなし
- パイプライン（`|>`）の前後にスペース 1 つ

### 行の折り返し

- 1 行 80 文字を目安（強制はしない、初版は折り返しなし）
- `match` の各腕は必ず別行
- `if`/`else` のボディが複数式の場合は別行に展開

### コメント

- `//` 行コメントは保持する（初版では AST にコメントが含まれない場合はスキップ可）

---

## 将来拡張（v9.2.0 スコープ外）

- `fav fmt src/`（ディレクトリ指定）
- `fav fmt --diff`（diff 出力のみ）
- 80 文字折り返し（長い式の自動改行）
- `.favfmt` 設定ファイル（インデント幅・行長のカスタマイズ）
- `v10.0.0` の CI: `fav fmt --check fav/self/` を必須チェックに追加
