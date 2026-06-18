# v13.3.0 Implementation Plan

Date: 2026-06-10

---

## Phase A — compiler.fav E0001 修正（`collect` 式）

### A-1: `ast_lower_checker.rs` に `Collect` ハンドラ追加

**ファイル**: `fav/src/middle/ast_lower_checker.rs`

`lower_expr` の `_ => v1("EVar", sv("_unsupported_"))` の直前に追加:

```rust
ast::Expr::Collect(block, _) => {
    v1("ECollect", lower_block(block))
}
```

### A-2: `checker.fav` に `ECollect` ケース追加

**ファイル**: `fav/self/checker.fav`

`infer_hm` の match ブロック（現在 5 ケース + `_` fallback）に追加:

```
ECollect(inner) =>
    Result.and_then(infer_hm(inner, env, state), |r|
        Result.ok(inf_result_of("Unknown", inf_state_of(r))))
```

**位置**: `ECall` ケースの直後、`_` fallback の直前。

### A-3: 動作確認

```bash
cargo run --bin fav -- check self/compiler.fav
# → exit 0 / no errors
```

---

## Phase B — 組み込み interface 登録（HttpClient / Io / Env）

**ファイル**: `fav/src/middle/checker.rs`

`register_builtin_capabilities()` に 3 interface を追加。

```rust
// Map<String, String> の型ヘルパー
let map_ss = || Type::Map(Box::new(s()), Box::new(s()));
let ro = || Type::Option(Box::new(s()));

// HttpClient
let http_get  = mk(vec![s(), map_ss()], rs());
let http_post = mk(vec![s(), s(), map_ss()], rs());
self.register_interface("HttpClient", vec![
    ("get",    http_get.clone()),
    ("post",   http_post.clone()),
    ("put",    http_post.clone()),
    ("delete", mk(vec![s(), map_ss()], rs())),
]);
// Io
self.register_interface("Io", vec![
    ("println",   mk(vec![s()], Type::Unit)),
    ("print",     mk(vec![s()], Type::Unit)),
    ("read_line", mk(vec![], rs())),
]);
// Env
self.register_interface("Env", vec![
    ("require", mk(vec![s()], rs())),
    ("get",     mk(vec![s()], ro())),
]);
```

`Type::Map` / `Type::Option` は既存の型なので追加依存なし。

---

## Phase C — Rune ファイル作成

### C-1: `runes/http/http_client_impl.fav`

HttpClientImpl が HttpClient を impl。
既存の `Http.get_raw` / `Http.post_raw` に委譲。

### C-2: `runes/io/io_impl.fav`

IoImpl が Io を impl。
既存の `IO.println` / `IO.print_raw` / `IO.read_line_raw` に委譲。

### C-3: `runes/io/io_capture.fav`

IoCapture（テスト用 no-op Io）。
`IoCapture.captured()` で記録した出力を返す（MVP では no-op で ok）。

### C-4: `runes/env/env_impl.fav`

EnvImpl が Env を impl。
`IO.env_require_raw` / `IO.env_get_raw` に委譲。

### C-5: `runes/env/mock_env.fav`

MockEnv（テスト用 Map ベース Env）。

---

## Phase D — W009 追加（IO.* / Http.*）

**ファイル**: `fav/src/lint.rs`

`DEPRECATED_RUNE_CALLS` 定数に追加:

```rust
("IO",   "println"),
("IO",   "print_raw"),
("IO",   "read_line_raw"),
("IO",   "env_require_raw"),
("IO",   "env_get_raw"),
("Http", "get_raw"),
("Http", "post_raw"),
```

`check_deprecated_rune_calls` は v13.2.0 実装済みなので、定数追加のみ。

---

## Phase E — テストと動作確認

**ファイル**: `fav/src/driver.rs`

`v133000_tests` モジュールに以下を追加:
- `version_is_13_3_0`
- `collect_expr_lowers_to_ecollect`（`scan_entries` 関数を含む .fav を check）
- `http_client_interface_registered`
- `io_interface_registered`
- `env_interface_registered`
- `io_interface_println_typecheck`
- `env_interface_require_typecheck`
- `w009_io_println_deprecated`
- `w009_http_get_deprecated`
- `compiler_fav_check_passes`（実際に `fav check self/compiler.fav` を実行）

`v132000_tests` の `version_is_13_2_0` テストをコメントアウト。

---

## Phase F — バージョンバンプ + コミット

1. `fav/Cargo.toml` → `version = "13.3.0"`
2. `versions/v13.3.0/tasks.md` 更新（全タスク完了チェック）
3. `git add` + `git commit` on `feat/v13-capability-context`

---

## 技術的注意点

### `ECollect` の infer について

`infer_hm` の match に `ECollect(inner)` を追加する位置は `ECall` ケースの直後が望ましい。
`ECall` は行が長いため、改行後に挿入する。

`inner` を type-check する理由: collect ブロック内に型エラーがあれば検出できるようにするため。
`"Unknown"` を返す理由: collect 式の要素型（yield 型）を checker.fav で追跡できないため。
`types_compatible("Unknown", declared)` は常に `true` → E0009 は発生しない。

### `Type::Map` の追加

`register_builtin_capabilities` で `Type::Map` を使う。
`Type::Map(Box<Type>, Box<Type>)` は既存の型ヴァリアントなので依存追加なし。

### `IO.env_require_raw` / `IO.env_get_raw` の存在確認

W009 に追加する前に、これらの primitive が実際に vm.rs に存在するか確認する。
存在しない場合は W009 対象から除外し、spec の差分として tasks.md にメモする。
