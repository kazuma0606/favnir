# v61.1.0 Plan — OR パターン強化（ネスト・型チェック・lint 統合）

Date: 2026-07-31
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/middle/checker.rs` | 変更 | `Pattern::Or` 全アーム型チェック |
| `fav/src/lint.rs` | 変更 | `pattern_lit_keys_all` 追加 + W037 拡張 |
| `fav/src/driver.rs` | 追加 | `v61100_tests` モジュール（テスト 2 件） |

AST 変更なし。新規ファイル追加なし。

---

## 実装ステップ

### Step 1: `checker.rs` — Pattern::Or 全アーム型チェック

`Pattern::Or` アームで `pats.first()` のみ処理している箇所を全アーム処理に変更。

**変更前**（Line ~4210）:
```rust
Pattern::Or(pats, _) => {
    if let Some(first) = pats.first() {
        self.check_pattern_bindings(first, ty);
    }
}
```

**変更後**:
```rust
// v61.1.0: OR パターン全アームを型チェック（従来は first のみ）
Pattern::Or(pats, _) => {
    for pat in pats {
        self.check_pattern_bindings(pat, ty);
    }
}
```

### Step 2: `lint.rs` — W037 OR パターン対応拡張

`pattern_lit_key` 関数の直前に `pattern_lit_keys_all` を追加する。

```rust
/// v61.1.0: パターン内の全リテラルキーを再帰的に収集（OR パターン対応）。
fn pattern_lit_keys_all(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Lit(lit, _) => vec![format!("{:?}", lit)],
        Pattern::Or(pats, _) => pats.iter().flat_map(|p| pattern_lit_keys_all(p)).collect(),
        _ => vec![],
    }
}
```

`check_expr_for_unreachable` 内の重複チェック部分（Line ~2984）を変更:

**変更前**:
```rust
// リテラル重複チェック
if let Some(lit_key) = pattern_lit_key(&arm.pattern) {
    if !seen_lits.insert(lit_key.clone()) {
        errors.push(LintError::new(
            "W037",
            format!(
                "unreachable pattern: literal `{}` already matched above",
                lit_key
            ),
            arm.pattern.span().clone(),
        ));
    }
}
```

**変更後**:
```rust
// v61.1.0: OR パターン内リテラルも重複チェック対象に拡張
for lit_key in pattern_lit_keys_all(&arm.pattern) {
    if !seen_lits.insert(lit_key.clone()) {
        errors.push(LintError::new(
            "W037",
            format!(
                "unreachable pattern: literal `{}` already matched above",
                lit_key
            ),
            arm.pattern.span().clone(),
        ));
        break; // 1 アームにつき最初の重複のみ報告
    }
}
```

### Step 3: `driver.rs` — `v61100_tests` モジュール追加

`v61000_tests` モジュールの直前（上側）に挿入する。

3アーム OR パターン（`"active" | "pending" | "inactive"`）を使用して
ロードマップ要件「3段階 OR パターン E2E 確認」を同時に満たす。

```rust
// -- v61100_tests (v61.1.0) -- OR パターン強化 --
#[cfg(test)]
mod v61100_tests {
    use super::*;

    /// OR パターン全アームが型チェックを通過することを確認（v61.1.0: 全アーム処理）
    /// 3アーム OR パターン ("active" | "pending" | "inactive") で 3段階 E2E も兼ねる
    #[test]
    fn pattern_or_type_check_arms_same() {
        let src = concat!(
            "fn classify(status: String) -> String {\n",
            "  match status {\n",
            "    \"active\" | \"pending\" | \"inactive\" => \"processing\"\n",
            "    \"deleted\" | \"archived\" => \"done\"\n",
            "    _ => \"unknown\"\n",
            "  }\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "OR pattern with consistent string literals (3-arm) should pass type check; \
             errors: {:?}",
            errors
        );
    }

    /// W037 が OR パターン内重複リテラルを検出することを確認（v61.1.0: lint 統合）
    #[test]
    fn pattern_or_lint_w037_integration() {
        let src = concat!(
            "fn f(x: String) -> String {\n",
            "  match x {\n",
            "    \"a\" | \"b\" => \"first\"\n",
            "    \"a\" => \"duplicate\"\n",
            "    _ => \"other\"\n",
            "  }\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let warnings = crate::lint::check_unreachable_patterns(&prog);
        assert!(
            warnings.iter().any(|w| w.code == "W037"),
            "W037 should fire when literal 'a' appears in OR pattern and later standalone; \
             warnings: {:?}",
            warnings
        );
    }
}
```

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `checker.rs` Pattern::Or | Line ~4210 の `if let Some(first)` を `for pat in pats` に変更 |
| `lint.rs` `pattern_lit_keys_all` | `pattern_lit_key` 関数の直前に追加 |
| `lint.rs` 重複チェック | `if let Some(lit_key) = pattern_lit_key(...)` を `for lit_key in pattern_lit_keys_all(...)` に変更 |
| `v61100_tests` | `driver.rs` の `v61000_tests` の直前（上側） |

---

## 注意点

- `check_pattern_bindings` は `&mut self` メソッドなので `for pat in pats` でのループは問題なし。
- `pattern_lit_keys_all` で OR パターンが重複する場合、最初のヒットで `break` して 1 件のみ報告する（W037 の既存方針に合わせる）。
- `pattern_lit_key` は差し替え後に未使用関数になる。`#[allow(dead_code)]` を付与するか、呼び出し元がなくなった時点で削除する（Rust `dead_code` 警告防止）。
- 部分重複 OR パターン（後続アームが OR パターンの場合）は OUT スコープ。実装上は後続 OR アームの個々のリテラルも `seen_lits` に登録されるが、既存の `catch_all_seen` チェックで先行 catch-all は検出済みのため問題なし。
- 実際のテスト数目標: **3353 + 2 = 3355**（ロードマップ記載 3354 + XSS オフセット +1）。
