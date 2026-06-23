# v22.7.0 仕様書 — OpenTelemetry 統合

## 概要

分散トレーシングを `fav run --trace` に統合する。
`OTEL_EXPORTER_OTLP_ENDPOINT` 環境変数を設定すれば、各 `stage` の実行を
**OpenTelemetry span** として OTLP/HTTP エンドポイントに送信できる。
Jaeger・Grafana Tempo など標準 OTel バックエンドで可視化可能。

環境変数未設定の場合は、既存の `[TRACE]` 出力に加えて `[OTEL]` span サマリーを
**stderr** に追記するのみで、動作に変化はない。

**テーマ**: 「分散トレーシングを型付き pipeline に標準搭載する」

---

## ロードマップ完了条件との対応

v22.7.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第七弾。
ロードマップ v22.7「OpenTelemetry 統合」を実装する。

完了条件:「OpenTelemetry の trace が Jaeger で確認できる」

---

## 機能仕様

### `fav run --trace` の拡張

`--trace` フラグは v12.5.0 から存在し、`verbose_level = 2` として VM の `[TRACE]` 出力を有効化する。
v22.7.0 ではこれに加えて **OTel span の収集と送信** を追加する。

既存の `[TRACE]` 出力は変更しない（後方互換）。

#### 環境変数 `OTEL_EXPORTER_OTLP_ENDPOINT`

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4318 fav run --trace pipeline.fav
```

| 設定 | 動作 |
|---|---|
| 設定あり | spans を OTLP/HTTP で POST する (`{endpoint}/v1/traces`) |
| 設定なし | stderr に `[OTEL]` span サマリーを出力（既存 `[TRACE]` と同時） |

#### Span 構造

| Span | 名前 | 種別 |
|---|---|---|
| Stage | `stage:<stage 名>` | span（pipeline level） |

> v22.7.0 では root span (pipeline) は実装しない。各 stage を独立した span として出力する。

#### Span Attributes

| Attribute キー | 値 | 例 |
|---|---|---|
| `favnir.stage.name` | stage 名 | `"FetchData"` |
| `favnir.stage.input_items` | 入力要素数（List は長さ、その他は 1） | `42` |
| `favnir.stage.output_items` | 出力要素数 | `36` |
| `favnir.stage.status` | `"ok"` または `"error"` | `"ok"` |
| `service.name` | `"favnir"` | — |
| `service.version` | `env!("CARGO_PKG_VERSION")` | `"22.7.0"` |

#### `[OTEL]` stderr フォーマット（エンドポイント未設定時）

```
[OTEL] trace_id=a1b2c3d4e5f67890a1b2c3d4e5f67890
[OTEL] span stage:LoadCsv      dur=120ms  status=ok  in=0  out=1000
[OTEL] span stage:Transform    dur=200ms  status=ok  in=1000  out=1000
[OTEL] span stage:Save         dur=130ms  status=ok  in=1000  out=1000
```

---

## アーキテクチャ

### 新規ファイル: `fav/src/otel.rs`

新しいモジュール（`#[cfg(not(target_arch = "wasm32"))]`）を追加する。

#### 型定義

```rust
pub type TraceId = String;   // hex 32 chars（128 bit random）
pub type SpanId  = String;   // hex 16 chars（64 bit random）

#[derive(Debug, Clone)]
pub struct OtelSpan {
    pub trace_id:       TraceId,
    pub span_id:        SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name:           String,
    pub start_unix_ns:  u128,
    pub end_unix_ns:    u128,
    pub input_items:    u64,
    pub output_items:   u64,
    pub status:         OtelStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OtelStatus { Ok, Error(String) }
```

#### Thread-local ストレージ

```rust
thread_local! {
    /// 完成した spans（end 済み）
    static OTEL_SPANS:    RefCell<Vec<OtelSpan>>       = const { RefCell::new(Vec::new()) };
    /// 現在のトレース ID
    static CURRENT_TRACE: RefCell<Option<TraceId>>     = const { RefCell::new(None) };
    /// 親 span スタック（ネスト対応）
    static PARENT_STACK:  RefCell<Vec<SpanId>>         = const { RefCell::new(Vec::new()) };
    /// OTel 有効フラグ
    static OTEL_ENABLED:  RefCell<bool>                = const { RefCell::new(false) };
    /// 進行中 spans（span_id → OtelSpan、end 時に OTEL_SPANS へ移動）
    static PENDING_SPANS: RefCell<std::collections::HashMap<SpanId, OtelSpan>>
        = const { RefCell::new(std::collections::HashMap::new()) };
}
```

#### 公開 API

```rust
pub fn otel_init();                 // トレース開始・thread-local 初期化
pub fn otel_is_enabled() -> bool;
pub fn otel_current_parent() -> Option<SpanId>;
pub fn otel_span_start(name: &str, parent_id: Option<&SpanId>) -> SpanId;
pub fn otel_span_end(span_id: &SpanId, input_items: u64, output_items: u64, status: OtelStatus);
pub fn otel_collected_spans() -> Vec<OtelSpan>;   // テスト用
pub fn otel_reset();                // thread-local 全クリア（テスト間クリーンアップ）
pub fn otel_export_http(endpoint: &str);           // OTLP/HTTP POST（失敗は stderr 警告のみ）
pub fn otel_export_stdout();        // [OTEL] を stderr に出力（エンドポイント未設定時）
```

---

### `vm.rs` 変更

#### `VM` struct に新フィールド追加

```rust
/// v22.7.0: 現在実行中 stage の OTel span ID。
pub current_otel_span_id: Option<crate::otel::SpanId>,
```

`VM::new` / struct literal に `current_otel_span_id: None` を追加。

#### `SeqStageEnter` 処理を修正

`stage_name` を `verbose_level > 0` ブロックの **外** に hoist してから OTel span を開始する:

```rust
x if x == Opcode::SeqStageEnter as u8 => {
    let name_idx = Self::read_u16(function, frame)? as usize;
    // v22.7.0: stage_name を verbose ブロックの外に hoist
    let stage_name = artifact.str_table.get(name_idx).map(|s| s.as_str()).unwrap_or("?");
    if Self::verbose_level() > 0 {
        trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: enter", stage_name));
    }
    // v22.7.0: OTel span 開始
    if crate::otel::otel_is_enabled() {
        let parent = crate::otel::otel_current_parent();
        let span_id = crate::otel::otel_span_start(
            &format!("stage:{}", stage_name),
            parent.as_ref(),
        );
        vm.current_otel_span_id = Some(span_id);
    }
}
```

#### `SeqStageCheck` 処理に OTel span 終了を追加

`stage_name` は `SeqStageCheck` では既に verbose ブロック外に定義されている（既存コード通り）。
`[TRACE] stage X: exit Ok(...)` の直後に追加:

```rust
// v22.7.0: OTel span 終了（Ok）
if let Some(ref sid) = vm.current_otel_span_id.take() {
    let out_items = otel_value_items(&unwrapped.to_vmvalue());
    crate::otel::otel_span_end(sid, 0, out_items, crate::otel::OtelStatus::Ok);
}
```

`[TRACE] stage X: exit Err(...)` の直後に追加:

```rust
// v22.7.0: OTel span 終了（Err）
if let Some(ref sid) = vm.current_otel_span_id.take() {
    crate::otel::otel_span_end(
        sid, 0, 0,
        crate::otel::OtelStatus::Error(inner_msg.clone()),
    );
}
```

ヘルパー（`vm.rs` 末尾に追加）:

```rust
fn otel_value_items(v: &VMValue) -> u64 {
    match v {
        VMValue::List(items, ..) => items.len() as u64,
        VMValue::Str(s)          => s.len() as u64,
        _                        => 1,
    }
}
```

---

### `driver.rs` 変更

`cmd_run` は `set_verbose_level(verbose_level)` を 1 回呼んだ後、リセット呼び出しなしで関数が終了する。
OTel の init は `set_verbose_level(verbose_level)` の**直後**、export は `cmd_run` の関数末尾（最後の `}`の直前）に追加する。

```rust
// cmd_run 末尾に追加（Favnir / legacy 両パス後）
if trace {
    match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(ep) => crate::otel::otel_export_http(&ep),
        Err(_) => crate::otel::otel_export_stdout(),
    }
    crate::otel::otel_reset();
}
```

---

### `lib.rs` / `main.rs` 変更

`lib.rs`（`mod pushdown;` / `mod arena;` 等の native-only mod の近くに追加）:

```rust
#[cfg(not(target_arch = "wasm32"))]
pub mod otel;
```

`main.rs`（同位置）:

```rust
#[cfg(not(target_arch = "wasm32"))]
mod otel;
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/otel.rs` | 新規 | OTel span 収集・OTLP/HTTP エクスポート・stderr フォールバック |
| `fav/src/backend/vm.rs` | 更新 | `SeqStageEnter` hoist + OTel span 開始、`SeqStageCheck` に OTel span 終了 |
| `fav/src/driver.rs` | 更新 | `cmd_run` に OTel init/export 追加、`v227000_tests` 5 件 |
| `fav/src/lib.rs` | 更新 | `mod otel` 追加（cfg guard あり） |
| `fav/src/main.rs` | 更新 | `mod otel` 追加（cfg guard あり） |
| `fav/Cargo.toml` | 更新 | `version = "22.6.0"` → `"22.7.0"`（新規依存なし） |
| `CHANGELOG.md` | 更新 | v22.7.0 エントリ追加 |
| `benchmarks/v22.7.0.json` | 新規 | ベンチマーク結果 |
| `site/content/docs/cli/otel.mdx` | 新規 | OTel 統合ドキュメント |

---

## 依存クレート

**新規追加なし**。既存の `rand`（ID 生成）・`ureq`（HTTP POST）・`std::time::SystemTime`（タイムスタンプ）を使用する。

---

## テスト一覧（v227000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_7_0` | Cargo.toml に `version = "22.7.0"` が含まれる |
| `otel_spans_collected_after_run` | pipeline 実行後に `otel_collected_spans()` が 1 件以上の span を返す |
| `otel_span_name_includes_stage_name` | span の name が `"stage:MyStage"` 形式になっている |
| `otel_export_stdout_does_not_panic` | `otel_export_stdout()` がパニックせず正常終了する |
| `changelog_has_v22_7_0` | CHANGELOG.md に `[v22.7.0]` が含まれる |

---

## スコープ外（v22.7.0 では実装しない）

- root span（pipeline 全体を包む span）
- `traceparent` HTTP ヘッダーによる外部 trace_id 注入
- OTLP/gRPC エクスポート（tonic バージョン競合を避けるため HTTP のみ）
- SLA アノテーション（`#[timeout]` 等）の実行時適用（v22.8+ 予定）
- Jaeger All-in-One の Docker Compose 設定ファイル
- `fav.toml` 経由での OTel 設定（`[otel]` セクション）
- baggage / span links / events

---

## 完了条件

- [ ] `fav/src/otel.rs` が作成され、全 API が実装される
- [ ] `vm.rs` の `SeqStageEnter` 処理（`stage_name` hoist 含む）に OTel span 開始が追加される
- [ ] `SeqStageCheck` の OK / Err 終了時に OTel span 終了が呼ばれる
- [ ] `cmd_run` の `trace = true` 時に OTel init / export が実行される
- [ ] `OTEL_EXPORTER_OTLP_ENDPOINT` が設定されている場合、OTLP/HTTP で POST される
- [ ] 環境変数未設定時は `[OTEL]` が stderr に出力される
- [ ] `lib.rs` / `main.rs` に `#[cfg(not(target_arch = "wasm32"))]` 付き `mod otel` が追加される
- [ ] `cargo test v227000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1874 件以上合格）
- [ ] `CHANGELOG.md` に v22.7.0 エントリ
- [ ] `benchmarks/v22.7.0.json` 作成済み
- [ ] `site/content/docs/cli/otel.mdx` 作成済み
