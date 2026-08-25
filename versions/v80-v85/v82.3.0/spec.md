# v82.3.0 — パイプライン間契約依存（`ContractDependency`）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 3 版。
v82.1.0 で定義した `IoContract` を使い、複数パイプラインが「上流の契約出力を
下流の入力として使う」依存関係を型で表現する。

`DependencyGraph` は契約間の有向エッジを保持し、
`detect_circular_dependencies` でサイクルを検出できる。
これにより「上流パイプラインが変わると下流が壊れる」リスクを型レベルで管理する。

---

## Goals

1. `ContractDependency` 構造体を定義する（`upstream: String`, `downstream: String`, `output_contract: String`）
2. `DependencyGraph` 構造体を定義する（`dependencies: Vec<ContractDependency>`）
3. `build_dependency_graph(contracts: &[IoContract]) -> DependencyGraph` を実装する
   - 各 IoContract の output フィールド名が別の IoContract の input フィールド名と一致する場合にエッジを作成する
4. `detect_circular_dependencies(graph: &DependencyGraph) -> Vec<Vec<String>>` を実装する
   - v82.3.0 スコープ: 2 ノードサイクル（A→B かつ B→A）の検出のみ対応。長いサイクル（A→B→C→A）は将来バージョンで対応予定。
5. `format_dependency_graph(graph: &DependencyGraph) -> String` を実装する

---

## API Examples（Rust テストコード）

```rust
// build_dependency_graph: output フィールド名が一致 → エッジ生成
let field_id = ContractField { name: "id".into(), field_type: ContractFieldType::Int, required: true };
let field_name = ContractField { name: "name".into(), field_type: ContractFieldType::Str, required: true };

let contract_a = IoContract {
    name: "orders".into(),
    version: "1.0.0".into(),
    input: vec![],
    output: vec![field_id.clone()],  // output: id
};
let contract_b = IoContract {
    name: "reports".into(),
    version: "1.0.0".into(),
    input: vec![field_id.clone()],   // input: id → orders に依存
    output: vec![],
};

let graph = build_dependency_graph(&[contract_a, contract_b]);
assert_eq!(graph.dependencies.len(), 1);
assert_eq!(graph.dependencies[0].upstream, "orders");
assert_eq!(graph.dependencies[0].downstream, "reports");

// detect_circular_dependencies: A→B かつ B→A → サイクル検出
let dep_ab = ContractDependency { upstream: "A".into(), downstream: "B".into(), output_contract: "A".into() };
let dep_ba = ContractDependency { upstream: "B".into(), downstream: "A".into(), output_contract: "B".into() };
let circular_graph = DependencyGraph { dependencies: vec![dep_ab, dep_ba] };
let cycles = detect_circular_dependencies(&circular_graph);
assert!(!cycles.is_empty(), "サイクルが検出されるはず");

// format_dependency_graph
let formatted = format_dependency_graph(&graph);
assert!(formatted.contains("orders"), "グラフ文字列に 'orders' が含まれるはず");
```

### `build_dependency_graph` のエッジ生成ロジック

- contracts を全ペア (i, j) で走査（i ≠ j）
- `contracts[i].output` の field name 群と `contracts[j].input` の field name 群に共通要素があれば、`i` → `j` のエッジを追加
  - `upstream = contracts[i].name`
  - `downstream = contracts[j].name`
  - `output_contract = contracts[i].name`

### `detect_circular_dependencies` の検出ロジック

- `dependencies` から隣接リスト（`HashMap<String, Vec<String>>`）を構築
- DFS でバックエッジを検出（訪問済みノードへの到達 = サイクル）
- 検出したサイクルを `Vec<Vec<String>>` で返す（各内側 Vec はサイクルのノード列）

### `format_dependency_graph` の出力形式

```
orders -> reports (orders)
```

エッジごとに `"{upstream} -> {downstream} ({output_contract})"` を改行区切りで結合。
エッジが空の場合は空文字列を返す。

---

## Success Criteria

- `cargo test` 全 pass（3,871 tests = 3,869 + 2）
- 新規テスト 2 件（`v82300_tests` モジュール）:
  - `dependency_graph_built_from_contracts`
  - `circular_dependency_detected`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `ContractDependency` / `DependencyGraph` / `build_dependency_graph` / `detect_circular_dependencies` / `format_dependency_graph` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82300_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.3.0 エントリを先頭に追加 |
