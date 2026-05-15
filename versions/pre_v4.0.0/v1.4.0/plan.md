# Favnir v1.4.0 実装計画

作成日: 2026-05-07

> スコープを守ることが最優先。各フェーズの Done definition を超えない。
>
> **前提**: v1.3.0 完了
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-explain-bundle.md`、`dev/post-v1/roadmap/favnir-graph-explain.md`

---

## 実装順序

```
Phase 0 (version bump)
  → Phase 1 (explain --format json)     ← explain JSON の基盤
  → Phase 2 (reachability analysis)     ← Phase 1 の reachable_from_entry を埋める / Phase 3 の前提
  → Phase 3 (fav bundle + manifest)     ← Phase 2 完了後
  → Phase 4 (bundle --explain + artifact explain)  ← Phase 1, 2, 3 完了後
  → Phase 5 (fav graph)                 ← Phase 1 完了後、独立
  → Phase 6 (trf 第一級値 + 動的注入)  ← Phase 3 と並行可
  → Phase 7 (abstract trf ジェネリック) ← Phase 6 と並行可
  → Phase 8 (テスト・ドキュメント)
```

---

## Phase 0: バージョン更新

```toml
# Cargo.toml
version = "1.4.0"
```

```rust
// main.rs HELP テキスト
"fav - Favnir language toolchain v1.4.0
...
  bundle <file> [-o <out>] [--manifest] [--explain]  bundle minimal artifact
  graph  <file> [--format text|mermaid] [--focus flw NAME]  visualize flw structure
..."
```

---

## Phase 1: `fav explain --format json`

### 1-1: CLI 拡張（`main.rs`）

```rust
"explain" => {
    let format = args.get("--format").map(|s| s.as_str()).unwrap_or("text");
    let focus  = args.get("--focus").cloned();
    let schema = args.contains("--schema");
    driver::cmd_explain(&file, schema, format, focus.as_deref());
}
```

`cmd_explain` のシグネチャ変更:
```rust
pub fn cmd_explain(file: &str, schema: bool, format: &str, focus: Option<&str>);
```

### 1-2: `ExplainJson` 構造体の定義（`driver.rs` または `src/explain.rs`）

serde_json の `Serialize` derive で JSON 出力:

```rust
#[derive(Serialize)]
pub struct ExplainJson {
    pub schema_version:   &'static str,
    pub favnir_version:   &'static str,
    pub entry:            String,
    pub source:           String,
    pub fns:              Vec<FnEntry>,
    pub trfs:             Vec<TrfEntry>,
    pub flws:             Vec<FlwEntry>,
    pub types:            Vec<TypeEntry>,
    pub effects_used:     Vec<String>,
    pub emits:            Vec<String>,
    pub runes_used:       Vec<String>,
}

#[derive(Serialize)]
pub struct FnEntry {
    pub name:                 String,
    pub kind:                 &'static str,   // "fn"
    pub params:               Vec<ParamEntry>,
    pub return_type:          String,
    pub effects:              Vec<String>,
    pub calls:                Vec<String>,
    pub reachable_from_entry: bool,
}

#[derive(Serialize)]
pub struct TrfEntry {
    pub name:                 String,
    pub kind:                 String,   // "trf" | "abstract_trf"
    pub input_type:           String,
    pub output_type:          String,
    pub effects:              Vec<String>,
    pub calls:                Option<Vec<String>>,  // abstract_trf は None
    pub reachable_from_entry: bool,
}

#[derive(Serialize)]
pub struct FlwEntry {
    pub name:                 String,
    pub kind:                 String,   // "flw" | "flw_binding" | "abstract_flw"
    pub input_type:           Option<String>,
    pub output_type:          Option<String>,
    pub effects:              Vec<String>,
    // flw / flw_binding
    pub steps:                Option<Vec<String>>,
    // flw_binding
    pub template:             Option<String>,
    pub type_args:            Option<Vec<String>>,
    pub bindings:             Option<HashMap<String, String>>,
    // abstract_flw
    pub type_params:          Option<Vec<String>>,
    pub slots:                Option<Vec<SlotEntry>>,
    pub reachable_from_entry: bool,
}

#[derive(Serialize)]
pub struct TypeEntry {
    pub name:       String,
    pub kind:       String,   // "record" | "sum"
    pub fields:     Option<Vec<FieldEntry>>,
    pub variants:   Option<Vec<String>>,
    pub invariants: Vec<String>,
}
```

### 1-3: `render_json` の実装

`ExplainPrinter` または `ExplainJsonBuilder` に以下を追加:

```rust
pub fn render_json(
    program: &Program,
    reachability: Option<&ReachabilityResult>,  // None → 全て true
    focus: Option<&str>,
) -> String {
    let mut explain = ExplainJson {
        schema_version: "1.0",
        favnir_version: "1.4.0",
        entry:  "main".into(),
        source: "".into(),
        fns:    build_fn_entries(program, reachability),
        trfs:   build_trf_entries(program, reachability),
        flws:   build_flw_entries(program, reachability),
        types:  build_type_entries(program),
        effects_used: collect_effects_used(program, reachability),
        emits:        vec![],
        runes_used:   vec![],
    };

    // --focus で絞り込み
    if let Some(focus) = focus {
        apply_focus(&mut explain, focus);
    }

    serde_json::to_string_pretty(&explain).unwrap()
}
```

各 `build_*_entries` は `program.items` をイテレートして対応する `Item` バリアントから情報を抽出する。

### 1-4: `calls` の収集

`fn` / `trf` / `flw_binding` の `calls` フィールドは、IR の `IRExpr::Global` / `IRExpr::Call` を再帰的にスキャンして収集:

```rust
fn collect_calls_in_ir(fn_def: &IRFnDef) -> Vec<String> {
    let mut calls = HashSet::new();
    collect_calls_expr(&fn_def.body, &mut calls);
    calls.into_iter().collect()
}
```

既存の `collect_deps` 関数（Phase 4 `fav explain` の DEPS 列）を再利用・拡張できる。

---

## Phase 2: 到達可能性解析

### 2-1: `src/middle/reachability.rs` の新規作成

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use crate::backend::ir::{IRProgram, IRFnDef, IRExpr, IRStmt};

pub struct ReachabilityResult {
    pub included:         HashSet<String>,
    pub excluded:         HashSet<String>,
    pub effects_required: Vec<String>,
    pub emits:            Vec<String>,
}

pub fn reachability_analysis(entry: &str, program: &IRProgram) -> ReachabilityResult {
    // fn_defs を名前 → IRFnDef のマップに変換
    let fn_map: HashMap<&str, &IRFnDef> = program.fn_defs.iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    queue.push_back(entry.to_string());

    while let Some(name) = queue.pop_front() {
        if visited.contains(&name) { continue; }
        visited.insert(name.clone());
        if let Some(fn_def) = fn_map.get(name.as_str()) {
            for dep in collect_calls_in_ir(fn_def) {
                if !visited.contains(&dep) {
                    queue.push_back(dep);
                }
            }
        }
    }

    let all_names: HashSet<String> = fn_map.keys().map(|s| s.to_string()).collect();
    let excluded: HashSet<String> = all_names.difference(&visited).cloned().collect();

    let effects_required: Vec<String> = visited.iter()
        .filter_map(|name| fn_map.get(name.as_str()))
        .flat_map(|f| f.effects.iter().map(|e| format!("{:?}", e)))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    ReachabilityResult {
        included: visited,
        excluded,
        effects_required,
        emits: vec![],
    }
}
```

### 2-2: `src/backend/ir.rs` の公開範囲確認

`IRFnDef.effects` フィールドが `pub` であることを確認（既存）。
`collect_calls_in_ir` を `ir.rs` または `reachability.rs` に実装。

---

## Phase 3: `fav bundle`

### 3-1: `cmd_bundle` の実装（`driver.rs`）

```rust
pub fn cmd_bundle(file: &str, out: Option<&str>, entry: &str, manifest: bool, explain: bool) {
    // 1. パース・型検査・IR 生成
    let (source, program, ir) = load_ir_program(file);

    // 2. 到達可能性解析
    let reachability = reachability_analysis(entry, &ir);

    // 3. minimal artifact 生成（included の関数だけを含む）
    let minimal_ir = filter_ir_program(&ir, &reachability.included);
    let artifact = build_artifact_from_ir(&minimal_ir)?;

    // 4. 出力パスの決定
    let out_path = out.map(String::from).unwrap_or_else(|| {
        format!("dist/{}.fvc", Path::new(file).file_stem().unwrap().to_str().unwrap())
    });
    ensure_dist_dir(&out_path);

    // 5. --manifest
    if manifest {
        let manifest_json = build_manifest_json(file, &out_path, &artifact, &reachability)?;
        let manifest_path = out_path.replace(".fvc", ".manifest.json");
        fs::write(&manifest_path, manifest_json)?;
        println!("manifest: {}", manifest_path);
    }

    // 6. --explain (Phase 4 で実装)
    if explain {
        let explain_json = render_json(&program, Some(&reachability), None);
        let explain_path = out_path.replace(".fvc", ".explain.json");
        fs::write(&explain_path, explain_json)?;
        println!("explain:  {}", explain_path);
    }

    write_artifact_to_path(&artifact, &out_path)?;
    println!("bundle:   {} ({} bytes)", out_path, artifact.len());
}
```

### 3-2: `filter_ir_program`

```rust
fn filter_ir_program(ir: &IRProgram, included: &HashSet<String>) -> IRProgram {
    IRProgram {
        fn_defs: ir.fn_defs.iter()
            .filter(|f| included.contains(&f.name))
            .cloned()
            .collect(),
        globals: ir.globals.iter()
            .filter(|g| included.contains(&g.name))
            .cloned()
            .collect(),
        // 他のフィールドはそのままコピー
        ..ir.clone()
    }
}
```

### 3-3: `build_manifest_json`

```rust
fn build_manifest_json(
    source: &str,
    artifact_path: &str,
    artifact: &[u8],
    r: &ReachabilityResult,
) -> String {
    let manifest = serde_json::json!({
        "schema_version": "1.0",
        "favnir_version":  "1.4.0",
        "entry":           "main",
        "source":          source,
        "artifact":        artifact_path,
        "artifact_size":   artifact.len(),
        "built_at":        chrono::Utc::now().to_rfc3339(),
        "included":        sorted_vec(&r.included),
        "excluded":        sorted_vec(&r.excluded),
        "effects_required": &r.effects_required,
        "emits":           &r.emits,
        "runes_used":      [],
    });
    serde_json::to_string_pretty(&manifest).unwrap()
}
```

`chrono` が依存に存在しない場合は `built_at` を固定文字列でも可（テスト用）。
→ Cargo.toml に `chrono = { version = "0.4", features = ["serde"] }` を追加。

---

## Phase 4: `fav bundle --explain` + artifact explain

### 4-1: `.fvc` フォーマットの拡張（`artifact.rs`）

```rust
pub struct FvcArtifact {
    // ... 既存フィールド ...
    pub explain_json: Option<String>,  // 追加
}

// マジックバイト / セクションヘッダの拡張
// 既存のセクション構造末尾に EXPLAIN セクションを追加:
const EXPLAIN_SECTION_MAGIC: &[u8; 4] = b"EXPL";
```

**書き込み**: `FvcWriter::write_explain_section(json: &str)` を追加。
**読み込み**: `FvcArtifact::read_explain_section` — `EXPL` マジックを探して JSON を読み込む。
後方互換: `EXPL` セクションがない場合 `explain_json = None`（既存 .fvc も読める）。

### 4-2: `fav bundle --explain` でのメタデータ埋め込み

```rust
// cmd_bundle に追加
if explain {
    let json = render_json(&program, Some(&reachability), None);
    artifact.explain_json = Some(json.clone());
    // explain.json ファイルも別途書き出す
    fs::write(&explain_path, &json)?;
}
write_artifact_to_path(&artifact, &out_path)?;
```

### 4-3: `fav explain dist/app.fvc`

`cmd_explain` でファイル拡張子が `.fvc` の場合:

```rust
if file.ends_with(".fvc") {
    let artifact = read_artifact_from_path(file)?;
    match (format, &artifact.explain_json) {
        ("json", Some(json)) => println!("{}", json),
        ("text", _)          => println!("{}", build_explain_skeleton(&artifact)),
        ("json", None)       => eprintln!("error: artifact has no embedded explain metadata"),
        _ => {}
    }
    return;
}
```

---

## Phase 5: `fav graph`（v1.3.0 残件）

### 5-1: `cmd_graph` の実装（`driver.rs`）

```rust
pub fn cmd_graph(file: &str, format: &str, focus: Option<&str>) {
    let (_, program) = load_and_parse_program(file);
    match format {
        "mermaid" => println!("{}", render_graph_mermaid(&program, focus)),
        _          => println!("{}", render_graph_text(&program, focus)),
    }
}
```

### 5-2: text レンダラー

```rust
fn render_graph_text(program: &Program, focus: Option<&str>) -> String {
    let mut out = String::new();
    for item in &program.items {
        match item {
            Item::FlwBindingDef(fd) if focus.map_or(true, |f| fd.name == f) => {
                let template = find_abstract_flw(&program, &fd.template);
                writeln!(out, "flw {} ({})", fd.name, fd.template).ok();
                for slot in template.map(|t| &t.slots).unwrap_or(&vec![]) {
                    let impl_name = fd.bindings.iter()
                        .find(|(s, _)| s == &slot.name)
                        .map(|(_, i)| i.as_str())
                        .unwrap_or("(unbound)");
                    let effs = format_effects(&slot.effects);
                    writeln!(out, "  [{:<10}] <- {:<20} : {} -> {}{}",
                        slot.name, impl_name,
                        format_type_expr(&slot.input_ty),
                        format_type_expr(&slot.output_ty), effs
                    ).ok();
                }
                // 解決済みシグネチャ
                writeln!(out).ok();
            }
            Item::FlwDef(fd) if focus.map_or(true, |f| fd.name == f) => {
                writeln!(out, "flw {}: {} steps", fd.name, fd.steps.len()).ok();
                for (i, step) in fd.steps.iter().enumerate() {
                    let sep = if i + 1 < fd.steps.len() { " ->" } else { "" };
                    write!(out, "  {}{}", step, sep).ok();
                }
                writeln!(out).ok();
            }
            _ => {}
        }
    }
    out
}
```

### 5-3: mermaid レンダラー

```rust
fn render_graph_mermaid(program: &Program, focus: Option<&str>) -> String {
    let mut out = String::from("flowchart LR\n");
    for item in &program.items {
        if let Item::FlwBindingDef(fd) = item {
            if focus.map_or(true, |f| fd.name == f) {
                let template = find_abstract_flw(&program, &fd.template);
                if let Some(t) = template {
                    for i in 0..t.slots.len() {
                        if i + 1 < t.slots.len() {
                            let from = &t.slots[i].name;
                            let to   = &t.slots[i+1].name;
                            let from_impl = impl_for_slot(fd, from);
                            let to_impl   = impl_for_slot(fd, to);
                            writeln!(out, "  {}[\"{}\"] --> {}[\"{}\"]", from, from_impl, to, to_impl).ok();
                        }
                    }
                }
            }
        }
    }
    out
}
```

---

## Phase 6: trf 第一級値 + 動的注入（v1.3.0 残件）

### 6-1: `TypeExpr` の拡張（`ast.rs`）

関数型の引数として `A -> B !Fx` を書けるよう `TypeExpr::TrfFn` を追加:

```rust
pub enum TypeExpr {
    // ... 既存 ...
    /// trf 型を値型として使う: `A -> B !Fx`
    TrfFn {
        input:   Box<TypeExpr>,
        output:  Box<TypeExpr>,
        effects: Vec<String>,
    },
}
```

### 6-2: パーサーの拡張（`parser.rs`）

`parse_param_type` で `->` を含む型式を `TypeExpr::TrfFn` としてパース:

```rust
// fn f(save: UserRow -> Int !Io)
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ パラメータ型として TrfFn を受け入れる
fn parse_fn_param_type(&mut self) -> Result<TypeExpr, ParseError> {
    let base = self.parse_type_expr()?;
    if self.peek() == &TokenKind::Arrow {
        self.advance();
        let output  = self.parse_type_expr()?;
        let effects = self.parse_effects()?;
        return Ok(TypeExpr::TrfFn {
            input:   Box::new(base),
            output:  Box::new(output),
            effects,
        });
    }
    Ok(base)
}
```

### 6-3: `SlotImpl` の追加（`ast.rs`）

```rust
pub enum SlotImpl {
    Global(String),  // グローバルな trf/fn 名
    Local(String),   // ローカル変数（関数引数 or bind 束縛値）
}

// FlwBindingDef の bindings を変更
pub struct FlwBindingDef {
    // ...
    pub bindings: Vec<(String, SlotImpl)>,
}
```

パーサーで束縛の右辺（`slot <- Impl`）が型環境にある場合 `SlotImpl::Local`、グローバルなら `SlotImpl::Global` とする。
この判断は型検査フェーズで行う（パーサーは文字列として保持し、チェッカーで解決）。

実用上は: パーサーでは一旦 `SlotImpl::Global(name)` として保持し、チェッカーが `env.lookup(name)` でローカル変数か確認して `Local` に変換する。

### 6-4: チェッカーの変更（`checker.rs`）

`check_flw_binding_def` でスロット実装の解決:

```rust
fn resolve_slot_impl(&self, name: &str) -> SlotImpl {
    // ローカルスコープに存在する → Local（関数引数など）
    if self.env.is_local(name) {
        SlotImpl::Local(name.to_string())
    } else {
        SlotImpl::Global(name.to_string())
    }
}
```

型照合は `SlotImpl::Local` の場合も `SlotImpl::Global` の場合も同じロジック（E048 条件は変わらない）。

### 6-5: VM 拡張（`vm.rs`）

```rust
// VMValue に追加
TrfRef(String),  // グローバル trf / fn への参照

// vm_call_builtin にはなく、CALL ハンドラで TrfRef を処理
VMValue::TrfRef(fn_name) => {
    self.call_fn(&fn_name, args)
}
```

### 6-6: コンパイラの変更（`compiler.rs`）

`compile_flw_binding_def` で `SlotImpl::Local(name)` の場合:

```rust
SlotImpl::Local(local_name) => {
    // ローカル変数に格納された trf 参照を経由して呼び出す
    IRExpr::CallTrfLocal {
        local: local_name.clone(),
        arg:   Box::new(compiled_input),
    }
}
```

---

## Phase 7: `abstract trf` ジェネリック型パラメータ（v1.3.0 残件）

### 7-1: `AbstractTrfDef` の変更（`ast.rs`）

```rust
pub struct AbstractTrfDef {
    pub visibility:  Option<Visibility>,
    pub name:        String,
    pub type_params: Vec<String>,  // 追加: `<T, U>` など
    pub input_ty:    TypeExpr,
    pub output_ty:   TypeExpr,
    pub effects:     Vec<Effect>,
    pub span:        Span,
}
```

### 7-2: パーサーの変更（`parser.rs`）

```rust
fn parse_abstract_trf_def(&mut self, vis: Option<Visibility>) -> Result<AbstractTrfDef, ParseError> {
    // ...
    let type_params = self.parse_type_params_opt()?;  // `<T>` or `<A, B>` or empty
    self.expect(&TokenKind::Colon)?;
    // ... 以下既存 ...
}
```

### 7-3: チェッカーの変更（`checker.rs`）

ジェネリック `AbstractTrfDef` を `abstract_trf_registry` に登録。
`check_flw_binding_def` でスロット型が `AbstractTrf<T>` の場合、`T` を束縛の型引数から推論して照合:

```rust
// AbstractTrf<T> で T = Row のとき、スロット期待型を具体化
fn resolve_abstract_trf_slot_type(
    &self,
    trf_name: &str,
    type_arg: &Type,   // FlwBindingDef の type_args から
) -> Option<(Type, Type, Vec<Effect>)> {
    let trf_def = self.abstract_trf_registry.get(trf_name)?;
    let subst = build_subst(&trf_def.type_params, &[type_arg.clone()]);
    let input  = apply_subst(&trf_def.input_ty,  &subst);
    let output = apply_subst(&trf_def.output_ty, &subst);
    Some((input, output, trf_def.effects.clone()))
}
```

---

## Phase 8: テスト・ドキュメント

### テスト追加場所

- `driver.rs` の `#[cfg(test)]`（explain JSON, bundle, graph テスト）
- `middle/reachability.rs` の `#[cfg(test)]`（到達可能性テスト）
- `middle/checker.rs` の `#[cfg(test)]`（動的注入・abstract trf ジェネリックテスト）

### example ファイル

```fav
// examples/bundle_demo.fav
// fav bundle の動作確認: unreachable コードが bundleに含まれないことを確認

fn unused_helper() -> Int {
    42
}

fn add(a: Int, b: Int) -> Int {
    a + b
}

public fn main() -> Unit !Io {
    bind result <- add(10, 20)
    IO.println_int(result)
}
// unused_helper は bundle 後の .fvc に含まれない
```

```fav
// examples/dynamic_inject.fav
// trf を関数引数として注入する動的注入パターン

type Row = { value: Int }

abstract flw Pipeline<T> {
    transform: T -> T!
    render:    T -> String
}

trf DoubleValue: Row -> Row! = |r| {
    Result.ok(Row { value: r.value * 2 })
}

trf ShowValue: Row -> String = |r| {
    Int.show.show(r.value)
}

fn make_pipeline(render: Row -> String) -> flw Row -> String {
    bind p <- Pipeline<Row> {
        transform <- DoubleValue
        render    <- render
    }
    p
}

public fn main() -> Unit !Io {
    bind pipeline <- make_pipeline(ShowValue)
    bind input    <- Row { value: 5 }
    bind result   <- pipeline(input)
    IO.println(result)
}
```

### `versions/v1.4.0/langspec.md` の作成

v1.3.0 langspec を起点に追加:
- `fav explain --format json` スキーマ仕様（JSON フィールド一覧）
- `fav bundle` コマンド（`--manifest` / `--explain` フラグ）
- `manifest.json` スキーマ
- `fav graph` コマンド
- trf 第一級値の構文（`fn f(save: A -> B !Fx)`）
- `abstract trf` ジェネリック構文
- artifact explain（`fav explain app.fvc`）

---

## 先送り一覧

| 制約 | バージョン |
|---|---|
| `fav explain diff` 専用コマンド（explain JSON の CI 差分） | v1.5.0 以降 |
| `fav graph --focus fn` での fn 依存グラフ | v1.5.0 以降 |
| artifact の gzip 圧縮（explain metadata の圧縮） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` へのリネーム（JSON 含む） | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking） | v2.0.0 以降 |
