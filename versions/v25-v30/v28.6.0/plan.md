# v28.6.0 Plan — grafana Rune 追加

## 実装順序

```
T1: Cargo.toml version bump (28.5.0 → 28.6.0)
T2: vm.rs に Grafana.*_raw 3 primitive 追加
T3: runes/grafana/grafana.fav 新規作成（3 関数）
T4: examples/observability/grafana_dashboard.fav 新規作成
T5: site/content/docs/runes/grafana.mdx 新規作成
T6: CHANGELOG.md に [v28.6.0] セクション追加
T7: benchmarks/v28.6.0.json 新規作成
T8: driver.rs に v286000_tests 9 件追加
T9a: fav/self/checker.fav ns_to_effect に "Grafana" → "IO" 追加（Phase 9a）
T9b: cargo test --bin fav v286000 — 9/9 PASS 確認
T9c: cargo test --bin fav grafana — 8 件以上 PASS 確認
T9d: cargo test --bin fav 全体 — 2281 PASS 確認
T10: tasks.md を COMPLETE に更新
```

---

## T2: vm.rs — Grafana primitives

`fav/src/backend/vm.rs` の Sentry.set_extra_raw の wasm32 アームの直後に追加する。

```rust
// ── Grafana primitives (v28.6.0) ─────────────────────────────────────
// Stub: Grafana HTTP API リクエストは v28.7 以降
#[cfg(not(target_arch = "wasm32"))]
"Grafana.create_annotation_raw" => {
    let mut it = args.into_iter();
    let _dashboard_id = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing dashboard_id")?)?;
    let _text = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing text")?)?;
    let _tags = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing tags")?)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Grafana.create_annotation_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Grafana.push_dashboard_raw" => {
    let mut it = args.into_iter();
    let _json = vm_string(it.next().ok_or("Grafana.push_dashboard_raw: missing json")?)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Grafana.push_dashboard_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Grafana.snapshot_raw" => {
    let mut it = args.into_iter();
    let _dashboard_id = vm_string(it.next().ok_or("Grafana.snapshot_raw: missing dashboard_id")?)?;
    // 注意: snapshot_raw のみ VMValue::Str を返す（スナップショット URL）。
    // create_annotation_raw / push_dashboard_raw は VMValue::Unit を返す点と非対称。
    Ok(ok_vm(VMValue::Str("https://grafana.example.com/dashboard/snapshot/stub".into())))
}
#[cfg(target_arch = "wasm32")]
"Grafana.snapshot_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),
```

---

## T3: runes/grafana/grafana.fav

```favnir
public fn create_annotation(dashboard_id: String, text: String, tags: String) -> Result<Unit, String> !Io =
    Grafana.create_annotation_raw(dashboard_id, text, tags)
public fn push_dashboard(json: String) -> Result<Unit, String> !Io =
    Grafana.push_dashboard_raw(json)
public fn snapshot(dashboard_id: String) -> Result<String, String> !Io =
    Grafana.snapshot_raw(dashboard_id)
```

---

## T4: examples/observability/grafana_dashboard.fav

```favnir
import runes/grafana

stage RecordDeploy: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Grafana.create_annotation("main-dashboard", "Deploy v28.6.0", "deploy,favnir")
    Result.ok(unit)
}
stage UpdateDashboard: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Grafana.push_dashboard("{\"title\": \"ETL Pipeline\", \"panels\": []}")
    Result.ok(unit)
}
seq GrafanaDashboardDemo = RecordDeploy |> UpdateDashboard
```

---

## T5: site/content/docs/runes/grafana.mdx

- frontmatter: `title: grafana Rune`, `description: Grafana ダッシュボード管理 Rune（v28.6.0）`
- 関数一覧テーブル（create_annotation / push_dashboard / snapshot のシグネチャ）
- 使用例（`create_annotation` + `push_dashboard` を `bind` で連鎖）
- エフェクト: `!Io`（Grafana HTTP API への送信）
- 注記: API キー（`[grafana]` セクション）は v28.7+、スナップショット URL はスタブ固定値、wasm32 非対応

---

## T9a: checker.fav — ns_to_effect に Grafana 追加

v28.5.0 時点の Sentry else ブロック末尾:

```favnir
if ns == "Sentry" {
    "IO"
} else {
    ""
}
```

v28.6.0 後（Sentry else ブロック内の `""` を Grafana 条件に置き換え）:

```favnir
if ns == "Sentry" {
    "IO"
} else {
    if ns == "Grafana" {
        "IO"
    } else {
        ""
    }
}
```

**重要**: `""` を直接 `if ns == "Grafana" { "IO" } else { "" }` に置き換える。
Sentry の else ブロック閉じ括弧との整合に注意。

---

## T8: driver.rs — v286000_tests

```rust
// ── v286000_tests (v28.6.0) — grafana Rune 追加 ────────────────────────────
#[cfg(test)]
mod v286000_tests {
    #[test]
    fn grafana_rune_has_create_annotation_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn create_annotation("), "grafana rune must define fn create_annotation(");
    }
    #[test]
    fn grafana_rune_has_push_dashboard_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn push_dashboard("), "grafana rune must define fn push_dashboard(");
    }
    #[test]
    fn grafana_rune_has_snapshot_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn snapshot("), "grafana rune must define fn snapshot(");
    }
    #[test]
    fn grafana_rune_uses_io_effect() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("!Io"), "grafana rune must use !Io effect");
    }
    #[test]
    fn vm_has_grafana_create_annotation_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Grafana.create_annotation_raw"), "vm.rs must implement Grafana.create_annotation_raw");
    }
    #[test]
    fn grafana_example_has_pipeline() {
        let src = include_str!("../../examples/observability/grafana_dashboard.fav");
        assert!(src.contains("GrafanaDashboardDemo"), "grafana_dashboard.fav must define GrafanaDashboardDemo seq");
    }
    #[test]
    fn checker_has_grafana_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"Grafana\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"Grafana\"' and map it to \"IO\""
        );
    }
    #[test]
    fn changelog_has_v28_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.6.0]") || src.contains("## v28.6.0"), "CHANGELOG.md must contain '[v28.6.0]'");
    }
    #[test]
    fn grafana_doc_exists() {
        let src = include_str!("../../site/content/docs/runes/grafana.mdx");
        assert!(src.contains("Grafana"), "grafana.mdx must mention Grafana");
    }
}
```

`include_str!` パス一覧:
- `../../runes/grafana/grafana.fav` — `fav/src/` → `fav/` → ルート → `runes/grafana/`
- `../../fav/self/checker.fav` — `fav/src/` → `fav/` → ルート → `fav/self/`
- `../../examples/observability/grafana_dashboard.fav`
- `../../CHANGELOG.md`
- `../../site/content/docs/runes/grafana.mdx`
- `backend/vm.rs` — 同じ `src/` 内
