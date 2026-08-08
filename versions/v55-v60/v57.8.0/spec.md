# Spec — v57.8.0 — ドキュメントサイト Enterprise Security 記事

## 概要

`site/content/docs/enterprise/` ディレクトリを新規作成し、Enterprise Security の 3 記事を追加する。
`driver.rs` に `v57800_tests` を追加して 2 件の Rust テストで各 MDX ファイルの存在と内容を検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.8.0 セクション
- ベーステスト数: **3267**（v57.7.0 完了時点の実績値）
- 目標テスト数: **3269**（+2）、かつ `cargo test` failures=0

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.8.0"
```

---

### 2. サイト MDX ファイル作成（3 件）

#### 2-1: `site/content/docs/enterprise/rbac.mdx`

RBAC 設定・ロールバインディング・checker 統合について記述。
必須キーワード（テスト検証対象）:
- `"RBAC"` — ページタイトルまたは見出し
- `"roles"` — ロール設定例
- `"bindings"` — ロールバインディング設定例
- `"E0424"` — RBAC アクセス拒否エラーコード

#### 2-2: `site/content/docs/enterprise/secrets.mdx`

シークレット管理・Vault / AWS SM 連携手順について記述。
必須キーワード（内容上必要、テスト対象外）:
- `"secrets"` — シークレット管理概要
- `"aws-secrets-manager"` / `"vault"` — プロバイダ設定例

#### 2-3: `site/content/docs/enterprise/compliance.mdx`

コンプライアンスレポート・GDPR / SOC2 対応について記述。
必須キーワード（テスト検証対象）:
- `"GDPR"` — フレームワーク名
- `"SOC2"` — フレームワーク名

---

### 3. `fav/src/driver.rs` — `v57800_tests` 追加

`v57700_tests` の直前に挿入する。

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

---

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.7.0" → "57.8.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.7.0" → "57.8.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.7.0" → "57.8.0"（rolling）
```

> `v57100_tests` 〜 `v57700_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `docs_rbac_page_exists` | `rbac.mdx` の存在・`RBAC` / `roles` / `bindings` / `E0424` キーワードを検証 |
| `docs_compliance_page_exists` | `compliance.mdx` の存在・`GDPR` / `SOC2` キーワードを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3269 tests passed, 0 failed**、ベース 3267 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57800_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.8.0]` エントリが追加されている
- `versions/current.md` が v57.8.0 / 3269 tests を反映

---

## 備考

- `site/content/docs/enterprise/` ディレクトリは v57.8.0 で新規作成
- `secrets.mdx` はコンテンツとして作成するが、対応する `docs_secrets_page_exists` テストは今バージョンのスコープ外（ロードマップの完了条件 2 件のみ対応）
- `include_str!` のパスは `fav/src/driver.rs` から見て `../../site/content/docs/enterprise/rbac.mdx`
- `v57100_tests` 〜 `v57700_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
