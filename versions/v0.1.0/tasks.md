# Favnir v0.1.0 タスク一覧

更新日: 2026-04-27

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Lexer

- [x] 1-1: `Token` 型を定義する (種別 + Span)
- [x] 1-2: `Span` 型を定義する (ファイル名 + 開始 + 終了)
- [x] 1-3: キーワードのトークン化 (`type`, `fn`, `trf`, `flw`, `bind`, `match`, `if`, `else`)
- [x] 1-4: 記号のトークン化 (`<-`, `|>`, `|`, `->`, `!`, `?`, `:`, `,`, `.`, `{`, `}`, `(`, `)`, `=`, `_`)
- [x] 1-5: 整数リテラルのトークン化
- [x] 1-6: 浮動小数点リテラルのトークン化
- [x] 1-7: 文字列リテラルのトークン化 (`"..."`, エスケープ対応)
- [x] 1-8: 真偽値リテラルのトークン化 (`true`, `false`)
- [x] 1-9: 識別子のトークン化
- [x] 1-10: コメントのスキップ (`//`)
- [x] 1-11: 空白・改行のスキップ
- [x] 1-12: 不正な文字に対するエラー報告
- [x] 1-13: Lexer の単体テストを書く

---

## Phase 2: AST 定義

- [x] 2-1: `Item` enum を定義する (TypeDef, FnDef, TrfDef, FlwDef)
- [x] 2-2: `TypeExpr` enum を定義する (Named, Optional, Fallible, Arrow)
- [x] 2-3: `TypeDef` 構造体を定義する (record / sum)
- [x] 2-4: `Variant` 構造体を定義する (Unit / Tuple / Record)
- [x] 2-5: `FnDef` 構造体を定義する
- [x] 2-6: `TrfDef` 構造体を定義する (effect 注釈含む)
- [x] 2-7: `FlwDef` 構造体を定義する
- [x] 2-8: `BindStmt` 構造体を定義する
- [x] 2-9: `Expr` enum を定義する (Lit, Ident, Pipeline, Apply, FieldAccess, Block, Match, If, Closure, BinOp)
- [x] 2-10: `Pattern` enum を定義する (Wildcard, Lit, Bind, Variant, Record)
- [x] 2-11: `MatchArm` 構造体を定義する
- [x] 2-12: `Effect` enum を定義する (Pure, Io)
- [x] 2-13: すべての AST ノードに `Span` を持たせる

---

## Phase 3: Parser

- [x] 3-1: パーサの基本構造を作る (Token 列の消費・先読み)
- [x] 3-2: エラー型を定義する (位置情報付き)
- [x] 3-3: `type` 定義のパース (record 形式)
- [x] 3-4: `type` 定義のパース (sum 形式)
- [x] 3-5: `fn` 定義のパース
- [x] 3-6: `trf` 定義のパース (effect 注釈含む)
- [x] 3-7: `flw` 定義のパース
- [x] 3-8: `bind <-` 束縛のパース
- [x] 3-9: 単純束縛のパターンパース (`bind x <- ...`)
- [x] 3-10: record 分解のパターンパース (`bind { name, email } <- ...`)
- [x] 3-11: variant 分解のパターンパース (`bind ok(v) <- ...`)
- [x] 3-12: 整数・浮動小数点・文字列・真偽値リテラルのパース
- [x] 3-13: 識別子式のパース
- [x] 3-14: 関数適用のパース (`f(x, y)`)
- [x] 3-15: `|>` パイプライン式のパース
- [x] 3-16: クロージャのパース (`|x| expr`)
- [x] 3-17: block のパース (`{ ... }`)
- [x] 3-18: `match` 式のパース
- [x] 3-19: `match` アームのパース (パターン + 式)
- [x] 3-20: `if` 式のパース (`if ... { } else { }`)
- [x] 3-21: 型式のパース (`T`, `T?`, `T!`, `A -> B`, `List<T>`)
- [x] 3-22: effect 注釈のパース (`!Pure`, `!Io`)
- [x] 3-23: パーサの単体テストを書く

---

## Phase 4: 型チェック

- [x] 4-1: `Type` enum を定義する
- [x] 4-2: `Effect` の合成関数を実装する (`Pure + X = X`, `Io + Io = Io`)
- [x] 4-3: 型環境 `TyEnv` を実装する (Map + 親環境への参照)
- [x] 4-4: 基本型の組み込み登録 (Bool, Int, Float, String, Unit, List, Map, Option, Result)
- [x] 4-5: 組み込み関数の型登録 (IO, List, String, Option, Result)
- [x] 4-6: `type` 定義の型チェック (record / sum)
- [x] 4-7: `fn` 定義の型チェック (引数型・戻り値型)
- [x] 4-8: `trf` 定義の型チェック (`Trf<Input, Output, Fx>`)
- [x] 4-9: `flw` 定義の型チェック (`|>` の Output-Input 一致確認)
- [x] 4-10: `bind <-` 束縛の型チェック
- [x] 4-11: パターンの型チェック (Wildcard, Literal, Bind, Variant, Record)
- [x] 4-12: `match` 式の型チェック (各アームの型一致確認)
- [x] 4-13: `if` 式の型チェック (then/else の型一致確認)
- [x] 4-14: `|>` 式の型チェック (接続型の一致確認)
- [x] 4-15: 関数適用の型チェック
- [x] 4-16: クロージャの型チェック
- [x] 4-17: block の型チェック (最後の式の型が block の型になる)
- [x] 4-18: `T?` / `T!` を内部型 (`Option<T>` / `Result<T, Error>`) に展開する
- [x] 4-19: 型エラーのメッセージ生成 (位置情報付き)
- [x] 4-20: 型チェックの単体テストを書く

---

## Phase 5: インタープリタ

- [x] 5-1: `Value` enum を定義する
- [x] 5-2: 実行環境 `Env` を実装する (lexical scope)
- [x] 5-3: リテラル値の評価 (Bool, Int, Float, String, Unit)
- [x] 5-4: 識別子の評価 (環境からの参照)
- [x] 5-5: 関数適用の評価
- [x] 5-6: クロージャの生成・適用
- [x] 5-7: `|>` パイプラインの評価
- [x] 5-8: `bind <-` 束縛の評価
- [x] 5-9: 単純束縛の評価
- [x] 5-10: record 分解束縛の評価
- [x] 5-11: variant 分解束縛の評価
- [x] 5-12: `match` 式の評価 (パターンマッチング)
- [x] 5-13: `if` 式の評価
- [x] 5-14: block の評価
- [x] 5-15: `fn` 定義の評価 (クロージャとして環境に登録)
- [x] 5-16: `trf` 定義の評価
- [x] 5-17: `flw` 定義の評価
- [x] 5-18: `type` 定義の評価 (コンストラクタを環境に登録)
- [x] 5-19: ADT variant の構築と取り出し
- [x] 5-20: record の構築とフィールドアクセス (`.` 記法)
- [x] 5-21: 組み込み関数の実装: `IO.print`, `IO.println`
- [x] 5-22: 組み込み関数の実装: `List.map`, `List.filter`, `List.fold`, `List.length`, `List.is_empty`, `List.first`, `List.last`
- [x] 5-23: 組み込み関数の実装: `String.trim`, `String.lower`, `String.upper`, `String.split`, `String.length`, `String.is_empty`
- [x] 5-24: 組み込み関数の実装: `Option.some`, `Option.none`, `Option.map`, `Option.unwrap_or`
- [x] 5-25: 組み込み関数の実装: `Result.ok`, `Result.err`, `Result.map`, `Result.unwrap_or`
- [x] 5-26: `Pure` / `Io` 以外の effect を使う `trf` がパース時にエラーになることを確認する
- [x] 5-27: インタープリタの単体テストを書く

---

## Phase 6: CLI

- [x] 6-1: `fav run <file>` コマンドの実装
- [x] 6-2: `fav check <file>` コマンドの実装
- [x] 6-3: ファイル読み込みとエラーハンドリング
- [x] 6-4: エラー表示の実装 (ファイル名・行番号・列番号・メッセージ)
- [x] 6-5: エラーコードの整理 (E001〜)
- [x] 6-6: `--help` の実装

---

## サンプル・テスト

- [x] 7-1: `examples/hello.fav` を書く (`IO.println` のみ)
- [x] 7-2: `examples/pipeline.fav` を書く (`trf` + `|>` + `flw`)
- [x] 7-3: `examples/adt_match.fav` を書く (`type` + `match`)
- [x] 7-4: `examples/hello.fav` が `fav run` で動くことを確認する
- [x] 7-5: `examples/pipeline.fav` が `fav run` で動くことを確認する
- [x] 7-6: `examples/adt_match.fav` が `fav run` で動くことを確認する
- [x] 7-7: 型エラーのあるファイルで `fav check` がエラーを報告することを確認する

---

## ドキュメント

- [x] 8-1: `README.md` に基本的な使い方を書く
- [x] 8-2: `examples/` の各ファイルにコメントを書く
