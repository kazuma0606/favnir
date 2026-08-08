# Plan — v57.9.0 — 安定化・コードフリーズ（Enterprise Security 前調整）

## 実装方針

`v57900_tests` を `v57800_tests` の直前に挿入する（最新優先の慣例に従う）。
`cargo_toml_version_is_57_9_0` は v57.0.0 と同形式の rolling チェックとして作成する。
`enterprise_security_overview_exists` は `include_str!` によるコンパイル時ファイル存在検証。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `57.8.0` → `57.9.0` |
| `site/content/docs/enterprise-security-overview.mdx` | 新規作成（Enterprise Security 概要骨子） |
| `fav/src/driver.rs` | `v57900_tests` 追加、バージョンチェックテスト 3 件更新 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
57.8.0 → 57.9.0
```

### Step 2: `site/content/docs/enterprise-security-overview.mdx` 作成

```mdx
# Enterprise Security — 概要

Favnir v57.0 スプリントで実装した Enterprise Security 機能群の概要。

## 機能一覧

| 機能 | バージョン | 説明 |
|---|---|---|
| RBAC | v57.1.0 | ロールベースアクセス制御 |
| Secrets 管理 | v57.2.0 | AWS SM / Vault 連携 |
| TLS / mTLS | v57.3.0 | HTTP / gRPC Rune 証明書設定 |
| 依存関係スキャン | v57.4.0 | CVE スキャン・--fail-on-high |
| 監査ログ署名 | v57.5.0 | HMAC-SHA256 署名・tamper-proof |
| compliance レポート | v57.6.0 | GDPR / SOC2 対応 |
| マルチテナント | v57.7.0 | テナント識別子・strict モード |
| ドキュメント | v57.8.0 | Enterprise Security 記事群 |

## 詳細ドキュメント

- [RBAC](./enterprise/rbac)
- [シークレット管理](./enterprise/secrets)
- [コンプライアンスレポート](./enterprise/compliance)
```

テスト検証キーワード: `"Enterprise Security"` / `"RBAC"` / `"TLS"` / `"compliance"`

### Step 3: `driver.rs` — `v57900_tests` 挿入

`v57800_tests` の直前に挿入:

```rust
// -- v57900_tests (v57.9.0) -- 安定化・コードフリーズ --
#[cfg(test)]
mod v57900_tests {
    #[test]
    fn cargo_toml_version_is_57_9_0() {
        // rolling check: function name is frozen at v57.9.0 by convention,
        // but this assertion is updated each release to the current version.
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"57.9.0\""),
            "Cargo.toml version should be 57.9.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn enterprise_security_overview_exists() {
        let content = include_str!("../../site/content/docs/enterprise-security-overview.mdx");
        assert!(content.contains("Enterprise Security"), "overview should mention Enterprise Security");
        assert!(content.contains("RBAC"), "overview should mention RBAC");
        assert!(content.contains("TLS"), "overview should mention TLS");
        assert!(content.contains("compliance"), "overview should mention compliance");
    }
}
```

### Step 4: バージョンチェックテスト更新（rolling）

| テスト | 変更前 | 変更後 |
|---|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | `"57.8.0"` | `"57.9.0"` |
| `v56900_tests::cargo_toml_version_is_56_9_0` | `"57.8.0"` | `"57.9.0"` |
| `v57000_tests::cargo_toml_version_is_57_0_0` | `"57.8.0"` | `"57.9.0"` |

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3272 tests passed, 0 failed`（ベース 3270 + 2）

```bash
cargo clippy -- -D warnings
```

期待: 警告ゼロ（`Finished` のみ出力）

---

## ポスト処理

1. `CHANGELOG.md` に `[v57.9.0]` エントリ追加
2. `versions/current.md` を v57.9.0 / 3272 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.9.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` のテスト数推移テーブルに v57.9.0 行を追加（v57.8.0 行の直後）
