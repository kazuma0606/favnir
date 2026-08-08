# Spec — v57.9.0 — 安定化・コードフリーズ（Enterprise Security 前調整）

## 概要

v57.1〜v57.8 の全機能が正しく通過することを確認し、
`site/content/docs/enterprise-security-overview.mdx` 骨子を作成する。
`v57900_tests` に `cargo_toml_version_is_57_9_0`（以降のバージョンで rolling 更新される）と
`enterprise_security_overview_exists` の 2 件を追加する。

> **ベーステスト数注記**: ロードマップ記載は「ベース 3269」だが、
> v57.8.0 の code-review 対応で `docs_secrets_page_exists` が追加され実績は **3270**。
> したがって目標は **3272**（3270 + 2）とする。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.9.0 セクション
- ベーステスト数: **3270**（v57.8.0 完了時点の実績値）
- 目標テスト数: **3272**（+2）、かつ `cargo test` failures=0

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.9.0"
```

---

### 2. `site/content/docs/enterprise-security-overview.mdx` 作成

Enterprise Security スプリント（v57.1〜v57.8）で実装した機能群の概要骨子。

必須キーワード（テスト検証対象）:
- `"Enterprise Security"` — ページタイトル
- `"RBAC"` — v57.1.0 機能
- `"TLS"` — v57.3.0 機能
- `"compliance"` — v57.6.0 機能

---

### 3. `fav/src/driver.rs` — `v57900_tests` 追加

`v57800_tests` の直前に挿入する。

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

---

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.8.0" → "57.9.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.8.0" → "57.9.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.8.0" → "57.9.0"（rolling）
```

> `v57100_tests` 〜 `v57800_tests` には `cargo_toml_version_is_*` がないため更新不要。
> `v57900_tests::cargo_toml_version_is_57_9_0` は今バージョンで新規追加のため今回は更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_57_9_0` | `Cargo.toml` に `version = "57.9.0"` が含まれることを検証（rolling） |
| `enterprise_security_overview_exists` | `enterprise-security-overview.mdx` の存在・Enterprise Security / RBAC / TLS / compliance キーワードを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3272 tests passed, 0 failed**、ベース 3270 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57900_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.9.0]` エントリが追加されている
- `versions/current.md` が v57.9.0 / 3272 tests を反映

---

## 備考

- `cargo_toml_version_is_57_9_0` は以降のバージョンで rolling 更新される（v56300 / v56900 / v57000 / v57900 の 4 件が rolling 更新対象となる）
- T0 確認で `v57100_tests`〜`v57800_tests` に予期しない `cargo_toml_version_is_*` が発見された場合、rolling 更新対象リストに加えて更新する（テスト数への影響はなし・既存テストの更新のみ）
- `enterprise-security-overview.mdx` は `site/content/docs/` 直下に配置（`enterprise/` サブディレクトリではない）
- `include_str!` パスは `../../site/content/docs/enterprise-security-overview.mdx`
- ロードマップ記載ベース（3269）と実績ベース（3270）の差異は v57.8.0 の code-review 対応（+1 test）によるもの
