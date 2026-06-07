# Favnir v12.1.0 実装計画

作成日: 2026-06-07

---

## 実装順序

```
Phase A: checker.fav の現状把握（EBind 処理箇所の特定）
    ↓
Phase B: checker.fav に bound_set ヘルパー関数追加
    ↓
Phase C: checker.fav の EBind 処理に E0018 チェック追加
    ↓
Phase D: checker.fav の chain（EBind として扱われる場合）も対応確認
    ↓
Phase E: Rust checker.rs に E0018 追加（--legacy モード用）
    ↓
Phase F: driver.rs に v12100_tests モジュール追加
    ↓
Phase G: cargo test v12100 / cargo test --lib 通過確認
    ↓
Phase H: Cargo.toml バージョン更新 + コミット
```

---

## Phase A — checker.fav 現状把握

調査対象:
- `EBind` を処理している関数（`infer_hm_let` / `infer_hm` / `infer_expr`）
- `infer_hm_let` の引数・戻り値の型
- 既存の「環境（env）」がどのように `EBind` で拡張されているか

確認ポイント:
- `env` は `List<{ name: String, ty: String }>` 形式か？
- `infer_hm_let` は現状 `(vname, val_e, cont_e, env, state)` を受け取っているか？
- `EBind` のネスト構造（`cont_e` に次の `EBind` が入る形式）

---

## Phase B — bound_set ヘルパー関数追加

`checker.fav` に以下を追加:

```favnir
fn bound_set_contains(set: List<String>, name: String) -> Bool =
  match List.first(List.filter(set, |s| s == name)) {
    None    => false
    Some(_) => true
  }

fn bound_set_add(set: List<String>, name: String) -> List<String> =
  List.push(set, name)
```

---

## Phase C — EBind 処理に E0018 チェック追加

`infer_hm_let` の変更:

1. 引数に `bound: List<String>` を追加
2. `vname != "_"` かつ `bound_set_contains(bound, vname)` の場合 E0018 を発行
3. 再帰呼び出しに `bound_set_add(bound, vname)` を渡す

`infer_hm` の `EBind` 分岐も `bound` を初期値 `List.empty()` で受け取るよう変更。

エラーメッセージ:

```favnir
fn fmt_e0018(name: String) -> String =
  String.concat(
    "E0018: variable '",
    String.concat(name,
      String.concat(
        "' is already bound in this scope\n",
        String.concat(
          "  = help: use a different name: `bind ",
          String.concat(name,
            String.concat("2 <- ...`\n",
              "  = help: or discard the value: `bind _ <- ...`"
            )
          )
        )
      )
    )
  )
```

---

## Phase D — chain の対応確認

Favnir の `chain` は AST 上で `EBind` と同じノードか `EChanin` 別ノードかを確認。
- `EBind` なら Phase C で自動的にカバーされる
- 別ノードなら同様のチェックを追加

---

## Phase E — Rust checker.rs に E0018 追加（--legacy 用）

`src/middle/checker.rs` の `infer_body` / `check_stmts` において、
`IRStmt::Bind(name, _)` の処理に「既出変数セット」を管理するロジックを追加。

```rust
// 既存の seen_names: HashSet<String> を追加
if name != "_" && seen_names.contains(name) {
    self.type_error("E0018",
        format!("E0018: variable '{}' is already bound in this scope\n\
                 = help: use a different name: `bind {}2 <- ...`\n\
                 = help: or discard the value: `bind _ <- ...`", name, name));
} else {
    seen_names.insert(name.clone());
}
```

---

## Phase F — driver.rs テスト追加

`v12100_tests` モジュールに 5 テストを追加:

```rust
#[test]
fn e0018_rebind_detected() {
    let src = r#"
stage Bad: Int -> Int = |n| {
  bind x <- n
  bind x <- n
  x
}
"#;
    let result = check_with_checker_fav(src, "e0018");
    assert!(result.is_err());
    assert!(result.unwrap_err().iter().any(|m| m.contains("E0018")));
}

#[test]
fn e0018_underscore_allowed() { ... }

#[test]
fn e0018_chain_rebind_detected() { ... }

#[test]
fn e0018_match_arm_independent() { ... }

#[test]
fn version_is_12_1_0() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "12.1.0");
}
```

---

## 注意事項

- `checker.fav` 変更後は `fav/self/checker.fav` を更新し、`cargo build` で再コンパイルされることを確認
- `bound` は fn/stage 本体ごとにリセット（match arm では arm スコープで新しい `bound` を使う）
- `infer_hm` が再帰的に `EBind` を処理する場合、外部からのトップレベル呼び出しは `List.empty()` で開始
- テスト実行: `cargo test v12100 -- --nocapture`
