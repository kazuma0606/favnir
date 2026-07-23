# Plan: v48.2.0 — import 構文刷新（ローカルファイル）

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/parser.rs` | `Str` ブランチで `./` / `../` prefix 検出 → `ImportKind::Local` |
| `fav/src/driver.rs` | `v482000_tests` モジュール追加（2テスト） |
| `fav/Cargo.toml` | version → `"48.2.0"` |
| `CHANGELOG.md` | v48.2.0 エントリ追加 |
| `versions/current.md` | v48.2.0 に更新、進行中 v48.3.0 |
| `versions/v45-v50/v48.2.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### Step 1: `parser.rs` — `Str` ブランチで `./` / `../` 検出

`parse_import_decl` 関数内の `let mut kind = ImportKind::Legacy;` から始まる `match` ブロックを編集する。

**変更前 (`Str` ブランチ):**

```rust
let mut kind = ImportKind::Legacy;
let path = match self.peek().clone() {
    TokenKind::Str(path) => { self.advance(); path }
    // ...
};
```

**変更後 (`Str` ブランチに prefix 判定を追加):**

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

`Ident` ブランチ（bare ident → `ImportKind::Package`）は v48.1.0 で既に実装済みのため変更不要。

### Step 2: `driver.rs` — `v482000_tests` 追加

挿入位置: `v481000_tests` の直前。

```rust
// -- v482000_tests (v48.2.0) -- import 構文刷新: Local import パース確認 --
#[cfg(test)]
mod v482000_tests {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;

    #[test]
    fn import_local_parses() {
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
        let src = "import \"../utils/common\"\nfn main() -> Bool { true }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let found = program.items.iter().any(|i| matches!(
            i,
            Item::ImportDecl { kind: ImportKind::Local, path, .. } if path == "../utils/common"
        ));
        assert!(found, "import \"../utils/common\" should parse as ImportKind::Local");
    }
}
```

### Step 3: `Cargo.toml` version → `"48.2.0"`

`v481000_tests::import_package_parses` 等のスタブ化は不要（`cargo_toml_version_is_48_1_0` テストは `v481000_tests` に存在しないため）。

### Step 4: `CHANGELOG.md` 更新

```markdown
## [v48.2.0] — 2026-07-18 — import 構文刷新（ローカルファイル）

### Added
- `parser.rs`: `"./..."` / `"../..."` prefix の import を `ImportKind::Local` として解析
- `driver.rs`: `v482000_tests` 追加（`import_local_parses` / `import_local_relative_path` 2テスト）

### Changed
- `Cargo.toml` version: `48.1.0` → `48.2.0`
```

---

## 実装順序

1. `parser.rs` の `Str` ブランチに `./` / `../` 判定を追加
2. `cargo build` でコンパイルエラーがないことを確認（ast.rs は変更不要なのでエラーなし）
3. `driver.rs` に `v482000_tests` を `v481000_tests` 直前に追加
4. `Cargo.toml` version → `"48.2.0"`
5. `CHANGELOG.md` v48.2.0 エントリ追加
6. `cargo test` で 3049 passed, 0 failed を確認
7. `cargo clippy -- -D warnings` クリーン確認
8. `versions/current.md` 更新（v48.2.0、次 v48.3.0）
9. `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.2.0 完了条件テスト数（3049）を実績として記入
10. `tasks.md` COMPLETE に更新

---

## 注意事項

- `ast.rs` は v48.1.0 で `ImportKind::Local` が既に定義済み → **変更不要**。
- `ImportDecl` フィールドも確定済み → `kind: _` の追記等は発生しない。
- `v48100_tests`（v48.1.0）に `cargo_toml_version_is_48_1_0` は存在しないため、スタブ化は不要。
- テストの raw string の扱い: `driver.rs` のテスト内は通常文字列（`\"`）でエスケープ。spec.md は raw string `r#"..."#` どちらでも可だが、driver.rs テスト内は `"..."` で `\"` エスケープを使うこと。
