# Favnir v10.4.0 Spec

Date: 2026-06-04
Theme: checker.fav 更新 — Snowflake 型チェック対応

---

## 概要

セルフホスト型チェッカー（`fav/self/checker.fav`）に Snowflake を認識させる。
`!Http`（v9.5.0）・`!Llm`（v9.6.0）と同じ 3 箇所の追加で対応できる。

v10.3.0 で Rust checker（`checker.rs`）は対応済み。
本バージョンでは `fav check` のデフォルトパス（checker.fav 経由）が
`!Snowflake` を正しく扱えるようにする。

---

## 前提（v10.3.0 完了時点）

- Rust checker（`checker.rs`）に `Effect::Snowflake`、`require_snowflake_effect`、E0314 追加済み
- `cargo test` 1267 件通過
- `checker.fav` はまだ `"Snowflake"` namespace を知らないため、
  `Snowflake.execute_raw` 呼び出しを含む Favnir ソースを `fav check` すると
  E0003「undeclared effect !Snowflake」ではなく未知の ns として扱われる

---

## checker.fav の effect チェック仕組み

```favnir
fn ns_to_effect(ns: String) -> String
```
→ namespace 文字列 → 必要エフェクト文字列を返す（例: `"Llm"` → `"Llm"`）

```favnir
fn infer_expr_effects(expr: Expr) -> List<String>
```
→ 式中の `ECall({ns, fname, args})` を走査し `ns_to_effect` でエフェクトを収集

```favnir
fn check_effects_all(declared, inferred) -> Option<String>
```
→ inferred に含まれるエフェクトが declared にない場合は E0003 を返す

エフェクトコードは E0003 で固定（checker.fav は E0314 を知らない）。
`fav check` 経由のエラーレポートは Rust checker の E0314 に対して
checker.fav は E0003 を出す差異があるが、「エラーが出る」という動作は一致する。

---

## 変更対象

```
fav/self/checker.fav   (3 箇所追加)
```

---

## 変更仕様

### 1. `snowflake_fn` 関数を追加（`llm_fn` の直後）

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

### 2. `builtin_ret_ty` に Snowflake 分岐を追加

`"Llm"` ブランチの直後（`"Debug"` ブランチの前）:

```favnir
// 変更前
                                if ns == "Llm" {
                                    llm_fn(fname)
                                } else {
                                    if ns == "Debug" {
                                        debug_fn(fname)
                                    } else {
                                        "Unknown"
                                    }
                                }

// 変更後
                                if ns == "Llm" {
                                    llm_fn(fname)
                                } else {
                                    if ns == "Snowflake" {
                                        snowflake_fn(fname)
                                    } else {
                                        if ns == "Debug" {
                                            debug_fn(fname)
                                        } else {
                                            "Unknown"
                                        }
                                    }
                                }
```

### 3. `ns_to_effect` に Snowflake エントリを追加

`"Llm"` ブランチの直後（`"Debug"` ブランチの前）:

```favnir
// 変更前
                                    if ns == "Llm" {
                                        "Llm"
                                    } else {
                                        if ns == "Debug" {
                                            "IO"
                                        } else {
                                            ""
                                        }
                                    }

// 変更後
                                    if ns == "Llm" {
                                        "Llm"
                                    } else {
                                        if ns == "Snowflake" {
                                            "Snowflake"
                                        } else {
                                            if ns == "Debug" {
                                                "IO"
                                            } else {
                                                ""
                                            }
                                        }
                                    }
```

---

## テスト設計（`driver.rs` の `v10400_tests` モジュール）

### テスト 1: `snowflake_effect_checker_fav_missing`

`check_source_str`（checker.fav 経由）で `!Snowflake` 未宣言の fn を渡す
→ E0003「undeclared effect !Snowflake」が出ること。

### テスト 2: `snowflake_effect_checker_fav_ok`

`check_source_str`（checker.fav 経由）で `!Snowflake` 宣言済みの fn を渡す
→ エラーが出ないこと。

### テスト 3: `checker_fav_wire_self_check`

既存テスト（`checker_fav_wire_self_check`）が引き続き通ること。
（このテストは `checker.fav` 自身を checker.fav で型チェックする。
`snowflake_fn` を追加した後もセルフチェックが通ることを確認する。）

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.4.0"`
- `fav/self/cli.fav`: `run_version` → `"10.4.0"`
