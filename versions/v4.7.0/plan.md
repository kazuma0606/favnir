# Favnir v4.7.0 実装計画 — Env Rune

作成日: 2026-05-17

---

## 実装フェーズ概要

| Phase | 内容                                  | 主要ファイル                        |
|-------|---------------------------------------|-------------------------------------|
| 0     | バージョン更新                         | Cargo.toml, main.rs                 |
| 1     | VM プリミティブ追加                    | vm.rs                               |
| 2     | fav.toml 拡張                         | toml.rs                             |
| 3     | checker.rs 変更（`!Env` エフェクト）   | middle/checker.rs                   |
| 4     | compiler.rs 変更                      | middle/compiler.rs                  |
| 5     | driver.rs 変更（設定反映・自動 dotenv）| driver.rs                           |
| 6     | rune ファイル作成                     | runes/env/                          |
| 7     | テスト追加                            | vm_stdlib_tests.rs, driver.rs       |
| 8     | examples 追加                         | examples/env_demo/                  |

---

## Phase 0: バージョン更新

`fav/Cargo.toml` の `version` を `"4.7.0"` に更新。`fav/src/main.rs` のヘルプ文字列を `v4.7.0` に更新。

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: `EnvConfig` thread_local

```rust
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub dotenv: Option<String>,  // dotenv ファイルパス
    pub prefix: String,          // キープレフィックス（デフォルト ""）
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfig { dotenv: None, prefix: String::new() }
    }
}

thread_local! {
    static ENV_CONFIG: RefCell<EnvConfig> = RefCell::new(EnvConfig::default());
}

pub fn set_env_config(cfg: EnvConfig) {
    ENV_CONFIG.with(|c| *c.borrow_mut() = cfg);
}
```

### 1-B: `env_resolve_key` ヘルパー

プレフィックスを適用してキーを解決する:

```rust
fn env_resolve_key(key: &str) -> String {
    ENV_CONFIG.with(|c| {
        let cfg = c.borrow();
        if cfg.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}{}", cfg.prefix, key)
        }
    })
}
```

### 1-C: `Env.get_raw` 実装

```rust
"Env.get_raw" => {
    let key = /* args[0] as String */;
    let resolved = env_resolve_key(&key);
    match std::env::var(&resolved) {
        Ok(val) => Ok(some_vm(VMValue::Str(val))),
        Err(_)  => Ok(none_vm()),
    }
}
```

`some_vm` / `none_vm` は既存パターン（`VMValue::Record { name: "Some"/"None", ... }`）を使用。

### 1-D: `Env.require_raw` 実装

```rust
"Env.require_raw" => {
    let key = /* args[0] as String */;
    let resolved = env_resolve_key(&key);
    match std::env::var(&resolved) {
        Ok(val) => Ok(ok_vm(VMValue::Str(val))),
        Err(_)  => Ok(err_vm(VMValue::Str(format!("ENV_MISSING: {}", resolved)))),
    }
}
```

### 1-E: `Env.get_int_raw` 実装

```rust
"Env.get_int_raw" => {
    let key = /* args[0] as String */;
    let resolved = env_resolve_key(&key);
    match std::env::var(&resolved) {
        Err(_)   => Ok(err_vm(VMValue::Str(format!("ENV_MISSING: {}", resolved)))),
        Ok(val)  => match val.trim().parse::<i64>() {
            Ok(n)  => Ok(ok_vm(VMValue::Int(n))),
            Err(_) => Ok(err_vm(VMValue::Str(format!("ENV_PARSE_INT: {}={}", resolved, val)))),
        },
    }
}
```

### 1-F: `Env.get_bool_raw` 実装

受理する値:
- `true`: `"true"`, `"1"`, `"yes"`, `"on"` (大文字小文字無視)
- `false`: `"false"`, `"0"`, `"no"`, `"off"` (大文字小文字無視)
- それ以外: `Err("ENV_PARSE_BOOL: ...")`

### 1-G: `Env.load_dotenv_raw` 実装

外部クレートを使わず自前でパース（`dotenvy` 等の追加依存なし）:

```rust
fn parse_dotenv(content: &str) -> Vec<(String, String)> {
    content.lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| {
            let l = l.trim();
            // KEY=VALUE または KEY="VALUE"
            l.splitn(2, '=').collect::<Vec<_>>().as_slice() {
                [k, v] => {
                    let key = k.trim().to_string();
                    let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    if key.is_empty() { None } else { Some((key, val)) }
                }
                _ => None
            }
        })
        .collect()
}
```

`Env.load_dotenv_raw(path)`:
- ファイル読み込み → `parse_dotenv` でパース
- `std::env::set_var(key, val)` で設定（既存変数は上書きしない: `std::env::var(&key).is_err()` のときのみ）
- 成功: `Ok(ok_vm(VMValue::Unit))`
- ファイル読み込み失敗: `Ok(err_vm(VMValue::Str("ENV_DOTENV_NOT_FOUND: path")))`

### 1-H: `Env.all_raw` 実装

```rust
"Env.all_raw" => {
    let map: BTreeMap<String, VMValue> = std::env::vars()
        .map(|(k, v)| (k, VMValue::Str(v)))
        .collect();
    Ok(VMValue::Record { name: "Map".to_string(), fields: map })
}
```

---

## Phase 2: `fav.toml` 拡張（`fav/src/toml.rs`）

```rust
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub dotenv: Option<String>,
    pub prefix: String,
}

impl Default for EnvConfig { ... }
```

`FavToml` に `pub env: Option<EnvConfig>` を追加。

`[env]` セクションのパース:
```rust
"env" => {
    let mut current: EnvConfig = env_cfg.take().unwrap_or_default();
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "dotenv" => current.dotenv = Some(val.to_string()),
            "prefix" => current.prefix = val.to_string(),
            _ => {}
        }
    }
    env_cfg = Some(current);
}
```

**注意**: `env` という変数名は Rust の `std::env` と衝突するリスクがあるため、内部変数名を `env_cfg` とする。また `let mut current: EnvConfig =` の型注釈を必ず付ける（`log` の前例から）。

`FavToml` リテラルに `env: None` を追加（checker.rs ×2、resolver.rs ×2、driver.rs ×1）。

---

## Phase 3: checker.rs 変更（`fav/src/middle/checker.rs`）

### `!Env` エフェクトの強制

```rust
fn require_env_effect(&self, span: Span) -> Option<TypeError> {
    let has_env = self.current_effects.iter().any(|e| {
        matches!(e, Effect::Unknown(s) if s == "Env")
    });
    if !has_env {
        Some(TypeError {
            code: "E0312".to_string(),
            message: "function uses Env but does not declare !Env effect".to_string(),
            span,
        })
    } else {
        None
    }
}
```

`check_builtin_apply` に追加:

```rust
("Env", "get_raw")          => { self.require_env_effect(span)?; Some(Type::Option(Box::new(Type::String))) }
("Env", "require_raw")      => { self.require_env_effect(span)?; Some(Type::Result(Box::new(Type::String), Box::new(Type::String))) }
("Env", "get_int_raw")      => { self.require_env_effect(span)?; Some(Type::Result(Box::new(Type::Int), Box::new(Type::String))) }
("Env", "get_bool_raw")     => { self.require_env_effect(span)?; Some(Type::Result(Box::new(Type::Bool), Box::new(Type::String))) }
("Env", "load_dotenv_raw")  => { self.require_env_effect(span)?; Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String))) }
("Env", "all_raw")          => { self.require_env_effect(span)?; Some(Type::Map(Box::new(Type::String), Box::new(Type::String))) }
("Env", _)                  => { self.require_env_effect(span)?; Some(Type::Unit) }
```

### test block への `!Env` 追加

`check_test_def` の `current_effects` に `Effect::Unknown("Env".to_string())` を追加。

---

## Phase 4: compiler.rs 変更（`fav/src/middle/compiler.rs`）

2 箇所の namespace リストに `"Env"` を追加。

---

## Phase 5: driver.rs 変更

### `cmd_run` での `EnvConfig` 適用

```rust
// dotenv の自動ロード（fav.toml の設定より）
if let Some(ref env_cfg) = toml.env {
    if let Some(ref dotenv_path) = env_cfg.dotenv {
        let path = root.join(dotenv_path);
        if path.exists() {
            // parse_dotenv_file(path) で読み込み・set_var
        }
    }
    set_env_config(vm::EnvConfig {
        dotenv: env_cfg.dotenv.clone(),
        prefix: env_cfg.prefix.clone(),
    });
} else {
    set_env_config(vm::EnvConfig::default());
}
```

`dotenv` の自動ロードは driver.rs 側で（VM プリミティブを呼ばず直接 Rust でロード）。

---

## Phase 6: rune ファイル作成

### `runes/env/access.fav`

```favnir
// get: 環境変数を取得。未設定なら default を返す。
public fn get(key: String, default: String) -> String !Env {
    match Env.get_raw(key) {
        Some(v) => v
        None    => default
    }
}

// get_opt: Option<String> を返す
public fn get_opt(key: String) -> Option<String> !Env {
    Env.get_raw(key)
}

// require: 必須変数。未設定なら Err を返す。
public fn require(key: String) -> Result<String, String> !Env {
    Env.require_raw(key)
}
```

### `runes/env/typed.fav`

```favnir
public fn get_int(key: String, default: Int) -> Int !Env {
    match Env.get_int_raw(key) {
        Ok(v)  => v
        Err(_) => default
    }
}

public fn require_int(key: String) -> Result<Int, String> !Env {
    Env.get_int_raw(key)
}

public fn get_bool(key: String, default: Bool) -> Bool !Env {
    match Env.get_bool_raw(key) {
        Ok(v)  => v
        Err(_) => default
    }
}

public fn require_bool(key: String) -> Result<Bool, String> !Env {
    Env.get_bool_raw(key)
}
```

### `runes/env/dotenv.fav`

```favnir
public fn load_dotenv(path: String) -> Result<Unit, String> !Env {
    Env.load_dotenv_raw(path)
}

public fn load_dotenv_or_ignore(path: String) -> Unit !Env {
    match Env.load_dotenv_raw(path) {
        Ok(_)  => ()
        Err(_) => ()
    }
}
```

### `runes/env/env.fav`（barrel）

```favnir
use access.{ get, get_opt, require }
use typed.{ get_int, require_int, get_bool, require_bool }
use dotenv.{ load_dotenv, load_dotenv_or_ignore }
```

### `runes/env/env.test.fav`（16 件）

テスト内では `std::env::set_var` 相当の操作は Rust 側テストで行う。Favnir テストは既知の環境変数（`PATH` など）または `Env.load_dotenv_raw` 経由でセットした変数を使う。

```favnir
import rune "env"

test "get missing key returns default" {
    assert(env.get("__FAV_TEST_MISSING_47__", "default_val") == "default_val")
}

test "get_opt missing key returns None" {
    assert(Option.is_none(env.get_opt("__FAV_TEST_MISSING_47B__")))
}

test "require missing key returns Err" {
    match env.require("__FAV_TEST_MISSING_47C__") {
        Ok(_)  => assert(false)
        Err(e) => assert(String.contains(e, "ENV_MISSING"))
    }
}

// ... etc.
```

---

## Phase 7: テスト追加

### vm_stdlib_tests.rs（8 件）

各テストで `std::env::set_var` を使って環境変数を設定してから VM プリミティブを呼ぶ。テスト間の干渉を防ぐためユニークなキー名を使う（`FAV_TEST_ENV_47_*`）。

### driver.rs 統合テスト（5 件）

`exec_project_main_source_with_runes` を使い、`-> Bool` か `-> Unit !Env` で動作確認。

---

## Phase 8: examples 追加

### `examples/env_demo/fav.toml`

```toml
[package]
name = "env_demo"
version = "0.1.0"

[runes]
path = "../../runes"

[env]
dotenv = ".env"
prefix = ""
```

### `examples/env_demo/.env`

```
DATABASE_URL=postgres://localhost/demo
PORT=5432
DEBUG=true
SERVICE_NAME=env-demo
```

### `examples/env_demo/src/main.fav`

```favnir
import rune "env"
import rune "log"

public fn main() -> Unit !Env !Io {
    env.load_dotenv_or_ignore(".env")
    log.info("I000", String.concat("Service: ", env.get("SERVICE_NAME", "unknown")))
    match env.require("DATABASE_URL") {
        Err(e) => log.error("LE010", e)
        Ok(url) => log.info("I010", String.concat("DB: ", url))
    }
    log.info("I010", String.concat("Port: ", Int.to_string(env.get_int("PORT", 8080))))
    log.info("I010", String.concat("Debug: ", Bool.to_string(env.get_bool("DEBUG", false))))
    log.info("I001", "env_demo finished")
}
```

---

## 注意点・落とし穴

1. **`env` 変数名の衝突**: toml.rs で `env` という変数名を使うと `std::env` モジュールと衝突する可能性。内部変数名は `env_cfg` を使う。
2. **`let mut current: EnvConfig =`**: 型注釈必須（`log` の `LogConfig` と同様）。
3. **テスト並列実行**: `std::env::set_var` はプロセスグローバルなので並列テストで干渉する。ユニークなキー名を使うこと。
4. **`Option<String>` 型の VM 表現**: `Env.get_raw` は `some_vm` / `none_vm` ヘルパーを使う（既存パターン確認必要）。
5. **`!Env` エフェクト名**: `Effect::Unknown("Env".to_string())` — 文字列は `"Env"` （大文字始まり）。
6. **`access` はキーワードか確認**: `access.fav` のモジュール名 `access` がキーワードでないか lexer.rs で確認すること（`emit` の前例から）。
7. **`Int.to_string` / `Bool.to_string`**: 存在するか確認。存在しなければ文字列変換は別の方法を探す。
