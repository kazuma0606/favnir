# Plan: v90.0.0 — SAP Integration 1.0 宣言

## 実装ステップ

### Step 1: `cargo test` で現在の状態確認

```bash
cd fav && cargo test 2>&1 | grep "test result"
```

4,037 tests, 0 failures を確認する。

### Step 2: `cargo clean` でビルドキャッシュをクリーンアップ

```bash
cargo clean
```

target/ ディレクトリを削除してクリーンな状態にする。
`fav/tmp/hello.fav` は消えないが、存在確認しておく。

### Step 3: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "89.0.0"` を `version = "90.0.0"` に変更する。

### Step 4: `driver.rs` の `cargo_toml_version_is_` テストを一括更新

driver.rs 内に存在する `"89.0.0"` 文字列（実測 42 件）を `90.0.0` に一括更新する:

```bash
# 確認
grep -c "89\.0\.0" src/driver.rs

# 一括置換（"89.0.0" → "90.0.0"）
sed -i 's/89\.0\.0/90.0.0/g' src/driver.rs
```

※ `"version = \"89.0.0\""` という文字列を含む行が対象。

### Step 5: `CHANGELOG.md` に v90.0.0 エントリを追加

`CHANGELOG.md` の先頭（`## [v89.0.0]` の前）に追加:

```markdown
## [v90.0.0] — 2026-08-25 — SAP Integration 1.0 宣言

> 「SAP が、Favnir の型になった。
>  `business_partners()` で得意先を取得し、
>  `sales_orders()` で受注を集計し、
>  `materials()` で在庫を確認し、
>  `journal_entries()` で支払を照合する。
>  世界最大の ERP データが、型安全なパイプラインとして流れる。
>  それが、Favnir SAP Integration 1.0 である。」

### Added
- `fav/src/driver.rs` — `mod v90000_tests`（テスト 4 件）を追加
- `MILESTONE.md` — SAP Integration 1.0 マイルストーンを追加
- 合計テスト数: **4,041**（+4）

### Changed
- `fav/Cargo.toml` — version を `89.0.0` → `90.0.0` に更新
- `versions/current.md` — v90.0.0 に更新

---
```

### Step 6: `MILESTONE.md` に SAP Integration 1.0 を追加

`MILESTONE.md` の最新マイルストーンとして追加:

```markdown
## SAP Integration 1.0 — v90.0.0（2026-08-25）

> 「SAP が、Favnir の型になった。」

- `business_partners()` / `sales_orders()` / `materials()` / `journal_entries()` による 4 業務シナリオ
- `fav infer --from sap --entity <name>` によるエンティティ型生成
- E2E デモ（全 4 シナリオ + Lambda デプロイ）
- `site/content/docs/runes/sap-odata.mdx` ドキュメント
- OSS 整備（CONTRIBUTING + ISSUE_TEMPLATE）
- パフォーマンス計測（benchmarks/sap-odata-v89.8.0.json）
```

### Step 7: `README.md` に SAP Integration 言及を追加

README.md の機能一覧または最新リリースセクションに `SAP Integration` を追加する。

### Step 8: `versions/current.md` を更新

v89.0.0 → v90.0.0 に更新する。

### Step 8.5: `roadmap-v85.1-v90.0.md` の全エントリを完了マークに更新

`versions/roadmap/roadmap-v85.1-v90.0.md` の Status を全て「完了」に更新する。
具体的には以下の行を更新する:
- `Status: 未着手` → `Status: 完了`
- バージョン一覧表の「未着手」→「完了」

### Step 9: `mod v90000_tests` を `driver.rs` に追加

`mod v89900_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v90000_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn cargo_toml_version_is_90_0_0() {
        let content = std::fs::read_to_string("Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(content.contains("version = \"90.0.0\""),
            "Cargo.toml should have version 90.0.0");
    }

    #[test]
    fn changelog_has_v90_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("v90.0.0"),
            "CHANGELOG.md should mention v90.0.0");
    }

    #[test]
    fn milestone_has_sap_integration() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(content.contains("SAP Integration"),
            "MILESTONE.md should mention SAP Integration");
    }

    #[test]
    fn readme_mentions_sap_integration() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(content.contains("SAP Integration"),
            "README.md should mention SAP Integration");
    }
}
```

### Step 10: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

4,037 + 4 = 4,041 tests, 0 failures を確認する。

### Step 11: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG の追加（Step 5）は `mod v90000_tests` 追加（Step 9）より前に行う。
`changelog_has_v90_0_0` テストが `cargo test` で通るには CHANGELOG の更新が先に必要。
