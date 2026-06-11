# v14.0.5 Spec — セルフホスト完全 capability-context 化

Date: 2026-06-11

---

## 概要

v14.0.0 で capability-context 設計の完成宣言を行ったが、セルフホスト実装（`self/compiler.fav` / `self/cli.fav`）および E2E デモに旧 `!Effect` 記法が残っていた。v14.0.5 はこれを完全に除去し、**Favnir のすべての `.fav` ファイルが capability-context 記法のみで書かれた状態**を実現する。

---

## 1. 現状の残存 `!Effect`

| ファイル | 件数 | 内容 |
|---|---|---|
| `self/compiler.fav` | 3 | `compile_file_quiet` / `print_bytes` / `main` |
| `self/cli.fav` | 17 | `run_*` 関数群 / `main` |
| `infra/e2e-demo/airgap/src/analyze.fav` | 2 | `read_txn_csv` / `main` |
| `infra/e2e-demo/fav2py/src/pipeline.fav` | 2 | `load_csv_rows_json` / `main` |

---

## 2. 設計方針

### 2-1. VM による AppCtx 自動注入

エントリーポイント `public fn main` が ctx 引数を持つ場合（`param_count == 1`）、VM が自動的に AppCtx 相当の値を注入する。

```
// Favnir 側（新記法）
public fn main(ctx: AppCtx) -> Bool {
    bind args <- ctx.io.argv();
    ...
}

// Rust VM 側（driver.rs の exec_artifact_main_with_source）
let initial_args = if artifact.functions[main_idx].param_count == 1 {
    vec![Value::Record(vec![])]  // AppCtx プレースホルダー
} else {
    vec![]
};
```

**なぜ `Value::Record(vec![])` で十分か:**
`ctx.io.println(...)` は checker/lowerer フェーズで `AppCtx.io.println(...)` → VM builtin 呼び出しに変換される。VM は ctx の値自体のフィールドを参照しない。ctx はシグネチャ上の型情報として機能し、実行時は空レコードで代替できる。

### 2-2. `IO.*` → `ctx.io.*` 変換パターン

`ctx.field.method()` 構文（v13.6.0 実装済み）を使う。

| 旧記法 | 新記法 |
|---|---|
| `IO.println(s)` | `ctx.io.println(s)` |
| `IO.read_file_raw(path)` | `ctx.io.read_file_raw(path)` |
| `IO.write_stderr_raw(s)` | `ctx.io.write_stderr_raw(s)` |
| `IO.exit_raw(n)` | `ctx.io.exit_raw(n)` |
| `IO.argv()` | `ctx.io.argv()` |

ctx は呼び出し元から渡す（関数シグネチャに `ctx: AppCtx` または `ctx: CommonCtx` を追加）。

### 2-3. Ctx 型の選択指針

| 使用 capability | 型 |
|---|---|
| IO のみ | `CommonCtx` |
| DB + IO | `LoadCtx` または `WriteCtx` |
| 全部 | `AppCtx` |

`compiler.fav` / `cli.fav` の `run_*` / helper は IO のみ使うため `CommonCtx`。
`main` は呼び出し元（VM）から注入されるため `AppCtx`（上位互換）。

---

## 3. 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `exec_artifact_main_with_source`: param_count 判定 + AppCtx 注入 |
| `fav/src/lint.rs` | `collect_ambient`: `has_io_effect` 除外ロジック削除（`!IO` がなくなるため不要） |
| `fav/src/driver.rs` `v140000_tests` | bootstrap 除外フィルター（`compile_file_quiet` / `print_bytes` / `main`）を削除 |
| `fav/self/compiler.fav` | 3関数移行 + `IO.*` → `ctx.io.*`（6箇所） |
| `fav/self/cli.fav` | 17関数移行 + `IO.*` → `ctx.io.*`（110箇所） |
| `infra/e2e-demo/airgap/src/analyze.fav` | 2関数移行 |
| `infra/e2e-demo/fav2py/src/pipeline.fav` | 2関数移行 |
| `fav/Cargo.toml` | `version = "14.0.5"` |

---

## 4. 完了条件

| 確認項目 | 状態 |
|---|---|
| `check_bang_notation(compiler.fav).is_empty()` が true（フィルターなし） | |
| `check_bang_notation(checker.fav).is_empty()` が true | |
| `check_bang_notation(cli.fav).is_empty()` が true | |
| `fn main(ctx: AppCtx) -> Bool` を `fav run` で実行できる | |
| `cargo test v140005` 全件パス | |
| `cargo test` 全件パス | |

---

## 5. 設計上の注意点

- **`compile_file_quiet(ctx, path)` の ctx 伝播**: `main` が受け取った ctx をそのまま渡す。
- **`print_bytes(ctx, bytes)` の ctx**: `IO.println` を `ctx.io.println` に変えるだけ。
- **cli.fav の `run_*` 関数**: `main(ctx: AppCtx)` → 各 `run_*(ctx, ...)` へ ctx を伝播する。
- **`has_io_effect` 除外の削除**: `!IO` 宣言がなくなるため、E0023 の除外条件が不要になる（削除しても既存テストに影響なし）。
- **`Value::Record(vec![])` の安全性**: ctx のフィールドへの直接アクセス（`ctx.db` 等）がコード中にある場合はパニックする。compiler.fav / cli.fav は IO のみ使うため問題なし。
