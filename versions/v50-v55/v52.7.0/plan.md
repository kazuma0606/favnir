# Plan: v52.7.0 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

Status: PLANNED
Date: 2026-07-21

---

## 実装順序

### Step 1 — `otel.rs` 更新

ファイル: `fav/src/otel.rs`

**1a. `OtelSpan` 構造体に `attrs` フィールド追加（行 14–25）**

`pub status: OtelStatus,` の直後に追加:
```rust
    pub attrs:          Vec<(String, String)>,  // v52.7.0: schema / lineage 属性
```

**1b. `otel_span_start` の初期化に `attrs: Vec::new()` 追加（行 95–105）**

`OtelSpan { ... }` の初期化ブロックに追加:
```rust
        attrs:          Vec::new(),
```

**1c. `otel_add_attr` 関数を追加**

挿入位置: `otel_span_end` と `otel_collected_spans` の間。

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

**1d. `otel_patch_attr_on_last` 関数を追加**

挿入位置: `otel_add_attr` の直後。

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

**1e. `build_otlp_json` 更新（行 211–240）**

`let attrs = format!(...)` の前に以下を追加:

```rust
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

`let attrs = format!(r#"[...]"#, ...)` の末尾に `{}]` → `{}{}]` として `extra_part` を追加:
- 末尾の `]` を `{}]` に変更し、`extra_part` を引数に追加する。

**1f. `otel_export_stdout` 更新（行 188–207）**

`eprintln!("[OTEL] span ...")` の直後に以下を追加:
```rust
            // v52.7.0: extra attrs（schema / lineage）
            for (k, v) in &span.attrs {
                eprintln!("       {:<30} = {}", k, v);
            }
```

`cargo build` → コンパイルエラーなし確認。

---

### Step 2 — `vm.rs` 更新

ファイル: `fav/src/backend/vm.rs`

**2a. `OTEL_PREV_STAGE` thread-local 追加**

挿入位置: `AUDIT_LOG_PATH` thread-local ブロック（`// v52.6.0: --audit-log 出力先パス。` ブロック）の直後。

```rust
// v52.7.0: OTel lineage 追跡 — 直前の stage 名。
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static OTEL_PREV_STAGE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}
```

**2b. `reset_stage_lineage` 関数追加**

挿入位置: `append_audit_event` 関数の直後（`/// Set the thread-local watch fields...` の前）。

```rust
/// v52.7.0: OTel lineage 追跡をリセットする（各 fav run の先頭で呼ぶ）。
#[cfg(not(target_arch = "wasm32"))]
pub fn reset_stage_lineage() {
    OTEL_PREV_STAGE.with(|p| *p.borrow_mut() = None);
}
```

**2c. `SeqStageEnter` opcode に lineage フック追加**

挿入位置: `otel_span_start(...)` 呼び出しの直後、`vm.current_otel_span_id = Some(span_id);` の前。

既存の `if crate::otel::otel_is_enabled() { ... }` ブロック内に追記:

```rust
        // v52.7.0: lineage attrs
        let prev = OTEL_PREV_STAGE.with(|p| p.borrow().clone());
        if let Some(ref prev_name) = prev {
            crate::otel::otel_patch_attr_on_last("lineage.downstream", stage_name);
            crate::otel::otel_add_attr("lineage.upstream", prev_name);
        }
        OTEL_PREV_STAGE.with(|p| *p.borrow_mut() = Some(stage_name.to_string()));
```

**2d. `AssertSchema` opcode に schema フック追加**

挿入位置: `vm.stack.push(NanVal::from_vmvalue(result));` の直前。

```rust
                // v52.7.0: OTel schema attrs
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

`cargo build` → コンパイルエラーなし確認。

---

### Step 3 — `driver.rs` 更新

**3a. `reset_stage_lineage` 呼び出し追加**

`rg -n "otel_init" fav/src/driver.rs` で挿入位置を確認。

`if trace { crate::otel::otel_init(); }` ブロックの直後に追加:

```rust
    // v52.7.0: OTel lineage 追跡をリセット（run ごとに前 stage をクリア）
    #[cfg(not(target_arch = "wasm32"))]
    crate::backend::vm::reset_stage_lineage();
```

**3b. `v52700_tests` モジュール追加**

挿入位置: `v52600_tests` の直前。

```rust
// -- v52700_tests (v52.7.0) -- OTel span 属性（schema / lineage）--
#[cfg(test)]
mod v52700_tests {
    #[test]
    fn otel_span_has_schema_attr() {
        let src = include_str!("otel.rs");
        assert!(src.contains("schema.name"), "otel.rs must have schema.name attribute key");
        assert!(src.contains("schema.fields"), "otel.rs must have schema.fields attribute key");
    }

    #[test]
    fn otel_span_has_lineage_attr() {
        let src = include_str!("otel.rs");
        assert!(src.contains("lineage.upstream"), "otel.rs must have lineage.upstream attribute key");
        assert!(src.contains("lineage.downstream"), "otel.rs must have lineage.downstream attribute key");
    }
}
```

---

### Step 4 — バージョン更新 + テスト

- `fav/Cargo.toml` version → `"52.7.0"`
- `cargo test` → 3151 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

---

### Step 5 — 後処理

- `CHANGELOG.md` に v52.7.0 エントリ追加
- `versions/current.md` を v52.7.0（3151 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.7.0 実績欄を更新
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `otel_add_attr` は PARENT_STACK の先頭 span（PENDING_SPANS 内）に追加する。
  `SeqStageEnter` で span が開始された直後、`SeqStageCheck` で終了する前に呼ぶため常に正しい span を指す。
- `otel_patch_attr_on_last` は OTEL_SPANS の最後エントリ（完了済み）を変更する。
  `SeqStageEnter` が発火する時点で直前 stage の `SeqStageCheck` は完了している（直後に呼ばれる）ため安全。
- `build_otlp_json` の `attrs` 文字列: `]` より前に `extra_part`（カンマ+追加 attr JSON）を挿入する。
  `extra_part` が空（attrs なし）の場合は空文字列 → JSON 構造に影響なし。
- `AssertSchema` の hook: `result` は既に確定しているため `vm.type_metas.get(&ty_name)` はフック内での参照（result 確定後の三度目）。
  一度目は `let result = if let Some(meta) = vm.type_metas.get(...)` でのフィールド検証、
  二度目以降はフック内での参照。`type_metas` は `HashMap<String, TypeMeta>` であり immutable borrow は安全（stack borrow は先に完了）。
