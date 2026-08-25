# Plan: v89.7.0 — OSS 整備

## 実装ステップ

### Step 1: `CONTRIBUTING.md` に SAP Rune エンティティ追加手順を追記

ファイル末尾に以下を追記する:

```markdown
---

## SAP Rune — 新エンティティの追加手順

sap-odata Rune に新しい SAP OData エンティティを追加する場合の手順:

### 1. 型定義ファイルの作成

`runes/sap-odata/<entity>.fav` を新規作成する:

```favnir
-- <EntityName> 型定義（v89.x.x）
use sap_odata.types

public type <EntityName> = {
    key_field: String,
    -- 必要なフィールドを追加
}

public type <EntityName>Filter = {
    top: Option<Int>
    -- フィルタフィールドを追加
}

public fn <entity_name>s(cfg: SapConfig, filter: <EntityName>Filter) -> Result<List<<EntityName>>, String> {
    Result.err("not implemented")
}
```

### 2. 関数実装（スタブから実装へ）

型定義ファイル内のスタブ（`Result.err("not implemented")`）を実際の OData 呼び出しに置き換える:

```favnir
public fn <entity_name>s(cfg: SapConfig, filter: <EntityName>Filter) -> Result<List<<EntityName>>, String> {
    -- OData エンドポイント呼び出し実装
    Result.err("not implemented")
}
```

### 3. `sap_odata.fav` への re-export 追加

`runes/sap-odata/sap_odata.fav` に追加する:

```favnir
use sap_odata.<entity>

public type <EntityName>       = <entity>.<EntityName>
public type <EntityName>Filter = <entity>.<EntityName>Filter
public fn <entity_name>s(cfg: SapConfig, filter: <EntityName>Filter) -> Result<List<<EntityName>>, String> {
    <entity>.<entity_name>s(cfg, filter)
}
```

### 4. `fav/src/driver.rs` テスト追加

`mod v<version>_tests` に 2 件のテストを追加する:
- `<entity_name>_type_defined_in_rune`: 型定義の存在確認
- `<entity_name>_function_exists`: 関数の存在確認

### 5. Registry デプロイ

```bash
/deploy-registry
```

詳細は `site/content/docs/runes/sap-odata.mdx` を参照。
```

### Step 2: `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` を作成

`quality-feedback.md` と同形式で以下を作成する:

```markdown
---
name: SAP Integration Feedback
about: SAP OData Rune（sap-odata）に関するフィードバック・不具合報告
title: "[SAP] "
labels: sap-integration, feedback
assignees: ""
---

## フィードバック種別

- [ ] エンティティ取得の誤動作
- [ ] 型定義の不一致
- [ ] 認証エラー
- [ ] 新エンティティのリクエスト
- [ ] その他

## 詳細

（詳細を記述してください）

## 再現手順

（再現手順があれば記述してください）

## 環境情報

- Favnir バージョン:
- SAP バージョン（S/4HANA Cloud / Business One / ECC）:
```

### Step 3: `mod v89700_tests` を `driver.rs` に追加

`mod v89600_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89700_tests {
    #[test]
    fn contributing_has_sap_section() {
        let content = std::fs::read_to_string("../CONTRIBUTING.md")
            .expect("CONTRIBUTING.md should exist");
        assert!(
            content.contains("SAP Rune"),
            "CONTRIBUTING.md should contain SAP Rune section"
        );
    }

    #[test]
    fn issue_template_sap_feedback_exists() {
        assert!(
            std::path::Path::new("../.github/ISSUE_TEMPLATE/sap-integration-feedback.md").exists(),
            ".github/ISSUE_TEMPLATE/sap-integration-feedback.md should exist"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,031 + 2 = 4,033 tests, 0 failures を確認する。

### Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
