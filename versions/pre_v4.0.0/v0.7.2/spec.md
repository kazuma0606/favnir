# Favnir v0.7.2 仕様書 — 三相アーキテクチャへのリファクタリング

更新日: 2026-04-30

---

## 概要

v0.7.2 は機能追加なしの純粋なリファクタリングリリース。
単一フラットな `src/*.rs` を **三相モジュール構造**に再編する。

目的：
1. フェーズ間の依存関係を明示化し、循環依存を防ぐ
2. バックエンドの差し替え（Wasm/LLVM）を容易にする
3. `main.rs` の肥大化（1368行）を解消する

---

## 三相モジュール構造

```
Frontend  →  Middle  →  Backend
（字句解析）  （意味解析）  （コード生成・実行）
```

### 依存の方向

```
frontend ← middle ← backend
    ↑          ↑
   ast        ast
```

- **Frontend** は他フェーズに依存しない
- **Middle** は Frontend と共有 ast に依存する
- **Backend** は Middle と Frontend の一部に依存する
- `ast.rs` / `toml.rs` はフェーズ横断の共有モジュールとしてルートに残す

---

## ディレクトリ構成（移行後）

```
src/
├── frontend/
│   ├── mod.rs          # pub mod lexer; pub mod parser;
│   ├── lexer.rs        # トークナイザ（外部依存なし）
│   └── parser.rs       # 再帰下降パーサ（deps: lexer, crate::ast）
├── middle/
│   ├── mod.rs          # pub mod checker; pub mod ir; pub mod compiler; pub mod resolver;
│   ├── checker.rs      # 型検査・型システム（deps: ast, frontend::lexer）
│   ├── ir.rs           # IR 定義（deps: ast）
│   ├── compiler.rs     # AST → IR（deps: ast, ir, checker）
│   └── resolver.rs     # モジュール解決（deps: ast, checker, frontend::*, toml）
├── backend/
│   ├── mod.rs          # pub mod artifact; pub mod codegen; pub mod vm;
│   ├── artifact.rs     # .fvc バイナリ形式（外部依存なし）
│   ├── codegen.rs      # IR → バイトコード（deps: ast::BinOp/Lit, ir, artifact）
│   └── vm.rs           # バイトコード VM（deps: artifact, codegen, eval::Value）
├── ast.rs              # AST 定義（共有ルート; deps: frontend::lexer::Span）
├── toml.rs             # fav.toml パーサ（共有ルート; 外部依存なし）
├── eval.rs             # ツリーウォーク実行（暫定; deps: ast, frontend::lexer）
├── driver.rs           # CLI コマンド実装（cmd_run/build/exec/check/explain）
└── main.rs             # エントリポイント + CLI 引数解析のみ（~150行目標）
```

---

## import パスの変換規則

| 移動前 | 移動後（同フェーズ内） | 移動後（クロスフェーズ） |
|---|---|---|
| `use crate::lexer::` | `use super::lexer::` | `use crate::frontend::lexer::` |
| `use crate::parser::` | `use super::parser::` | `use crate::frontend::parser::` |
| `use crate::checker::` | `use super::checker::` | `use crate::middle::checker::` |
| `use crate::ir::` | `use super::ir::` | `use crate::middle::ir::` |
| `use crate::artifact::` | `use super::artifact::` | `use crate::backend::artifact::` |
| `use crate::codegen::` | `use super::codegen::` | `use crate::backend::codegen::` |
| `use crate::ast::` | `use crate::ast::` | （変化なし、ルート共有） |
| `use crate::toml::` | `use crate::toml::` | （変化なし、ルート共有） |
| `use crate::eval::` | `use crate::eval::` | （変化なし、ルート） |

---

## main.rs の分割

### 移行前（1368行、全責務混在）
```
main.rs
├── mod 宣言・use
├── HELP 定数
├── fn main()           ← CLI ディスパッチ
├── fn cmd_run/build/exec/check/explain  ← コマンド実装
├── fn artifact_info_string + 分析関数群
└── fn format_visibility/type_expr/effects
```

### 移行後

**`src/main.rs`**（~150行）
- `mod` 宣言
- `HELP` 定数
- `fn main()` — 引数解析とコマンドディスパッチのみ

**`src/driver.rs`**（残り全部）
- `cmd_run`, `cmd_build`, `cmd_exec`, `cmd_check`, `cmd_explain`
- `load_file`, `find_entry`, `build_artifact`, `exec_artifact_main`
- `artifact_info_string` と分析関数群
- `format_visibility`, `format_type_expr`, `format_effects`

---

## 変更しないもの

- API（`fav run / check / build / exec / explain`）
- テスト（`#[cfg(test)]` ブロックはファイルと一緒に移動）
- `Cargo.toml`（ワークスペース分割は行わない）
- `eval.rs` の実装内容（暫定実行系はそのまま）

---

## ARCHITECTURE.md

リポジトリルートに配置。内容：
- データフロー図（`.fav` → AST → IR → `.fvc` → VM）
- 各モジュールの責務一覧
- 将来の拡張ポイント（Wasm バックエンド、LSP サーバ）
