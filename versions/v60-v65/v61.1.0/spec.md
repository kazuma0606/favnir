# v61.1.0 Spec — OR パターン強化（ネスト・型チェック・lint 統合）

Date: 2026-07-31
Status: 未着手

---

## 概要

`Pattern::Or` は v17.2.0 で `ast.rs` に実装済み（`Or(Vec<Pattern>, Span)`）。
本バージョンでは **AST 変更なし** で以下 2 点を強化する。

1. **checker.rs** — `Pattern::Or` の全アームを型チェック対象に拡張
   （現行は `pats.first()` のみ処理→全アームに拡張）
2. **lint.rs** — W037（到達不能パターン）を OR パターン内リテラル重複に対応拡張
   （現行の `pattern_lit_key` は `Pattern::Or` に対し `None` を返す → 全リテラル抽出に拡張）

---

## 検証対象

| 確認項目 | 実装箇所 | 検証テスト |
|---|---|---|
| OR パターン全アームが型チェックを通過する | `checker.rs` Pattern::Or arm | `pattern_or_type_check_arms_same` |
| W037 が OR パターン内重複リテラルを検出する | `lint.rs` W037 / `pattern_lit_keys_all` | `pattern_or_lint_w037_integration` |

---

## 実装スコープ

| 項目 | スコープ | 理由 |
|---|---|---|
| OR パターン全アームを型チェック対象に拡張 | **IN** | checker.rs の `pats.first()` → 全アームループに変更 |
| 全アーム処理による E0009 自然発火 | **IN** | 各アームが既存の `check_pattern_bindings` を通ることで型不一致時に E0009 が自然に発行される |
| W037: OR パターン内重複リテラル検出 | **IN** | `pattern_lit_keys_all` 追加 + 重複チェック拡張 |
| 3段階（3アーム）OR パターンの E2E 確認 | **IN** | `"active" \| "pending" \| "inactive"` を含む 3アームテストを追加 |
| アーム間型一貫性の独立検証（スクルーティニー型と独立した比較） | **OUT** | v61.1.0 では各アームを既存 checker で独立チェック。アーム同士を比較する専用ロジックは v63.x 以降 |
| 部分重複 OR パターン（後続アームが OR パターンの場合の overlap 解析） | **OUT** | `"a"\|"b"` の後に `"b"\|"c"` が来る場合の partial overlap は対象外 |
| OR パターン各アーム個別ガード | **OUT** | v61.3.0 スコープ（AST 変更が必要） |
| OR パターン内 catch-all 伝播 | **OUT** | 型情報が必要な複雑な解析（既存 Known limitation を維持） |

---

## コードベース現状（調査済み）

### `ast.rs`（変更なし）

```rust
/// "a" | "b" | "c" — or-pattern (v17.2.0)
Or(Vec<Pattern>, Span),
```

### `checker.rs` 現行（Line ~4210）

```rust
Pattern::Or(pats, _) => {
    if let Some(first) = pats.first() {
        self.check_pattern_bindings(first, ty);  // 最初のアームのみ
    }
}
```

→ **全アームに変更**:
```rust
Pattern::Or(pats, _) => {
    for pat in pats {
        self.check_pattern_bindings(pat, ty);  // 全アームを型チェック
    }
}
```

### `lint.rs` 現行（Line ~3074）

```rust
/// リテラルパターンの一意キーを返す（重複検出用）。
/// リテラル以外（OR パターン含む）は None を返す。
fn pattern_lit_key(pat: &Pattern) -> Option<String> {
    if let Pattern::Lit(lit, _) = pat {
        Some(format!("{:?}", lit))
    } else {
        None  // OR パターンは None → 重複検出不可
    }
}
```

→ **新関数 `pattern_lit_keys_all` を追加**（OR 内の全リテラルを再帰抽出）:
```rust
fn pattern_lit_keys_all(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Lit(lit, _) => vec![format!("{:?}", lit)],
        Pattern::Or(pats, _) => pats.iter().flat_map(pattern_lit_keys_all).collect(),
        _ => vec![],
    }
}
```

`check_expr_for_unreachable` 内の重複チェックを `pattern_lit_key` から `pattern_lit_keys_all` に差し替え。

---

## テスト仕様

### `pattern_or_type_check_arms_same`

```
1. "active" | "pending" | "inactive" => ... / "deleted" | "archived" => ... / _ => ...
   の match を型チェック（3アームの OR パターンを含む）
   → errors.is_empty() を確認（全アームが String 型で一致、全アーム処理で E0009 なし）

   ※ 3アーム OR パターンを含むことで「3段階 OR パターン E2E 確認」を兼ねる（ロードマップ要件）
```

### `pattern_or_lint_w037_integration`

```
1. match x { "a" | "b" => ... / "a" => ... / _ => ... } を lint 実行
   → W037 が発火することを確認
   （OR パターン内の "a" が後続アームの単独リテラル "a" と重複）

   ※ 部分重複（後続アームが "a" | "c" のような OR パターン）は対象外
```

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3352 + 2 = 3354」は v60.8.0 XSS テスト追加前の想定値。
実際の v61.0.0 テスト数: **3353**（ロードマップ記載 3352 + XSS テスト +1）

実際のテスト数目標: **3353 + 2 = 3355** tests passed, 0 failed

---

## 完了条件

- `pattern_or_type_check_arms_same` pass
- `pattern_or_lint_w037_integration` pass
- 総テスト数: **3355** tests passed, 0 failed
- `cargo build` でコンパイルエラーなし
