# v43.6.0 仕様書 — パイプライン型伝播（Pipeline stage typing）

## 概要

ロードマップ: "stage 内の中間型を明示せずとも checker.fav が伝播・確定"

```favnir
// 複数の bind ステップを持つパイプラインで型が連鎖して伝播する
fn process(xs: List<Int>) -> List<Int> {
    bind doubled <- List.map(xs, |x| x * 2)   // doubled: List<Int> — 推論
    List.filter(doubled, |x| x > 0)            // x: Int — 推論
}
```

---

## 現状と問題

v43.5.0 でラムダ引数型推論（`infer_list_lambda_call`）が実装され、`List.map(xs, |x| ...)` の `x` の型が `xs: List<Int>` から伝播するようになった。

しかし多段パイプライン（bind → bind → ...）の **end-to-end** での動作検証が行われていない。

特に、以下の流れが正しく機能するか検証が必要:

1. `bind doubled <- List.map(xs, |x| x * 2)` → `doubled: List<Int>` が env に追加される
2. `List.filter(doubled, |x| x > 0)` → `doubled: List<Int>` から `x: Int` が伝播する

---

## 解決

v43.6.0 は **実装追加なし** のバリデーションリリース。

既存の仕組みの組み合わせで多段パイプラインが機能する:
- **`infer_hm_let`**（`checker.fav` line 2037）: `EBind` を処理し、`vr.ty` を `env_insert` で次式の環境に追加
- **`infer_list_lambda_call`**（v43.5.0）: `List.map`/`List.filter` 呼び出しでリスト要素型をラムダパラメータに伝播

```
check_fn_def
  └─ infer_hm(EBind("doubled", List.map_call, List.filter_call), env)
       └─ infer_hm_let("doubled", List.map_call, List.filter_call, env, state)
            ├─ infer_hm(List.map_call) → infer_call_hm → infer_call → infer_list_lambda_call → "List<Int>"
            │   (v43.5.0: x: Int propagated from xs: List<Int>)
            └─ env_insert(env, "doubled", "List<Int>")
                └─ infer_hm(List.filter_call, env+doubled, ...) → "List<Int>"
                    (v43.5.0: x: Int propagated from doubled: List<Int>)
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v43600_tests` 追加（3 件） |
| `fav/Cargo.toml` | version 43.5.0 → 43.6.0 |
| `CHANGELOG.md` | v43.6.0 エントリ追加 |
| `versions/current.md` | v43.6.0 最新安定版に更新 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.6.0 を COMPLETE に更新 |

**`fav/self/checker.fav` は変更不要**: `infer_hm_let` + v43.5.0 `infer_list_lambda_call` の組み合わせで多段パイプラインが既に機能する。

---

## テスト設計

### v43.5.0 との差分

v43.5.0 のテストは **単一の `List.map` / `List.filter` 呼び出し** でラムダパラメータへの型伝播を検証した。
v43.6.0 のテストは **`bind` チェーンを跨ぐ `env_insert` 経路** を追加検証する。具体的には:

- `bind doubled <- List.map(xs, |x| x * 2)` → `infer_hm_let` が `doubled: List<Int>` を env に追加
- 続く `List.filter(doubled, |x| x > 0)` → `doubled: List<Int>` から `x: Int` が伝播

この「前の `bind` で確定した型が次式の `infer_list_lambda_call` に届く」パスが v43.6.0 の検証対象。

### T1 — `v43600_tests`

#### `cargo_toml_version_is_43_6_0`
バージョン確認テスト（次バージョン bump 時にスタブ化）。

#### `pipeline_two_step_bind_infers_types`

```rust
let src = r#"
fn process(xs: List<Int>) -> List<Int> {
    bind doubled <- List.map(xs, |x| x * 2)
    List.filter(doubled, |x| x > 0)
}
"#;
```

- `doubled: List<Int>` が環境に追加され、`List.filter` の `x: Int` が伝播する
- `run_checker_fav` → `Ok` を期待

#### `pipeline_three_step_bind_infers_types`

```rust
let src = r#"
fn three_step(xs: List<Int>) -> List<Int> {
    bind step1 <- List.map(xs, |x| x + 1)
    bind step2 <- List.filter(step1, |x| x > 0)
    List.map(step2, |x| x * 2)
}
"#;
```

- 3 段階の `bind` チェーンで型が連鎖して伝播する
- `run_checker_fav` → `Ok` を期待

---

## 完了条件

- `cargo test` 2920 tests passed, 0 failed（2917 + 3）
- `v43600_tests` 3 件 pass
- `pipeline_two_step_bind_infers_types`: `run_checker_fav` が `Ok(())` を返す（`Err` 時はその内容を `{:?}` で表示）
- `pipeline_three_step_bind_infers_types`: `run_checker_fav` が `Ok(())` を返す（`Err` 時はその内容を `{:?}` で表示）

---

## 影響範囲

- **checker.fav 変更なし**: 既存コードで機能する
- **既知制限（スコープ外）**:
  - `ECollect`（`[1, 2, 3]` リストリテラル）は `infer_expr` が `"Unknown"` を返すため、リテラルからの要素型伝播は非対応（v43.5.0 既知制限）
  - `EAccess`（フィールドアクセス `r.value`）は常に `"Unknown"` を返す（v41.5.0 コメント: "型精度は Unknown で近似"）→ フィールド型追跡は v42.0+ 以降の課題
  - ロードマップ例の `Csv.read("data.csv")` → `builtin_ret_ty("Csv", "read")` は `"Unknown"` を返すため、stage 先頭の Csv 呼び出しからの型伝播は非対応（既知制限）
- **型付きパラメータ経由（`xs: List<Int>`）では全段で正常動作** ✓

---

## 前提条件

- v43.5.0 COMPLETE（2917 tests）
- `infer_hm_let`: `Result.and_then(infer_hm(val_e, env, state), |vr| infer_hm(cont_e, env_insert(env, vname, vr.ty), inf_state_of(vr)))` ← `vr.ty` が続く式の env に追加される
- `infer_call_hm` for `ns != ""`: `Result.and_then(infer_call(ns, fname, args, env), |ty| Result.ok(inf_result_of(ty, state)))` ← `infer_call` → `infer_list_lambda_call`（v43.5.0）のパスを使用
