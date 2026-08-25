# v82.6.0 実装計画

## 方針

**前提**: v82.5.0 完了済み（3,875 tests pass）。

`test_framework.rs` に契約バージョニング型・関数を追加し、`driver.rs` に `v82600_tests` を追加する。
`IoContract` / `ContractField` / `ContractFieldType` は v82.1.0 で定義済み。

---

## 実装ステップ

### Step 1: `ContractVersion` 構造体を追加

`fav/src/test_framework.rs` の v82.5.0 セクション末尾に続けて追加する。

```rust
// ── v82.6.0: ContractVersion / CompatibilityResult ───────────────────────────

/// セマンティックバージョニング形式の契約バージョン。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ContractVersion {
    /// `"major.minor.patch"` 形式の文字列をパースする。
    /// フォーマット不正な場合は `Err("invalid version: {s}")` を返す。
    pub fn parse(s: &str) -> Result<ContractVersion, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("invalid version: {s}"));
        }
        let parse_u32 = |p: &str| {
            p.parse::<u32>().map_err(|_| format!("invalid version: {s}"))
        };
        Ok(ContractVersion {
            major: parse_u32(parts[0])?,
            minor: parse_u32(parts[1])?,
            patch: parse_u32(parts[2])?,
        })
    }
}
```

### Step 2: `CompatibilityResult` enum を追加

```rust
/// 契約の後方互換性チェック結果。
#[derive(Debug, PartialEq)]
pub enum CompatibilityResult {
    /// 変更なし。
    Compatible,
    /// 既存消費者への影響なしの変更（非必須フィールドの追加など）。
    BackwardsCompatible(Vec<String>),
    /// 既存消費者を壊す変更（必須フィールドの削除・型変更など）。
    Breaking(Vec<String>),
}
```

### Step 3: `check_contract_compatibility` を実装

優先度: Breaking > BackwardsCompatible > Compatible。

```rust
/// 2 つの `IoContract` の input フィールドを比較して後方互換性を判定する。
///
/// 判定順序（優先度降順）:
/// 1. Breaking: old の required フィールドが new_ に存在しない
/// 2. Breaking: 同名フィールドの型が変わった
/// 3. BackwardsCompatible: new_ に old にないフィールドが追加された
/// 4. Compatible: 変更なし
pub fn check_contract_compatibility(old: &IoContract, new_: &IoContract) -> CompatibilityResult {
    let mut breaking: Vec<String> = Vec::new();

    // 1. required フィールドが削除された
    for old_field in &old.input {
        if old_field.required && !new_.input.iter().any(|f| f.name == old_field.name) {
            breaking.push(old_field.name.clone());
        }
    }

    // 2. 同名フィールドの型が変わった
    for old_field in &old.input {
        if let Some(new_field) = new_.input.iter().find(|f| f.name == old_field.name) {
            if new_field.field_type != old_field.field_type {
                breaking.push(old_field.name.clone());
            }
        }
    }

    if !breaking.is_empty() {
        breaking.sort();
        breaking.dedup();
        return CompatibilityResult::Breaking(breaking);
    }

    // 3. 非必須フィールドが追加された
    let added: Vec<String> = new_
        .input
        .iter()
        .filter(|nf| !old.input.iter().any(|of| of.name == nf.name))
        .map(|nf| nf.name.clone())
        .collect();

    if !added.is_empty() {
        return CompatibilityResult::BackwardsCompatible(added);
    }

    CompatibilityResult::Compatible
}
```

### Step 4: `format_compatibility_result` を実装

```rust
/// `CompatibilityResult` を人間が読める文字列に変換する。
pub fn format_compatibility_result(result: &CompatibilityResult) -> String {
    match result {
        CompatibilityResult::Compatible => "Compatible".into(),
        CompatibilityResult::BackwardsCompatible(fields) => {
            format!("BackwardsCompatible: added [{}]", fields.join(", "))
        }
        CompatibilityResult::Breaking(fields) => {
            format!("Breaking: [{}]", fields.join(", "))
        }
    }
}
```

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.6.0 エントリを追加する。

### Step 6: `v82600_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82600_tests` を追加する。

- `contract_version_parsed`:
  - `"1.2.3"` → `major=1, minor=2, patch=3` を確認
  - 不正フォーマット → `Err` を確認
  - `format_compatibility_result(&CompatibilityResult::Compatible)` が `"Compatible"` を返すことを確認
- `breaking_change_detected_on_field_removal`:
  - old に必須フィールドが 2 つ、new_ に 1 つ（削除）のコントラクトで `Breaking` が返ることを確認
  - `format_compatibility_result` の出力に削除フィールド名が含まれることを確認

### Step 7: `cargo test` 全通過確認

3,877 tests pass（+2）、0 failures であることを確認する。
