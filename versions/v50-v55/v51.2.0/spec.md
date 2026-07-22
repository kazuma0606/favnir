# Spec: v51.2.0 — `par` Phase 2（Merge.ordered / Merge.any）

## 概要

v51.1.0 で `par [A, B]` が `IRExpr::Par` → `Opcode::ParStages` 経由で並列実行できるようになった。
本バージョンでは **`par [A, B] |> Merge.ordered`** と **`par [A, B] |> Merge.any`** の構文を実装する。

- **`Merge.ordered`**: par の結果を宣言順に Unwrap して `List<T>` を返す
- **`Merge.any`**: par の結果を完了順に Unwrap して `List<T>` を返す（`std::thread` では実質 Ordered と同一だが、将来の tokio タスク化を見越した意図的分離）

ロードマップ上の「`ast.rs` に `MergeMode` enum を追加」に対応する。

**スコープ外（v51.3.0 以降）**: ロードマップ行 54「`Stream<T>` を返す stage にも `par` が適用できることを確認」は本バージョンのスコープ外。v51.3.0 バックプレッシャー実装時に合わせて検討する。

---

## 背景・現状確認

| 項目 | 現状 |
|---|---|
| `par [A, B]` → `Opcode::ParStages` | ✅ v51.1.0 実装済み |
| `ParStages` の戻り値 | `List<Result<T>>` — stage が `Result.ok(v)` を返す場合、`Variant("ok", Some(Box(v)))` のリスト |
| `Merge.ordered` / `Merge.any` | **未実装**（本バージョンで追加） |
| `FlwStep::Merge(MergeMode)` | **未定義**（本バージョンで追加） |
| `MergeMode` enum | **未定義**（本バージョンで追加） |

---

## 成果物仕様

### 1. `ast.rs` — `MergeMode` enum + `FlwStep::Merge` variant 追加

```rust
/// par Merge モード (v51.2.0)
#[derive(Debug, Clone, PartialEq)]
pub enum MergeMode {
    /// 宣言順に結果を収集（デフォルト）
    Ordered,
    /// 完了順に結果を収集（将来の tokio 対応を見越した意図的分離）
    Any,
}

pub enum FlwStep {
    // ... 既存 ...
    /// par [A, B] |> Merge.ordered / Merge.any (v51.2.0)
    Merge(MergeMode),
}
```

`FlwStep` の既存 impl ブロックも更新:
- `stage_names()` → `FlwStep::Merge(_)` は `vec![]`
- `display_str()` → `FlwStep::Merge(MergeMode::Ordered)` は `"Merge.ordered"`、`FlwStep::Merge(MergeMode::Any)` は `"Merge.any"`

### 2. `frontend/parser.rs` — `Merge.ordered` / `Merge.any` パース

`parse_flw_step` 内で、peek が `Merge` の場合に `.ordered` / `.any` を先読み:

```
"Merge" "." "ordered" → FlwStep::Merge(MergeMode::Ordered)
"Merge" "." "any"     → FlwStep::Merge(MergeMode::Any)
"Merge"               → FlwStep::Stage("Merge")  // フォールバック（既存テスト互換）
```

### 3. `middle/compiler.rs` — `FlwStep::Merge` コンパイル

`build_step_call` 関数内で追加（`ctx.resolve_global("IO")` で IO グローバルインデックスを取得するパターン — `IO_GLOBAL_IDX` 定数は存在しないため）:

```rust
FlwStep::Merge(mode) => {
    let io_idx = ctx.resolve_global("IO").unwrap_or(u16::MAX);
    let io_expr = IRExpr::Global(io_idx, Type::Unknown);
    let method = match mode {
        MergeMode::Ordered => "merge_ordered_raw",
        MergeMode::Any    => "merge_any_raw",
    };
    IRExpr::Call(
        Box::new(IRExpr::FieldAccess(Box::new(io_expr), method.into(), Type::Unknown)),
        vec![input],
        Type::Unknown,
    )
}
```

`flw_step_name`（`stage_info_str` ではなく `flw_step_name` — compiler.rs 行 740 付近の実際の関数名）にも `FlwStep::Merge(mode)` arm を追加:

```rust
FlwStep::Merge(MergeMode::Ordered) => "merge.ordered",
FlwStep::Merge(MergeMode::Any)     => "merge.any",
```

`build_step_call_ctx`（ctx 付きバリアント）にも同様の arm を追加。

### 4. `backend/vm.rs` — `IO.merge_ordered_raw` / `IO.merge_any_raw` ハンドラ

```
IO.merge_ordered_raw(list: List<Any>) -> List<Any>
  - list の各要素を順に処理:
      Variant("ok" | "some", Some(payload)) → payload を Unwrap して results に追加
      Variant("ok" | "some", None)          → VMValue::Unit を results に追加（0引数 ok の場合）
      Variant("err", payload)               → fail-fast で Err を返す
      その他の値                             → そのまま results に追加（非 Result stages 互換）
  - Ok(List(results)) を返す

IO.merge_any_raw(list: List<Any>) -> List<Any>
  - std::thread 実装では merge_ordered_raw と同一動作
  - コメントで「将来 tokio 実装時に FuturesUnordered 相当の再実装を予定」と明記
```

### 5. 更新が必要な `FlwStep` match 箇所

新 variant `FlwStep::Merge(MergeMode)` が non-exhaustive にならないよう以下を更新:

| ファイル | 関数 / match | 追加内容 |
|---|---|---|
| `ast.rs` | `stage_names()` / `display_str()` | `FlwStep::Merge(_)` arm |
| `middle/compiler.rs` | `build_step_call` / `build_step_call_ctx` / `flw_step_name` | `FlwStep::Merge` arm |
| `middle/checker.rs` | 5 箇所（下記参照） | `FlwStep::Merge(_)` pass-through |
| `emit_python.rs` | 2 箇所の FlwStep match（各関数 1 行ずつ） | `FlwStep::Merge(_) => { /* skip */ }` |
| `middle/ast_lower_checker.rs` | `lower_flw_step` match | `FlwStep::Merge(MergeMode::Ordered) => v1("SMergeOrdered", ...)` / `FlwStep::Merge(MergeMode::Any) => v1("SMergeAny", ...)` |

**`checker.rs` の FlwStep match 5 箇所の期待動作:**

1. メイン step match（行 3476 付近）— `FlwStep::Merge(_)` は入力型をそのまま出力型として pass-through（par の出力 `List<T>` に対して型チェックをスキップ）
2. `FlwStep::Par(names)` arm 付近（行 3541）— `FlwStep::Merge(_)` は到達しないが網羅性のため arm 追加
3. `FlwStep::ParDistributed(names)` arm 付近（行 3580）— 同上
4. `FlwStep::Tap(_) | FlwStep::Inspect` arm（行 3616）— `| FlwStep::Merge(_)` を追加
5. `step_last_stage` / `step_first_stage` 相当の match（行 3624〜3630）— `FlwStep::Merge(_)` は false を返す（最初でも最後でもない特殊ステップ）

**`emit_python.rs` の `has_par` チェックへの影響:**
`FlwStep::Par | FlwStep::ParDistributed` を参照している `has_par` チェック（行 1185）には `FlwStep::Merge` を**追加しない**（Merge は par の後続ステップであり、par フラグとは独立）。

### 6. `driver.rs` — `v51200_tests` 追加

```rust
mod v51200_tests {
    fn cargo_toml_version_is_51_2_0() { ... }
    fn par_stage_merge_ordered()       { ... }  // Merge.ordered が List を返す（長さ 2）
    fn par_stage_merge_unordered()     { ... }  // Merge.any が List を返す（長さ 2）
}
```

`v51100_tests::cargo_toml_version_is_51_1_0` を削除（慣例）。

---

## テスト仕様

### `par_stage_merge_ordered`

```favnir
stage AddOne: Int -> Int = |n| { Result.ok(n + 1) }
stage AddTwo: Int -> Int = |n| { Result.ok(n + 2) }
seq P = par [AddOne, AddTwo] |> Merge.ordered
public fn main() -> Int { 0 |> P }
```

- `VM::run` が `Ok` を返すことを assert
- 戻り値が `Value::List` であり、長さ 2 であることを assert

### `par_stage_merge_unordered`

```favnir
stage AddOne: Int -> Int = |n| { Result.ok(n + 1) }
stage AddTwo: Int -> Int = |n| { Result.ok(n + 2) }
seq P = par [AddOne, AddTwo] |> Merge.any
public fn main() -> Int { 0 |> P }
```

- `VM::run` が `Ok` を返すことを assert
- 戻り値が `Value::List` であり、長さ 2 であることを assert

---

## バージョン要件

- `fav/Cargo.toml` version: `51.2.0`
- テスト数: 3115 → **3117**（純増 +2）
  - `v51200_tests` 3 件追加
  - `v51100_tests::cargo_toml_version_is_51_1_0` 1 件削除
  - 純増: +3 − 1 = **+2**

---

## 完了条件

- `par [A, B] |> Merge.ordered` / `|> Merge.any` がパース・コンパイル・実行できる
- `cargo test` 3117 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51200_tests` 3 件 pass:
  - `cargo_toml_version_is_51_2_0`
  - `par_stage_merge_ordered`
  - `par_stage_merge_unordered`

---

## リスク・制約

- `FlwStep` に `Merge(MergeMode)` 1 variant 追加するため、全 match 箇所（5 ファイル）が non-exhaustive になる → `cargo build` でコンパイルエラーとして検出される。
- `Merge.any` の `std::thread` 実装は `Merge.ordered` と同一動作。将来の tokio タスク化（v51.3+）まで意味的区別なし。コメントで明記する。
- `FlwStep::Merge` step は SeqStageCheck を通らない（par と同様、seq チェックは stage 単位）。
- `merge_ordered_raw` で `Variant("err", ...)` を検出したとき fail-fast する（`ParStages` の fail-fast と対称的な設計）。
- `Variant("ok" | "some", None)` の場合は `VMValue::Unit` を返す（0 引数 ok は実使用上ほぼ発生しないが、フォールバックを明示）。
- `Stream<T>` stage への `par` 対応はスコープ外（v51.3.0 以降）。

---

## ロードマップ対応

roadmap-v51.1-v52.0.md v51.2.0 より:

> `Merge.ordered`（`tokio::join_all` 相当）と `Merge.any`（`FuturesUnordered` 相当）を実装。
> `ast.rs` に `MergeMode` enum を追加。
>
> `Stream<T>` を返す stage にも `par` が適用できることを確認（→ 本バージョンスコープ外、v51.3.0 で実施）。
