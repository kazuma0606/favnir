# Spec: v48.5.0 — import エイリアス完全化 + 旧構文 deprecation

## 概要

旧 `import rune "kafka"` 構文を **W035 警告（非推奨）** 化する。
`import kafka as k` の完全サポートを確認テストで明示し、
`lint.rs` に `W035: legacy_import_rune` ルールを追加する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/lint.rs` | `check_w035_legacy_import_rune` 関数追加・`run_lint` に登録 |
| `fav/src/driver.rs` | `v485000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.5.0"` |
| `CHANGELOG.md` | v48.5.0 エントリ追加 |

---

## 実装詳細

### W035 ルール仕様

- **コード**: `W035`
- **名称**: `legacy_import_rune`
- **対象**: `Item::ImportDecl { kind: ImportKind::Legacy, .. }`
- **発火条件**: `ImportKind::Legacy` の import（`import rune "kafka"` / `import "kafka"` / `import runes/kafka`）

W035 は `lint.rs` の `run_lint` 関数に追加する（W034 コメントの直後）。

### `check_w035_legacy_import_rune` 実装

```rust
// ── W035: legacy_import_rune (v48.5.0) ────────────────────────────────────────

/// W035: 旧 `import rune "kafka"` / `import "kafka"` 構文は非推奨。
/// `import kafka` または `import "./path"` を使用すること。
fn check_w035_legacy_import_rune(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::ImportDecl { kind, path, span, .. } = item {
            if *kind == ImportKind::Legacy {
                errors.push(LintError {
                    code: "W035".to_string(),
                    message: format!(
                        "legacy import syntax `import rune \"{}\"` is deprecated; \
                         use `import {}` (package) or `import \"./path\"` (local) instead",
                        path, path
                    ),
                    span: span.clone(),
                });
            }
        }
    }
}
```

### `run_lint` への登録

W034 コメントの直後に追加:

```rust
// v48.5.0: W035
check_w035_legacy_import_rune(program, &mut errors);
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `import_alias_resolves` | `import postgres as db` → `ImportDecl { kind: Package, alias: Some("db") }` であることを確認 |
| `legacy_import_rune_w035` | `import rune "kafka"` を `run_lint` に渡すと W035 が 1 件発生することを確認 |

```rust
#[test]
fn import_alias_resolves() {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;
    let src = "import postgres as db\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let Item::ImportDecl { kind, alias, path, .. } = &program.items[0] else {
        panic!("expected ImportDecl");
    };
    assert_eq!(*kind, ImportKind::Package);
    assert_eq!(alias.as_deref(), Some("db"));
    assert_eq!(path, "postgres");
}

#[test]
fn legacy_import_rune_w035() {
    use crate::frontend::parser::Parser;
    use crate::lint::run_lint;
    let src = "import rune \"kafka\"\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let warnings = run_lint(&program);
    let w035: Vec<_> = warnings.iter().filter(|w| w.code == "W035").collect();
    assert_eq!(w035.len(), 1, "W035 must fire for legacy import rune syntax");
    assert!(w035[0].message.contains("kafka"));
}
```

テスト数: 3053 → **3055**（+2）

---

## 注意事項

- `ImportKind::Legacy` は AST に既存（v48.1.0 追加）— 新規 enum 追加不要。
- W035 は `lint.rs` 経由（`run_lint` の返り値）。`checker.rs` / `type_warning` チャネルは使わない。
- W034 は checker.rs 発行なので混同しないこと。
- `check_w035_legacy_import_rune` で `ImportKind` を参照するため、`use crate::ast::ImportKind;` が必要（`use crate::ast::*` で既にインポート済みのため追加不要）。
- `import rune "kafka"` 構文は **削除しない**（W035 警告のみ）。削除は v49.0.0 以降のスコープ。
- **E0417 実発行（`checker.rs` での `ImportKind::Package` × `FavToml.runes` 突き合わせ）は v48.5.0 のスコープ外**。ロードマップ v48.3.0 の「スコープ分割注記」に記載があるが、v48.5.0 では lint W035 の追加のみを行う。

---

## 完了条件

- `cargo test` 3055 passed, 0 failed（3053 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.5.0"`
- `CHANGELOG.md` に v48.5.0 エントリ追加
- `versions/current.md` を v48.5.0（3055 tests）に更新、進行中バージョンを `v48.6.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
