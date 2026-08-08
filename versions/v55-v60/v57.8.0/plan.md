# Plan — v57.8.0 — ドキュメントサイト Enterprise Security 記事

## 実装方針

v24.7.0 の `include_str!` テストパターンを踏襲する。
`site/content/docs/enterprise/` ディレクトリを新規作成し、3 MDX ファイルを作成後、
`v57800_tests` で `include_str!` によるコンパイル時ファイル存在チェックを行う。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `57.7.0` → `57.8.0` |
| `site/content/docs/enterprise/rbac.mdx` | 新規作成（RBAC 設定記事） |
| `site/content/docs/enterprise/secrets.mdx` | 新規作成（シークレット管理記事） |
| `site/content/docs/enterprise/compliance.mdx` | 新規作成（コンプライアンスレポート記事） |
| `fav/src/driver.rs` | `v57800_tests` 追加、バージョンチェックテスト 3 件更新 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
57.7.0 → 57.8.0
```

### Step 2: MDX ファイル 3 件作成

#### 2-1: `site/content/docs/enterprise/rbac.mdx`

テスト検証キーワード: `RBAC` / `roles` / `bindings` / `E0424` が含まれること。

    # RBAC — ロールベースアクセス制御

    Favnir は `fav.toml` の `[security.rbac]` セクションで
    ロールベースアクセス制御（RBAC）を設定できます。

    ## 設定例

        [security.rbac]
        roles = ["reader", "writer", "admin"]

        [security.rbac.bindings]
        "kafka"     = ["reader", "writer", "admin"]
        "snowflake" = ["writer", "admin"]

    ## ロールバインディング

    `bindings` セクションでは Rune 名とアクセス可能なロールの一覧を指定します。
    `roles` に列挙されていないロールは `E0424`（RBAC アクセス拒否）エラーになります。

    ## エラーコード

    | コード | 内容 |
    |---|---|
    | E0424 | RBAC アクセス拒否 — 現在のロールに Rune へのアクセス権がありません |

#### 2-2: `site/content/docs/enterprise/secrets.mdx`

    # シークレット管理

    Favnir は `fav.toml` の `[secrets]` セクションで
    AWS Secrets Manager / HashiCorp Vault からシークレットを取得できます。

    ## 設定例

        [secrets]
        provider = "aws-secrets-manager"
        region   = "ap-northeast-1"

        [secrets.bindings]
        SNOWFLAKE_PASSWORD = "prod/snowflake/password"
        KAFKA_API_KEY      = "prod/kafka/api-key"

    ## 対応プロバイダ

    | プロバイダ | 設定値 |
    |---|---|
    | AWS Secrets Manager | `aws-secrets-manager` |
    | HashiCorp Vault | `vault` |

#### 2-3: `site/content/docs/enterprise/compliance.mdx`

テスト検証キーワード: `GDPR` / `SOC2` が含まれること。

    # コンプライアンスレポート

    Favnir は GDPR・SOC2 フレームワークに対応したコンプライアンスレポートを生成できます。

    ## 対応フレームワーク

    ### GDPR

    データアクセスログと削除記録を含む GDPR レポートを生成します。

    ### SOC2

    アクセス制御と監査証跡を含む SOC2 レポートを生成します。

### Step 3: `driver.rs` — `v57800_tests` 挿入

`v57700_tests` の直前に挿入:

```rust
// -- v57800_tests (v57.8.0) -- ドキュメントサイト Enterprise Security 記事 --
#[cfg(test)]
mod v57800_tests {
    #[test]
    fn docs_rbac_page_exists() {
        let content = include_str!("../../site/content/docs/enterprise/rbac.mdx");
        assert!(content.contains("RBAC"), "rbac.mdx should mention RBAC");
        assert!(content.contains("roles"), "rbac.mdx should mention roles");
        assert!(content.contains("bindings"), "rbac.mdx should mention bindings");
        assert!(content.contains("E0424"), "rbac.mdx should mention E0424 error code");
    }

    #[test]
    fn docs_compliance_page_exists() {
        let content = include_str!("../../site/content/docs/enterprise/compliance.mdx");
        assert!(content.contains("GDPR"), "compliance.mdx should mention GDPR");
        assert!(content.contains("SOC2"), "compliance.mdx should mention SOC2");
    }
}
```

### Step 4: バージョンチェックテスト更新（rolling）

| テスト | 変更前 | 変更後 |
|---|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | `"57.7.0"` | `"57.8.0"` |
| `v56900_tests::cargo_toml_version_is_56_9_0` | `"57.7.0"` | `"57.8.0"` |
| `v57000_tests::cargo_toml_version_is_57_0_0` | `"57.7.0"` | `"57.8.0"` |

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3269 tests passed, 0 failed`（ベース 3267 + 2）

---

## ポスト処理

1. `CHANGELOG.md` に `[v57.8.0]` エントリ追加
2. `versions/current.md` を v57.8.0 / 3269 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.8.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` のテスト数推移テーブルに v57.8.0 行を追加
