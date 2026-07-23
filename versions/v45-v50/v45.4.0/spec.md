# Spec: v45.4.0 — `match` 網羅性改善 + W034 / E0416

Date: 2026-07-16
Sprint: Language Refinement (v45.1〜v46.0)

---

## 概要

`checker.rs` の `check_match_arms` に **Sum 型の網羅性チェック**を追加する。
非網羅 match を文脈によって W034（警告）または E0416（ハードエラー）として報告する。

- **文として使う match** (Stmt::Expr) → W034 — 「カバーされていないバリアント」警告
- **値として使う match** (let/bind/return 等の式文脈) → E0416 — 非網羅ハードエラー

## 動機

```favnir
type Color = Red | Green | Blue

// W034: 文として使う match — Blue がカバーされていない
match color {
  Red   -> process_red()
  Green -> process_green()
}

// E0416: 値として使う match — Blue がカバーされていない → コンパイルエラー
let label = match color {
  Red   -> "red"
  Green -> "green"
}
```

現在 (`check_match_arms`) はアーム間の型一貫性チェック (E0101) しか行わない。
本バージョンで **全バリアント列挙 → 未カバー検出** を追加する。

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `check_match_arms` に `value_ctx: bool` 追加 / `collect_covered_variants` ヘルパー追加 / 網羅性チェックロジック追加 / `check_stmt` で Stmt::Expr の Match を stmt ctx で呼び出すように変更 |
| `fav/src/error_catalog.rs` | E0416 エントリ追加（予約コメント → 実装） |
| `fav/src/driver.rs` | `v454000_tests` モジュール追加（3件） |
| `fav/Cargo.toml` | version `45.3.0` → `45.4.0` |
| `CHANGELOG.md` | v45.4.0 エントリ追加 |

## 設計詳細

### 1. `check_match_arms` シグネチャ変更

```rust
fn check_match_arms(
    &mut self,
    arms: &[MatchArm],
    scrutinee_ty: &Type,
    span: &Span,
    value_ctx: bool,   // true = 値文脈(E0416) / false = 文文脈(W034)
) -> Type
```

既存の 1 か所の呼び出し元 (`Expr::Match` in `check_expr`) は `value_ctx: true` で呼ぶ。
`check_stmt` の `Stmt::Expr` で `Expr::Match` を検出したときは `value_ctx: false` で呼ぶ。

### 2. `check_stmt` の変更

```rust
Stmt::Expr(e) => {
    if let Expr::Match(scrutinee, arms, span) = e {
        let scrutinee_ty = self.check_expr(scrutinee);
        self.check_match_arms(arms, &scrutinee_ty, span, false); // stmt ctx
    } else {
        self.check_expr(e);
    }
}
```

### 3. `collect_covered_variants` / `collect_pattern_variants` ヘルパー

```rust
/// Match アームのパターン群から「カバーされたバリア���ト名」の集合と catch-all フラグを返す。
/// ガード付きアームは網羅性に寄与しない（ガードが失敗すれば通過しないため）。
fn collect_covered_variants(arms: &[MatchArm]) -> (Vec<String>, bool)
// → (covered_variant_names, has_catch_all)

/// collect_covered_variants の内部補助関数。1 パターンを再帰的に解析する。
fn collect_pattern_variants(pat: &Pattern, covered: &mut Vec<String>, has_catch_all: &mut bool)
```

- `Pattern::Wildcard` / `Pattern::Bind(_)` かつガードなし → `has_catch_all = true`
- `Pattern::Variant(name, ..)` かつガードなし → `name` を追加
- `Pattern::Or(pats)` かつガードなし → 各パターンを再帰処理
- ガード付きアームはスキップ

### 4. 網羅性チェックロジック

```rust
// check_match_arms 末尾に追加（既存の型一貫性チェック後）
if let Type::Named(type_name, _) = scrutinee_ty {
    if let Some(TypeBody::Sum(variants)) = self.type_defs.get(type_name.as_str()) {
        let all_variants: Vec<String> = variants.iter().map(|v| v.name().to_string()).collect();
        let (covered, has_catch_all) = collect_covered_variants(arms);
        if !has_catch_all {
            let missing: Vec<&str> = all_variants
                .iter()
                .filter(|v| !covered.contains(*v))
                .map(|s| s.as_str())
                .collect();
            if !missing.is_empty() {
                let msg = format!(
                    "non-exhaustive match: {} not covered",
                    missing.join(", ")
                );
                if value_ctx {
                    self.type_error("E0416", msg, span);
                } else {
                    self.type_warning("W034", msg, span);
                }
            }
        }
    }
}
```

`collect_covered_variants` は `check_match_arms` の外側でフリー関数として定義（`&self` 不要）。

### 5. E0416 エントリ (`error_catalog.rs`)

```rust
ErrorEntry {
    code: "E0416",
    title: "non-exhaustive match in value context",
    category: "types",
    description: "A `match` expression used in value context does not cover all variants of the \
                  scrutinee's sum type. All variants must be handled when a `match` produces a value.",
    example: "type C = A | B\nlet x = match c { A -> 1 }  // E0416: B not covered",
    fix: "Add arms for missing variants, or add a wildcard `_ -> ...` arm.",
},
```

既存の `// ── E0416〜E0419: 予約（将来拡張用）` コメントを置換する。

### スコープ制限（このバージョン外）

- `Result<T, E>` / `Option<T>` のような組み込みジェネリック型の網羅チェック → 対象外（Sum 型のみ）
- ネストしたパターン (`Some(Red)` 等) の完全な網羅解析 → 対象外（トップレベルバリアント名のみ）
- `--deny-warnings` による W034 → E0416 格上げは既存の `--deny-warnings` フラグで対応済み
- `lint.rs` の変更は不要（W034 は checker 内部の `type_warning` で発行する。`lint.rs` 先頭コメントへの追記は任意）

### E0101 と W034/E0416 の共起

E0101（アーム型不一致）と W034/E0416 が同一 match で発生した場合、両エラー/警告を独立して発行する。一方の発行が他方を抑制することはない。

## 完了条件

- `cargo test` 全通過（**2977 tests** passed, 0 failed）
- `v454000_tests` の 3 件が pass:
  - `match_exhaustive_ok`
  - `match_w034_missing_variant`
  - `match_e0416_value_context`
- `cargo clippy --locked -D warnings` クリーン
- `CHANGELOG.md` に v45.4.0 エントリ追加
