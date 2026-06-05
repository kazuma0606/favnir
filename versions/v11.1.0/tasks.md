# Favnir v11.1.0 Tasks

Date: 2026-06-06
Theme: emit_python 基盤 — AST → Python コード生成の土台

---

## Phase A — `src/emit_python.rs` 新規作成

- [ ] A-1: `src/emit_python.rs` ファイル作成（モジュール骨格）
  - `pub fn emit_python(program: &Program) -> String`
  - インデント管理用 `Emitter` 構造体
- [ ] A-2: 型定義変換 — `type Foo = { ... }` → `@dataclass class Foo:`
  - `from dataclasses import dataclass, asdict` を import に追加
  - フィールド型マッピング: `String→str`, `Int→int`, `Float→float`, `Bool→bool`, `List<T>→List[T]`
- [ ] A-3: `fn` 変換 — `fn foo(x: T) -> R` → `def foo(x: T) -> R:`
  - 引数型アノテーション付き
  - エフェクト宣言（`!IO` 等）はコメントとして出力: `# effects: IO`
- [ ] A-4: 基本式変換
  - `Int` / `Float` / `String` / `Bool` リテラル
  - `if/else` → Python `if/else`
  - `List.empty()` → `[]`、`List.concat(a, b)` → `a + b`
  - `String.concat(a, b)` → `a + b`
  - `Int.to_string(n)` → `str(n)`
- [ ] A-5: `bind x <- expr` 脱糖 → `x = expr`
- [ ] A-6: `match` 変換
  - `Some(v) => ...` / `None => ...` → `if x is not None: v = x ... else: ...`
  - `Ok(v) => ...` / `Err(e) => ...` → `if isinstance(x, Ok): ...` (Result ヘルパー生成)
- [ ] A-7: `Result` / `Option` ヘルパークラス生成（先頭に埋め込み）
  ```python
  class Ok:
      def __init__(self, value): self.value = value
  class Err:
      def __init__(self, error): self.error = error
  ```

---

## Phase B — CLI エントリ

- [ ] B-1: `driver.rs` に `cmd_transpile(args)` 追加
  - `--target python`（現在は python のみ）
  - 入力: `.fav` ファイルパス
  - 出力: 同名の `.py` ファイル（デフォルト）/ `--out` で指定可
- [ ] B-2: `cli.fav` に `cmd_transpile` エントリ追加
  - `fav transpile --target python <file.fav>`
  - usage メッセージ
- [ ] B-3: `src/main.rs` の dispatch に `"transpile"` ケース追加

---

## Phase C — テスト

- [ ] C-1: `emit_python.rs` の `#[cfg(test)] mod v11100_tests` 作成
- [ ] C-2: `transpile_dataclass_simple` — `type TxnRow = { ... }` → `@dataclass` スナップショット
- [ ] C-3: `transpile_fn_basic` — `fn add(a: Int, b: Int) -> Int` → `def add(a: int, b: int) -> int:`
- [ ] C-4: `transpile_bind_desugars` — `bind x <- foo()` → `x = foo()`
- [ ] C-5: `transpile_match_option` — `match expr { Some(v) => ... None => ... }` → if/else
- [ ] C-6: `transpile_match_result` — `match expr { Ok(v) => ... Err(e) => ... }` → if/else
- [ ] C-7: `transpile_list_ops` — `List.concat` / `List.filter` / `List.length` → Python 相当
- [ ] C-8: `cargo test v11100 --lib` — 全件通過

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `fav transpile --target python hello.fav` で `.py` ファイルが生成される | |
| 生成 Python が `python3 -c "import ast; ast.parse(open('hello.py').read())"` で構文エラーなし | |
| `cargo test v11100` 全件通過 | |
