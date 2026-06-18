# Favnir v9.3.0 仕様書 — fav lint（静的解析ルールエンジン）

作成日: 2026-06-01

---

## 概要

`checker.fav` に型エラー（E0xxx）とは独立した警告（W0xx）を追加し、
`fav lint <file>` コマンドで静的解析を実行できるようにする。

「型は正しいが設計上疑問がある」コードをユーザーに伝えることで、
データパイプラインの品質向上を支援する。

**Rust 変更: なし**（`checker.fav` + `cli.fav` のみ）

---

## 警告ルール一覧

### W001 — EffectlessSink

`stage` の戻り型が `Unit` かつエフェクト宣言がない。

```favnir
// 警告
stage DoNothing: String -> Unit = |s| ()

// 正常（エフェクトあり）
stage SaveToDb: String -> Unit !Db = |s| db.exec(...)
```

> `"W001: stage DoNothing の戻り型が Unit ですがエフェクトがありません"`

---

### W002 — NoWriteInSeq

`seq` の最終 `stage` に `!Db` / `!AWS` / `!IO` 等の書き込みエフェクトがない。
「読んで捨てる」パイプラインをユーザーに気づかせる。

```favnir
// 警告
seq ReadOnly = FetchOrders |> Transform   // Transform に書き込みエフェクトなし

// 正常
seq Full = FetchOrders |> Transform |> SaveToS3  // SaveToS3 に !AWS あり
```

> `"W002: seq ReadOnly の最終 stage に書き込みエフェクトがありません"`

---

### W003 — UnusedBinding

`bind x <- expr` または `let x = expr` で束縛した `x` が後続の式で一度も参照されない。

```favnir
// 警告
bind result <- some_computation()
42  // result を使っていない

// 正常
bind result <- some_computation()
result + 1
```

> `"W003: 変数 result は定義されていますが使用されていません"`

---

### W004 — TooManyArgs

`stage` の引数型（パラメータ数）が 4 個以上。タプル化・レコード化を推奨。

```favnir
// 警告
stage Process: (String, Int, Bool, String, Float) -> Result<String, String> = ...

// 推奨
type ProcessInput = { name: String  count: Int  flag: Bool  tag: String  rate: Float }
stage Process: ProcessInput -> Result<String, String> = ...
```

> `"W004: stage Process の引数型が 5 個です。レコード型へのまとめを検討してください"`

---

### W005 — WildcardOnlyMatch

`match` 式の腕が `_` のみ（実質的に分岐していない）。

```favnir
// 警告
match result {
    _ => "unknown"
}

// 正常
match result {
    Ok(v) => v
    Err(e) => "error"
}
```

> `"W005: match 式の腕が _ のみです。網羅的なパターンを検討してください"`

---

## CLI インターフェース

```
fav lint <file>                  # 警告を表示（終了コード 0）
fav lint --warn-as-error <file>  # 警告があれば終了コード 1（CI 用）
```

出力フォーマット:

```
warning W001: stage DoNothing の戻り型が Unit ですがエフェクトがありません
warning W003: 変数 result は定義されていますが使用されていません
2 warning(s)
```

`--warn-as-error` 時:

```
error W001: stage DoNothing の戻り型が Unit ですがエフェクトがありません
1 error(s) (--warn-as-error)
```

---

## 実装方針

### `checker.fav` への追加

```favnir
type LintWarning = {
    code:    String   // "W001"
    message: String   // 人間向けメッセージ
    name:    String   // 対象の関数名・変数名等
}

fn lint_program(prog: Program) -> List<LintWarning>
fn lint_item(item: Item) -> List<LintWarning>
fn lint_fn_def(f: FnDef) -> List<LintWarning>     // W003（未使用変数）
fn lint_stage_def(sd: StageDef) -> List<LintWarning>  // W001 / W004
fn lint_seq_def(sd: SeqDef) -> List<LintWarning>       // W002
fn lint_expr(expr: Expr, bound: List<String>, used: List<String>) -> List<LintWarning>  // W003 / W005
```

新規エントリポイント（`checker.fav` の public fn として追加）:

```favnir
public fn lint_source(src: String) -> Result<List<LintWarning>, String>
```

### `cli.fav` への追加

```favnir
| CmdLint(String, Bool)   // (path, warn_as_error)

fn parse_lint_cmd(args: List<String>) -> CliCmd
fn run_lint(path: String, warn_as_error: Bool) -> Unit !IO
```

### Rust ブリッジ（最小）

`Compiler.lint_source_raw(src: String) -> Result<String, String>`
— `List<LintWarning>` を JSON 風文字列にシリアライズして返す
（既存 `Compiler.check_raw` パターンと同様）

または、`checker.fav` の `check_and_lint` 関数として check + lint を統合して
単一の出力文字列（改行区切り）にまとめる方が実装が簡潔。

---

## 完了条件

| 条件 | 確認 |
|---|---|
| W001〜W005 が対象コードで警告を出す | |
| `fav lint fav/self/compiler.fav` が実行できる | |
| `--warn-as-error` で警告時に終了コード 1 を返す | |
| `fav check fav/self/checker.fav` が self-check を通る | |
| `cargo test` 全件通過（1172 件以上） | |
