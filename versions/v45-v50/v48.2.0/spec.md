# Spec: v48.2.0 — import 構文刷新（ローカルファイル）

## 概要

`import "./src/helpers" as helpers`（`./` prefix を持つ文字列パス）を
`ImportKind::Local` として解析できるようにする。
v48.1.0 で追加した `ImportKind` enum の `Local` バリアントを活用し、
`parser.rs` の文字列 import ブランチで `./` prefix を検出して分岐する。

既存の `import rune "kafka"` / `import "models/user"` は引き続き `ImportKind::Legacy` として共存維持。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/parser.rs` | `Str` ブランチで `./` prefix 検出 → `ImportKind::Local`、それ以外は `ImportKind::Legacy` |
| `fav/src/driver.rs` | `v482000_tests` 追加（2テスト）・`v481000_tests::import_package_parses` は変更不要 |
| `fav/Cargo.toml` | version → `"48.2.0"` |
| `CHANGELOG.md` | v48.2.0 エントリ追加 |

---

## Parser 変更詳細

### 変更前（v48.1.0 時点）

`Str` ブランチは path を取り出すだけで `kind` は `Legacy` のまま:

```rust
let mut kind = ImportKind::Legacy;
let path = match self.peek().clone() {
    TokenKind::Str(path) => { self.advance(); path }
    // ...
};
```

### 変更後

`./` prefix の有無で `Local` / `Legacy` を判定:

```rust
let mut kind = ImportKind::Legacy;
let path = match self.peek().clone() {
    TokenKind::Str(path) => {
        self.advance();
        if path.starts_with("./") || path.starts_with("../") {
            kind = ImportKind::Local;
        }
        path
    }
    // ...
};
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `import_local_parses` | `import "./src/helpers" as helpers` が `ImportKind::Local`、`path == "./src/helpers"` でパースされる |
| `import_local_relative_path` | `import "../utils/common"` が `ImportKind::Local`、`path == "../utils/common"` でパースされる |

```rust
#[test]
fn import_local_parses() {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;
    let src = "import \"./src/helpers\" as helpers\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let found = program.items.iter().any(|i| matches!(
        i,
        Item::ImportDecl { kind: ImportKind::Local, path, alias: Some(a), .. }
            if path == "./src/helpers" && a == "helpers"
    ));
    assert!(found, "import \"./src/helpers\" as helpers should parse as ImportKind::Local");
}

#[test]
fn import_local_relative_path() {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;
    let src = "import \"../utils/common\"\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let found = program.items.iter().any(|i| matches!(
        i,
        Item::ImportDecl { kind: ImportKind::Local, path, alias: None, .. }
            if path == "../utils/common"
    ));
    assert!(found, "import \"../utils/common\" should parse as ImportKind::Local with no alias");
}
```

テスト数: 3047 → **3049**（+2）

---

## 注意事項

- `ast.rs` の `ImportKind` enum は v48.1.0 で追加済みのため変更不要（`Local` バリアントは既に定義済み）。
- `ImportDecl` のフィールド構造も v48.1.0 で確定済みのため `ast.rs` の変更は不要。
- `../` prefix も Local として扱う（相対パス全般）。
- `driver.rs` の実ファイル解決ロジック（`Local` import に対応した `collect_sources` 等）は v48.3.0 以降の対象。本バージョンは parser のみ。
- `is_rune` フラグとの相互作用: `"./"` prefix を持つパスは `.` を含むため、現行の `is_rune` 判定（`!path.contains('/') && !path.contains('.')`）により常に `false` となる。`ImportKind::Local` が設定された場合は必ず `is_rune = false` になることが保証される。
- driver.rs テストコードは raw string `r#"..."#` ではなくエスケープ文字列 `"..."` 形式で記述すること（v48.1.0 テストとスタイルを統一）。
- site/ MDX の更新（`module-system.mdx` 等）は v48.9.0 のスコープ。本バージョンでは不要。

---

## 完了条件

- `cargo test` 3049 passed, 0 failed（3047 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.2.0"`
- `CHANGELOG.md` に v48.2.0 エントリ追加
- `versions/current.md` を v48.2.0（3049 tests）に更新、進行中バージョンを `v48.3.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T2 全 `[x]`）
