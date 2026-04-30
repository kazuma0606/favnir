# Favnir Compiler Architecture

v0.7.2 以降の `src/` 構成と各モジュールの責務。

---

## データフロー

```
.fav ソースファイル
    │
    ▼ frontend::lexer
  Token 列
    │
    ▼ frontend::parser
  AST (ast.rs)
    │
    ├─▶ middle::checker    型検査・エフェクト検査
    │
    ▼ middle::compiler
  IR (middle::ir)
    │
    ▼ backend::codegen
  バイトコード
    │
    ▼ backend::artifact
  .fvc バイナリ
    │
    ▼ backend::vm
  実行結果
```

---

## モジュール構成

```
src/
├── frontend/           フェーズ 1: 字句解析・構文解析
│   ├── mod.rs
│   ├── lexer.rs        トークナイザ。外部依存なし。
│   └── parser.rs       再帰下降パーサ。AST を生成。
│
├── middle/             フェーズ 2: 意味解析・IR 生成
│   ├── mod.rs
│   ├── checker.rs      型検査・エフェクト検査。Type / Subst / unify。
│   ├── ir.rs           IR 定義 (IRProgram / IRExpr / IRStmt / IRPattern)
│   ├── compiler.rs     AST → IR の変換。
│   └── resolver.rs     モジュール解決 (use / namespace)。
│
├── backend/            フェーズ 3: コード生成・実行
│   ├── mod.rs
│   ├── artifact.rs     .fvc バイナリ形式の読み書き (FvcWriter / FvcArtifact)
│   ├── codegen.rs      IR → バイトコード列。Opcode / Constant 定義。
│   └── vm.rs           スタック VM。バイトコードを実行。
│
├── ast.rs              AST 定義 (共有ルート。frontend / middle が参照)
├── toml.rs             fav.toml パーサ (共有ルート)
├── eval.rs             ツリーウォーク実行系 (暫定; fav run で使用)
├── driver.rs           CLI コマンド実装 (cmd_run/build/exec/check/explain)
└── main.rs             エントリポイント + CLI 引数解析のみ (~160行)
```

---

## 依存関係

```
                  ast.rs (共有)
                 /     \
         frontend       middle ─── backend
         (lexer,         (checker,   (codegen,
          parser)         ir,         artifact,
                          compiler,   vm)
                          resolver)
```

- **Frontend** は他フェーズに依存しない
- **Middle** は Frontend (`lexer::Span`, `parser::Parser`) と `ast` に依存
- **Backend** は Middle (`ir::*`, `checker::Type`) と `ast` に依存
- `eval.rs` は `ast` と `frontend::lexer` に依存 (暫定実行系)
- `driver.rs` は全フェーズをオーケストレート

---

## 将来の拡張ポイント

| 拡張 | 追加場所 |
|---|---|
| Wasm バックエンド | `backend/wasm.rs` |
| LLVM バックエンド | `backend/llvm.rs` |
| LSP サーバ | `src/lsp/` (独立バイナリ) |
| フォーマッタ | `src/fmt.rs` |
| eval.rs の廃止 | vm.rs が eval::Value を再利用する形で統合 |

---

## ファイル規模 (v0.7.2 時点)

| ファイル | 行数 | 備考 |
|---|---|---|
| `frontend/lexer.rs` | ~690 | テスト含む |
| `frontend/parser.rs` | ~1452 | テスト含む |
| `middle/checker.rs` | ~2529 | テスト含む |
| `middle/ir.rs` | ~82 | |
| `middle/compiler.rs` | ~645 | テスト含む |
| `middle/resolver.rs` | ~310 | テスト含む |
| `backend/artifact.rs` | ~295 | テスト含む |
| `backend/codegen.rs` | ~631 | |
| `backend/vm.rs` | ~3261 | テスト含む |
| `ast.rs` | ~349 | |
| `eval.rs` | ~3377 | テスト含む (暫定) |
| `driver.rs` | ~1328 | テスト含む |
| `main.rs` | ~160 | エントリポイントのみ |
