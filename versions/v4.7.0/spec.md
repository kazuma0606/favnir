# Favnir v4.7.0 仕様書 — Env Rune（環境変数管理）

作成日: 2026-05-17

---

## 概要

データエンジニアリングパイプラインにおいて、データベース接続文字列・API キー・AWS 認証情報などを安全に管理するための環境変数アクセス機能を追加する。

**主な追加機能:**
- `Env.*` VM プリミティブ（get / require / 型変換 / `.env` ロード）
- `!Env` エフェクト（環境変数アクセスの明示的な宣言）
- `runes/env/` Rune（高レベル API）
- `schemas/env.yaml` による必須変数の定義・バリデーション
- `fav.toml [env]` セクション（dotenv パス・プレフィックス設定）

---

## 動機

データパイプラインの本番運用では以下のパターンが頻発する:

```
# Bad: ハードコード
DB.connect("postgres://user:password@localhost/mydb")

# Good: 環境変数
DB.connect(env.require("DATABASE_URL"))
```

v4.7.0 では Favnir で環境変数を型安全に扱うための基盤を整備する。

---

## エフェクトシステムへの統合

### `!Env` エフェクト

環境変数を読み取る関数は `!Env` エフェクトを宣言する。これにより、どの関数が外部設定に依存するかがシグネチャから明確になる。

```favnir
import rune "env"

public fn get_db_url() -> Result<String, String> !Env {
    env.require("DATABASE_URL")
}
```

実装: `Effect::Unknown("Env".to_string())`（`!Auth` と同パターン）。`Effect` enum への新バリアント追加は不要。

### checker.rs での扱い

- `check_builtin_apply` の `("Env", _)` アームは `require_env_effect` を呼ぶ（E0312）
- `check_test_def` の `current_effects` に `Effect::Unknown("Env")` を追加

---

## `Env.*` VM プリミティブ

### `Env.get_raw(key: String) -> Option<String>`

環境変数を読む。存在しない場合は `None`。

```favnir
match Env.get_raw("PORT") {
    Some(p) => p
    None    => "8080"
}
```

VM 表現: `VMValue::Record { name: "Some"/"None", ... }`（既存 Option パターン）

### `Env.require_raw(key: String) -> Result<String, String>`

必須環境変数を読む。未設定の場合は `Err("ENV_MISSING: KEY")` を返す。

```favnir
match Env.require_raw("DATABASE_URL") {
    Ok(url)  => DB.connect(url)
    Err(msg) => Result.err(msg)
}
```

### `Env.get_int_raw(key: String) -> Result<Int, String>`

環境変数を `Int` として読む。未設定 → `Err("ENV_MISSING: KEY")`。パース失敗 → `Err("ENV_PARSE_INT: KEY=val")`。

### `Env.get_bool_raw(key: String) -> Result<Bool, String>`

環境変数を `Bool` として読む（`"true"/"1"/"yes"` → `true`、`"false"/"0"/"no"` → `false`）。

### `Env.load_dotenv_raw(path: String) -> Result<Unit, String>`

`.env` ファイルを読み込み、プロセスの環境変数に設定する。既に設定済みの変数は上書きしない（ `--no-override` 方式）。

`.env` フォーマット:
```
# コメント
DATABASE_URL=postgres://localhost/mydb
PORT=5432
DEBUG=true
```

### `Env.all_raw() -> Map<String, String>`

現在のプロセス環境変数を全て取得する（デバッグ用）。

---

## `runes/env/` Rune 構成

```
runes/env/
  env.fav        — barrel (use access.*, use dotenv.*, use typed.*)
  access.fav     — get, require, get_or
  typed.fav      — get_int, get_bool, require_int, require_bool
  dotenv.fav     — load_dotenv, load_dotenv_or_ignore
  env.test.fav   — 16 件のテスト
```

### `access.fav`

```favnir
// env.require(key) → Result<String, String> !Env
public fn require(key: String) -> Result<String, String> !Env {
    Env.require_raw(key)
}

// env.get(key, default) → String !Env
public fn get(key: String, default: String) -> String !Env {
    match Env.get_raw(key) {
        Some(v) => v
        None    => default
    }
}

// env.get_opt(key) → Option<String> !Env
public fn get_opt(key: String) -> Option<String> !Env {
    Env.get_raw(key)
}
```

### `typed.fav`

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

### `dotenv.fav`

```favnir
public fn load_dotenv(path: String) -> Result<Unit, String> !Env {
    Env.load_dotenv_raw(path)
}

public fn load_dotenv_or_ignore(path: String) -> Unit !Env {
    match Env.load_dotenv_raw(path) {
        Ok(_)    => ()
        Err(_)   => ()
    }
}
```

---

## `schemas/env.yaml` — 環境変数スキーマ定義

`fav build --env-schema` で定義済み変数の一覧をドキュメント化できる（オプション機能）。

```yaml
# schemas/env.yaml
DATABASE_URL:
  type: string
  required: true
  description: "PostgreSQL connection string"
PORT:
  type: int
  required: false
  default: "8080"
  description: "HTTP server port"
DEBUG:
  type: bool
  required: false
  default: "false"
  description: "Enable debug logging"
AWS_REGION:
  type: string
  required: false
  default: "ap-northeast-1"
  description: "AWS region"
```

### コンパイル時バリデーション（オプション）

`fav build --check-env` は `schemas/env.yaml` の `required: true` 変数が現在の環境に存在するか確認する。CI/CD パイプラインでの使用を想定。

---

## `fav.toml [env]` セクション

```toml
[env]
dotenv = ".env"           # 起動時に自動ロードする .env ファイルのパス（オプション）
prefix = ""               # 全キーに付けるプレフィックス（例: "APP_"）
```

- `dotenv` が設定されている場合、`cmd_run` 起動前に自動ロード
- `prefix` が設定されている場合、`Env.get_raw("PORT")` は実際には `"APP_PORT"` を参照

---

## エラーコード体系

| コード   | 内容                                      |
|---------|-------------------------------------------|
| E0312   | `!Env` エフェクトが宣言されていない（require_env_effect） |
| EE010   | 必須環境変数が未設定（runtime `Env.require_raw`）      |
| EE020   | 環境変数の型変換失敗（runtime `Env.get_int_raw` 等）   |

---

## テスト方針

### vm_stdlib_tests.rs（8 件）

- `env_get_raw_returns_some` — `std::env::set_var` で設定後 `Env.get_raw` が `Some` を返す
- `env_get_raw_returns_none` — 未設定キーで `None` を返す
- `env_require_raw_ok` — 設定済みキーで `Ok(val)` を返す
- `env_require_raw_err` — 未設定キーで `Err(...)` を返す
- `env_get_int_raw_ok` — 整数文字列を `Ok(Int)` に変換
- `env_get_int_raw_parse_err` — 非整数で `Err(...)` を返す
- `env_get_bool_raw_true` — `"true"/"1"/"yes"` が `Ok(true)`
- `env_load_dotenv_raw_ok` — テスト用 `.env` ファイルを読み込み変数がセットされる

### driver.rs 統合テスト（5 件）

- `env_get_in_favnir_source` — Favnir ソースで `env.get("KEY", "default")` が動く
- `env_require_in_favnir_source` — `env.require` が Ok を返す
- `env_get_int_in_favnir_source` — `env.get_int` が Int を返す
- `env_get_bool_in_favnir_source` — `env.get_bool` が Bool を返す
- `env_rune_test_file_passes` — `log.test.fav` 全件 pass

### rune テスト（`env.test.fav`、16 件）

- `get_existing_key_returns_value`
- `get_missing_key_returns_default`
- `get_opt_existing_returns_some`
- `get_opt_missing_returns_none`
- `require_existing_returns_ok`
- `require_missing_returns_err`
- `get_int_valid_returns_default_on_missing`
- `get_int_valid_parse`
- `require_int_valid`
- `require_int_missing_returns_err`
- `get_bool_true_values`
- `get_bool_false_values`
- `get_bool_missing_returns_default`
- `require_bool_valid`
- `load_dotenv_or_ignore_no_file`
- `multiple_env_calls_in_sequence`

---

## 利用シナリオ

### データパイプライン設定

```favnir
import rune "env"
import rune "db"
import rune "log"

public fn run_pipeline() -> Result<Unit, String> !Env !Db !Io {
    match env.require("DATABASE_URL") {
        Err(e) => Result.err(e)
        Ok(url) =>
            match DB.connect(url) {
                Err(e) => Result.err(e)
                Ok(conn) => {
                    log.info("I010", "Pipeline started")
                    // ... pipeline logic ...
                    Result.ok(())
                }
            }
    }
}
```

### 環境別設定

```favnir
import rune "env"

public fn main() -> Unit !Env !Io {
    env.load_dotenv_or_ignore(".env")
    // ...
}
```

---

## 既知の制約

- `!Env` エフェクトは `Effect::Unknown("Env")` として実装（`Effect` enum 変更なし）
- `Env.all_raw()` はデバッグ用途のみ（機密情報漏洩リスクあり）
- `.env` ファイルは既存変数を上書きしない（OS 環境変数が優先）
- `fav.toml [env] prefix` の適用は VM 側で透過的に行う（Favnir コードから意識しない）
