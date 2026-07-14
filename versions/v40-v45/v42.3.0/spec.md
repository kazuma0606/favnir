# v42.3.0 仕様書 — CEP checker.fav 統合

## 概要

v42.1.0 / v42.2.0 で構築した `cep pattern` AST・パーサーに対して、
型チェッカー（checker.rs Pass 2）でセマンティクス検証を追加する。
`within` 値が 0 の場合に E0420 を報告する。

---

## 背景・動機

v42.2.0 時点の `checker.rs` は `CepPatternDef` をスタブアームで無視している。
v42.3.0 でスタブを取り除き、`within_secs == Some(0)` を不正な値として検出する。
`within 0` は「0 秒以内に発生する」という意味で意味論的に不正である。

---

## 実装スコープ

### 1. `error_catalog.rs` — E0420 追加

E0406 の直後（E05xx セクションの直前）に追加:

```rust
// ── E042x: CEP パターン (v42.3.0) ─────────────────────────────────────────
ErrorEntry {
    code: "E0420",
    title: "cep pattern within_secs must be positive",
    category: "types",
    description: "The `within` value in a `cep pattern` clause must be a positive integer (≥ 1). `within 0` is semantically invalid.",
    example: "cep pattern P { Login within 0 }  // E0420",
    fix: "Use `within N` where N ≥ 1 (e.g., `within 60`).",
},
```

### 2. `checker.rs` — Pass 2 スタブ解除・`check_cep_pattern_def` 追加

**Pass 2 の変更**（line 2413 付近）:

```rust
// 変更前:
Item::CepPatternDef(_) => {} // v42.1.0: 型チェックは v42.3.0

// 変更後:
Item::CepPatternDef(cd) => self.check_cep_pattern_def(cd), // v42.3.0
```

**`check_cep_pattern_def` メソッド追加**（`check_abstract_trf_def` の直前付近）:

```rust
fn check_cep_pattern_def(&mut self, cd: &CepPatternDef) {
    for clause in &cd.body {
        if clause.within_secs == Some(0) {
            self.errors.push(TypeError::new(
                "E0420",
                "`within 0` is not valid; use a positive integer (within N where N ≥ 1)",
                clause.span.clone(),
            ));
        }
    }
}
```

> **型確認メモ**:
> - `clause.within_secs` は `Option<i64>` 型。`== Some(0)` は Rust 標準の `PartialEq for Option<i64>` が適用される。`CepClause` 自体に `PartialEq` derive は不要。
> - `clause.span` は `Span` 型（`ast.rs` の `CepClause` フィールド定義参照）。`TypeError::new` の第3引数 `span: Span` と一致する。

Pass 1 のスタブ（`| Item::CepPatternDef(..) => {}`）は変更なし（署名登録不要）。

### 3. `checker.fav` — 設計コメント更新

v42.1.0 で追加した「v42.3.0 以降に実装」コメントを「E0420 実装済み」に更新。

### 4. テスト（3件）— `v42300_tests`

- `cargo_toml_version_is_42_3_0`（NOTE コメント付き）
- `cep_e0420_within_zero` — `cep pattern P { Login within 0 }` → checker が E0420 を 1 件返す
- `e0420_in_error_catalog` — `error_catalog::lookup("E0420")` が `Some` を返す

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_3_0` | Cargo.toml に "42.3.0" が含まれる |
| `cep_e0420_within_zero` | `within 0` で E0420 が報告される |
| `e0420_in_error_catalog` | E0420 がカタログに登録されている |

**推定テスト数**: 2880 + 3 = **2883**

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/error_catalog.rs` | 変更 | E0420 エントリ追加 |
| `fav/src/middle/checker.rs` | 変更 | Pass 2 スタブ解除、`check_cep_pattern_def` 追加 |
| `fav/self/checker.fav` | 変更 | 設計コメント更新（E0420 実装済みに）|
| `fav/src/driver.rs` | 変更 | `v42300_tests` 3 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.2.0` → `42.3.0` |
| `CHANGELOG.md` | 変更 | `[v42.3.0]` エントリ追加 |

---

## 非スコープ

- `seq` / `any` / `not` 内部の再帰的な `within_secs` チェック（CepExpr は現状 within を持たない）
- イベント名の型環境検証（ロードマップ原文の「型変数を checker.fav で検証」に相当）: イベント型システム（`Event<T>` 型）が未整備のため **v44.x に延期**。本バージョンの実装は `within` 値の数値検証のみ
- checker.fav（Favnir 自己ホスト側）への E0420 ロジック移植（v43.x 以降）
- Pass 1 での CepPatternDef 署名登録（不要）
