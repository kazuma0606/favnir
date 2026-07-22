# Spec: v52.7.0 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

Status: PLANNED
Date: 2026-07-21

---

## 目的

v52.6.0 でデータアクセスログを追加した。
v52.7.0 では OTel の stage span に以下の属性を付与する:

- `schema.name` / `schema.fields` — `assert_schema<T>` 呼び出し時に設定
- `lineage.upstream` / `lineage.downstream` — `seq` パイプライン実行順序から自動追跡

これにより Jaeger / Grafana Tempo 等で以下のような span 詳細が表示可能になる:

```
span: stage.Validate
  schema.name        = "OrderRow"
  schema.fields      = "id,amount,status"
  lineage.upstream   = "Parse"
  lineage.downstream = "snowflake.insert"
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/otel.rs` | `OtelSpan.attrs` 追加、`otel_add_attr` / `otel_patch_attr_on_last` 追加、`build_otlp_json` / `otel_export_stdout` 更新 |
| `fav/src/backend/vm.rs` | `OTEL_PREV_STAGE` thread-local 追加、`reset_stage_lineage` 追加、`SeqStageEnter` に lineage フック、`AssertSchema` に schema フック |
| `fav/src/driver.rs` | `otel_init()` ブロックに `reset_stage_lineage()` 呼び出し追加、`v52700_tests` 追加 |
| `fav/Cargo.toml` | version → `"52.7.0"` |
| `CHANGELOG.md` | v52.7.0 エントリ追加 |
| `versions/current.md` | v52.7.0（3151 tests）に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.7.0 実績欄を更新 |

---

## 詳細仕様

### 1. `otel.rs` — span attrs 拡張

#### 1a. `OtelSpan` 構造体に `attrs` フィールド追加

```rust
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
    pub attrs:          Vec<(String, String)>,  // ← v52.7.0 追加（末尾）
}
```

`otel_span_start` 内の初期化に `attrs: Vec::new()` を追加。

#### 1b. `otel_add_attr` 追加

PARENT_STACK の先頭 span ID に対して attr を追加するヘルパー。

```rust
/// v52.7.0: 現在実行中 span に文字列属性を追加する。OTel 有効時のみ呼ぶこと。
pub fn otel_add_attr(key: &str, val: &str) {
    let span_id = PARENT_STACK.with(|s| s.borrow().last().cloned());
    if let Some(sid) = span_id {
        PENDING_SPANS.with(|p| {
            if let Some(span) = p.borrow_mut().get_mut(&sid) {
                span.attrs.push((key.to_string(), val.to_string()));
            }
        });
    }
}
```

挿入位置: `otel_patch_attr_on_last` の直前（`otel_span_end` と `otel_collected_spans` の間）。

#### 1c. `otel_patch_attr_on_last` 追加

OTEL_SPANS（完了済み span リスト）の最後の span に attr を後付けするヘルパー。
lineage.downstream を直前 stage の完了済み span に追記するために使用する。

```rust
/// v52.7.0: 完了済み span リストの最後のエントリに属性を後付けする。
/// lineage.downstream の遡及追加に使用。
pub fn otel_patch_attr_on_last(key: &str, val: &str) {
    OTEL_SPANS.with(|s| {
        if let Some(span) = s.borrow_mut().last_mut() {
            span.attrs.push((key.to_string(), val.to_string()));
        }
    });
}
```

挿入位置: `otel_add_attr` の直後。

#### 1d. `build_otlp_json` 更新

`attrs` フィールドの追加 attr を OTLP JSON の `attributes` 配列に追加する。

```rust
// 既存 attrs に続けて span.attrs を追加（v52.7.0）
let extra_attrs: String = s.attrs.iter().map(|(k, v)| {
    format!(
        r#"{{"key":"{}","value":{{"stringValue":"{}"}}}}"#,
        escape_json_str(k), escape_json_str(v)
    )
}).collect::<Vec<_>>().join(",");
let extra_part = if extra_attrs.is_empty() {
    String::new()
} else {
    format!(",{}", extra_attrs)
};
```

`attrs` 文字列生成部分を上記 `extra_part` を末尾に追加する形に変更する:

```rust
let attrs = format!(
    r#"[{{"key":"favnir.stage.name",...}},{{"key":"favnir.stage.status",...}}{}]"#,
    // 既存 4 フィールド, extra_part を末尾に追加
    ..., extra_part,
);
```

#### 1e. `otel_export_stdout` 更新

`for span in &spans { ... }` ループ内の `eprintln!("[OTEL] span ...")` の直後に追加。
span に `attrs` がある場合は各 attr を追加出力する。

```rust
// v52.7.0: extra attrs（schema / lineage）
for (k, v) in &span.attrs {
    eprintln!("       {:<30} = {}", k, v);
}
```

---

### 2. `vm.rs` — OTel フック

#### 2a. `OTEL_PREV_STAGE` thread-local 追加

挿入位置: `AUDIT_LOG_PATH` thread-local ブロックの直後。

```rust
// v52.7.0: OTel lineage 追跡 — 直前の stage 名。
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static OTEL_PREV_STAGE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}
```

#### 2b. `reset_stage_lineage` 関数追加

挿入位置: `append_audit_event` の直後。

```rust
/// v52.7.0: OTel lineage 追跡をリセットする（各 fav run の先頭で呼ぶ）。
#[cfg(not(target_arch = "wasm32"))]
pub fn reset_stage_lineage() {
    OTEL_PREV_STAGE.with(|p| *p.borrow_mut() = None);
}
```

#### 2c. `SeqStageEnter` opcode に lineage フック追加

挿入位置: `otel_span_start` 呼び出しの直後（`vm.current_otel_span_id = Some(span_id);` の前）。

```rust
// v52.7.0: lineage attrs — patch previous span with downstream, add upstream to current
let prev = OTEL_PREV_STAGE.with(|p| p.borrow().clone());
if let Some(ref prev_name) = prev {
    crate::otel::otel_patch_attr_on_last("lineage.downstream", stage_name);
    crate::otel::otel_add_attr("lineage.upstream", prev_name);
}
OTEL_PREV_STAGE.with(|p| *p.borrow_mut() = Some(stage_name.to_string()));
```

このブロックは既存の `if crate::otel::otel_is_enabled() { ... }` 内に含める（`#[cfg(not(wasm32))]` で外側がガードされているため内側の追加 `#[cfg]` は不要）。

**重要**: `otel_span_start` は内部で `PARENT_STACK.push(span_id)` を実行してから返るため、
`otel_span_start` 呼び出し直後の時点で PARENT_STACK 先頭は新 span_id を指している。
したがって `otel_add_attr` / `otel_patch_attr_on_last` はこの直後に正しく動作する。
`vm.current_otel_span_id = Some(span_id)` の前後は問わない。

#### 2d. `AssertSchema` opcode に schema フック追加

挿入位置: `vm.stack.push(NanVal::from_vmvalue(result));` の直前。

```rust
// v52.7.0: OTel schema attrs — assert_schema 成功時に schema.name / schema.fields を付与
#[cfg(not(target_arch = "wasm32"))]
if crate::otel::otel_is_enabled() {
    if let VMValue::Variant(ref tag, _) = result {
        if tag == "ok" {
            if let Some(meta) = vm.type_metas.get(&ty_name) {
                let fields_str: String = meta.fields.iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                crate::otel::otel_add_attr("schema.name", &ty_name);
                crate::otel::otel_add_attr("schema.fields", &fields_str);
            }
        }
    }
}
```

---

### 3. `driver.rs` — reset 呼び出し追加

挿入位置: `if trace { crate::otel::otel_init(); }` ブロックの直後。

```rust
// v52.7.0: OTel lineage 追跡をリセット（run ごとに前 stage をクリア）
#[cfg(not(target_arch = "wasm32"))]
crate::backend::vm::reset_stage_lineage();
```

---

## テスト（2 件）

追加先: `driver.rs` の `v52700_tests` モジュール（`v52600_tests` の直前）

### `otel_span_has_schema_attr`

```rust
#[test]
fn otel_span_has_schema_attr() {
    let src = include_str!("otel.rs");
    assert!(src.contains("schema.name"), "otel.rs must have schema.name attribute key");
    assert!(src.contains("schema.fields"), "otel.rs must have schema.fields attribute key");
}
```

### `otel_span_has_lineage_attr`

```rust
#[test]
fn otel_span_has_lineage_attr() {
    let src = include_str!("otel.rs");
    assert!(src.contains("lineage.upstream"), "otel.rs must have lineage.upstream attribute key");
    assert!(src.contains("lineage.downstream"), "otel.rs must have lineage.downstream attribute key");
}
```

---

## テスト数

- ベース: **3149** tests（v52.6.0 完了時点）
- `v52600_tests` に version テストなし → 削除 0 件
- 追加: `v52700_tests` 2 件（`otel_span_has_schema_attr` + `otel_span_has_lineage_attr`）
- **合計: 3151 tests**（ロードマップ記載の推定 3149 から +2 補正: v52.6.0 実績が 3149 だったため）

---

## 完了条件

- `cargo test` 3151 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `otel.rs` に `"schema.name"` / `"schema.fields"` / `"lineage.upstream"` / `"lineage.downstream"` が含まれる
- `otel_export_stdout` が span.attrs を出力する
- `build_otlp_json` が span.attrs を OTLP JSON の `attributes` 配列に追加する

---

## 注意事項

- `otel_patch_attr_on_last` は OTEL_SPANS（完了済み）の最後エントリを変更する。
  `SeqStageEnter` が直前 stage の `SeqStageCheck`（span 終了）後に呼ばれるため、
  直前 stage の span は必ず OTEL_SPANS に移動済みである（安全に操作可能）。
  **確認方法**: `rg -n "otel_span_end" fav/src/backend/vm.rs` で SeqStageCheck 内の
  `otel_span_end` 呼び出しを確認してから実装を開始すること。
- `OTEL_PREV_STAGE` は `reset_stage_lineage()` で初期化する。
  呼び出し元は `cmd_run` の `if trace { otel_init(); }` ブロック直後（driver.rs）。
  `reset_stage_lineage()` は `if trace` の**外側**に置く（`#[cfg(not(wasm32))]` のみガード）。
  これにより `--trace` なし時も OTEL_PREV_STAGE をリセットし、前 run の残留値を防ぐ。
  `OTEL_PREV_STAGE` は `if crate::otel::otel_is_enabled()` 内でのみ参照されるため、OTel 無効時に実害はない。
- `otel_add_attr` は PARENT_STACK の先頭（現在実行中 span）に追加するため、
  `SeqStageEnter`（span 開始直後）および `AssertSchema`（stage 実行中）からの呼び出しは正しい。
- `OtelSpan.attrs` の追加は `otel_span_start` 内の初期化に `attrs: Vec::new()` を追加することで対応する。
- `v52600_tests` に version テストなし → 削除対象なし。
