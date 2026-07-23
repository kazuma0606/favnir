# Spec: v46.6.0 — `fav explain` 2.0 Phase 1（パイプライン図改善）

Date: 2026-07-17
Status: 計画中

---

## 概要

`fav explain --format mermaid` 出力の品質向上。
`return` 早期脱出パスを dead path（点線 `-.->` ）として図示し、
`Err(...)` 返却パスをエラーパス（赤ノード）として区別する新しい Mermaid レンダラーを追加する。

---

## スコープ

### 新関数: `pub(crate) render_pipeline_mermaid_v2(program: &Program) -> String`

`driver.rs` に追加（`render_graph_mermaid_with_opts` とは独立した新関数）。

- `pub(crate)` とする理由: `driver.rs` テストモジュール（`v466000_tests`）から直接呼び出すため
- **v46.6.0 は関数追加のみ**。`fav explain --format mermaid` コマンドへの差し替えは v46.7.0 以降に行う
- **スキャン範囲**: `fn` ボディの**トップレベル** `stmts` のみ（ネストした `if`/`match`/`for` 内の `return` は Phase 1 スコープ外）

**出力フォーマット**:
```mermaid
flowchart LR
    classDef deadPath stroke-dasharray: 5 5
    classDef errPath fill:#ffcccc,stroke:#cc0000
    process["fn process"]
    process_dead_return(["return"])
    process -.-> process_dead_return
    class process_dead_return deadPath
```

`Err(...)` を返す `return` の場合:
```mermaid
    validate_err_return(["return Err"])
    validate -.-> validate_err_return
    class validate_err_return errPath
```

**実装詳細**:
1. `flowchart LR` を出力
2. `classDef deadPath stroke-dasharray: 5 5` と `classDef errPath fill:#ffcccc,stroke:#cc0000` を出力
3. 各 `FnDef` をスキャン:
   - ノード `fn_id["fn <name>"]` を追加
   - `body.stmts` の `Stmt::Return(r)` を検出
   - 検出ごとに return ノード `<fn_id>_dead_return(["return"])` を追加し `<fn_id> -.-> <fn_id>_dead_return` (dotted)
   - return の expr が `Expr::Apply(Expr::Ident("Err", _), _, _)` なら ノードを `(["return Err"])` とし `class <fn_id>_dead_return errPath` を付与
   - それ以外の dead return は `class <fn_id>_dead_return deadPath` を付与

**ヘルパー**:
```rust
/// Stmt::Return が存在するか、かつ Err(...) を返すかを判定
fn scan_returns(stmts: &[Stmt]) -> (bool, bool) {
    // (has_any_return, has_err_return)
}
```

---

## 実装ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `render_pipeline_mermaid_v2` + `scan_returns` + `v466000_tests` 追加 |
| `fav/Cargo.toml` | version → `46.6.0` |
| `CHANGELOG.md` | v46.6.0 エントリ追加 |
| `versions/current.md` | v46.6.0（3005 tests）に更新 |

---

## テスト

| テスト名 | 内容 |
|---|---|
| `explain_mermaid_includes_dead_path` | `return` を含む `fn` をパースし `render_pipeline_mermaid_v2` が `"-.-> "` を含む出力を返すことを確認 |
| `explain_pipeline_v2` | `return Err(...)` を含む `fn` で `"errPath"` と `"deadPath"` の両方が出力に含まれることを確認 |

**テスト数**: 3003 + 2 = **3005 tests**

---

## 完了条件

- `cargo test` 3005 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version = `46.6.0`
- `CHANGELOG.md` に v46.6.0 エントリ
- `versions/current.md` を v46.6.0（3005 tests）に更新
