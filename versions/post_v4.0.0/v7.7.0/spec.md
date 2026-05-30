# Favnir v7.7.0 仕様書

Date: 2026-05-28
Theme: checker.fav 基本機能パリティ（エフェクト追跡 / builtin 全登録 / エラーコード / match 網羅性）

---

## 目的

現在の `fav/self/checker.fav`（513 行）は Bootstrap 検証用の簡略版。
本バージョンで Rust 版 `checker.rs` の基本機能と同等の検査能力を持たせる。

---

## 現状ギャップ

| 機能 | checker.rs | checker.fav（現状） |
|------|-----------|---------------------|
| エフェクト追跡 | !IO / !Cache / !Queue / !Email | なし |
| builtin 名前空間 | 全 40+ 関数 | 限定的（IO.argv のみ等） |
| エラーコード | E0xxx 形式 | 生文字列 |
| match 網羅性 | Option / Result / Bool | なし |
| 型ミスマッチ検出 | エラー蓄積 | Unknown を返す（サイレント） |

---

## Phase A: エフェクト追跡

### A-1: FnDef に effects フィールド追加

```favnir
type FnDef = {
    is_public: Bool
    name: String
    params: List<Param>
    ret: TypeExpr
    effects: List<String>   // 新規追加: ["IO", "Cache"] 等
    body: Expr
}
```

### A-2: `infer_expr_effects(expr: Expr) -> List<String>`

式を再帰走査してエフェクトを収集する純粋関数。

| ECall(ns, ...) | 生成エフェクト |
|----------------|---------------|
| `"IO"` | `"IO"` |
| `"Compiler"` | `"IO"` |
| `"Cache"` | `"Cache"` |
| `"Queue"` | `"Queue"` |
| `"Email"` | `"Email"` |
| その他 | なし |

複合式（EBind / EIf / EBlock / EMatch 等）は部分式を再帰してマージ。

### A-3: `has_effect(effects: List<String>, eff: String) -> Bool`

`List.any(effects, |e| eq_str(e, eff))` — 宣言済みかチェック。

### A-4: `check_fn_effects(declared: List<String>, inferred: List<String>) -> Option<String>`

推論エフェクト中で宣言に含まれないものがあれば E0003 を返す。

### A-5: `check_fn_def` に効果チェックを統合

```
infer_expr_effects(body) → inferred
check_fn_effects(fd.effects, inferred) → エラーがあれば追記
```

---

## Phase B: builtin 名前空間全登録

`builtin_ret_ty(ns, fname) -> String` を拡張。

### B-1: `io_fn(fname)` 拡張

| 関数 | 戻り型 |
|------|--------|
| `argv` | `"List"` |
| `read_file_raw` | `"Result"` |
| `write_stdout_raw` | `"Unit"` |
| `write_stderr_raw` | `"Unit"` |
| `exit_raw` | `"Unit"` |
| `list_dir_raw` | `"Result"` |
| `file_stat_raw` | `"Map"` |
| `path_join_raw` | `"String"` |
| `home_dir_raw` | `"Option"` |
| `cwd_raw` | `"String"` |
| `is_dir_raw` | `"Bool"` |
| `println` | `"Unit"` |

### B-2: `compiler_fn(fname)`

| 関数 | 戻り型 |
|------|--------|
| `check_raw` | `"Result"` |
| `lineage_text_raw` | `"String"` |

### B-3: `cache_fn(fname)`

| 関数 | 戻り型 |
|------|--------|
| `get_raw` | `"Option"` |
| `set_raw` | `"Unit"` |
| `del_raw` | `"Unit"` |
| `exists_raw` | `"Bool"` |
| `del_prefix_raw` | `"Unit"` |

### B-4: `queue_fn(fname)`

| 関数 | 戻り型 |
|------|--------|
| `send_raw` | `"Result"` |
| `recv_raw` | `"Result"` |
| `ack_raw` | `"Result"` |
| `delete_raw` | `"Result"` |

### B-5: `email_fn(fname)`

| 関数 | 戻り型 |
|------|--------|
| `send_raw` | `"Result"` |

### B-6: `str_fn` 拡張（追加分）

`starts_with`, `ends_with`, `trim`, `upper`, `lower`, `to_upper`, `to_lower`,
`join`, `repeat`, `replace`, `split`, `compare`, `to_bytes`

### B-7: `list_fn` 拡張（追加分）

`fold`, `fold_left`, `find`, `partition`, `empty`, `sort_by`,
`zip_with`, `group_by`, `intersperse`, `zip`, `chunk`

### B-8: `map_fn` 拡張（追加分）

`empty`, `contains_key`, `delete`, `to_list`, `from_entries`, `merge_with`, `count_by`

### B-9: `int_fn` 拡張 / `float_fn` 追加

`int_fn`: `to_string`, `compare`, `band`, `shr`, `bor`
`float_fn`: `to_string`, `from_int`

---

## Phase C: エラーコード

### C-1: `fmt_err(code: String, msg: String) -> String`

```favnir
fn fmt_err(code: String, msg: String) -> String {
    String.concat(code, String.concat(": ", msg))
}
```

### C-2: エラーコード一覧

| コード | 意味 |
|--------|------|
| `E0001` | arithmetic type mismatch |
| `E0002` | logical operator requires Bool |
| `E0003` | undeclared effect |
| `E0004` | non-exhaustive match |
| `E0005` | condition must be Bool |

全ての `Result.err(...)` を `Result.err(fmt_err("E0xxx", ...))` に置き換え。

---

## Phase D: match 網羅性チェック（基本）

### D-1: `collect_arm_ctors(arms: Expr) -> List<String>`

`EArm(pat, _, rest)` を再帰走査してトップレベルコンストラクタ名を収集。

```
PWild      → "_"
PVariant(name) → name
PVariantP(name, _) → name
PVar(_)    → "_"
その他     → ""
```

### D-2: `has_wildcard(ctors: List<String>) -> Bool`

`List.any(ctors, |c| eq_str(c, "_"))` で `_` / `PVar` の有無を確認。

### D-3: `check_option_exhaustive(ctors: List<String>) -> Bool`

`has_wildcard(ctors) || (contains "None" && contains "Some")` → true なら網羅済み。

### D-4: `check_result_exhaustive(ctors: List<String>) -> Bool`

`has_wildcard(ctors) || (contains "Ok" && contains "Err")` → true なら網羅済み。

### D-5: `check_match_exhaustive(scrut_ty: String, arms: Expr) -> Option<String>`

```
scrut_ty == "Option" → check_option_exhaustive
scrut_ty == "Result" → check_result_exhaustive
その他               → 常に Ok（v7.7.0 スコープ外）
```

網羅性エラー: `fmt_err("E0004", "non-exhaustive match on " + scrut_ty)`

### D-6: `infer_arms` に `scrut_ty: String` パラメータを追加

`infer_arms(arms, env, scrut_ty)` → 末尾で D-5 を呼び出す。

---

## Phase E: テスト（driver.rs / checker.fav 内）

### checker.fav 内テスト（8 件追加）

| テスト | 確認内容 |
|--------|----------|
| `effect_io_detected` | IO.write_stdout_raw → `"IO"` エフェクト検出 |
| `effect_cache_detected` | Cache.get_raw → `"Cache"` エフェクト検出 |
| `effect_undeclared_error` | 宣言なしで IO 使用 → E0003 |
| `builtin_io_fn` | `builtin_ret_ty("IO", "list_dir_raw") == "Result"` |
| `builtin_cache_fn` | `builtin_ret_ty("Cache", "get_raw") == "Option"` |
| `error_code_format` | `fmt_err("E0001", "x") == "E0001: x"` |
| `match_option_exhaustive_ok` | None + Some → 網羅済み |
| `match_option_missing_some` | None のみ → E0004 |

### driver.rs 統合テスト（3 件追加）

`checker_v77_tests` モジュール:

| テスト | 確認内容 |
|--------|----------|
| `checker_fav_effect_tracking_test` | checker.fav をロードし、IO エフェクト違反を検出 |
| `checker_fav_builtin_coverage_test` | Cache.get_raw の戻り型が "Option" と推論される |
| `checker_fav_exhaustiveness_test` | Option match が非網羅のとき E0004 を返す |

---

## Phase F: ドキュメント

`site/content/docs/language/self-host-checker.mdx` 作成:

- checker.fav アーキテクチャ（型推論パス / エフェクト追跡パス / 網羅性パス）
- エラーコード一覧（E0001〜E0005）
- v7.7.0〜v7.9.0 ロードマップ（ジェネリクス → HM 推論）

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- checker.fav が IO/Cache/Queue/Email エフェクト違反を E0003 で検出できる
- `builtin_ret_ty` が v7.6.0 までに追加した全 VM primitive をカバーする
- Option/Result の非網羅 match を E0004 で検出できる
- 統合テスト 11 件追加（checker.fav 内 8 + driver.rs 3）
- 既存テスト全通過（1102+ passing）

---

## 実装ノート

- `FnDef.effects` を追加すると既存の `IFn(FnDef)` のすべての構築箇所でフィールド追加が必要
  → テスト内の `FnDef { ... }` リテラルも全更新
- `infer_arms` のシグネチャを `(arms, env, scrut_ty)` に変更すると呼び出し側も変更が必要
  → `infer_expr` の `EMatch` ハンドラを更新
- `collect_arm_ctors` は bind-in-closure を避けるため再帰関数で実装
- `has_wildcard` / `has_ctor` はヘルパーを外部関数化（closure 内 bind 禁止のため）
