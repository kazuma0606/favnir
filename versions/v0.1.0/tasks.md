# Favnir v0.1.0 タスク一覧

更新日: 2026-04-26

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Lexer

- [ ] 1-1: `Token` 型を定義する (種別 + Span)
- [ ] 1-2: `Span` 型を定義する (ファイル名 + 開始 + 終了)
- [ ] 1-3: キーワードのトークン化 (`type`, `fn`, `trf`, `flw`, `bind`, `match`, `if`, `else`)
- [ ] 1-4: 記号のトークン化 (`<-`, `|>`, `|`, `->`, `!`, `?`, `:`, `,`, `.`, `{`, `}`, `(`, `)`, `=`, `_`)
- [ ] 1-5: 整数リテラルのトークン化
- [ ] 1-6: 浮動小数点リテラルのトークン化
- [ ] 1-7: 文字列リテラルのトークン化 (`"..."`, エスケープ対応)
- [ ] 1-8: 真偽値リテラルのトークン化 (`true`, `false`)
- [ ] 1-9: 識別子のトークン化
- [ ] 1-10: コメントのスキップ (`//`)
- [ ] 1-11: 空白・改行のスキップ
- [ ] 1-12: 不正な文字に対するエラー報告
- [ ] 1-13: Lexer の単体テストを書く

---

## Phase 2: AST 定義

- [ ] 2-1: `Item` enum を定義する (TypeDef, FnDef, TrfDef, FlwDef, Bind)
- [ ] 2-2: `TypeExpr` enum を定義する (Named, Optional, Fallible, Arrow)
- [ ] 2-3: `TypeDef` 構造体を定義する (record / sum)
- [ ] 2-4: `Variant` 構造体を定義する
- [ ] 2-5: `FnDef` 構造体を定義する
- [ ] 2-6: `TrfDef` 構造体を定義する (effect 注釈含む)
- [ ] 2-7: `FlwDef` 構造体を定義する
- [ ] 2-8: `BindStmt` 構造体を定義する
- [ ] 2-9: `Expr` enum を定義する (Literal, Ident, Pipeline, Apply, Block, Match, If, Closure)
- [ ] 2-10: `Pattern` enum を定義する (Wildcard, Literal, Bind, Variant, Record)
- [ ] 2-11: `MatchArm` 構造体を定義する
- [ ] 2-12: `Effect` enum を定義する (Pure, Io, Db, Network, Emit)
- [ ] 2-13: すべての AST ノードに `Span` を持たせる

---

## Phase 3: Parser

- [ ] 3-1: パーサの基本構造を作る (Token 列の消費・先読み)
- [ ] 3-2: エラー型を定義する (位置情報付き)
- [ ] 3-3: `type` 定義のパース (record 形式)
- [ ] 3-4: `type` 定義のパース (sum 形式)
- [ ] 3-5: `fn` 定義のパース
- [ ] 3-6: `trf` 定義のパース (effect 注釈含む)
- [ ] 3-7: `flw` 定義のパース
- [ ] 3-8: `bind <-` 束縛のパース
- [ ] 3-9: 単純束縛のパターンパース (`bind x <- ...`)
- [ ] 3-10: record 分解のパターンパース (`bind { name, email } <- ...`)
- [ ] 3-11: variant 分解のパターンパース (`bind ok(v) <- ...`)
- [ ] 3-12: 整数・浮動小数点・文字列・真偽値リテラルのパース
- [ ] 3-13: 識別子式のパース
- [ ] 3-14: 関数適用のパース (`f(x, y)`)
- [ ] 3-15: `|>` パイプライン式のパース
- [ ] 3-16: クロージャのパース (`|x| expr`)
- [ ] 3-17: block のパース (`{ ... }`)
- [ ] 3-18: `match` 式のパース
- [ ] 3-19: `match` アームのパース (パターン + 式)
- [ ] 3-20: `if` 式のパース (`if ... { } else { }`)
- [ ] 3-21: 型式のパース (`T`, `T?`, `T!`, `A -> B`, `List<T>`)
- [ ] 3-22: effect 注釈のパース (`!Io`, `!Db`, `!Emit<E>`)
- [ ] 3-23: パーサの単体テストを書く

---

## Phase 4: 型チェック

- [ ] 4-1: `Type` enum を定義する
- [ ] 4-2: `Effect` の合成関数を実装する (`Pure + X = X`, `Io + Io = Io`)
- [ ] 4-3: 型環境 `TyEnv` を実装する (Map + 親環境への参照)
- [ ] 4-4: 基本型の組み込み登録 (Bool, Int, Float, String, Unit, List, Map, Option, Result)
- [ ] 4-5: 組み込み関数の型登録 (IO, List, String, Option, Result)
- [ ] 4-6: `type` 定義の型チェック (record / sum)
- [ ] 4-7: `fn` 定義の型チェック (引数型・戻り値型)
- [ ] 4-8: `trf` 定義の型チェック (`Trf<Input, Output, Fx>`)
- [ ] 4-9: `flw` 定義の型チェック (`|>` の Output-Input 一致確認)
- [ ] 4-10: `bind <-` 束縛の型チェック
- [ ] 4-11: パターンの型チェック (Wildcard, Literal, Bind, Variant, Record)
- [ ] 4-12: `match` 式の型チェック (各アームの型一致確認)
- [ ] 4-13: `if` 式の型チェック (then/else の型一致確認)
- [ ] 4-14: `|>` 式の型チェック (接続型の一致確認)
- [ ] 4-15: 関数適用の型チェック
- [ ] 4-16: クロージャの型チェック
- [ ] 4-17: block の型チェック (最後の式の型が block の型になる)
- [ ] 4-18: `T?` / `T!` を内部型 (`Option<T>` / `Result<T, Error>`) に展開する
- [ ] 4-19: 型エラーのメッセージ生成 (位置情報付き)
- [ ] 4-20: 型チェックの単体テストを書く

---

## Phase 5: インタープリタ

- [ ] 5-1: `Value` enum を定義する
- [ ] 5-2: 実行環境 `Env` を実装する (lexical scope)
- [ ] 5-3: リテラル値の評価 (Bool, Int, Float, String, Unit)
- [ ] 5-4: 識別子の評価 (環境からの参照)
- [ ] 5-5: 関数適用の評価
- [ ] 5-6: クロージャの生成・適用
- [ ] 5-7: `|>` パイプラインの評価
- [ ] 5-8: `bind <-` 束縛の評価
- [ ] 5-9: 単純束縛の評価
- [ ] 5-10: record 分解束縛の評価
- [ ] 5-11: variant 分解束縛の評価
- [ ] 5-12: `match` 式の評価 (パターンマッチング)
- [ ] 5-13: `if` 式の評価
- [ ] 5-14: block の評価
- [ ] 5-15: `fn` 定義の評価 (クロージャとして環境に登録)
- [ ] 5-16: `trf` 定義の評価
- [ ] 5-17: `flw` 定義の評価
- [ ] 5-18: `type` 定義の評価 (コンストラクタを環境に登録)
- [ ] 5-19: ADT variant の構築と取り出し
- [ ] 5-20: record の構築とフィールドアクセス (`.` 記法)
- [ ] 5-21: 組み込み関数の実装: `IO.print`, `IO.println`
- [ ] 5-22: 組み込み関数の実装: `List.map`, `List.filter`, `List.fold`, `List.length`, `List.is_empty`, `List.first`, `List.last`
- [ ] 5-23: 組み込み関数の実装: `String.trim`, `String.lower`, `String.upper`, `String.split`, `String.length`, `String.is_empty`
- [ ] 5-24: 組み込み関数の実装: `Option.some`, `Option.none`, `Option.map`, `Option.unwrap_or`
- [ ] 5-25: 組み込み関数の実装: `Result.ok`, `Result.err`, `Result.map`, `Result.unwrap_or`
- [ ] 5-26: `Pure` / `Io` 以外の effect を使う `trf` がパース時にエラーになることを確認する
- [ ] 5-27: インタープリタの単体テストを書く

---

## Phase 6: CLI

- [ ] 6-1: `fav run <file>` コマンドの実装
- [ ] 6-2: `fav check <file>` コマンドの実装
- [ ] 6-3: ファイル読み込みとエラーハンドリング
- [ ] 6-4: エラー表示の実装 (ファイル名・行番号・列番号・メッセージ)
- [ ] 6-5: エラーコードの整理 (E001〜)
- [ ] 6-6: `--help` の実装

---

## サンプル・テスト

- [ ] 7-1: `examples/hello.fav` を書く (`IO.println` のみ)
- [ ] 7-2: `examples/pipeline.fav` を書く (`trf` + `|>` + `flw`)
- [ ] 7-3: `examples/adt_match.fav` を書く (`type` + `match`)
- [ ] 7-4: `examples/hello.fav` が `fav run` で動くことを確認する
- [ ] 7-5: `examples/pipeline.fav` が `fav run` で動くことを確認する
- [ ] 7-6: `examples/adt_match.fav` が `fav run` で動くことを確認する
- [ ] 7-7: 型エラーのあるファイルで `fav check` がエラーを報告することを確認する

---

## ドキュメント

- [ ] 8-1: `README.md` に基本的な使い方を書く
- [ ] 8-2: `examples/` の各ファイルにコメントを書く
