# v41.1.0 仕様書 — Refinement type 基盤

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v41.1.0 |
| テーマ | Type Precision — Refinement type 基盤 |
| 前バージョン | v41.0.0（2845 tests） |
| 目標テスト数 | 2848（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.1.0 |

---

## 背景と目的

v41.0「Streaming Foundations」でウィンドウ・Watermark 基盤を整備した。
本バージョンから「Type Precision」スプリントを開始する。

第 1 弾として **Refinement type 基盤** を追加する。
型エイリアス宣言に `where (条件)` 節を付けることで、値の制約を型レベルで表現できるようにする。

```fav
type Age  = Int    where |v| v >= 0 && v <= 150
type Name = String where |v| String.length(v) > 0

fn greet(name: Name) -> String { "Hello, " ++ name }
```

本バージョンは **構文解析（parser）と checker.fav スタブ** の追加に留まる。
静的違反検出（E0400 系）は v41.2.0 で追加する。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `fav/src/frontend/parser.rs` | `parse_type_def` の Alias 分岐に `where (expr)` 節を追加 |
| `fav/self/checker.fav` | `check_refinement_alias` スタブ関数を追加（実装は v41.2.0） |
| `fav/Cargo.toml` | version: `41.0.0` → `41.1.0` |
| `CHANGELOG.md` | `[v41.1.0]` エントリ追加（`[v41.0.0]` の直後） |
| `fav/src/driver.rs` | `cargo_toml_version_is_41_0_0` stub 化 + v41100_tests 追加 |

---

## `parser.rs` 変更設計

`parse_type_def` の Alias 分岐（`type Name = TypeExpr` の解析）に `where` 節を追加する。

**変更前（line 1394〜1405）:**

```rust
} else {
    // type alias: type Name = TypeExpr
    let target = self.parse_type_expr()?;
    return Ok(TypeDef {
        visibility,
        name,
        type_params,
        with_interfaces,
        invariants: vec![],
        body: TypeBody::Alias(target),
        span: self.span_from(&start),
    });
};
```

**変更後:**

```rust
} else {
    // type alias: type Name = TypeExpr
    let target = self.parse_type_expr()?;
    // v41.1.0: refinement constraint `where (expr)` for type aliases
    let invariants = if self.peek() == &TokenKind::Where {
        self.advance(); // consume `where`
        vec![self.parse_expr()?]
    } else {
        vec![]
    };
    return Ok(TypeDef {
        visibility,
        name,
        type_params,
        with_interfaces,
        invariants,
        body: TypeBody::Alias(target),
        span: self.span_from(&start),
    });
};
```

`TypeDef.invariants` フィールドは既存（v9.7.5 時点から）のため AST 変更不要。

---

## `checker.fav` 変更設計

`fav/self/checker.fav` に `check_refinement_alias` スタブを追加する。

```fav
// v41.1.0: Refinement type alias constraint check (stub — full impl in v41.2.0)
fn check_refinement_alias(ty_name: String, invariants: List<Expr>) -> Bool {
    // TODO: v41.2.0 で E0400 違反検出を実装
    true
}
```

**注意**: checker.fav の `TypeDef` 構造体（`fav/self/checker.fav`）は v41.1.0 時点では `invariants` フィールドを持たない。
`check_refinement_alias` スタブは v41.1.0 では呼び出し箇所を設けない（構文的な追加のみ）。
v41.2.0 のスコープで checker.fav の `TypeDef` に `invariants: List<Expr>` フィールドを追加し、このスタブを統合する。

---

## テスト設計（v41100_tests）

```rust
#[cfg(test)]
mod v41100_tests {
    #[test]
    fn cargo_toml_version_is_41_1_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.1.0"), "Cargo.toml must contain version 41.1.0");
    }

    #[test]
    fn changelog_has_v41_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v41.1.0]"), "CHANGELOG.md must contain [v41.1.0]");
    }

    #[test]
    fn refinement_type_alias_where_parseable() {
        use crate::frontend::parser::Parser;
        let src = "type Age = Int where |v| v >= 0";
        let result = Parser::parse_str(src, "test.fav");
        assert!(result.is_ok(), "Refinement type alias with where clause should parse without error");
    }
}
```

- `cargo_toml_version_is_41_1_0` / `changelog_has_v41_1_0`: `include_str!` のみ → `use super::*` 不要
- `refinement_type_alias_where_parseable`: `crate::frontend::parser::Parser` を直接参照 → `use super::*` 不要
- `v41100_tests` モジュールとして `use super::*` は **不要**

テスト数: 2845 + 3 = **2848**

**注**: ロードマップは「推定 2843（+3）」と記載しているが、v41.0.0 の実績（2845）を起点に 3 テスト構成（2848）とする（確立パターンに合わせる）。

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 | 検証方法 |
|---|------|----------|
| 1 | `Cargo.toml` の version が `41.1.0` | `cargo_toml_version_is_41_1_0` テスト |
| 2 | `CHANGELOG.md` に `[v41.1.0]` エントリが存在する | `changelog_has_v41_1_0` テスト |
| 3 | `type Age = Int where (>= 0)` が parser でエラーなく解析できる | `refinement_type_alias_where_parseable` テスト |
| 4 | `cargo test` 全通過（failures=0、テスト数 ≥ 2848） | cargo test |
| 5 | `v41100_tests` 3 件すべて pass | cargo test |

---

## ロードマップとの差異

- ロードマップは「推定 2843（+3）」と記載しているが、実績 2845 を起点に 3 テスト構成（2848）とする。
