# v13.3.0 Spec — HttpClient / Io / Env interface 実装 + compiler.fav E0001 修正

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## テーマ

I/O 操作 capability の interface 型を言語に導入する。
v13.2.0 で確立したデータ操作 capability（DbRead / DbWrite / StorageRead / StorageWrite）に続き、
HTTP クライアント・標準 I/O・環境変数の 3 capability interface を実装する。

また、`fav check self/compiler.fav` が `collect { ... }` 式で E0001 クラッシュする
既知バグを本バージョンで修正する。

---

## 1. compiler.fav E0001 修正（`collect` 式未対応）

### 問題

`self/compiler.fav:499` に `collect { scan_collect(chars) }` がある。
`ast_lower_checker.rs` の `lower_expr` はこの式を処理できず、
fallback `_ => v1("EVar", sv("_unsupported_"))` を emit する。
checker.fav が `_unsupported_` という未定義変数を参照しようとして E0001 が発生する。

### 修正方針

**`ast_lower_checker.rs`**:
- `ast::Expr::Collect(block, _) =>` ケースを追加
- `v1("ECollect", lower_block(block))` を emit

**`self/checker.fav`**:
- `infer_hm` の match に `ECollect(inner)` ケースを追加
- inner を type-check した上で `"Unknown"` を返す:
  ```
  ECollect(inner) =>
      Result.and_then(infer_hm(inner, env, state), |r|
          Result.ok(inf_result_of("Unknown", inf_state_of(r))))
  ```
- `"Unknown"` は `types_compatible` で常に `true` → E0009 も発生しない

---

## 2. HttpClient interface

### 型定義

```
interface HttpClient {
    get(url: String, headers: Map<String, String>) -> Result<String, String>
    post(url: String, body: String, headers: Map<String, String>) -> Result<String, String>
    put(url: String, body: String, headers: Map<String, String>) -> Result<String, String>
    delete(url: String, headers: Map<String, String>) -> Result<String, String>
}
```

### Rune 実装（`runes/http/http_client_impl.fav`）

```favnir
type HttpClientImpl(String)

public fn HttpClientImpl.new() -> HttpClientImpl { HttpClientImpl("default") }

impl HttpClient for HttpClientImpl {
    fn get(c: HttpClientImpl, url: String, headers: Map<String, String>) -> Result<String, String> !Http {
        Http.get_raw(url)
    }
    fn post(c: HttpClientImpl, url: String, body: String, headers: Map<String, String>) -> Result<String, String> !Http {
        Http.post_raw(url, body)
    }
    fn put(c: HttpClientImpl, url: String, body: String, headers: Map<String, String>) -> Result<String, String> !Http {
        Http.post_raw(url, body)
    }
    fn delete(c: HttpClientImpl, url: String, headers: Map<String, String>) -> Result<String, String> !Http {
        Http.get_raw(url)
    }
}
```

---

## 3. Io interface

### 型定義

```
interface Io {
    println(msg: String) -> Unit
    print(msg: String) -> Unit
    read_line() -> Result<String, String>
}
```

### Rune 実装（`runes/io/io_impl.fav`）

```favnir
type IoImpl(String)

public fn IoImpl.new() -> IoImpl { IoImpl("default") }

impl Io for IoImpl {
    fn println(io: IoImpl, msg: String) -> Unit !IO {
        IO.println(msg)
    }
    fn print(io: IoImpl, msg: String) -> Unit !IO {
        IO.print_raw(msg)
    }
    fn read_line(io: IoImpl) -> Result<String, String> !IO {
        IO.read_line_raw()
    }
}
```

### IoCapture 型（`runes/io/io_capture.fav`）

テスト用 — stdout をキャプチャして文字列として返す。

```favnir
type IoCapture(List<String>)

public fn IoCapture.empty() -> IoCapture { IoCapture(List.empty()) }

impl Io for IoCapture {
    fn println(io: IoCapture, msg: String) -> Unit {
        // capture — no-op at runtime (captured via IoCapture.captured)
        ()
    }
    fn print(io: IoCapture, msg: String) -> Unit { () }
    fn read_line(io: IoCapture) -> Result<String, String> {
        Result.ok("")
    }
}

public fn IoCapture.captured(io: IoCapture) -> String {
    String.join(io, "\n")
}
```

---

## 4. Env interface

### 型定義

```
interface Env {
    require(key: String) -> Result<String, String>
    get(key: String) -> Option<String>
}
```

### Rune 実装（`runes/env/env_impl.fav`）

```favnir
type EnvImpl(String)

public fn EnvImpl.new() -> EnvImpl { EnvImpl("process") }

impl Env for EnvImpl {
    fn require(e: EnvImpl, key: String) -> Result<String, String> !IO {
        IO.env_require_raw(key)
    }
    fn get(e: EnvImpl, key: String) -> Option<String> !IO {
        IO.env_get_raw(key)
    }
}
```

### MockEnv 型（`runes/env/mock_env.fav`）

```favnir
type MockEnv(Map<String, String>)

public fn MockEnv.empty() -> MockEnv { MockEnv(Map.empty()) }
public fn MockEnv.set(env: MockEnv, key: String, val: String) -> MockEnv {
    MockEnv(Map.insert(env, key, val))
}

impl Env for MockEnv {
    fn require(e: MockEnv, key: String) -> Result<String, String> {
        match Map.get(e, key) {
            None => Result.err(String.concat("missing env var: ", key))
            Some(v) => Result.ok(v)
        }
    }
    fn get(e: MockEnv, key: String) -> Option<String> {
        Map.get(e, key)
    }
}
```

---

## 5. W009 追加（IO.* / Http.* 直接呼び出し deprecated）

v13.2.0 で Postgres / AWS / Snowflake の直接呼び出しに W009 を追加済み。
本バージョンでは IO.* / Http.* を追加する:

```
W009: direct Rune call is deprecated — use capability interface instead
  --> pipeline.fav:8:10
   |
 8 | bind _ <- IO.println("done")
   |           ^^^^^^^^^^^^ deprecated
   |
   = help: migrate to `ctx.io.println("done")`
   = note: direct Rune calls will be an error in v14.0
```

追加対象: `IO.println`, `IO.print_raw`, `IO.read_line_raw`, `IO.env_require_raw`, `IO.env_get_raw`,
          `Http.get_raw`, `Http.post_raw`

---

## 6. 組み込み interface 登録

`checker.rs` の `InterfaceRegistry::new()` → `register_builtin_capabilities()` に追加:

```rust
// HttpClient
self.register_interface("HttpClient", vec![
    ("get",    mk(vec![s(), map_ss()], rs())),
    ("post",   mk(vec![s(), s(), map_ss()], rs())),
    ("put",    mk(vec![s(), s(), map_ss()], rs())),
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
    ("get",     mk(vec![s()], Type::Option(Box::new(s())))),
]);
```

---

## 7. テスト

| テスト名 | 内容 |
|---|---|
| `version_is_13_3_0` | Cargo.toml バージョン確認 |
| `collect_expr_lowers_to_ecollect` | `scan_entries` 関数を含む .fav を check → no error |
| `ecollect_infers_unknown` | ECollect の infer_hm が "Unknown" を返す |
| `http_client_interface_registered` | HttpClient interface が登録済みか確認 |
| `io_interface_registered` | Io interface が登録済みか確認 |
| `env_interface_registered` | Env interface が登録済みか確認 |
| `io_interface_println_typecheck` | `ctx: Io` → `ctx.println("x")` が通る |
| `env_interface_require_typecheck` | `ctx: Env` → `ctx.require("KEY")` が通る |
| `w009_io_println_deprecated` | `IO.println(...)` が W009 を出力する |
| `w009_http_get_deprecated` | `Http.get_raw(...)` が W009 を出力する |
| `compiler_fav_check_passes` | `fav check self/compiler.fav` が 0 エラーで終了 |
