# v22.6.0 仕様書 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

## 概要

本番パイプラインの信頼性を型システムレベルで保証する SLA アノテーションを導入する。
`#[timeout]` / `#[retry]` / `#[circuit_breaker]` を `stage` 定義に付与し、
**コンパイル時に値の妥当性を検証**する。
`fav explain --sla` でパイプライン全体の最悪実行時間を計算・表示できる。

v22.6.0 は **構文解析・コンパイル時検証・`fav explain --sla` 出力**を実装する。
実行時の timeout / retry の実際の適用（VM インターセプト）は v22.7+ で対応する。

**テーマ**: 「SLA を型システムに統合する」

---

## ロードマップ完了条件との対応

v22.6.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第六弾。
ロードマップ v22.6「SLA 宣言」の静的解析・コンパイル時チェック部分を実装する。

---

## 機能仕様

### SLA アノテーション構文

`stage` 定義に最大 3 種のアノテーションを付与できる（順序は問わない）。

```favnir
#[timeout(seconds = 30)]
#[retry(max = 3, backoff = "exponential")]
#[circuit_breaker(threshold = 0.5, window = 60)]
stage CallExternalAPI: Request -> Response = |req| {
  http.post(req)
}
```

#### `#[timeout(seconds = N)]`

| パラメータ | 型 | 説明 |
|---|---|---|
| `seconds` | 正の数値（整数または小数） | 最大実行時間（秒） |

コンパイル時チェック: `seconds > 0` でなければ E0401。

#### `#[retry(max = N, backoff = "...")]`

| パラメータ | 型 | 説明 |
|---|---|---|
| `max` | 正の整数 | 最大リトライ回数（1 以上） |
| `backoff` | 文字列リテラル | `"exponential"` / `"linear"` / `"none"` のいずれか |

コンパイル時チェック:
- `max >= 1` でなければ E0402
- `backoff` が `"exponential"` / `"linear"` / `"none"` 以外なら E0402

#### `#[circuit_breaker(threshold = F, window = N)]`

| パラメータ | 型 | 説明 |
|---|---|---|
| `threshold` | 0.0 超〜1.0 以下の数値 | 失敗率の閾値（0.5 = 50% 失敗でブレーカーが開く） |
| `window` | 正の整数 | 計測ウィンドウ（秒） |

コンパイル時チェック:
- `threshold > 0.0 && threshold <= 1.0` でなければ E0403
- `window > 0` でなければ E0403

---

### エラーコード

| コード | トリガー条件 | メッセージ例 |
|---|---|---|
| E0401 | `#[timeout]` の `seconds` が 0 以下 | `E0401: timeout seconds must be > 0, got -1` |
| E0402 | `#[retry]` の `max` が 0 以下、または `backoff` が不正 | `E0402: retry max must be >= 1` / `E0402: unknown backoff strategy "fast"` |
| E0403 | `#[circuit_breaker]` の `threshold` が範囲外、または `window` が 0 以下 | `E0403: circuit_breaker threshold must be in (0.0, 1.0]` |

E0401〜E0403 は `fav/src/error_catalog.rs` の `ERROR_CATALOG` 定数に追加する。

---

### `fav explain --sla [file]`

```bash
fav explain --sla src/pipeline.fav
```

すべての `stage` を検索し、SLA アノテーションを持つ stage の一覧と最悪実行時間を出力する。
アノテーションのない stage（`Transform` 等）も一覧に含めるが、`worst_case` は `—` とし、合計には含めない。

#### 出力フォーマット

```
SLA Summary — src/pipeline.fav
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
stage             timeout   retry   circuit_breaker   worst_case
CallExternalAPI   30s       3×      0.5/60s           90s
FetchData         10s       2×      —                 20s
Transform         —         —       —                 —
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total worst-case (SLA-annotated stages only): 110s
```

最悪実行時間の計算式（stage ごと）:
- `timeout` のみ: `timeout_secs`
- `timeout` + `retry`: `timeout_secs * retry.max`
- アノテーションなし: `—`（合計に含めない）

---

## アーキテクチャ

### 新規 struct（`ast.rs`）

```rust
// ── SLA Annotations (v22.6.0) ────────────────────────────────────────────────

/// `#[timeout(seconds = 30)]` annotation on stage definitions.
#[derive(Debug, Clone)]
pub struct TimeoutAnnotation {
    pub seconds: f64,
    pub span: Span,
}

/// `#[retry(max = 3, backoff = "exponential")]` annotation on stage definitions.
#[derive(Debug, Clone)]
pub struct RetryAnnotation {
    pub max: u32,
    pub backoff: String,   // "exponential" | "linear" | "none"
    pub span: Span,
}

/// `#[circuit_breaker(threshold = 0.5, window = 60)]` annotation on stage definitions.
#[derive(Debug, Clone)]
pub struct CircuitBreakerAnnotation {
    pub threshold: f64,    // 0.0 < threshold <= 1.0
    pub window: u64,       // seconds > 0
    pub span: Span,
}
```

### `TrfDef` への新フィールド追加（`ast.rs`）

既存の `pub checkpoint: bool` フィールドの直後（`pub span: Span` の前）に追加:

```rust
    /// v22.6.0: `#[timeout(seconds = N)]` annotation.
    pub timeout: Option<TimeoutAnnotation>,
    /// v22.6.0: `#[retry(max = N, backoff = "...")]` annotation.
    pub retry_ann: Option<RetryAnnotation>,
    /// v22.6.0: `#[circuit_breaker(threshold = F, window = N)]` annotation.
    pub circuit_breaker: Option<CircuitBreakerAnnotation>,
```

### パーサー変更（`frontend/parser.rs`）

3 つのアノテーションパーサーを追加し、`parse_item()` 内で `trigger_ann` の直後に呼ぶ:

```rust
fn parse_timeout_annotation(&mut self) -> Result<Option<TimeoutAnnotation>, ParseError>
fn parse_retry_annotation(&mut self) -> Result<Option<RetryAnnotation>, ParseError>
fn parse_circuit_breaker_annotation(&mut self) -> Result<Option<CircuitBreakerAnnotation>, ParseError>
```

`parse_item()` での呼び出し順（既存行も含めた完全コンテキスト）:
```rust
let checkpoint_ann      = self.parse_checkpoint_annotation()?;
let trigger_ann         = self.parse_trigger_annotation()?;     // 既存（v22.4.0）
let timeout_ann         = self.parse_timeout_annotation()?;     // v22.6.0
let retry_ann           = self.parse_retry_annotation()?;       // v22.6.0
let circuit_breaker_ann = self.parse_circuit_breaker_annotation()?; // v22.6.0
```

Stage ブランチの構造体初期化に追加:
```rust
td.checkpoint      = checkpoint_ann;
td.trigger         = trigger_ann;         // 既存
td.timeout         = timeout_ann;         // v22.6.0
td.retry_ann       = retry_ann;           // v22.6.0
td.circuit_breaker = circuit_breaker_ann; // v22.6.0
```

**`#` / `[` の消費パターン**: 既存の `parse_checkpoint_annotation` / `parse_trigger_annotation` に合わせて、`#` は `self.advance()`、`[` は `self.expect(&TokenKind::LBracket)?` で消費する（`advance()` ではなく `expect()` を使うことで不正トークンを検出できる）。

**数値トークン**:
- `seconds = 30` → `TokenKind::Int(30)` → `30.0 as f64`
- `seconds = 0.5` → `TokenKind::Float(0.5)`
- `threshold` / `seconds` は両方対応; `max` / `window` は Int のみ

### チェッカー変更（`middle/checker.rs`）

`check_trf_def()` 末尾（`check_item` からは `check_trf_def` に委譲されているため、実際の追加先は `check_trf_def`）に SLA バリデーションを追加:

```rust
// ── SLA バリデーション (v22.6.0) ──────────────────────────────────────────────
if let Some(t) = &td.timeout {
    if t.seconds <= 0.0 {
        self.type_error("E0401", format!("timeout seconds must be > 0, got {}", t.seconds), &t.span);
    }
}
if let Some(r) = &td.retry_ann {
    if r.max == 0 {
        self.type_error("E0402", "retry max must be >= 1".to_string(), &r.span);
    }
    if !matches!(r.backoff.as_str(), "exponential" | "linear" | "none") {
        self.type_error("E0402", format!("unknown backoff strategy {:?}; expected \"exponential\", \"linear\", or \"none\"", r.backoff), &r.span);
    }
}
if let Some(cb) = &td.circuit_breaker {
    if cb.threshold <= 0.0 || cb.threshold > 1.0 {
        self.type_error("E0403", format!("circuit_breaker threshold must be in (0.0, 1.0], got {}", cb.threshold), &cb.span);
    }
    if cb.window == 0 {
        self.type_error("E0403", "circuit_breaker window must be > 0".to_string(), &cb.span);
    }
}
```

> **注意**: エラー追加は `self.type_error(code, message, &span)` を使う（`CheckError::new` / `self.errors.push` は不正）。`type_error` の第 3 引数は `&Span`（参照）なので `&t.span` を渡す。

### `cmd_explain_sla`（`driver.rs`）

SLA アノテーションを持つ全 `stage` を表形式で出力。アノテーションなし stage も行に含めるが `worst_case` は `—` とし、合計（`Total`）には加算しない。

### CLI（`main.rs`）

`Some("explain")` ブランチ内の `--lineage` チェックの直後に追加:

```rust
if args.iter().any(|a| a == "--sla") {
    let file = args.iter().skip(2).find(|a| !a.starts_with('-')).map(|s| s.as_str());
    crate::driver::cmd_explain_sla(file);
    return;
}
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` struct 追加、`TrfDef` に 3 フィールド追加 |
| `fav/src/frontend/parser.rs` | 更新 | 3 アノテーションパーサー追加、`parse_item()` 統合、`TrfDef` 初期化箇所への代入追加 |
| `fav/src/middle/checker.rs` | 更新 | `check_trf_def()` 末尾に E0401 / E0402 / E0403 バリデーション追加 |
| `fav/src/error_catalog.rs` | 更新 | E0401 / E0402 / E0403 エントリ追加 |
| `fav/src/driver.rs` | 更新 | `cmd_explain_sla` 追加、`v226000_tests` 8 件 |
| `fav/src/main.rs` | 更新 | `fav explain --sla` フラグ対応、`cmd_explain_sla` import |
| `fav/Cargo.toml` | 更新 | `version = "22.5.0"` → `"22.6.0"` |
| `CHANGELOG.md` | 更新 | v22.6.0 エントリ追加 |
| `benchmarks/v22.6.0.json` | 新規 | ベンチマーク結果 |
| `site/content/docs/cli/sla.mdx` | 新規 | SLA アノテーション・`fav explain --sla` ドキュメント |

---

## テスト一覧（v226000_tests、8 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_6_0` | Cargo.toml に `version = "22.6.0"` が含まれる |
| `timeout_annotation_parsed` | `#[timeout(seconds = 30)]` が正しくパースされ `td.timeout.seconds == 30.0` |
| `retry_annotation_parsed` | `#[retry(max = 3, backoff = "exponential")]` が正しくパースされる |
| `circuit_breaker_annotation_parsed` | `#[circuit_breaker(threshold = 0.5, window = 60)]` が正しくパースされる |
| `sla_invalid_timeout_checker_err` | `seconds = 0` で E0401 が `check_program` に報告される |
| `sla_invalid_retry_checker_err` | `max = 0` で E0402 が報告される |
| `sla_invalid_circuit_breaker_checker_err` | `threshold = 0.0` で E0403 が報告される |
| `changelog_has_v22_6_0` | CHANGELOG.md に `[v22.6.0]` が含まれる |

---

## スコープ外（v22.6.0 では実装しない）

- 実行時 timeout の VM インターセプト（実際に N 秒で中断する）
- 実行時 retry の実装（失敗時に自動再実行する）
- circuit_breaker の実行時状態管理（Redis 等への状態保存）
- `fav.toml` への SLA デフォルト設定
- `#[sla_group(...)]` でグループ化した SLA 宣言
- `fn` 定義への SLA アノテーション付与（v22.6.0 ではサイレントに無視される。E0404 は v22.7+ 予定）

---

## 完了条件

- [ ] `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` が `ast.rs` に追加される
- [ ] `TrfDef` に `timeout` / `retry_ann` / `circuit_breaker` フィールドが追加される
- [ ] `#[timeout(seconds = N)]` がパースされ、`TrfDef.timeout` に設定される
- [ ] `#[retry(max = N, backoff = "...")]` がパースされ、`TrfDef.retry_ann` に設定される
- [ ] `#[circuit_breaker(threshold = F, window = N)]` がパースされ、`TrfDef.circuit_breaker` に設定される
- [ ] E0401 / E0402 / E0403 がコンパイル時に `check_trf_def()` 経由で報告される
- [ ] E0401〜E0403 が `error_catalog.rs` に登録される
- [ ] `fav explain --sla [file]` が SLA 一覧と最悪実行時間（SLA 付き stage の合計）を出力する
- [ ] `cargo test v226000 --bin fav` — 8/8 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1865 件以上合格）
- [ ] `CHANGELOG.md` に v22.6.0 エントリ
- [ ] `benchmarks/v22.6.0.json` 作成済み
- [ ] `site/content/docs/cli/sla.mdx` 作成済み
