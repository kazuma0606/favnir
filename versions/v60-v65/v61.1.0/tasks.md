# v61.1.0 Tasks — OR パターン強化（ネスト・型チェック・lint 統合）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3353 tests passed, 0 failed であることを確認
  （注: ロードマップ記載は 3352 だが v60.8.0 で XSS テスト追加のため実際は 3353）
- [x] `fav/Cargo.toml` のバージョンが `"61.0.0"` であることを確認
  - `grep '^version' fav/Cargo.toml` → `version = "61.0.0"`
- [x] `v61100_tests` がまだ存在しないことを確認
  - `grep -c 'v61100_tests' fav/src/driver.rs` = 0 件
- [x] `v61000_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v61000_tests' fav/src/driver.rs` ≥ 1 件
- [x] `Pattern::Or` の checker 処理が `pats.first()` のみであることを確認
  - `grep -n 'pats.first' fav/src/middle/checker.rs` ≥ 1 件
- [x] `pattern_lit_key` が `lint.rs` に存在することを確認
  - `grep -c 'fn pattern_lit_key' fav/src/lint.rs` ≥ 1 件
- [x] `pattern_lit_keys_all` がまだ存在しないことを確認
  - `grep -c 'fn pattern_lit_keys_all' fav/src/lint.rs` = 0 件

---

## T1: `checker.rs` — Pattern::Or 全アーム型チェック

`Pattern::Or(pats, _)` の処理で `if let Some(first) = pats.first()` を
`for pat in pats` ループに変更する。

```rust
// 変更前
Pattern::Or(pats, _) => {
    if let Some(first) = pats.first() {
        self.check_pattern_bindings(first, ty);
    }
}

// 変更後
// v61.1.0: OR パターン全アームを型チェック（従来は first のみ）
Pattern::Or(pats, _) => {
    for pat in pats {
        self.check_pattern_bindings(pat, ty);
    }
}
```

- [x] `checker.rs` の `Pattern::Or` アームを全アーム処理に変更した
- [x] `if let Some(first) = pats.first()` が削除され `for pat in pats` になっている
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T2: `lint.rs` — W037 OR パターン対応拡張

### T2-1: `pattern_lit_keys_all` 関数を追加

`pattern_lit_key` 関数の直前に追加する。
また、`pattern_lit_key` は差し替え後に未使用になるため `#[allow(dead_code)]` を付与する。

```rust
/// v61.1.0: パターン内の全リテラルキーを再帰的に収集（OR パターン対応）。
fn pattern_lit_keys_all(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Lit(lit, _) => vec![format!("{:?}", lit)],
        Pattern::Or(pats, _) => pats.iter().flat_map(|p| pattern_lit_keys_all(p)).collect(),
        _ => vec![],
    }
}

// 既存関数（v61.1.0 以降は pattern_lit_keys_all で代替。将来削除予定）
#[allow(dead_code)]
fn pattern_lit_key(pat: &Pattern) -> Option<String> { ... }
```

- [x] `pattern_lit_keys_all` を `pattern_lit_key` 直前に追加した
- [x] `pattern_lit_key` に `#[allow(dead_code)]` を付与した
- [x] `cargo build` でコンパイルエラーがないことを確認

### T2-2: 重複チェック部分を `pattern_lit_keys_all` に差し替え

`check_expr_for_unreachable` 内の重複チェック部分を変更する。

```rust
// 変更前
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

// 変更後
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

- [x] 重複チェック部分を `pattern_lit_keys_all` を使うように変更した
- [x] `break` が追加されている（1 アームにつき最初の重複のみ報告）
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T3: `driver.rs` — `v61100_tests` モジュール追加

`v61000_tests` モジュールの直前（上側）に挿入する。

```rust
// -- v61100_tests (v61.1.0) -- OR パターン強化 --
#[cfg(test)]
mod v61100_tests {
    use super::*;

    /// OR パターン全アームが型チェックを通過することを確認（v61.1.0: 全アーム処理）
    /// 3アーム OR パターン ("active" | "pending" | "inactive") でロードマップ要件の 3段階 E2E も兼ねる
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
            "OR pattern with consistent string literals should pass type check; errors: {:?}",
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

- [x] `v61100_tests` モジュールを `v61000_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `pattern_or_type_check_arms_same` テストが含まれている
- [x] `pattern_or_lint_w037_integration` テストが含まれている

---

## T4: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61100_tests::pattern_or_type_check_arms_same` pass
- [x] `v61100_tests::pattern_or_lint_w037_integration` pass
- [x] 総テスト数 **3355** tests passed, 0 failed を確認

---

## T5: 事後処理

- [x] `versions/current.md` を v61.1.0 / 3355 tests に更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.1.0 実績欄を更新
  - 実績欄に実際のテスト数（3355）と注記（ベース 3353 = ロードマップ記載 3352 + XSS テスト +1）を記録
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v62.0 でまとめて記載）
  - v62.0 記載範囲: v61.1〜v61.9 全機能
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

### 実装後 code-reviewer 指摘（4件）

- **[HIGH] checker.rs — OR パターン内 Bind 変数が複数回 `define` される**:
  `for pat in pats` ループに `HashSet<String> bound` を追加し、top-level `Pattern::Bind` の重複定義をスキップするよう修正。
- **[MED] lint.rs — 同一アーム内重複（`"a" | "a"`）が inter-arm W037 として誤報告される**:
  `arm_lits: HashSet<String>` を追加して intra-arm 重複を `continue` で除去し、`seen_lits` による inter-arm 重複チェックと分離。
- **[LOW] `pattern_lit_key` の `#[allow(dead_code)]` に TODO コメント追記**: 対応保留（v62.0 削除予定はコメント済み）
- **[LOW] テストにバインド変数を含む OR パターンがない**: スコープ外（v61.3.0 以降）

### 実装前の spec-reviewer 指摘（6件）を対応済み:
- [HIGH] E0009 スコープ明記（全アーム処理による自然発火 IN、独立アーム間比較 OUT）
- [HIGH] 3段階 OR パターンテスト追加（3アーム `"active"|"pending"|"inactive"` で E2E 兼用）
- [HIGH] ロードマップのテスト数全行 +1 修正
- [MED] 部分重複 OR パターンを OUT スコープとして明示
- [MED] `pattern_lit_key` に `#[allow(dead_code)]` 付与
- [LOW] T0 に `grep '^version' fav/Cargo.toml` 確認追記

---

Status: COMPLETE
