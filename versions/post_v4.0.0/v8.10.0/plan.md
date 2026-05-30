# Favnir v8.10.0 実装計画

Date: 2026-05-30

---

## Phase A: ヘルパー関数追加

**変更ファイル**: `fav/self/checker.fav`

`is_type_var_extended` 関数の直後（`list_dedup_inner` の前）に追加:

```fav
fn outer_type(s: String) -> String {
    match List.first(String.split(s, "<")) {
        None        => s
        Some(outer) => outer
    }
}

fn types_compatible(inferred: String, declared: String) -> Bool {
    if inferred == declared { true }
    else if inferred == "Unknown" { true }
    else if is_type_var_extended(inferred) { true }
    else { outer_type(inferred) == outer_type(declared) }
}
```

---

## Phase B: `check_fn_def` 更新

**変更ファイル**: `fav/self/checker.fav`

`Result.and_then(infer_hm(fd.body, param_env, init_state), |r| Result.ok(fd.name))` を
戻り型照合付きバージョンに変更:

```fav
// Before:
Result.and_then(infer_hm(fd.body, param_env, init_state), |r|
Result.ok(fd.name))

// After:
Result.and_then(infer_hm(fd.body, param_env, init_state), |r|
    bind inferred <- apply_subst(r.subst, r.ty)
    bind declared <- type_expr_to_str(fd.ret)
    if types_compatible(inferred, declared) {
        Result.ok(fd.name)
    } else {
        Result.err(fmt_err("E0009",
            String.concat(fd.name,
            String.concat(": declared return ",
            String.concat(declared,
            String.concat(" but body infers ", inferred))))))
    })
```

---

## Phase C: テスト追加

**変更ファイル**: `fav/src/driver.rs`

```rust
// ── checker_v810_tests (v8.10.0) ─────────────────────────────────────────────
#[cfg(test)]
mod checker_v810_tests {
    // check_errors ヘルパーは checker_v87_tests と同じパターン

    /// 戻り型ミスマッチ → E0009
    #[test]
    fn return_type_mismatch_e0009() {
        let errors = check_errors(
            "fn bad() -> Int { \"hello\" }\npublic fn main() -> Int { bad() }",
        );
        assert!(
            errors.iter().any(|e| e.contains("E0009")),
            "expected E0009 for return type mismatch, got: {:?}", errors
        );
    }

    /// リテラルの正しい戻り型 → エラーなし
    #[test]
    fn return_type_correct_literal() {
        let errors = check_errors(
            "public fn main() -> Int { 42 }",
        );
        assert!(errors.is_empty(), "Int literal fn should pass: {:?}", errors);
    }

    /// 関数呼び出しの正しい戻り型 → エラーなし
    #[test]
    fn return_type_correct_call() {
        let errors = check_errors(r#"
fn double(x: Int) -> Int { x + x }
public fn main() -> Int { double(21) }
"#);
        assert!(errors.is_empty(), "fn call with matching return type should pass: {:?}", errors);
    }
}
```

---

## Phase D: テスト実行・確認

```
cargo test checker_v810
cargo test checker_fav         ← 20 + 3 = 23 件通ること
cargo test                     ← 全件通ること（目標 1134 tests）
```

---

## Phase E: 最終確認

- `cargo build` — コンパイルエラーなし
- `checker_fav_wire_self_check` — 64MB スタックで通ること
- tasks.md 完了・commit

---

## 実装ノート

### `apply_subst(r.subst, r.ty)` の戻り値は `String`（非 Result）

`bind` を使って束縛するが、`apply_subst` は Result を返さない（String を返す）。
Favnir の `bind x <- expr` は let-binding であり、Result unwrap ではないため問題なし。

### `types_compatible` の寛容さについて

現状の推論精度（`infer_expr` が "Unknown" を返すケースが多い）を考慮し、
完全な型一致ではなく outer base type の一致を確認する寛容なチェックを採用。
これにより:
- 誤検知（False Positive）を最小化
- 真に自明な型ミスマッチ（Int vs String 等）を確実に検出

将来バージョン（v8.x 以降）でより精密な unification ベースの check に強化可能。

### `checker_fav_wire_self_check` の通過根拠

checker.fav 内の全関数で:
1. `Result<...>` を返す関数 → 宣言 "Result" == 推論 "Result" ✓
2. `List<KVPair>` を返す関数 → exact match か outer_type 互換 ✓
3. `String`/`Int`/`Bool` を返す関数 → exact match ✓
4. 推論が "Unknown" になる関数 → スキップ ✓
