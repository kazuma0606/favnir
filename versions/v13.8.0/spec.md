# v13.8.0 Spec — ambient effect 禁止（W008 → E0023）

Date: 2026-06-11

---

## 概要

v13.1.0 で警告（W008）として導入した ambient effect 検出を、
標準 `fav check` のコンパイルエラー（E0023）に昇格する。

「capability 引数がなければ純粋」を言語レベルで強制する第一歩。
`--legacy` フラグによる後方互換パスを維持しつつ、
新規コードは必ず `ctx.io.println(...)` スタイルを強制される。

---

## 現状（v13.7.0）

```fav
// 現状: IO.println を使っても警告のみ（W008）
fn process(rows: List<TxnRow>) -> Unit {
    bind _ <- IO.println("processing...")  // W008: only with --ambient flag
    ()
}
```

問題:
- `--ambient` フラグを付けない限り W008 は表示されない
- ライブラリ関数に ambient IO が混入していても検出されない
- 「ctx なし = 純粋」という設計目標が実際には保証されていない

---

## 新しい動作

### 非 legacy モード（標準）

```
$ fav check pipeline.fav

error[E0023]: ambient effect call — `IO.println` called without ctx argument
  --> pipeline.fav:5:18
4  |
5  |     bind _ <- IO.println("processing...")
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: pass an io capability through the function signature
   = help: use `ctx.io.println(...)` instead of `IO.println(...)`
   = help: use `--legacy` flag to allow ambient calls during migration
error: 1 ambient effect call(s) rejected (E0023)
  = note: use `--legacy` to allow ambient calls during migration
```

### legacy モード

```
$ fav check --legacy pipeline.fav
pipeline.fav: no errors found
```

W008 は `--ambient` フラグ経由でのみ表示される（変化なし）。

---

## E0023 の適用ルール

### エラーになる例

```fav
// ctx なし関数で IO.* を直接呼ぶ → E0023
fn log_rows(rows: List<Row>) -> Unit {
    bind _ <- IO.println("done")  // E0023
    ()
}

// ctx なし関数で Postgres.* を直接呼ぶ → E0023
fn fetch(sql: String) -> Result<List<Row>, String> {
    Postgres.query_raw(sql, "[]")  // E0023
}
```

### エラーにならない例

```fav
// !IO アノテーション付き関数 → 明示的にオプトイン（v13.10.0 で廃止予定）
fn compile_file(path: String) -> Result<Artifact, String> !IO {
    Result.and_then(IO.read_file_raw(path), |src| compile(src))  // OK
}

// ctx.io.println スタイル → E0023 なし
fn process(ctx: AppCtx, rows: List<Row>) -> Unit {
    bind _ <- ctx.io.println("done")  // OK
    ()
}

// 純粋関数（IO なし）→ E0023 なし
fn double(n: Int) -> Int {
    n * 2  // OK
}
```

### 免除ルール（v13.8.0 時点）

| 条件 | 扱い |
|---|---|
| `!IO` アノテーション付き関数 | E0023 免除（明示的オプトイン） |
| `ctx.io.println(...)` スタイル | 検出対象外（FieldAccess ネストのため） |
| 純粋関数（IO 呼び出しなし） | E0023 なし |

`!IO` 免除は v13.10.0 で `!` 記法が廃止されるまでの暫定措置。

---

## 対象となる ambient namespaces

W008 から引き継ぎ（変更なし）:

```rust
const AMBIENT_NAMESPACES: &[&str] = &[
    "IO", "Postgres", "AWS", "Snowflake", "Http", "Grpc",
    "Llm", "Queue", "Cache", "Slack", "Email",
];
const AMBIENT_GEN_FNS: &[&str] = &["uuid_raw", "uuid_v7_raw", "nano_id"];
```

---

## compiler.fav の移行

### 移行対象（IO 呼び出しを含んでいた関数）

| 関数 | 移行方針 |
|---|---|
| `compile_file_after_prog` / `_after_parse` / `_after_lex` | 削除（デバッグ用 `IO.println` のみ） |
| `compile_file(path)` | 削除（`compile_file_quiet` に統合） |
| `compile_bytes(path)` | 削除（Rust 側でファイル読み込みに移行） |
| `compile_file_quiet(path)` | `!IO` 付きで保持（bootstrap 用） |
| `print_bytes(bytes)` | `!IO` 付きで保持（bootstrap 用） |
| `main()` | `!IO` 付きで保持（bootstrap entry point） |

### パブリック API の変化

| 関数 | v13.7.0 | v13.8.0 |
|---|---|---|
| `compile_bytes_from_src(src)` | あり（IO なし） | 維持（メイン API） |
| `compile_bytes(path)` | あり（IO あり） | **削除** |
| `fmt_source(src)` | あり（IO なし） | 維持 |
| `lint_source(src)` | あり（IO なし） | 維持 |
| `main()` | あり（`!IO`） | 維持（`!IO`） |

### compiler_fav_runner.rs の変化

`compile_file_to_bytes(path)` — Rust 側でファイルを読んでから `compile_bytes_from_src` に委譲:

```rust
// Before: compiler.fav の compile_bytes(path) を VM 経由で呼び出す
pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String> {
    let fn_idx = artifact.fn_idx_by_name("compile_bytes")?;
    VM::run(&artifact, fn_idx, vec![Value::Str(path.to_string())])
    // ...
}

// After: Rust でファイル読み込み → compile_bytes_from_src に委譲
pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String> {
    let src = std::fs::read_to_string(path)?;
    compile_src_str_to_bytes(&src)
}
```

---

## エラーコード

### E0023: ambient effect call is not allowed

```
E0023: ambient effect call is not allowed
  --> pipeline.fav:5:18
4  |
5  |     bind _ <- IO.println("processing...")
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: pass an io capability through the function signature
   = help: use `ctx.io.println(...)` instead of `IO.println(...)`
   = help: use `--legacy` flag to allow ambient calls during migration
```

トリガー条件:
- 非 legacy モードの標準 `fav check`
- `!IO` アノテーションを持たない関数のボディで AMBIENT_NAMESPACES の名前空間を直接呼び出す

---

## `--ambient` フラグの扱い

v13.8.0 では `--ambient` を廃止しない（後方互換）。
`fav check --ambient` は引き続き W008 を表示する（挙動変化なし）。
標準 `fav check` が E0023 を出すため、`--ambient` は実質的に重複になるが、
W008 の詳細表示（行数・下線）を確認したい場合に引き続き使用可能。

---

## スコープ外（v13.8.0 では実装しない）

- JSON 出力モード（`--json`）での E0023 統合 — フォーマット設計が必要
- `fav check --ambient` の廃止 — v13.10.0 で `!` 記法廃止と同時に検討
- `self/checker.fav` と `self/cli.fav` の ctx 移行 — 現在 IO 呼び出しなし or 別バージョン
- `par` ステップを含む関数の ambient チェック — 現行と同じ動作を維持

---

## 後方互換性

- `--legacy` フラグで E0023 はゼロ（W008 は `--ambient` で確認可能）
- `!IO` アノテーション付き関数は E0023 免除（既存コードの大半は影響なし）
- テスト: 既存の W008 テスト（`w008_ambient_*`）が全件パスすること
