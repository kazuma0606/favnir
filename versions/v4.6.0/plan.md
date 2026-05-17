# Favnir v4.6.0 実装計画 — Log Rune

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.6.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.6.0` に更新

新規 Cargo 依存はなし。タイムスタンプは Rust 標準ライブラリの `std::time::SystemTime` で生成する。
JSON フォーマットは `serde_json`（既存依存）を使用する。

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: `LogConfig` thread_local と設定関数

```rust
// vm.rs に追加
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,    // "debug" | "info" | "warn" | "error"
    pub format: String,   // "json" | "text"
    pub output: String,   // "stdout" | "stderr"
    pub service: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            format: "text".to_string(),
            output: "stdout".to_string(),
            service: String::new(),
        }
    }
}

thread_local! {
    static LOG_CONFIG: RefCell<LogConfig> = RefCell::new(LogConfig::default());
}

pub fn set_log_config(cfg: LogConfig) {
    LOG_CONFIG.with(|c| *c.borrow_mut() = cfg);
}
```

### 1-B: ログレベルのフィルタ判定ヘルパー

```rust
fn log_level_passes(emit_level: &str) -> bool {
    LOG_CONFIG.with(|c| {
        let cfg = c.borrow();
        match cfg.level.as_str() {
            "error" => emit_level == "ERROR",
            "warn"  => matches!(emit_level, "ERROR" | "WARN"),
            "info"  => matches!(emit_level, "ERROR" | "WARN" | "INFO" | "SUCCESS"),
            _       => true, // "debug" — 全て通過
        }
    })
}
```

### 1-C: `Log.emit_raw` 実装

```rust
"Log.emit_raw" => {
    // args: (level: String, code: String, message: String, ctx_json: String)
    let level   = vm_str(&args[0])?;
    let code    = vm_str(&args[1])?;
    let message = vm_str(&args[2])?;
    let ctx_json= vm_str(&args[3])?;

    if !log_level_passes(&level) {
        return Ok(VMValue::Unit);
    }

    LOG_CONFIG.with(|c| {
        let cfg = c.borrow();
        let line = if cfg.format == "json" {
            log_format_json(&level, &code, &message, &ctx_json, &cfg.service)
        } else {
            log_format_text(&level, &code, &message, &ctx_json)
        };
        if cfg.output == "stderr" {
            eprintln!("{}", line);
        } else {
            println!("{}", line);
        }
    });
    Ok(VMValue::Unit)
}
```

**`log_format_text` の出力形式:**
```
[2026-05-17 10:30:00] SUCCESS S010  Pipeline completed  rows=1500
```
- 時刻: `%Y-%m-%d %H:%M:%S`（UTC）
- レベル: 7文字で右パディング
- コード: 5文字で右パディング
- コンテキスト: `ctx_json` の各キーを `key=value  ` 形式でスペース区切り（空 `{}` のとき省略）

**`log_format_json` の出力形式:**
```json
{"ts":"2026-05-17T10:30:00Z","level":"SUCCESS","code":"S010","msg":"Pipeline completed","service":"my-pipeline","ctx":{"rows":"1500"}}
```
- `serde_json` を使い Map を直接構築して `to_string()`
- `service` が空文字のときは `"service"` フィールドを省略

### 1-D: `Log.metric_raw` 実装

```rust
"Log.metric_raw" => {
    // args: (name: String, value: Int, unit: String)
    let name  = vm_str(&args[0])?;
    let value = vm_int(&args[1])?;
    let unit  = vm_str(&args[2])?;

    LOG_CONFIG.with(|c| {
        let cfg = c.borrow();
        let line = if cfg.format == "json" {
            log_metric_emf(&name, value, &unit)
        } else {
            format!("[{}] METRIC  {}={} {}",
                log_timestamp_text(), name, value, unit)
        };
        if cfg.output == "stderr" {
            eprintln!("{}", line);
        } else {
            println!("{}", line);
        }
    });
    Ok(VMValue::Unit)
}
```

**CloudWatch EMF 形式（json フォーマット時）:**
```json
{"_aws":{"Timestamp":1747471801000,"CloudWatchMetrics":[{"Namespace":"favnir","Dimensions":[[]],"Metrics":[{"Name":"processed_rows","Unit":"Count"}]}]},"processed_rows":1500}
```

### 1-E: タイムスタンプ生成ヘルパー

```rust
fn log_timestamp_text() -> String {
    // std::time::SystemTime を使って "2026-05-17 10:30:00" 形式
}

fn log_timestamp_iso() -> String {
    // "2026-05-17T10:30:00Z" 形式
}

fn log_timestamp_millis() -> u128 {
    // Unix epoch ミリ秒
}
```

`chrono` クレートは追加しない。
`SystemTime::now().duration_since(UNIX_EPOCH)` から計算する（年月日の分解は手動実装）。

### 1-F: ctx_json パースとテキスト展開ヘルパー

```rust
fn log_ctx_to_text(ctx_json: &str) -> String {
    // serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ctx_json) でパース
    // "key1=val1  key2=val2" 形式の文字列を返す
    // パース失敗時は ctx_json をそのまま返す
}
```

---

## Phase 2: `fav.toml` 拡張（`fav/src/toml.rs`）

### 2-A: `LogConfig` 構造体の追加

```rust
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,    // "debug" | "info" | "warn" | "error"; デフォルト "info"
    pub format: String,   // "json" | "text"; デフォルト "text"
    pub output: String,   // "stdout" | "stderr"; デフォルト "stdout"
    pub service: String,  // デフォルト ""
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            format: "text".to_string(),
            output: "stdout".to_string(),
            service: String::new(),
        }
    }
}
```

`FavToml` に `pub log: Option<LogConfig>` を追加する。
既存の `FavToml` リテラル初期化箇所（checker.rs ×2、resolver.rs ×2、driver.rs ×1）に
`log: None` を追加する。

### 2-B: `[log]` セクションのパース追加

`parse_fav_toml` 関数内で以下を追加:

```rust
"[log]" => current_section = "log",
// ...
"log" => match key {
    "level"   => log_config.level   = value.to_string(),
    "format"  => log_config.format  = value.to_string(),
    "output"  => log_config.output  = value.to_string(),
    "service" => log_config.service = value.to_string(),
    _ => {}
},
```

### 2-C: `cmd_run` での設定反映

`driver.rs` の `cmd_run` に `set_log_config` の呼び出しを追加:

```rust
if let Some(lc) = toml.log.as_ref() {
    crate::backend::vm::set_log_config(crate::backend::vm::LogConfig {
        level:   lc.level.clone(),
        format:  lc.format.clone(),
        output:  lc.output.clone(),
        service: lc.service.clone(),
    });
}
```

---

## Phase 3: checker.rs への変更（`fav/src/middle/checker.rs`）

### 3-A: `("Log", *)` アームの追加

`check_builtin_apply` の `("Crypto", _)` / `("Auth", _)` アームの近くに追加:

```rust
// Log.* (v4.6.0)
("Log", "emit_raw") => Some(Type::Unit),
("Log", "metric_raw") => Some(Type::Unit),
("Log", _) => Some(Type::Unit),
```

`require_auth_effect` は不要 — Log プリミティブにエフェクト制約なし。

### 3-B: `"Log"` namespace の登録

`compiler.rs` の global loop に `"Log"` を追加する:
```rust
// 既存の "Crypto", "Auth" と同様
"Log" => { /* グローバル namespace として登録 */ }
```

---

## Phase 4: `logs/*.yaml` ロード（`fav/src/driver.rs`）

### 4-A: `logs/*.yaml` の読み込み

`cmd_run` で `fav.toml` がある場合、プロジェクトルートの `logs/` ディレクトリを走査する。

```rust
fn load_log_codes(root: &Path) -> HashMap<String, String> {
    // root/logs/*.yaml を全て読み込み
    // キー: "S100", 値: message String
    // serde_yaml（既存依存）でパース
}
```

v4.6.0 では **ロードして HashMap に格納するだけ**（コード未定義の警告等は未実装）。
将来的に `fav check` でコード文字列リテラルの検証に使用する。

### 4-B: thread_local への格納

```rust
thread_local! {
    static LOG_CODES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub fn set_log_codes(codes: HashMap<String, String>) { ... }
```

---

## Phase 5: VM プリミティブのエラー自動ログ出力

v4.6.0 では対象を絞り込んで実装する。

### 5-A: 対象プリミティブとコード

| プリミティブ群 | ログコード | 対象 |
|-------------|-----------|------|
| `DB.*` | `LE010` | `DB.query_raw`, `DB.execute_raw` |
| `Http.*` | `LE020` | `Http.get_raw`, `Http.post_raw` |
| `Crypto.*` | `LE030` | `Crypto.jwt_verify_raw`（verify 失敗時のみ） |
| `Grpc.*` | `LE040` | `Grpc.call_raw` |

### 5-B: 実装パターン

`err_vm(...)` を返す直前に `log_auto_emit` を呼ぶ:

```rust
fn log_auto_emit(level: &str, code: &str, message: &str) {
    if !log_level_passes(level) { return; }
    // LOG_CONFIG を参照して出力する（log_format_text / log_format_json を流用）
}
```

既存の各プリミティブのエラーパスに1行追加する（侵略的変更は最小限）:

```rust
// 例: DB.query_raw のエラーパス
Err(e) => {
    log_auto_emit("ERROR", "LE010", &format!("DB error: {}", e));
    Ok(err_vm(...))
}
```

---

## Phase 6: rune ファイル作成（`runes/log/`）

### 6-A: `runes/log/codes.fav`

```favnir
// codes.fav — 組み込みログコード定数 (v4.6.0)

public fn app_started()   -> String { "I000" }
public fn app_stopped()   -> String { "I001" }
public fn processing()    -> String { "I010" }
public fn completed()     -> String { "S000" }
public fn pipeline_done() -> String { "S010" }
public fn retrying()      -> String { "W001" }
public fn slow_op()       -> String { "W002" }
public fn db_error()      -> String { "LE010" }
public fn net_error()     -> String { "LE020" }
public fn auth_error()    -> String { "LE030" }
public fn rpc_error()     -> String { "LE040" }
public fn aws_error()     -> String { "LE050" }
```

### 6-B: `runes/log/emit.fav`

```favnir
// emit.fav — ログ出力関数 (v4.6.0)

public fn info(code: String, message: String) -> Unit !Io {
    Log.emit_raw("INFO", code, message, "{}")
}

public fn success(code: String, message: String) -> Unit !Io {
    Log.emit_raw("SUCCESS", code, message, "{}")
}

public fn warn(code: String, message: String) -> Unit !Io {
    Log.emit_raw("WARN", code, message, "{}")
}

public fn error(code: String, message: String) -> Unit !Io {
    Log.emit_raw("ERROR", code, message, "{}")
}

fn map_to_json(ctx: Map<String, String>) -> String {
    // Map<String, String> を JSON 文字列に変換するヘルパー
    // 実装: Map.keys でキー一覧を取り出してキー="値" をカンマ結合
    // 簡易実装: "{" + entries + "}" 形式
    // NOTE: キー・値のエスケープは行わない（ログ用途なので十分）
    String.concat("{", String.concat(
        List.join(Map.entries(ctx), ","),
    "}"))
}

public fn info_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io {
    Log.emit_raw("INFO", code, message, map_to_json(ctx))
}

public fn success_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io {
    Log.emit_raw("SUCCESS", code, message, map_to_json(ctx))
}

public fn warn_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io {
    Log.emit_raw("WARN", code, message, map_to_json(ctx))
}

public fn error_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io {
    Log.emit_raw("ERROR", code, message, map_to_json(ctx))
}
```

> **注意**: `map_to_json` は Map を単純な JSON 文字列に変換する。
> `Map.entries` や `List.join` が存在しない場合は VM primitive で対応する
> （`Log.map_to_json_raw(ctx)` として追加し、rune 側から呼ぶ）。

### 6-C: `runes/log/metric.fav`

```favnir
// metric.fav — メトリクス出力 (v4.6.0)

public fn metric(name: String, value: Int) -> Unit !Io {
    Log.metric_raw(name, value, "Count")
}

public fn metric_with_unit(name: String, value: Int, unit: String) -> Unit !Io {
    Log.metric_raw(name, value, unit)
}
```

### 6-D: `runes/log/log.fav`（barrel file）

```favnir
// log.fav — Log Rune public API (v4.6.0)
use emit.{ info, success, warn, error, info_ctx, success_ctx, warn_ctx, error_ctx }
use metric.{ metric, metric_with_unit }
use codes.*
```

### 6-E: `runes/log/log.test.fav`

14 件のテストを実装:
1. `"info emits without crash"` — `log.info` が例外なく動く
2. `"success emits without crash"` — 同上
3. `"warn emits without crash"` — 同上
4. `"error emits without crash"` — 同上
5. `"info_ctx emits without crash"` — `log.info_ctx` が動く
6. `"error_ctx with map context"` — コンテキストあり
7. `"metric emits without crash"` — `log.metric` が動く
8. `"metric_with_unit emits without crash"` — 単位付きメトリクス
9. `"app_started returns I000"` — コード定数確認
10. `"app_stopped returns I001"` — 同上
11. `"pipeline_done returns S010"` — 同上
12. `"db_error returns LE010"` — 同上
13. `"net_error returns LE020"` — 同上
14. `"multiple log calls in sequence"` — 複数呼び出しが順番通り動く

---

## Phase 7: テスト追加

### 7-A: `fav/src/backend/vm_stdlib_tests.rs`（8 件）

```rust
fn log_emit_text_format_runs()          // Log.emit_raw が Unit を返す（クラッシュしない）
fn log_emit_json_format_runs()          // json フォーマット設定で Unit を返す
fn log_emit_level_filter_suppresses()   // level="error" で INFO が抑制される
fn log_emit_level_filter_passes()       // level="error" で ERROR が通過する
fn log_metric_runs()                    // Log.metric_raw が Unit を返す
fn log_metric_json_format_runs()        // json フォーマットで EMF 形式
fn log_emit_ctx_json_included()         // ctx_json あり（クラッシュしない）
fn log_emit_service_field()             // service 名設定（クラッシュしない）
```

vm_stdlib_tests は **出力内容ではなくクラッシュしないこと**を主に確認する
（stdout の内容キャプチャは複雑なため回避）。
`set_log_config` で設定を切り替えて、Unit が返ることで動作確認する。

### 7-B: `fav/src/driver.rs` 統合テスト（5 件）

```rust
fn log_rune_test_file_passes()          // log.test.fav 全件 pass
fn log_info_in_favnir_source()          // Favnir ソースで log.info が Unit を返す
fn log_error_ctx_in_favnir_source()     // log.error_ctx が Unit を返す
fn log_metric_in_favnir_source()        // log.metric が Unit を返す
fn log_config_format_json_runs()        // json フォーマット設定で動く
```

---

## Phase 8: examples 追加（`examples/log_demo/`）

```
examples/log_demo/
  fav.toml      ← [log] format = "text", service = "log-demo"
  src/
    main.fav    ← 各ログレベル・メトリクス・コード定数のデモ
```

`main.fav` のデモ内容:
1. `log.info(log.app_started(), "Log Demo started")`
2. ループ処理のシミュレーション（`log.success_ctx` でカウント表示）
3. エラーパスのシミュレーション（`log.error_ctx`）
4. `log.metric("demo_rows", 1000)`
5. `log.metric_with_unit("demo_duration_ms", 150, "Milliseconds")`
6. `log.info(log.app_stopped(), "Log Demo finished")`

---

## 実装順序と依存関係

```
Phase 0 (バージョン更新)
  ↓
Phase 1 (VM プリミティブ: 1-A〜1-F)  ← コア。1-C / 1-D が最優先
  ↓              ↓
Phase 2        Phase 3                ← 並列可
(toml.rs)      (checker.rs)
  ↓
Phase 4 (logs/*.yaml ロード)
  ↓
Phase 5 (自動エラーログ)
  ↓
Phase 6 (rune ファイル)
  ↓
Phase 7 (テスト)
  ↓
Phase 8 (examples)
```

Phase 1 の中では `1-A`（LogConfig）→ `1-B`（level filter）→ `1-C`（emit_raw）→ `1-D`（metric_raw）の順。
`1-E` / `1-F`（タイムスタンプ・ctx ヘルパー）は `1-C` の実装に必要なので先に書く。

---

## 注意事項・実装のポイント

### `map_to_json` の実装

`emit.fav` の `map_to_json` では `Map.entries` / `List.join` 等の VM primitive が必要。
これらが存在しない場合は `Log.map_to_json_raw(ctx: Map<String, String>) -> String` を
VM primitive として追加し、rune から呼ぶ。

実装:
```rust
"Log.map_to_json_raw" => {
    // args: (ctx: VMValue::Record)
    // {"key":"val","key2":"val2"} 形式の JSON 文字列を返す
}
```

### chrono 不使用

`chrono` を追加せず `std::time::SystemTime` で実装する。
テキスト形式の年月日時分秒は `duration_since(UNIX_EPOCH)` から計算する:

```rust
fn log_timestamp_text() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Unix epoch 秒から年月日を計算（proleptic Gregorian calendar 手動実装）
    // 精度: 秒まで（ミリ秒不要）
}
```

簡易実装として `seconds_to_ymd` ヘルパーを追加する（うるう年・うるう秒に対応）。

### `FavToml` リテラル初期化への `log: None` 追加

v4.5.0 で `auth: None` を追加した箇所と同様に、以下すべてに `log: None` を追加:
- `checker.rs` 内の `FavToml` リテラル（2箇所）
- `resolver.rs` 内の `FavToml` リテラル（2箇所）
- `driver.rs` 内の `FavToml` リテラル（1箇所）

### リスクと対策

| リスク | 対策 |
|-------|------|
| `log_timestamp_text` の手動実装でバグ | テストで既知のフォーマットパターンを確認 |
| `map_to_json` で `Map.entries`/`List.join` が未実装 | `Log.map_to_json_raw` VM primitive で代替 |
| 既存テストが stdout に出力されてテスト出力が乱れる | `set_log_config(level="error")` でサイレント化する |
| Phase 5（自動エラーログ）で既存テストに副作用 | テスト用に `LOG_CONFIG.level = "error"` を設定してフィルタ |
