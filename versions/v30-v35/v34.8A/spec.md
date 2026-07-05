# v34.8A spec — `!Effect` 構文のパースエラー化（E0374）

**バージョン**: v35.4.0
**日付**: 2026-07-05
**前提**: v34.7A (v35.3.0) COMPLETE

---

## 目的

`!Effect` アノテーションを `.fav` 構文レベルで完全に廃止する。
パーサーが `!Xxx` を fn/stage シグネチャ内で検出した時点で E0374 のハードエラーとして返す。
これにより「書けば動く非推奨構文」から「書けばコンパイルエラーになる削除済み構文」に格上げする。

---

## 背景

| バージョン | 実施内容 |
|---|---|
| v34.5A | W022 警告追加（`!Effect` は非推奨） |
| v34.6A | `runes/` 100 件を ctx 構文に移行 |
| v34.7A | `examples/` 31 件 + `infra/` 10 件を ctx 構文に移行 |
| **v34.8A** | `!Effect` 構文自体を parse error E0374 に格上げ |
| v34.9A | `Effect` enum / `effects` フィールドの完全削除（後続） |

---

## 変更内容

### 1. error_catalog.rs — E0374 追加

```rust
ErrorEntry {
    code: "E0374",
    title: "removed syntax: !Effect annotation",
    description: "The `!Effect` annotation syntax has been removed. \
                  Use `ctx: AppCtx` as the first parameter instead.",
    example: "// Error: fn f(x: Int) -> Int !Io { ... }\n\
              // Fix:   fn f(ctx: AppCtx, x: Int) -> Int { ... }",
}
```

### 2. parser.rs — `parse_effects_acc` をエラー化

現行の `parse_effects_acc` は `!Xxx` をパースして `Vec<Effect>` を返す。
v34.8A では `!` が fn/stage の戻り型の後ろに現れた場合、
`ParseError(E0374)` を直ちに返すよう変更する。

変更対象関数（grep 確認済み）:
- `parse_fn_def_after_ret`（行 ~1870）— `parse_effects_acc` 呼び出し箇所
- `parse_stage_def`（行 ~2100 付近）— 同上

変更方針:
```rust
// 変更前
let effects = self.parse_effects_acc();

// 変更後
if self.peek_is_bang_effect() {
    return Err(ParseError::new(
        "E0374",
        "!Effect annotation syntax removed — use `ctx: AppCtx` parameter instead",
        self.current_span(),
    ));
}
let effects = vec![]; // always empty after v34.8A
```

### 3. lint.rs — W022 を削除

パーサーが先に E0374 を返すため W022 は到達不能になる。
`check_w022_deprecated_effect_annotation` 関数を削除し、
`run_lint` からの呼び出しも除去する。

`ast.rs` の `Effect::is_deprecated()` は v34.9A（`Effect` 削除）まで残す。

### 4. driver.rs

- `cargo_toml_version_is_35_3_0` をスタブ化
- `v35400_tests` 5 件を追加（`v35300_tests` 直後に挿入）

---

## 完了条件

- `fn f(x: Int) -> Int !Http { x }` を `fav check` すると E0374 が返る
- `fn f(ctx: AppCtx, x: Int) -> Int { x }` は問題なくコンパイルできる
- W022 が lint 結果に現れない（関数ごと削除済み）
- `cargo test` 全件 PASS
- `cargo clippy --locked -- -D warnings` PASS
