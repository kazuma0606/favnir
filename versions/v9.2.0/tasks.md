# Favnir v9.2.0 Tasks

Date: 2026-06-01
Theme: fav fmt — コードフォーマッタ（Rust 変更なし、Favnir のみ）

---

## Phase A: `compiler.fav` — pretty_expr 実装

- [x] A-1: `fn spaces(n: Int) -> String` ヘルパーを `compiler.fav` に追加
  — `String.repeat(" ", n)` で n スペースを生成
- [x] A-2: `fn pretty_expr(expr: Expr, indent: Int) -> String` を実装
  — リテラル（EInt / EFloat / EStr / EBool）
  — EVar（変数名そのまま）
  — EBinOp（`l op r`、演算子前後スペース）
  — ECall（`fn(arg1, arg2)` 形式）
  — EList（`[item1, item2]` 形式）
  — ERecord（`{ field: val }` 形式）
  — ELambda（`|param| body` 形式）
  — ELet / EBind（`let/bind name = val\nbody` 2 行形式）
  — EIf（`if cond {\n  then\n} else {\n  else_\n}` インデント展開）
  — EMatch（`match expr { Pattern -> body }` 各腕を別行）
- [x] A-3: `fav check fav/self/compiler.fav` — コンパイルエラーなし確認

---

## Phase B: `compiler.fav` — pretty_stmt / pretty_program 実装

- [x] B-1: `fn pretty_stmt(stmt: Stmt, indent: Int) -> String` を実装
  — SFn: `fn name(params) -> ret = body`（params を `, ` で join）
  — SStage: `stage Name: In -> Out !Eff = body`（エフェクト空なら省略）
  — SSeq: `seq Name = Stage1 |> Stage2`（` |> ` で join）
  — SType: `type Name = { field: Type  field: Type }`（フィールドを 2 スペース区切り）
  — SImport: `import "path"` / `import rune "name"`
- [x] B-2: `fn pretty_program(prog: Program) -> String` を実装
  — import 文を先頭に集約（`List.filter` で分離）
  — import ブロックと定義の間: 改行 2 つ
  — 定義間: 改行 2 つ
  — 末尾に改行 1 つ
- [x] B-3: `pub fn fmt_source(src: String) -> Result<String, String>` を追加
  — `lex → parse → pretty_program` のパイプラインをラップ
  — 外部から呼び出せる公開 API として定義
- [x] B-4: `fav check fav/self/compiler.fav` — コンパイルエラーなし確認

---

## Phase C: `cli.fav` — cmd_fmt 実装

- [x] C-1: `fn cmd_fmt(path: String, check: Bool) -> Unit !Io` を `cli.fav` に追加
  — `IO.read_file_raw(path)` → `fmt_source(src)` → 上書きまたは比較
  — `check: true` の場合: 差分あり → エラー表示 + `IO.exit(1)`
  — `check: false` の場合: `IO.write_file_raw(path, formatted)` + 完了メッセージ
- [x] C-2: `cli.fav` の dispatch に `"fmt"` コマンドを追加
  — `"fmt"` → `cmd_fmt(path, check_flag)`
  — `--check` フラグのパース
- [x] C-3: `fav check fav/self/cli.fav` — コンパイルエラーなし確認

---

## Phase D: 統合テスト

- [x] D-1: `fmt_simple_fn` — `fn` 定義の整形が期待通りであることを確認
  — 入力: スペース・改行が乱れた `fn` 定義
  — 出力: 正規化された `fn name(p: T) -> R = body` 形式
- [x] D-2: `fmt_idempotent` — 冪等性テスト
  — `fmt_source(src)` → `fmt_source(result)` の 2 回適用で同じ出力になること
- [x] D-3: `fmt_check_mode` — `--check` モードのテスト
  — フォーマット済みソース → 差分なし → 正常終了
  — 未フォーマットソース → 差分あり → エラー終了コード
- [x] D-4（オプション）: `fmt_stage_def` — `stage` 定義の整形テスト
- [x] D-5（オプション）: `fmt_seq_def` — `seq` 定義の整形テスト
- [x] D-6: `cargo test fmt` — D-1〜D-3（必須 3 件）通過確認

---

## Phase E: self-check + Bootstrap 検証

- [x] E-1: `fav check fav/self/compiler.fav` — self-check 通過
- [x] E-2: `cargo test bootstrap` — `bytecode_A == bytecode_B` 維持確認
- [x] E-3: `cargo test` — 全件通過（1165 件以上）確認

---

## Phase F: ドキュメント・バージョン更新

- [x] F-1: `fav/Cargo.toml` の `version` を `"9.2.0"` に更新
- [x] F-2: `versions/v9.2.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] F-3: `memory/MEMORY.md` に v9.2.0 完了を記録
- [x] F-4: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav fmt <file>` がファイルを整形して上書きする | ✓ |
| `fav fmt --check <file>` が差分あり → 終了コード 1 を返す | ✓ |
| 2 回適用しても差分が出ない（冪等性） | ✓ |
| `fav fmt fav/self/compiler.fav` が適用できる | ✓ |
| `fav check fav/self/compiler.fav` が引き続き通る（self-check） | ✓ |
| `bytecode_A == bytecode_B` を維持（Bootstrap） | ✓ |
| `cargo test` 全件通過（1167 件） | ✓ |
