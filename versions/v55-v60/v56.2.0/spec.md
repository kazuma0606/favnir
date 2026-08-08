# Spec — v56.2.0 — 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）

## 概要

`T with Ord with Serialize` 形式の複数 `with` constraint の動作確認（既存 parser サポート済み）と、
coherence ルール（同一型に対する重複 `impl` の禁止）の checker ロジック強化を行う。
違反は新規エラーコード E0423 で報告する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.2.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.2.0 行
- ベーステスト数: **3229**（v56.1.0 完了時点の実績値）
- 目標テスト数: **3231**（+2）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `where T: Interface`（`T with Ord` 形式）解析 | v33.0 | 実装済み |
| 複数 `with` bound 解析（`T with A with B`） | v33.0 | 実装済み（parser 対応済み） |
| `TypeConstraint::Interface(String)` | v18.2.0 | 実装済み |
| `InterfaceRegistry.impls: HashMap<(String,String), InterfaceImplEntry>` | v13.0 | 実装済み |
| `InterfaceImplEntry { methods, is_auto: bool }` | v13.0 | 実装済み |
| E0422（Interface 境界違反） | v56.1.0 | 実装済み |
| E0423（coherence 違反） | v56.2.0 | **本バージョンで追加** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.2.0"
```

（v56.1.0 で Cargo.toml 更新が未実施だったため、本バージョンで `56.2.0` へ直接更新した。）

---

### 2. `fav/src/error_catalog.rs` — E0423 エントリ追加

E0422 エントリの直後（E05xx セクションの直前）に挿入する。

```rust
// v56.2.0: impl coherence violation
ErrorEntry {
    code: "E0423",
    title: "duplicate impl: coherence violation",
    category: "types",
    description: "An interface is implemented more than once for the same type. \
                  Favnir enforces coherence: each (interface, type) pair may have at most one `impl` block.",
    example: "interface Greet { hello: Self -> String }\ntype Foo = { x: Int }\nimpl Greet for Foo { hello = |s| \"hello\" }\nimpl Greet for Foo { hello = |s| \"world\" }  // E0423: duplicate impl",
    fix: "Remove the duplicate `impl` block, or merge the methods into a single `impl Greet for Foo`.",
    suggestion: Some("Ensure each interface is implemented at most once per type."),
},
```

> **注意**: impl メソッド構文は `method_name = expr`（`=` で body を定義）。
> interface メソッド宣言構文は `method_name: TypeExpr`（`:` で型を宣言）。

---

### 3. `fav/src/middle/checker.rs` — coherence check 追加

#### 3a. `InterfaceRegistry` に `is_explicitly_implemented` メソッドを追加

`is_implemented` の直後（`lookup_declared_method` の直前）に挿入する。

```rust
/// Returns true only when the existing impl is user-declared (is_auto = false).
/// Used for coherence checking (E0423, v56.2.0) — built-in impls must not trigger duplicates.
pub fn is_explicitly_implemented(&self, interface_name: &str, type_name: &str) -> bool {
    self.impls
        .get(&(interface_name.to_string(), type_name.to_string()))
        .map(|e| !e.is_auto)
        .unwrap_or(false)
}
```

#### 3b. `check_interface_impl_decl` に coherence check を追加

`for interface_name in &id.interface_names` ループ内、`register_impl` 呼び出しの直前に挿入する。

```rust
// Coherence check (E0423, v56.2.0): detect duplicate impl for same (interface, type) pair.
// Only fires when the existing impl is also user-declared (!is_auto) to avoid
// flagging built-in stdlib impls as coherence violations.
if !id.is_auto
    && self
        .interface_registry
        .is_explicitly_implemented(interface_name, &id.type_name)
{
    self.type_error(
        "E0423",
        format!(
            "duplicate impl of `{}` for `{}`: coherence violation",
            interface_name, id.type_name
        ),
        &id.span,
    );
    continue; // skip registration — duplicate impl rejected
}
```

> **注意**: `continue` で registration をスキップするのは意図的。
> `TypeConstraint::HasField`（E0337）は変更しない。
> built-in impl は `is_auto = true` であり `is_explicitly_implemented` は `false` を返すため、
> stdlib の組み込み impl が誤って E0423 を発行することはない。

---

### 4. `fav/src/driver.rs` — v56200_tests 追加 + v56000_tests 更新

#### 4a. `v56000_tests` から `cargo_toml_version_is_56_0_0` を削除

Cargo.toml が `56.2.0` に更新されるため、バージョン検証テストは v56200_tests に移す。

#### 4b. `v56200_tests` モジュールを `v56100_tests` の直前に挿入

```rust
// -- v56200_tests (v56.2.0) -- 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）--
#[cfg(test)]
mod v56200_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v56200_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_56_2_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"56.2.0\""),
            "Cargo.toml version should be 56.2.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn where_multiple_constraints() {
        // 複数 bound (T with Ord with Serialize) が正しく動作することを確認
        let errors = check_errors(r#"
interface Serialize { to_str: Self -> String }
impl Serialize for Int { to_str = |s| "int" }
fn pick<T with Ord with Serialize>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn main() -> Int {
    pick(3, 7)
}
"#);
        assert!(
            errors.is_empty(),
            "Multiple bounds should not emit errors for valid type, got: {:?}",
            errors
        );
    }

    #[test]
    fn impl_coherence_violation() {
        // 同一 (interface, type) ペアに重複 impl があると E0423 が出る
        let errors = check_errors(r#"
interface Greet { hello: Self -> String }
type Foo = { x: Int }
impl Greet for Foo { hello = |s| "hello" }
impl Greet for Foo { hello = |s| "world" }
"#);
        assert!(
            errors.iter().any(|e| e == "E0423"),
            "Expected E0423 for duplicate impl, got: {:?}",
            errors
        );
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_56_2_0` | Cargo.toml が `56.2.0` を反映 |
| `where_multiple_constraints` | `Int with Ord with Serialize` — 両 bound を満たす → エラーなし（`errors.is_empty()`） |
| `impl_coherence_violation` | Greet for Foo を 2 回 impl → E0423 が emitted される |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3231 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_56_2_0` pass
- `where_multiple_constraints` pass（`errors.is_empty()` assert）
- `impl_coherence_violation` pass（E0423 assert）
- `v56000_tests::cargo_toml_version_is_56_0_0` が削除されている
- `error_catalog.rs` に E0423 エントリが含まれる（正しい Favnir 構文 `hello = |s|`）
- `checker.rs` に `is_explicitly_implemented` が追加されている
- `checker.rs` の coherence check が E0423 を emit する（built-in impl は対象外）
- `checker.rs` の `continue` に `// skip registration — duplicate impl rejected` コメントあり
- `CHANGELOG.md` に v56.2.0 エントリが追加されている（version: `56.1.0 → 56.2.0`）
- `versions/current.md` が v56.2.0 / 3231 tests を反映
- `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.2.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.2.0 実績欄も COMPLETE に更新

---

## 備考

- 複数 `with` 制約（`T with Ord with Serialize`）は v33.0 の parser で既にサポート済み。
  本バージョンでは新規 parser 変更なく、checker の coherence check 追加のみ。
- `is_auto` フラグにより built-in（stdlib）の自動登録 impl は coherence check 対象外。
  ユーザー宣言の重複 impl のみ E0423 を発行する。
- `InterfaceImplDecl.is_auto` は、`impl Interface for Type` に `{ body }` ブロックがない場合に
  `true` になる（parser が自動設定）。body ありの場合は `false`（ユーザー宣言）。
- **Favnir impl/interface 構文（重要）**:
  - interface メソッド宣言: `method_name: TypeExpr`（コロン、`fn` キーワード不使用）
  - impl メソッド定義: `method_name = expr`（等号、`fn` キーワード不使用）
