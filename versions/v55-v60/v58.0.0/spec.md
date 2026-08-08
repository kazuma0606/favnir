# Spec — v58.0.0 — Enterprise Security 宣言 ★クリーンアップ

## 概要

v57.1〜v57.9 で構築した Enterprise Security 基盤を宣言する。
`MILESTONE.md` に宣言文エントリを追加し、`README.md` を更新する。
`v58000_tests` 4 件で CHANGELOG / MILESTONE / README のすべてが正しく更新されたことを検証する。
最後に `cargo clean` を実行する（★クリーンアップ）。

**宣言文**（`MILESTONE.md` に追記）:
> 「アクセスはロールで制御され、シークレットはコードに現れず、
>  通信は mTLS で守られ、監査ログは改ざんできない。
>  コンプライアンスレポートはボタン一つで生成される。
>  Favnir は企業のセキュリティ要件を満たす言語になった。
>
>  これが Favnir v58.0 — Enterprise Security の姿である。」

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v58.0.0 セクション
- ベーステスト数: **3272**（v57.9.0 完了時点の実績値）— `roadmap-v57.1-v58.0.md` v57.9.0 実績欄より確定。v57.1.0 のテスト数（2 件 vs 3 件）に既存の不整合があるが v58.0.0 のベース値には影響しない
- 目標テスト数: **3276**（+4）、かつ `cargo test` failures=0

---

## スコープ外項目

| 項目 | 備考 |
|---|---|
| v57.1〜v57.9 の個別機能の新規実装 | すでに各バージョンで完了済み |
| `cargo clean` 後の再ビルド | ★クリーンアップ後は手動で `cargo build` を確認すれば十分 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "58.0.0"
```

### 2. `MILESTONE.md` — Enterprise Security エントリ追加

既存エントリの**先頭**に追加する（最新順）:

```markdown
## v58.0.0（2026-07-28）— Enterprise Security

> 「アクセスはロールで制御され、シークレットはコードに現れず、
>  通信は mTLS で守られ、監査ログは改ざんできない。
>  コンプライアンスレポートはボタン一つで生成される。
>  Favnir は企業のセキュリティ要件を満たす言語になった。
>
>  これが Favnir v58.0 — Enterprise Security の姿である。」

**Enterprise Security** の宣言バージョン。v57.1〜v57.9 の全機能統合を経て、
RBAC・シークレット管理・TLS/mTLS・監査ログ署名・コンプライアンスレポート・
マルチテナント分離の成熟を宣言する。

**v57.1〜v57.9 達成内容:**
- v57.1（RBAC）: ロールベースアクセス制御・E0424 エラーコード
- v57.2（Secrets 管理）: AWS SM / Vault 連携・実行時シークレット注入
- v57.3（TLS / mTLS）: HTTP / gRPC Rune 証明書設定・`is_mtls()` メソッド
- v57.4（依存関係スキャン）: CVE スキャン・`--fail-on-high`
- v57.5（監査ログ署名）: HMAC-SHA256 署名・tamper-proof audit
- v57.6（コンプライアンスレポート）: GDPR / SOC2 フレームワーク対応
- v57.7（マルチテナント分離）: `TenancyConfig` / strict モード・`is_strict()`
- v57.8（ドキュメント）: Enterprise Security 記事群（rbac / secrets / compliance）
- v57.9（安定化）: コードフリーズ・enterprise-security-overview.mdx
```

### 3. `README.md` — Enterprise Security 宣言の追記

README.md のマイルストーン/バージョン履歴テーブルに v58.0.0 行を追加し、
「Enterprise Security」を明示する。

### 4. `CHANGELOG.md` — v58.0.0 エントリ追加

> **重要**: `changelog_has_v58_0_0` テストが CHANGELOG を `include_str!` で読むため、
> テスト追加（Step 5）の**前**に CHANGELOG を更新すること。

### 5. `fav/src/driver.rs` — `v58000_tests` 追加

`v57900_tests` の直前に挿入する。

```rust
// -- v58000_tests (v58.0.0) -- Enterprise Security 宣言 --
#[cfg(test)]
mod v58000_tests {
    #[test]
    fn cargo_toml_version_is_58_0_0() {
        // rolling check: function name is frozen at v58.0.0 by convention,
        // but this assertion is updated each release to the current version.
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"58.0.0\""),
            "Cargo.toml version should be 58.0.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn changelog_has_v58_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("v58.0.0"),
            "CHANGELOG.md should have a v58.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_enterprise_security() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Enterprise Security"),
            "MILESTONE.md should contain Enterprise Security declaration"
        );
    }

    #[test]
    fn readme_mentions_enterprise_security() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Enterprise Security"),
            "README.md should mention Enterprise Security"
        );
    }
}
```

### 6. バージョンチェックテスト更新（rolling）

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.9.0" → "58.0.0"
v56900_tests::cargo_toml_version_is_56_9_0  : "57.9.0" → "58.0.0"
v57000_tests::cargo_toml_version_is_57_0_0  : "57.9.0" → "58.0.0"
v57900_tests::cargo_toml_version_is_57_9_0  : "57.9.0" → "58.0.0"
```

> `v57100_tests` 〜 `v57800_tests` には `cargo_toml_version_is_*` がないため更新不要。
> `v58000_tests::cargo_toml_version_is_58_0_0` は今バージョン新規追加のため今回は更新しない。

### 7. ★クリーンアップ（`cargo clean`）

すべてのテストが通過した後に実施する。
`cargo clean` の**前**に `fav/tmp/hello.fav` が存在することを確認してから実行すること（`cargo clean` は `target/` を削除するが `fav/tmp/hello.fav` は削除しない）。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_58_0_0` | `Cargo.toml` に `version = "58.0.0"` が含まれることを検証（rolling） |
| `changelog_has_v58_0_0` | `CHANGELOG.md` に `v58.0.0` が含まれることを検証 |
| `milestone_has_enterprise_security` | `MILESTONE.md` に `Enterprise Security` が含まれることを検証 |
| `readme_mentions_enterprise_security` | `README.md` に `Enterprise Security` が含まれることを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3276 tests passed, 0 failed**、ベース 3272 + 4）
- `cargo clippy -- -D warnings` クリーン
- `v58000_tests` 4 件全 pass
- `MILESTONE.md` に `"Enterprise Security"` 宣言文エントリが追加されている
- `CHANGELOG.md` に `## [v58.0.0]` エントリが追加されている
- `versions/current.md` が v58.0.0 / 3276 tests を反映
- `cargo clean` 完了（★クリーンアップ）

---

## 備考

- `changelog_has_v58_0_0` は `include_str!` コンパイル時評価のため、CHANGELOG の v58.0.0 エントリはテストコードを driver.rs に追加する**前**に記載する必要がある
- rolling チェック更新対象は v57.9.0 から **4 件**（v56300 / v56900 / v57000 / v57900）
- `README.md` はすでに「Enterprise Security」という文字列を含むため、テストは追加変更なしでも通過する。ただし v58.0.0 の完成を反映する正式な更新を行う
- `cargo clean` は `fav/tmp/hello.fav` を削除しないが、`target/` を削除する。`hello.fav` の存在は CI で必要なため注意（今回は `hello.fav` を使うテストが残るため、`cargo clean` 後も `hello.fav` が `fav/tmp/` に存在することを確認する）
