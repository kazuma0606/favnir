# Spec: v86.0.0 — SAP Foundation 1.0 宣言 ★クリーンアップ

## Background

v85.1.0〜v85.9.0 で SAP Foundation スプリントの全機能を実装した。
本バージョンでは宣言文を刻み、`cargo clean` / バージョン更新 / ドキュメント整備のクリーンアップを行う。

**宣言文:**
> 「SAP に、型安全に接続できるようになった。
>  `fav.toml [sap]` を書けば、Favnir が SAP OData v4 と話せる。」

**SAP Foundation 1.0（v85.1〜v85.9）達成内容:**
- **v85.1**: `SapTomlConfig` + `inject_sap_config()`（Rust 基盤）
- **v85.2**: `SapConfig` Favnir 型 + `sap_config_from_env()`
- **v85.3**: Docker Compose — SAP OData モックサーバー構築
- **v85.4**: `runes/sap-odata/` 骨格 + `rune.toml`
- **v85.5**: OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`）
- **v85.6**: `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク）
- **v85.7**: `fav new` テンプレート + `fav.toml [sap]` セクション追加
- **v85.8**: SSM Parameter Store 設定（`infra/sap/`）
- **v85.9**: 安定化・コードフリーズ

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v86.0.0 セクション）

## Goals

- Cargo.toml バージョンを `86.0.0` に更新する
- driver.rs 内の `version = \"85.0.0\"` アサーション（35 件）を `86.0.0` に一括更新する
- CHANGELOG.md / MILESTONE.md / README.md / `versions/current.md` を更新する
- `v86000_tests` 4 件を追加して **3,953 tests** を達成する
- `cargo clean` を実施する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 修正 | `version = "85.0.0"` → `version = "86.0.0"` |
| `fav/src/driver.rs` | 修正 | `version = \"85.0.0\"` アサーション 35 件を `86.0.0` に一括更新 |
| `fav/src/driver.rs` | 追記 | `mod v86000_tests`（テスト 4 件） |
| `CHANGELOG.md` | 追記 | v86.0.0 エントリ（先頭に追加） |
| `MILESTONE.md` | 追記 | SAP Foundation 1.0 エントリ（先頭に追加） |
| `README.md` | 修正 | v86.0 セクション追加（SAP Integration 言及） |
| `versions/current.md` | 修正 | v86.0.0 に更新 |

## `mod v86000_tests` 内容

```rust
#[cfg(test)]
mod v86000_tests {
    #[test]
    fn cargo_toml_version_is_86_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("version = \"86.0.0\""),
            "Cargo.toml should have version = \"86.0.0\""
        );
    }

    #[test]
    fn changelog_has_v86_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v86.0.0"), "CHANGELOG.md should mention v86.0.0");
    }

    #[test]
    fn milestone_has_sap_foundation() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("SAP Foundation"),
            "MILESTONE.md should mention SAP Foundation"
        );
    }

    #[test]
    fn readme_mentions_sap_integration() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("SAP"),
            "README.md should mention SAP integration"
        );
    }
}
```

## Success Criteria

- `cargo test` が **3,953 tests**, 0 failures（前バージョン 3,949 から +4 の増加。削除・スキップがないことを確認）
- `cargo_toml_version_is_86_0_0`: Cargo.toml に `version = "86.0.0"` が含まれる
- `changelog_has_v86_0_0`: CHANGELOG.md に `v86.0.0` が含まれる
- `milestone_has_sap_foundation`: MILESTONE.md に `SAP Foundation` が含まれる
- `readme_mentions_sap_integration`: README.md に `SAP` が含まれる

## Error Codes

新規エラーコードなし。

## 注記

- `cargo clean` 後は `fav/tmp/hello.fav` が消えないことを確認する（target/ のみ削除）
- `version = \"85.0.0\"` の一括更新は `sed` ではなく Edit ツールの `replace_all: true` で行う
- CHANGELOG の v86.0.0 エントリを追加した後に `cargo test changelog_has_v86_0_0` が通ることを確認してから `v86000_tests` モジュールを追加する（順序注意）
