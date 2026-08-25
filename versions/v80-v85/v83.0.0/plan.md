# v83.0.0 実装計画 — Pipeline Contracts 1.0 宣言 ★クリーンアップ

---

## 実装ステップ

### Step 1: CHANGELOG 更新

`CHANGELOG.md` 先頭に v83.0.0 エントリを追加する。
（`changelog_has_v83_0_0` テストがこのステップの完了で pass できるようにする）

---

### Step 2: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "82.0.0"` を `version = "83.0.0"` に変更する。

---

### Step 3: `MILESTONE.md` 更新

Pipeline Contracts 1.0 達成の宣言文を追加する。
以下の内容を含む:
- "Pipeline Contracts 1.0" という文字列
- 宣言文テキスト
- 完了日付

---

### Step 4: `README.md` 更新

`ContractRegistry` への言及を追加する。
例: "Pipeline Contracts 1.0 により `ContractRegistry` でチーム間の契約を共有できるようになりました。"

---

### Step 5: `v83000_tests` テストモジュール追加

`fav/src/driver.rs` 末尾に 4 件のテストを追加する:

```rust
#[cfg(test)]
mod v83000_tests {
    #[test]
    fn cargo_toml_version_is_83_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"83.0.0\""), "Cargo.toml のバージョンが 83.0.0 のはず");
    }

    #[test]
    fn changelog_has_v83_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v83.0.0"), "CHANGELOG に v83.0.0 が含まれるはず");
    }

    #[test]
    fn milestone_has_pipeline_contracts() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Pipeline Contracts"), "MILESTONE に 'Pipeline Contracts' が含まれるはず");
    }

    #[test]
    fn readme_mentions_contract_registry() {
        let content = include_str!("../../README.md");
        assert!(content.contains("ContractRegistry"), "README に 'ContractRegistry' が含まれるはず");
    }
}
```

---

### Step 6: `cargo test` 実行

3,887 tests pass（+4）を確認する。

---

### Step 7: `cargo clean` 実施

build artifacts をクリアする。

---

### Step 8: `versions/current.md` 更新

現行バージョンを v83.0.0 に更新する。

---

### Step 9: `roadmap-v80.1-v85.0.md` 更新

Sprint 3（v82.1〜v83.0）バージョン一覧テーブルを全行「完了」に更新する。

---

## 依存関係

```
CHANGELOG 更新（Step 1）
    └── changelog_has_v83_0_0 テスト（Step 5）

Cargo.toml 更新（Step 2）
    └── cargo_toml_version_is_83_0_0 テスト（Step 5）

MILESTONE 更新（Step 3）
    └── milestone_has_pipeline_contracts テスト（Step 5）

README 更新（Step 4）
    └── readme_mentions_contract_registry テスト（Step 5）

Step 5 テスト → Step 6 cargo test → Step 7 cargo clean
```

## 注意事項

- `cargo clean` は Step 6 の後に実施する（テスト通過を確認してから）
- `v83000_tests` は `use super::*` 不要（外部シンボル未使用 — `include_str!` マクロのみ使用）
