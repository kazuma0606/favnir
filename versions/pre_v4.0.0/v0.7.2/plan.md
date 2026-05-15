# Favnir v0.7.2 実装計画

更新日: 2026-04-30

---

## フェーズ構成

```
Phase 1: ディレクトリ作成 + mod.rs 生成
Phase 2: Frontend 移動（lexer, parser）
Phase 3: Middle 移動（checker, ir, compiler, resolver）
Phase 4: Backend 移動（artifact, codegen, vm）
Phase 5: main.rs 分割（main.rs + driver.rs）
Phase 6: ast.rs の import パス修正
Phase 7: cargo test で全テスト通過確認
Phase 8: ARCHITECTURE.md 作成
```

---

## Phase 1: ディレクトリ作成 + mod.rs

```
src/frontend/mod.rs  → pub mod lexer; pub mod parser;
src/middle/mod.rs    → pub mod checker; pub mod ir; pub mod compiler; pub mod resolver;
src/backend/mod.rs   → pub mod artifact; pub mod codegen; pub mod vm;
```

`main.rs` の `mod` 宣言を更新：
```rust
// 削除: mod lexer; mod parser; mod checker; mod ir; mod compiler; mod artifact; mod codegen; mod vm; mod resolver;
// 追加: mod frontend; mod middle; mod backend;
// 残す: mod ast; mod toml; mod eval; mod driver;
```

---

## Phase 2: Frontend 移動

### lexer.rs
- `src/lexer.rs` → `src/frontend/lexer.rs`
- import 変更: なし（外部依存なし）

### parser.rs
- `src/parser.rs` → `src/frontend/parser.rs`
- import 変更:
  ```rust
  // before:
  use crate::lexer::{Lexer, LexError, Span, Token, TokenKind};
  use crate::ast::*;
  // after:
  use super::lexer::{Lexer, LexError, Span, Token, TokenKind};
  use crate::ast::*;
  ```

---

## Phase 3: Middle 移動

### checker.rs
- `src/checker.rs` → `src/middle/checker.rs`
- import 変更:
  ```rust
  // before:
  use crate::ast::*;
  use crate::lexer::Span;
  // after:
  use crate::ast::*;
  use crate::frontend::lexer::Span;
  ```

### ir.rs
- `src/ir.rs` → `src/middle/ir.rs`
- import 変更:
  ```rust
  // before: use crate::ast::{...};
  // after:  use crate::ast::{...};  // 変化なし（ルート共有）
  ```

### compiler.rs
- `src/compiler.rs` → `src/middle/compiler.rs`
- import 変更:
  ```rust
  // before:
  use crate::checker::Type;
  use crate::ir::{...};
  // after:
  use super::checker::Type;
  use super::ir::{...};
  ```

### resolver.rs
- `src/resolver.rs` → `src/middle/resolver.rs`
- import 変更:
  ```rust
  // before:
  use crate::ast::Visibility;
  use crate::checker::{Type, Checker};
  use crate::lexer::Span;
  use crate::parser::Parser;
  use crate::toml::FavToml;
  // after:
  use crate::ast::Visibility;
  use super::checker::{Type, Checker};
  use crate::frontend::lexer::Span;
  use crate::frontend::parser::Parser;
  use crate::toml::FavToml;
  ```

---

## Phase 4: Backend 移動

### artifact.rs
- `src/artifact.rs` → `src/backend/artifact.rs`
- import 変更: なし（外部依存なし）

### codegen.rs
- `src/codegen.rs` → `src/backend/codegen.rs`
- import 変更:
  ```rust
  // before:
  use crate::ast::{BinOp, Lit};
  use crate::artifact::{...};
  use crate::ir::{...};
  // after:
  use crate::ast::{BinOp, Lit};
  use super::artifact::{...};
  use crate::middle::ir::{...};
  ```

### vm.rs
- `src/vm.rs` → `src/backend/vm.rs`
- import 変更:
  ```rust
  // before:
  use crate::artifact::FvcArtifact;
  use crate::codegen::{Constant, Opcode};
  use crate::eval::Value;
  // after:
  use super::artifact::FvcArtifact;
  use super::codegen::{Constant, Opcode};
  use crate::eval::Value;
  ```

---

## Phase 5: main.rs 分割

### src/driver.rs（新規作成）
抽出する関数：
- `cmd_run`, `cmd_build`, `cmd_exec`, `cmd_check`, `cmd_explain`
- `load_file`, `load_all_items`, `find_entry`, `collect_fav_files`
- `make_resolver`, `load_and_check_program`
- `build_artifact`, `write_artifact_to_path`
- `read_artifact_from_path`, `exec_artifact_main`, `exec_artifact_main_with_emits`
- `artifact_info_string` と全分析関数（~350行）
- `format_visibility`, `format_type_expr`, `format_effects`

### src/main.rs（残す）
- `mod` 宣言
- `use driver::*;`
- `HELP` 定数
- `fn main()` のみ

---

## Phase 6: ast.rs 修正

```rust
// before:
use crate::lexer::Span;
// after:
use crate::frontend::lexer::Span;
```

---

## Phase 7: テスト確認

```bash
cargo test
```

全 302 テストが通ることを確認。

---

## Phase 8: ARCHITECTURE.md

`C:\Users\yoshi\favnir\fav\ARCHITECTURE.md` に作成。
