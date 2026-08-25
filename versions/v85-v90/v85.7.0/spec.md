# Spec: v85.7.0 — `fav new` テンプレート + `fav.toml [sap]` セクション追加

## Background

v85.1.0 で `fav.toml [sap]` の Rust 解析基盤を実装した。
本バージョンでは `fav new` コマンドが生成する `fav.toml` テンプレートに `[sap]` セクションのコメントを追加し、
開発者が SAP 接続設定をすぐに参照できるようにする。
既存の `[snowflake]` コメントブロックと同じパターンで追加する。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.7.0 セクション）

## Goals

- `fav/src/driver.rs` の `default_fav_toml()` に `[sap]` テンプレートコメントを追加する
- Rust テスト 2 件を追加して **3,945 tests** を達成する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 修正 | `default_fav_toml()` に `[sap]` コメントブロック追加 |
| `fav/src/driver.rs` | 追記 | `mod v85700_tests`（テスト 2 件） |

## `default_fav_toml()` 変更内容

現在の `default_fav_toml()` は `[snowflake]` コメントブロックで終わっている。
その後に `[sap]` コメントブロックを追加する。

```rust
fn default_fav_toml(name: &str) -> String {
    format!(
        "[project]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\nsrc     = \"src\"\n\n\
         # [snowflake]\n\
         # account   = \"${{SNOWFLAKE_ACCOUNT}}\"\n\
         # user      = \"${{SNOWFLAKE_USER}}\"\n\
         # warehouse = \"COMPUTE_WH\"\n\
         # database  = \"MY_DB\"\n\
         # schema    = \"PUBLIC\"\n\n\
         # [sap]\n\
         # base_url = \"${{SAP_BASE_URL}}\"   # SAP S/4HANA エンドポイント\n\
         # client   = \"100\"                  # SAP クライアント番号\n\
         # username = \"${{SAP_USER}}\"\n\
         # password = \"${{SAP_PASS}}\"\n\
         # auth     = \"basic\"                # \"basic\" | \"oauth2\"\n"
    )
}
```

## Success Criteria

- `cargo test` が **3,945 tests**, 0 failures
- `fav_new_template_contains_sap_comment`:
  - `default_fav_toml("test")` の出力に `"# [sap]"` が含まれる
- `sap_toml_section_parses_correctly`:
  - `parse_fav_toml_pub` に `[sap]\nbase_url = \"https://example.com\"\n` を渡すと `sap.base_url` が `Some("https://example.com")` になる
  - ※ v85100_tests の `sap_toml_config_parses_base_url` と実装内容は同等だが、v85.7.0 ではテンプレート変更後もパーサーが正しく動作するかを確認する明示的なリグレッションガードとして配置する（将来の削除防止のため意図的に重複させる）

## Error Codes

新規エラーコードなし。

## 注記

- `default_fav_toml()` の変更は `fav new` で生成されるファイルのみに影響する（既存プロジェクトの `fav.toml` は変更されない）
- Rust フォーマット文字列内の `{` は `{{`、`}` は `}}` にエスケープが必要
- テストのパス: `fav/src/driver.rs` 内のテスト（ファイル I/O 不要）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
