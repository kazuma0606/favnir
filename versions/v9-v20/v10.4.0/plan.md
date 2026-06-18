# Favnir v10.4.0 Plan

Date: 2026-06-04
Theme: checker.fav 更新 — Snowflake 型チェック対応

---

## Phase A: checker.fav 更新（3 箇所）

### A-1: `snowflake_fn` 関数を `llm_fn` の直後に追加

`fav/self/checker.fav` の `llm_fn` 終端（〜line 603）の直後:

```favnir
fn snowflake_fn(fname: String) -> String {
    if fname == "execute_raw" {
        "Result"
    } else {
        if fname == "query_raw" {
            "Result"
        } else {
            "Result"
        }
    }
}
```

### A-2: `builtin_ret_ty` に Snowflake 分岐を追加

`fav/self/checker.fav` の `builtin_ret_ty` 内、`"Llm"` ブランチの直後（〜line 1019）:

```favnir
                                if ns == "Llm" {
                                    llm_fn(fname)
                                } else {
                                    if ns == "Snowflake" {      // ← 追加
                                        snowflake_fn(fname)     // ← 追加
                                    } else {                    // ← 追加
                                        if ns == "Debug" {
                                            debug_fn(fname)
                                        } else {
                                            "Unknown"
                                        }
                                    }                           // ← 追加
                                }
```

### A-3: `ns_to_effect` に Snowflake エントリを追加

`fav/self/checker.fav` の `ns_to_effect` 内、`"Llm"` ブランチの直後（〜line 1073）:

```favnir
                                    if ns == "Llm" {
                                        "Llm"
                                    } else {
                                        if ns == "Snowflake" {  // ← 追加
                                            "Snowflake"         // ← 追加
                                        } else {                // ← 追加
                                            if ns == "Debug" {
                                                "IO"
                                            } else {
                                                ""
                                            }
                                        }                       // ← 追加
                                    }
```

---

## Phase B: テスト追加（`driver.rs` 末尾に `v10400_tests` を追加）

`check_source_str`（checker.fav 経由）を使う。
効果チェックは E0003（checker.fav の汎用エフェクトエラー）で確認。

```rust
// ── v10400_tests (v10.4.0) — checker.fav Snowflake support ──────────────────
#[cfg(test)]
mod v10400_tests {
    #[test]
    fn snowflake_effect_checker_fav_missing() {
        // checker.fav 経由: !Snowflake 未宣言で E0003
        let src = r#"
fn run(sql: String) -> Result<String, String> {
  Snowflake.execute_raw(sql)
}
"#;
        let errors = super::check_source_str(src);
        let has_e0003 = errors.iter().any(|e| e.code == "E0003");
        assert!(has_e0003, "expected E0003 for missing !Snowflake via checker.fav: {:?}", errors);
    }

    #[test]
    fn snowflake_effect_checker_fav_ok() {
        // checker.fav 経由: !Snowflake 宣言済みでエラーなし
        let src = r#"
fn run(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.execute_raw(sql)
}
"#;
        let errors = super::check_source_str(src);
        let e0003: Vec<_> = errors.iter().filter(|e| e.code == "E0003").collect();
        assert!(e0003.is_empty(), "unexpected E0003 with !Snowflake via checker.fav: {:?}", e0003);
    }
}
```

---

## Phase C: self-check + `fav fmt --check` + テスト

### C-1: `fav fmt --check self/checker.fav`

```bash
fav fmt --check self/checker.fav
```

フォーマット差分がないことを確認。差分があれば `fav fmt self/checker.fav` で整形。

### C-2: `fav check self/checker.fav`

```bash
fav check self/checker.fav
```

エラーなしを確認。

### C-3: `cargo test checker_fav_wire_self_check`

```bash
cargo test checker_fav_wire_self_check
```

checker.fav 自身のセルフチェックが通ること。

### C-4: `cargo test v10400`

新規テスト 2 件が通ること。

### C-5: `cargo test`

全件通過（目標: 1269 件 = 1267 + 2 新規）。

---

## Phase D: バージョン更新

### D-1: `fav/Cargo.toml` version → `"10.4.0"`

### D-2: `fav/self/cli.fav` の `run_version` → `"10.4.0"`
