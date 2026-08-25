# v82.8.0 実装計画 — 契約レジストリ（`ContractRegistry` / ローカルキャッシュ）

---

## 実装ステップ

### Step 1: `ContractRegistryEntry` 構造体追加

`fav/src/test_framework.rs` に追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ContractRegistryEntry {
    pub name: String,
    pub version: ContractVersion,
    pub contract: IoContract,
    pub registered_at: String,
}
```

依存: `ContractVersion`（v82.6.0）、`IoContract`（v82.1.0）

---

### Step 2: `ContractRegistry` 構造体と `new()` 追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ContractRegistry {
    pub entries: Vec<ContractRegistryEntry>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        ContractRegistry { entries: vec![] }
    }
}
```

---

### Step 3: `register` メソッド追加

```rust
impl ContractRegistry {
    pub fn register(&self, entry: ContractRegistryEntry) -> ContractRegistry {
        let mut entries = self.entries.clone();
        entries.push(entry);
        ContractRegistry { entries }
    }
}
```

---

### Step 4: `lookup` メソッド追加

```rust
impl ContractRegistry {
    pub fn lookup(&self, name: &str, version: Option<&str>) -> Option<&ContractRegistryEntry> {
        match version {
            Some(v) => {
                // バージョン文字列をパースして完全一致
                let target = ContractVersion::parse(v).ok()?;
                self.entries.iter().find(|e| e.name == name && e.version == target)
            }
            None => {
                // 同名の最後のエントリ
                self.entries.iter().rev().find(|e| e.name == name)
            }
        }
    }
}
```

---

### Step 5: `list_all` メソッド追加

```rust
impl ContractRegistry {
    pub fn list_all(&self) -> Vec<&ContractRegistryEntry> {
        self.entries.iter().collect()
    }
}
```

---

### Step 6: `format_registry_listing` 関数追加

```rust
pub fn format_registry_listing(registry: &ContractRegistry) -> String {
    let mut lines = vec![format!("Registry ({} entries):", registry.entries.len())];
    for e in &registry.entries {
        lines.push(format!(
            "  {} v{}.{}.{} — registered_at: {}",
            e.name, e.version.major, e.version.minor, e.version.patch, e.registered_at
        ));
    }
    lines.join("\n")
}
```

---

### Step 7: CHANGELOG 更新

`CHANGELOG.md` 先頭に v82.8.0 エントリを追加する。

---

### Step 8: `v82800_tests` テストモジュール追加

`fav/src/driver.rs` 末尾に追加:

```rust
#[cfg(test)]
mod v82800_tests {
    use fav_core::test_framework::*;

    #[test]
    fn contract_registry_register_and_lookup() {
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

        // バージョン指定ありで lookup
        let found = registry.lookup("orders", Some("1.0.0"));
        assert!(found.is_some(), "バージョン指定で lookup できるはず");
        assert_eq!(found.unwrap().name, "orders");

        // バージョン指定なしで lookup（最後のエントリ）
        let found2 = registry.lookup("orders", None);
        assert!(found2.is_some(), "バージョン指定なしで lookup できるはず");

        // 存在しない名前は None
        let not_found = registry.lookup("nonexistent", None);
        assert!(not_found.is_none(), "存在しない名前は None のはず");

        // format に名前とバージョンが含まれる
        let s = format_registry_listing(&registry);
        assert!(s.contains("orders"), "listing に 'orders' が含まれるはず: {s}");
        assert!(s.contains("1.0.0"), "listing に '1.0.0' が含まれるはず: {s}");
    }

    #[test]
    fn contract_registry_list_all() {
        let v1 = ContractVersion { major: 1, minor: 0, patch: 0 };
        let v2 = ContractVersion { major: 2, minor: 1, patch: 0 };
        let c1 = IoContract { name: "orders".into(), version: "1.0.0".into(), input: vec![], output: vec![] };
        let c2 = IoContract { name: "payments".into(), version: "2.1.0".into(), input: vec![], output: vec![] };

        let e1 = ContractRegistryEntry {
            name: "orders".into(), version: v1, contract: c1,
            registered_at: "2026-08-20T00:00:00Z".into(),
        };
        let e2 = ContractRegistryEntry {
            name: "payments".into(), version: v2, contract: c2,
            registered_at: "2026-08-20T01:00:00Z".into(),
        };

        let registry = ContractRegistry::new()
            .register(e1)
            .register(e2);

        let all = registry.list_all();
        assert_eq!(all.len(), 2, "2件登録したので list_all は 2 件のはず");
        assert_eq!(all[0].name, "orders");
        assert_eq!(all[1].name, "payments");

        let s = format_registry_listing(&registry);
        assert!(s.contains("Registry (2 entries)"), "ヘッダに件数が含まれるはず: {s}");
        assert!(s.contains("payments"), "listing に 'payments' が含まれるはず: {s}");
        assert!(s.contains("2.1.0"), "listing に '2.1.0' が含まれるはず: {s}");
    }
}
```

---

### Step 9: テスト通過確認

`cargo test` を実行し 3,881 tests pass（+2）を確認する。

---

## 依存関係

```
ContractVersion (v82.6.0)
    └── ContractRegistryEntry
            └── ContractRegistry::new / register / lookup / list_all
                    └── format_registry_listing
                            └── v82800_tests
```
