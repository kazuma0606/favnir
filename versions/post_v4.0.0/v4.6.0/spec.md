# Favnir v4.6.0 仕様書 — Log Rune（構造化ログ + メトリクス）

作成日: 2026-05-17

## 概要

Favnir プログラムに構造化ログとメトリクス出力を統合する。
`log.info` / `log.success` / `log.warn` / `log.error` でアプリケーションイベントを記録し、
`log.metric` で CloudWatch EMF 形式のメトリクスを出力する。
ローカルでは人間が読みやすいテキスト形式、本番（CloudWatch / Grafana Loki）では JSON 形式に切り替える。
`!Log` という独立エフェクトは持たない — ログ出力は `!Io` の一部として扱う。

---

## 1. 設計方針

### 1.1 `!Log` エフェクトを持たない理由

```
純粋関数（エフェクトなし）   → ログなし（副作用ゼロを保証）
!Io / !Db / !Network / ...  → VM primitive 層がエラー時に自動で LE コードを出力
アプリケーションイベント     → log.info / log.success / log.warn / log.error で明示的に出力
```

`log.*` 関数は `!Io` エフェクトを要求する。専用の `!Log` エフェクトを追加しないことで、
既存の `Effect` enum を変更せず（exhaustive match の修正ゼロ）、後方互換性を完全に保つ。

### 1.2 ログ出力レベル

| レベル | Favnir API | 用途 |
|--------|-----------|------|
| INFO | `log.info` | 処理開始・進行状況 |
| SUCCESS | `log.success` | 正常完了 |
| WARN | `log.warn` | リトライ・スロー操作 |
| ERROR | `log.error` | エラー発生（処理継続） |

### 1.3 ログコード体系

コンパイラエラーコード（`E0001`〜、4桁）と区別するため**3桁**を使用する。

| プレフィクス | 区分 | アプリ定義範囲 |
|------------|------|--------------|
| `I` | INFO | `I100`〜 |
| `S` | SUCCESS | `S100`〜 |
| `W` | WARN | `W100`〜 |
| `LE` | LOG ERROR | `LE100`〜 |

**組み込みコード（予約済み、`logs/*.yaml` 不要）**
```
I000  Application started        LE010 DB error
I001  Application stopped        LE020 Network error
I010  Processing started         LE030 Auth error
S000  Operation completed        LE040 RPC error
S010  Pipeline completed         LE050 AWS error
W001  Retry attempted
W002  Slow operation detected
```

アプリ定義コードは `I100`〜 / `S100`〜 / `W100`〜 / `LE100`〜 を使い、組み込みと衝突しない。

---

## 2. `fav.toml` 拡張

```toml
[log]
level   = "info"        # debug | info | warn | error
format  = "json"        # json | text
output  = "stdout"      # stdout | stderr
service = "my-pipeline"
```

| キー | デフォルト | 説明 |
|-----|-----------|------|
| `level` | `"info"` | このレベル以上を出力。debug < info < warn < error |
| `format` | `"text"` | `"text"`（ローカル）/ `"json"`（CloudWatch / Loki） |
| `output` | `"stdout"` | `"stdout"` / `"stderr"` |
| `service` | `""` | JSON フォーマット時に `"service"` フィールドとして出力 |

```rust
pub struct LogConfig {
    pub level: String,    // "debug" | "info" | "warn" | "error"
    pub format: String,   // "json" | "text"
    pub output: String,   // "stdout" | "stderr"
    pub service: String,  // JSON の "service" フィールド
}
```

`FavToml` に `pub log: Option<LogConfig>` を追加する。

---

## 3. VM プリミティブ（`Log.*`）

### 3-A: `Log.emit_raw`

```
Log.emit_raw(level, code, message, ctx_json) -> Unit
```

- `level`: `"INFO"` / `"SUCCESS"` / `"WARN"` / `"ERROR"`
- `code`: ログコード文字列（`"I000"`, `"LE010"` 等）
- `message`: 人間が読めるメッセージ
- `ctx_json`: `"{\"key\":\"val\"}"` 形式の JSON 文字列（`"{}"` も可）

**text フォーマット出力例:**
```
[2026-05-17 10:30:00] SUCCESS S100  Pipeline completed  inserted=1500
```

**json フォーマット出力例（CloudWatch Logs / Grafana Loki 対応）:**
```json
{"ts":"2026-05-17T10:30:00Z","level":"SUCCESS","code":"S100","msg":"Pipeline completed","service":"my-pipeline","ctx":{"inserted":"1500"}}
```

レベルフィルタリング: `log.level` 設定に基づき、閾値以下のログは無視する。
- `"error"` 設定時: ERROR のみ出力
- `"warn"` 設定時: WARN + ERROR を出力
- `"info"` 設定時: INFO + SUCCESS + WARN + ERROR を出力（デフォルト）
- `"debug"` 設定時: 全て出力

### 3-B: `Log.metric_raw`

```
Log.metric_raw(name, value, unit) -> Unit
```

- `name`: メトリクス名（例: `"processed_rows"`）
- `value`: 値（Int）
- `unit`: 単位文字列（`"Count"` / `"Milliseconds"` / `"Bytes"` 等、CloudWatch EMF 準拠）

**json フォーマット時（CloudWatch EMF 形式）:**
```json
{"_aws":{"Timestamp":1747471800000,"CloudWatchMetrics":[{"Namespace":"favnir","Dimensions":[[]],"Metrics":[{"Name":"processed_rows","Unit":"Count"}]}]},"processed_rows":1500}
```

**text フォーマット時:**
```
[2026-05-17 10:30:01] METRIC  processed_rows=1500 Count
```

`format = "json"` かつ出力先が CloudWatch Logs の場合、EMF として自動抽出される。
ローカル開発（`format = "text"`）では単純なテキスト行として表示される。

---

## 4. `logs/*.yaml` — カスタムログコード定義

```yaml
# logs/success.yaml
S100:
  message: "Pipeline completed"
  tags: [pipeline]

S101:
  message: "Batch inserted"
  tags: [pipeline, db]
```

```yaml
# logs/error.yaml
LE100:
  message: "External API unreachable"
  severity: critical
  tags: [network]

LE101:
  message: "Schema validation failed"
  severity: error
  tags: [validation]
```

- `logs/` ディレクトリはプロジェクトルートに配置する（`fav.toml` と同階層）
- `fav run` / `fav test` 起動時に自動ロードする
- 将来的に: `fav check` でコード定義のない文字列リテラルを警告（v4.6.0 では未実装）

---

## 5. rune ファイル構成

```
runes/log/
  log.fav        ← public API（barrel file）
  emit.fav       ← info / success / warn / error（+ ctx バリアント）
  metric.fav     ← metric
  codes.fav      ← 組み込みコード定数
  log.test.fav   ← テスト
```

---

## 6. rune API 仕様

### 6-A: `emit.fav`

```favnir
// 基本ログ（コンテキストなし）
public fn info(code: String, message: String) -> Unit !Io
public fn success(code: String, message: String) -> Unit !Io
public fn warn(code: String, message: String) -> Unit !Io
public fn error(code: String, message: String) -> Unit !Io

// コンテキスト付きログ（ctx: フィールド名→値のマップ）
public fn info_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io
public fn success_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io
public fn warn_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io
public fn error_ctx(code: String, message: String, ctx: Map<String, String>) -> Unit !Io
```

### 6-B: `metric.fav`

```favnir
// メトリクス出力
// value は Int（CloudWatch EMF に変換時は float として扱う）
public fn metric(name: String, value: Int) -> Unit !Io

// 単位を指定するバリアント（"Count" | "Milliseconds" | "Bytes" | "None" 等）
public fn metric_with_unit(name: String, value: Int, unit: String) -> Unit !Io
```

### 6-C: `codes.fav` — 組み込みコード定数

```favnir
public fn app_started()  -> String { "I000" }
public fn app_stopped()  -> String { "I001" }
public fn processing()   -> String { "I010" }
public fn completed()    -> String { "S000" }
public fn pipeline_done()-> String { "S010" }
public fn retrying()     -> String { "W001" }
public fn slow_op()      -> String { "W002" }
public fn db_error()     -> String { "LE010" }
public fn net_error()    -> String { "LE020" }
public fn auth_error()   -> String { "LE030" }
public fn rpc_error()    -> String { "LE040" }
public fn aws_error()    -> String { "LE050" }
```

### 6-D: `log.fav`（barrel file）

```favnir
use emit.{ info, success, warn, error, info_ctx, success_ctx, warn_ctx, error_ctx }
use metric.{ metric, metric_with_unit }
use codes.*
```

---

## 7. checker.rs への変更

### 7-A: `("Log", *)` アームの追加

`check_builtin_apply` に `("Log", *)` アームを追加する。`!Io` は要求しない
（VM primitive 層で直接 stdout に書くため、checker 側でのエフェクト検査は不要）。

| メソッド | 戻り値型 |
|---------|---------|
| `emit_raw` | `Unit` |
| `metric_raw` | `Unit` |
| `_`（その他） | `Unit` |

### 7-B: `"Log"` namespace の登録

`compiler.rs` の global loop に `"Log"` を追加する（`"Crypto"` / `"Auth"` と同様）。

---

## 8. 自動エラーログ（VM primitive 層）

v4.6.0 では、`DB.query_raw` / `Http.get_raw` / `Http.post_raw` がエラー時に
`Log.emit_raw("ERROR", "LE010", message, "{}")` 相当の出力を自動的に行う。

対象プリミティブ:
- `DB.*` → `LE010`
- `Http.*` → `LE020`
- `Crypto.*`（jwt_verify エラー） → `LE030`
- `Grpc.*` → `LE040`

出力条件: `err_vm(...)` を返す直前に emit する。`log.level` 設定を参照し、
`"error"` レベル以上の場合のみ出力する。

---

## 9. 使用イメージ

### 9-A: パイプライン処理

```favnir
import rune "log"
import rune "duckdb"

public fn main() -> Unit !Io !Db {
    log.info(log.app_started(), "Pipeline started")
    bind conn <- duckdb.open(":memory:")
    bind rows <- duckdb.query(conn, "SELECT * FROM 'data/*.parquet'")
    match rows {
        Ok(data) => {
            log.success_ctx(log.pipeline_done(), "Pipeline completed",
                Map.set((), "rows", String.from_int(List.length(data))))
            log.metric("processed_rows", List.length(data))
        }
        Err(e) => log.error_ctx(log.db_error(), "Query failed", Map.set((), "error", e))
    }
}
```

**ローカル（text フォーマット）:**
```
[2026-05-17 10:30:00] INFO    I000  Pipeline started
[2026-05-17 10:30:01] SUCCESS S010  Pipeline completed  rows=1500
[2026-05-17 10:30:01] METRIC  processed_rows=1500 Count
```

**本番（json フォーマット）:**
```json
{"ts":"2026-05-17T10:30:00Z","level":"INFO","code":"I000","msg":"Pipeline started","service":"my-pipeline","ctx":{}}
{"ts":"2026-05-17T10:30:01Z","level":"SUCCESS","code":"S010","msg":"Pipeline completed","service":"my-pipeline","ctx":{"rows":"1500"}}
{"_aws":{"Timestamp":1747471801000,"CloudWatchMetrics":[{"Namespace":"favnir","Dimensions":[[]],"Metrics":[{"Name":"processed_rows","Unit":"Count"}]}]},"processed_rows":1500}
```

### 9-B: エラーログとリトライ

```favnir
import rune "log"
import rune "http"

fn fetch_with_retry(url: String, max: Int) -> Result<String, String> !Io !Network {
    match http.get(url) {
        Ok(resp) => Result.ok(resp)
        Err(e) => match max <= 0 {
            true  => {
                log.error_ctx(log.net_error(), "All retries exhausted", Map.set((), "url", url))
                Result.err(e)
            }
            false => {
                log.warn_ctx(log.retrying(), "Retrying request", Map.set((), "remaining", String.from_int(max)))
                fetch_with_retry(url, max - 1)
            }
        }
    }
}
```

---

## 10. テスト方針

### 10-A: vm_stdlib_tests.rs（8 件以上）

| テスト名 | 内容 |
|---------|------|
| `log_emit_text_format` | `Log.emit_raw` が text フォーマットで stdout に出力する |
| `log_emit_json_format` | `Log.emit_raw` が json フォーマットで出力する |
| `log_emit_level_filter_suppresses` | レベルフィルタで抑制されることを確認 |
| `log_emit_level_filter_passes` | レベルフィルタを通過することを確認 |
| `log_metric_text_format` | `Log.metric_raw` が text フォーマットで出力する |
| `log_metric_json_emf_format` | `Log.metric_raw` が json フォーマットで EMF 形式を出力する |
| `log_emit_ctx_json_included` | ctx_json が出力に含まれる |
| `log_emit_service_in_json` | service 名が json 出力に含まれる |

### 10-B: driver.rs 統合テスト（5 件以上）

| テスト名 | 内容 |
|---------|------|
| `log_rune_test_file_passes` | log.test.fav 全件 pass |
| `log_info_in_favnir_source` | Favnir ソースで `log.info` が動く |
| `log_error_ctx_in_favnir_source` | `log.error_ctx` でコンテキスト付きエラーログ |
| `log_metric_in_favnir_source` | `log.metric` が動く |
| `log_config_from_toml_in_favnir_source` | `fav.toml [log]` 設定が反映される |

### 10-C: `runes/log/log.test.fav`（12 件以上）

- `log.info` / `log.success` / `log.warn` / `log.error` が例外なく動く
- `log.info_ctx` がコンテキスト付きで動く
- `log.metric` / `log.metric_with_unit` が動く
- `log.app_started()` 等のコード定数が正しい文字列を返す
- 複数ログ呼び出しが順番通り動く

---

## 11. 完了条件

- `cargo build` が通る
- 既存 848 件が全て pass
- 新規テスト 25 件以上が pass（Rust 8 件 + Favnir 12 件 + 統合 5 件）
- `log.info("I000", "started")` が text / json 両フォーマットで出力できる
- `log.error_ctx` でコンテキスト付きエラーが出力できる
- `log.metric("rows", 1500)` が text では `METRIC` 行、json では EMF 形式で出力される
- `fav.toml [log] format = "json"` の切り替えが動く
- `fav.toml [log] level = "error"` でレベルフィルタが動く
- `logs/*.yaml` のカスタムコードが `cmd_run` 起動時にロードされる
- `examples/log_demo/` が `fav run` で動く
