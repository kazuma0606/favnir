# Favnir v9.2.0 実装計画 — fav fmt

Date: 2026-06-01

---

## 実装方針

**Rust は一切触れない。** `compiler.fav` と `cli.fav` のみ変更する。

### 依存関係

```
compiler.fav の AST 型定義（既存）
  └─ pretty_expr / pretty_stmt / pretty_program を追加
        └─ cli.fav の cmd_fmt が呼び出す
              └─ driver.rs の cmd dispatch が呼び出す（既存 Favnir pipeline 経由）
```

`compiler.fav` にはすでに `Expr` / `Stmt` / `Program` 型と lexer/parser が実装済み。
pretty-printer はこれらの型を受け取り `String` を返すだけなので、Rust 変更は不要。

---

## フェーズ構成

### Phase A: `compiler.fav` — pretty_expr 実装

`Expr` の各バリアントを整形する関数を実装する。

**実装の進め方:**
1. リテラル（EInt / EFloat / EStr / EBool）から始める（自明）
2. EVar / EBinOp（演算子前後スペース）
3. ECall（`fn(arg1, arg2)` 形式）
4. ELet / EBind（2 行形式）
5. EIf / EMatch（インデント付き複数行）
6. ELambda / EList / ERecord

**設計上の注意:**
- `indent: Int` を受け取り、インデント付き文字列を組み立てる
- インデントヘルパー: `fn spaces(n: Int) -> String = String.repeat(" ", n)`
- `EIf` のネスト: `else if` は `else` ブロック内に `if` を展開（同一インデント）
- `EMatch` の腕: `arm.pattern + " -> " + pretty_expr(arm.body, indent + 2)` の各行

### Phase B: `compiler.fav` — pretty_stmt / pretty_program 実装

トップレベル定義の整形。

**`pretty_stmt` の各ケース:**
- `SFn`: `fn name(params) -> ret = body`
  - params: `List.map(params, |p| p.name + ": " + p.ty)` を `, ` で join
  - ボディが複数式の場合は `{\n  ...\n}` ブロック形式
- `SStage`: `stage Name: In -> Out [!Eff] = body`
  - エフェクトリストが空なら省略、あれば ` !Eff1 !Eff2`
- `SSeq`: `seq Name = Stage1 |> Stage2 |> ...`
  - stages を ` |> ` で join
- `SType`: `type Name = { field1: T1  field2: T2 }`
  - フィールドをスペース 2 つで区切り（Favnir 慣習）
- `SImport`: `import "path"` または `import rune "name"`

**`pretty_program`:**
- import 文を先頭にまとめる（`List.filter` で分離）
- import 間: 改行 1 つ
- import ブロックと定義の間: 改行 2 つ
- 定義間: 改行 2 つ
- 末尾に改行 1 つ

### Phase C: `cli.fav` — cmd_fmt 実装

```favnir
fn cmd_fmt(path: String, check: Bool) -> Unit !Io = {
  bind src    <- IO.read_file_raw(path)
  bind tokens <- Result.from_option(lex(src), "lex error")
  bind prog   <- parse(tokens)
  let formatted = pretty_program(prog)
  if check {
    if src == formatted {
      IO.println("ok: " + path)
    } else {
      IO.println("diff: " + path + " is not formatted")
      IO.exit(1)
    }
  } else {
    IO.write_file_raw(path, formatted)
    IO.println("formatted: " + path)
  }
}
```

**CLI dispatch 追加（`cli.fav`の `main` / dispatch 関数）:**
- `"fmt"` コマンドを dispatch に追加
- `--check` フラグのパース

### Phase D: 統合テスト

`fav/src/driver.rs` に `fmt_tests` モジュールを追加。

**テスト方針:**

テスト 1: `fmt_simple_fn` — シンプルな `fn` 定義のフォーマット
```favnir
fn add(x: Int, y: Int) -> Int = x + y
```
→ フォーマット後の文字列が期待形式と一致すること

テスト 2: `fmt_idempotent` — 冪等性テスト
1 回目のフォーマット結果を 2 回目の入力に使い、結果が同一であることを確認

テスト 3: `fmt_check_mode` — `--check` モード
未フォーマットのファイルで `--check` → エラー終了
フォーマット済みのファイルで `--check` → 正常終了

**オプション（テスト数を増やす場合）:**
- テスト 4: `fmt_stage_def` — `stage` 定義のフォーマット
- テスト 5: `fmt_seq_def` — `seq` 定義のフォーマット

### Phase E: self-check + Bootstrap 検証

- `fav check fav/self/compiler.fav` が引き続き通ること
- `cargo test bootstrap` — `bytecode_A == bytecode_B` を維持

---

## リスクと対策

| リスク | 対策 |
|---|---|
| AST が全バリアントを網羅していない | `_` ケースに `"<unknown>"` フォールバックを入れて段階的に実装 |
| コメントが AST に含まれない | 初版はコメント保持なし。警告なしでスキップ |
| `pretty_program` の冪等性が崩れる | テスト 2（冪等性テスト）で早期発見 |
| `cli.fav` の dispatch 変更で既存コマンドが壊れる | 既存コマンドのテストがすべて通ることを確認 |
| `compiler.fav` 自身に適用した結果が self-check を壊す | Phase E で明示的に確認 |

---

## 実装順序

```
Phase A: pretty_expr     [compiler.fav]  ← リテラル〜ネスト式
Phase B: pretty_stmt     [compiler.fav]  ← SFn / SStage / SSeq / SType
         pretty_program  [compiler.fav]  ← import 整理、定義間空行
Phase C: cmd_fmt         [cli.fav]       ← dispatch 追加、--check フラグ
Phase D: 統合テスト      [driver.rs]     ← 3 件以上
Phase E: self-check      [検証]          ← fav check + bootstrap
```

各 Phase は前の Phase 完了後に着手する（Phase A が安定しないと Phase B は始めない）。

---

## テスト目標

| 種別 | 件数 |
|---|---|
| fmt 統合テスト（新規）| 3〜5 件 |
| 既存テスト全件通過 | 1162 件 |
| **合計** | **1165 件以上** |
