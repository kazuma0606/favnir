# Plan: v51.2.0 — `par` Phase 2（Merge.ordered / Merge.any）

## 実装方針

`FlwStep` に `Merge(MergeMode)` variant を 1 つ追加するだけのシンプルな実装。
VM は `List<Any>` を受け取り Unwrap するだけ — tokio 不要、std::thread も不要。

`MergeMode` は `FlwStep::Merge(MergeMode)` のフィールドとして使用するため dead_code 警告は発生しない。

---

## 実装ステップ

### Step 1: `ast.rs` — `MergeMode` enum + `FlwStep::Merge` variant 追加

`MergeMode` enum を `ast.rs` に追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MergeMode {
    Ordered,
    Any,
}
```

`FlwStep` enum に 1 variant 追加（2 unit variant ではなく 1 tuple variant）:

```rust
pub enum FlwStep {
    // ... 既存 ...
    Merge(MergeMode),
}
```

`FlwStep` の impl ブロック更新:
- `stage_names()` — `Merge(_) => vec![]`
- `display_str()` — `Merge(MergeMode::Ordered) => "Merge.ordered"` / `Merge(MergeMode::Any) => "Merge.any"`

### Step 2: `frontend/parser.rs` — Merge.ordered / Merge.any パース

`parse_flw_step` 内、`"Merge"` peek 時の分岐を追加:

```
peek("Merge") + peek(".") + peek("ordered") → FlwStep::Merge(MergeMode::Ordered)
peek("Merge") + peek(".") + peek("any")     → FlwStep::Merge(MergeMode::Any)
peek("Merge") (フォールバック)               → FlwStep::Stage("Merge")（既存テスト互換）
```

Dot トークンと Ident("ordered"/"any") トークンを消費する点に注意。

### Step 3: `middle/compiler.rs` — build_step_call / build_step_call_ctx / flw_step_name 更新

**実際の関数名（`compile_flow_step` / `stage_info_str` ではない）:**
- `build_step_call`（行 751 付近）— メイン FlwStep → IRExpr 変換
- `build_step_call_ctx`（行 849 付近）— ctx 付きバリアント
- `flw_step_name`（行 740 付近）— 表示文字列

`build_step_call` に追加:

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

`flw_step_name` に追加:
```rust
FlwStep::Merge(MergeMode::Ordered) => "merge.ordered",
FlwStep::Merge(MergeMode::Any)     => "merge.any",
```

`build_step_call_ctx` にも同様の arm 追加（`build_step_call` の委譲パターンに従う）。

### Step 4: `backend/vm.rs` — merge_ordered_raw / merge_any_raw ハンドラ

`IO` namespace の builtin dispatch 内に追加。`merge_ordered_raw`:

```rust
"merge_ordered_raw" => {
    let list = match args.into_iter().next() {
        Some(VMValue::List(l)) => l,
        _ => return Err(vm.error(artifact, "merge_ordered_raw: expected List")),
    };
    let mut results = Vec::with_capacity(list.len());
    for item in list.iter() {
        match item {
            VMValue::Variant(tag, payload) if tag == "ok" || tag == "some" => {
                let v = match payload {
                    Some(b) => *b.clone(),
                    None    => VMValue::Unit,
                };
                results.push(v);
            }
            VMValue::Variant(tag, payload) if tag == "err" => {
                let msg = match payload {
                    Some(b) => match b.as_ref() {
                        VMValue::Str(s) => format!("merge: stage returned err: {}", s),
                        p => format!("merge: stage returned err: {:?}", p),
                    },
                    None => "merge: stage returned err".to_string(),
                };
                return Err(vm.error(artifact, &msg));
            }
            other => results.push(other.clone()),
        }
    }
    Ok(VMValue::List(FavList::new(results)))
}
```

`merge_any_raw`:

```rust
"merge_any_raw" => {
    // std::thread 実装では merge_ordered_raw と同一動作。
    // 将来 tokio 実装時に FuturesUnordered 相当の再実装を予定（v51.3+）。
    // (merge_ordered_raw と同じ実装)
}
```

### Step 5: match 網羅性更新（FlwStep 追加に伴う全箇所）

| ファイル | 関数 | 追加 arm |
|---|---|---|
| `ast.rs` | `stage_names` / `display_str` | `Merge(_)` / `Merge(Ordered)` / `Merge(Any)` |
| `middle/compiler.rs` | `build_step_call` / `build_step_call_ctx` / `flw_step_name` | `Merge(mode)` arm |
| `middle/checker.rs` | 5 箇所（下記） | `Merge(_)` arm |
| `emit_python.rs` | 2 箇所（各関数 1 行） | `Merge(_) => { /* skip */ }` |
| `middle/ast_lower_checker.rs` | `lower_flw_step` | `Merge(Ordered) => v1("SMergeOrdered", ...)` / `Merge(Any) => v1("SMergeAny", ...)` |

**checker.rs 5 箇所の追加内容:**
1. メイン step match — `Merge(_) => { /* 入力型を出力型として pass-through */ }`
2. `FlwStep::Par` arm 付近 — `| FlwStep::Merge(_)` を追加（あるいは単独 arm）
3. `FlwStep::ParDistributed` arm 付近 — 同上
4. `FlwStep::Tap(_) | FlwStep::Inspect` arm — `| FlwStep::Merge(_)` を追加
5. `step_first_stage` / `step_last_stage` match — `Merge(_) => false`

**emit_python.rs `has_par` チェック（行 1185）には `Merge` を追加しない。**

### Step 6: `driver.rs` — v51200_tests 追加

3 件追加、1 件削除:
- 追加: `cargo_toml_version_is_51_2_0`, `par_stage_merge_ordered`, `par_stage_merge_unordered`
- 削除: `v51100_tests::cargo_toml_version_is_51_1_0`

各テストは `run_src_to_value` (または同等の test helper) を使用し:
- `par_stage_merge_ordered` / `par_stage_merge_unordered` は戻り値が `VMValue::List` かつ `len() == 2` を assert

### Step 7: バージョン更新・完了処理

- `fav/Cargo.toml` → `"51.2.0"`
- `cargo test` 3117 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` エントリ追加
- `versions/current.md` 更新
- `roadmap-v51.1-v52.0.md` v51.2.0 実績欄更新

---

## ファイル変更リスト

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `MergeMode` enum 追加、`FlwStep::Merge(MergeMode)` 追加、impl 更新 |
| `fav/src/frontend/parser.rs` | `parse_flw_step` に Merge.ordered/any 分岐追加 |
| `fav/src/middle/compiler.rs` | `build_step_call` + `build_step_call_ctx` + `flw_step_name` arm 追加 |
| `fav/src/middle/checker.rs` | 5 箇所の FlwStep match に arm 追加 |
| `fav/src/emit_python.rs` | 2 箇所の FlwStep match に arm 追加（`has_par` は変更なし） |
| `fav/src/middle/ast_lower_checker.rs` | `lower_flw_step` に `SMergeOrdered` / `SMergeAny` arm 追加 |
| `fav/src/backend/vm.rs` | `IO.merge_ordered_raw` / `IO.merge_any_raw` ハンドラ追加 |
| `fav/src/driver.rs` | `v51200_tests` 追加、`cargo_toml_version_is_51_1_0` 削除 |
| `fav/Cargo.toml` | version → `"51.2.0"` |
| `CHANGELOG.md` | v51.2.0 エントリ追加 |
| `versions/current.md` | v51.2.0、3117 tests に更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v51.2.0 実績欄更新 |

---

## リスク

1. **FlwStep 非網羅 match**: `cargo build` でコンパイルエラーとして検出されるため見落とし不可。
2. **`ctx.resolve_global("IO")`**: `build_step_call` の実際のシグネチャ（ctx 引数の型）を確認してから実装する。`build_step_call_ctx` との差異に注意。
3. **`ast_lower_checker.rs` の `lower_flw_step`**: `SMergeOrdered` / `SMergeAny` タグを checker.fav 側が認識するか確認が必要。checker.fav の FlwStep タグ一覧を grep して対応するパターンを確認すること。
4. **Stream<T> 対応はスコープ外**: ロードマップ行 54 の記述は v51.3.0 以降で対応。
