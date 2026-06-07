# Favnir v12.2.0 Spec

Date: 2026-06-07
Theme: lint 強化 — W006（Result を `bind _` で捨てる）+ W007（深い match ネスト）

---

## 背景

### W006: `bind _` で Result を捨てる問題

`bind _ <- Postgres.execute_raw(...)` のようにエフェクト付き呼び出しの `Result` を
アンダースコアで捨てても現状は何の警告も出ない。
E2E デバッグ中に「Postgres が全エラー無視で exit 0」になる根本原因がこれだった。
AI も人間も「捨てても安全」と誤解しやすい。

### W007: 深い match ネスト問題

v12.1.0 の実装中に、`match` の中に `match` が 2 段以上ネストして
その内側の arm に `bind` があると **"global index out of bounds"** でランタイムクラッシュすることが判明。
これは Favnir の Rust コンパイラのバグだが、根本修正より
「そもそも深いネストを書かないようにリントで誘導する」方が設計として正しい。

Python 的な「ネストしまくり」コードは可読性が低く、Favnir の設計思想
（パイプライン指向・フラットな構造）にも反する。

---

## W006 仕様

### 検出条件

`EBind("_", expr, cont)` において `expr` の推論型が `Result<T, E>` 形式の場合。

- 型推論は checker.fav の HM 型推論を使用（`infer_hm` の `EBind` ハンドラ内）
- 型文字列が `"Result<"` で始まる場合にフラグ

### 対象となる例

```favnir
// 警告対象
bind _ <- Postgres.execute_raw("INSERT ...", params)
bind _ <- AWS.s3_put_object_raw(bucket, key, body)
bind _ <- IO.write_file(path, content)

// 対象外（Result を返さない）
bind _ <- IO.println("hello")       // Unit を返す
bind _ <- List.empty()              // List を返す
```

### エラーメッセージ

```
W006: discarding Result value with `bind _`
  --> pipeline.fav:10:3
   |
10 | bind _ <- Postgres.execute_raw(...)
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^ this expression returns Result<Unit, String>
   |
   = help: use `chain _` to propagate errors automatically
   = help: or handle explicitly:
           match Postgres.execute_raw(...) {
             Ok(_) => ...
             Err(e) => Result.err(e)
           }
   = note: using `bind _` silently discards errors — the pipeline will continue even on failure
```

### 実装場所

`checker.fav` の `infer_hm` — `EBind` 分岐内で型推論後にチェック:
```
EBind({ _0: "_", ... }) で inferred_ty が "Result<" で始まる → W006 を warn_list に追加
```

checker.fav はエラーを `Result.err(msg)` で返すが、警告は別途 warn_list に蓄積して
全チェック完了後に `Result.ok("ok\nW006: ...")` 形式で返す方式（既存 W001-W005 と同様）。

---

## W007 仕様

### 検出条件

`match` の中に `match` が **3 段以上**ネストしている場合。

```
match { ... }                        // 1段 — OK
match { ... match { ... } }          // 2段 — OK
match { ... match { ... match { } } } // 3段 — W007
```

### エラーメッセージ

```
W007: deeply nested match (depth 3)
  --> pipeline.fav:15:7
   |
15 |       match inner {
   |       ^^^^^ this match is nested 3 levels deep
   |
   = help: extract inner logic into a helper function
   = help: consider using Result.and_then / Option.map to flatten
   = note: deeply nested match reduces readability and can cause compiler issues
```

### 実装場所

`compiler.fav` の lint エンジン（W001-W005 が既に存在する場所）。
AST を再帰的に走査し、`EMatch` に到達するたびに深さカウンタをインクリメント。
深さが 3 以上になった時点で W007 を発行。

```favnir
fn lint_fn_w007(expr: Expr, depth: Int) -> Option<String> {
    match expr {
        EMatch({ _0: scrut, _1: arms }) =>
            if depth >= 3 {
                Option.some("W007: deeply nested match ...")
            } else {
                lint_arms_w007(arms, depth + 1)
            }
        ...
    }
}
```

---

## lint 抑制

`fav.toml` の `[lint]` セクションで個別に抑制可能:

```toml
[lint]
allow = ["W006"]   # W006 を無視（危険！理由コメントを必ず書くこと）
allow = ["W007"]   # W007 を無視
```

---

## スコープ外（v12.2.0 には含めない）

- W006b: `bind x <- result_expr` の後 `x` が未使用 — 未使用変数追跡が必要で複雑
- W007 の自動修正（`fav fmt --fix`）— v12.8.0 以降
- `--legacy` モードでの W007 — checker.fav 経由でのみ

---

## テストケース

### W006

| テスト名 | 内容 | 期待結果 |
|---|---|---|
| `w006_bind_underscore_result` | `bind _ <- Postgres.execute_raw(...)` | W006 警告 |
| `w006_bind_underscore_unit` | `bind _ <- IO.println("hello")` | 警告なし |
| `w006_chain_underscore_ok` | `chain _ <- Postgres.execute_raw(...)` | 警告なし |
| `w006_explicit_match_ok` | `match expr { Ok(_) => ... Err(e) => ... }` | 警告なし |
| `w006_fav_toml_allow` | `[lint] allow = ["W006"]` で抑制 | 警告なし |

### W007

| テスト名 | 内容 | 期待結果 |
|---|---|---|
| `w007_depth_2_ok` | match { match {} } | 警告なし |
| `w007_depth_3_warn` | match { match { match {} } } | W007 警告 |
| `w007_depth_4_warn` | 4段ネスト | W007 警告 |
| `w007_helper_fn_ok` | 内部をヘルパー関数に切り出し | 警告なし |
| `w007_fav_toml_allow` | `[lint] allow = ["W007"]` で抑制 | 警告なし |

### バージョン確認

| テスト名 | 内容 |
|---|---|
| `version_is_12_2_0` | `CARGO_PKG_VERSION == "12.2.0"` |
