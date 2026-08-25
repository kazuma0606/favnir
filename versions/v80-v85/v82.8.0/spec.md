# v82.8.0 — 契約レジストリ（`ContractRegistry` / ローカルキャッシュ）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 8 版。
v82.1.0〜v82.7.0 で構築した `IoContract` / `SlaContract` / `ContractVersion` / `cmd_verify_contract` を
チーム間で共有・検索・バージョン管理するローカルレジストリ層を追加する。

`ContractRegistry` は `ContractRegistryEntry` のコレクションとして実装し、
`register` / `lookup` / `list_all` の 3 操作を提供する。
`format_registry_listing` で登録内容を一覧表示できる。

---

## Goals

1. `ContractRegistryEntry` 構造体を定義する（`name: String`, `version: ContractVersion`, `contract: IoContract`, `registered_at: String`）
2. `ContractRegistry` 構造体を定義する（`entries: Vec<ContractRegistryEntry>`）
3. `ContractRegistry::new() -> ContractRegistry` を実装する（空レジストリを作成）
4. `ContractRegistry::register(&self, entry: ContractRegistryEntry) -> ContractRegistry` を実装する
   - 既存エントリをクローンし、末尾に `entry` を追加した新しい `ContractRegistry` を返す
5. `ContractRegistry::lookup(&self, name: &str, version: Option<&str>) -> Option<&ContractRegistryEntry>` を実装する
   - `version` が Some のとき: `ContractVersion::parse(v)` でパースし `name` とバージョンが一致するエントリを返す
   - `version` が None のとき: `name` が一致するエントリのうち最後に登録されたものを返す
6. `ContractRegistry::list_all(&self) -> Vec<&ContractRegistryEntry>` を実装する
7. `format_registry_listing(registry: &ContractRegistry) -> String` を実装する

---

## API Examples（Rust テストコード）

```rust
let version = ContractVersion { major: 1, minor: 0, patch: 0 };
let contract = IoContract {
    name: "orders".into(), version: "1.0.0".into(),
    input: vec![], output: vec![],
};
let entry = ContractRegistryEntry {
    name: "orders".into(),
    version: version.clone(),
    contract: contract.clone(),
    registered_at: "2026-08-20T00:00:00Z".into(),
};

let registry = ContractRegistry::new();
let registry = registry.register(entry.clone());

// lookup: バージョン指定あり
let found = registry.lookup("orders", Some("1.0.0"));
assert!(found.is_some());
assert_eq!(found.unwrap().name, "orders");

// lookup: バージョン指定なし（最後に登録されたもの）
let found2 = registry.lookup("orders", None);
assert!(found2.is_some());

// list_all
let all = registry.list_all();
assert_eq!(all.len(), 1);

// format
let s = format_registry_listing(&registry);
assert!(s.contains("orders"));
assert!(s.contains("1.0.0"));
```

### `ContractRegistryEntry` フィールド

| フィールド | 型 | 説明 |
|---|---|---|
| `name` | `String` | 契約名（検索キー） |
| `version` | `ContractVersion` | セマンティックバージョン |
| `contract` | `IoContract` | 本体の契約 |
| `registered_at` | `String` | 登録日時（ISO 8601 文字列） |

### `ContractRegistry::register` の設計

- `self` を消費せず `&self` で受け取り、新しい `ContractRegistry` を返す（不変更新スタイル）
- 重複登録チェックは本バージョンでは行わない（同名・同バージョンを複数登録可能）

### `ContractRegistry::lookup` の `version` 引数

- `Some("1.0.0")` → `major=1, minor=0, patch=0` のエントリにマッチ
- `None` → 同名エントリが複数あれば最後（インデックス最大）のものを返す

### `format_registry_listing` の出力形式

```
Registry (2 entries):
  orders v1.0.0 — registered_at: 2026-08-20T00:00:00Z
  payments v2.1.0 — registered_at: 2026-08-20T01:00:00Z
```

- `"Registry ({n} entries):"` を 1 行目に出力
- 各エントリは `"  {name} v{major}.{minor}.{patch} — registered_at: {registered_at}"` 形式

---

## Success Criteria

- `cargo test` 全 pass（3,881 tests = 3,879 + 2）※ drift 補正後
- 新規テスト 2 件（`v82800_tests` モジュール）:
  - `contract_registry_register_and_lookup`
  - `contract_registry_list_all`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `ContractRegistryEntry` / `ContractRegistry` / `register` / `lookup` / `list_all` / `format_registry_listing` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82800_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.8.0 エントリを先頭に追加 |
