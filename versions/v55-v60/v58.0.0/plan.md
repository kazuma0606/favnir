# Plan — v58.0.0 — Enterprise Security 宣言 ★クリーンアップ

## 実装方針

v30.0.0（Ecosystem Maturity 宣言）と同じ宣言バージョンパターンを踏襲する。
4 テストのうち `changelog_has_v58_0_0` は `include_str!` でコンパイル時に CHANGELOG を読むため、
**CHANGELOG を先に更新してから** `v58000_tests` を driver.rs に追加する順序を守る。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `57.9.0` → `58.0.0` |
| `MILESTONE.md` | Enterprise Security エントリ（先頭）追加 |
| `README.md` | v58.0.0 マイルストーン達成を追記 |
| `CHANGELOG.md` | `[v58.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v58000_tests` 追加、rolling バージョンチェック 4 件更新 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
57.9.0 → 58.0.0
```

### Step 2: `MILESTONE.md` — Enterprise Security エントリ追加（先頭）

既存の `## v57.0.0` エントリの前に挿入:

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

### Step 3: `README.md` — v58.0.0 達成追記

README.md のマイルストーン/バージョン履歴テーブルで v58.0.0 行を追加（または既存の「計画中」行を更新）し、Enterprise Security 達成を明示する。

### Step 4: `CHANGELOG.md` — v58.0.0 エントリ追加（テスト追加前に必須）

```markdown
## [v58.0.0] — 2026-07-28 — Enterprise Security 宣言

### Added
- `MILESTONE.md`: Enterprise Security 宣言文エントリを追加（v57.1〜v57.9 達成内容）
- `v58000_tests` 追加（4 件）— 3276 tests
  - `cargo_toml_version_is_58_0_0`: Cargo.toml バージョンを検証（rolling チェック）
  - `changelog_has_v58_0_0`: CHANGELOG.md に v58.0.0 エントリが存在することを検証
  - `milestone_has_enterprise_security`: MILESTONE.md に Enterprise Security が含まれることを検証
  - `readme_mentions_enterprise_security`: README.md に Enterprise Security が含まれることを検証
```

### Step 5: `driver.rs` — `v58000_tests` 挿入（CHANGELOG 更新後に実施）

`v57900_tests` の直前に挿入:

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

### Step 6: バージョンチェックテスト更新（rolling）

| テスト | 変更前 | 変更後 |
|---|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | `"57.9.0"` | `"58.0.0"` |
| `v56900_tests::cargo_toml_version_is_56_9_0` | `"57.9.0"` | `"58.0.0"` |
| `v57000_tests::cargo_toml_version_is_57_0_0` | `"57.9.0"` | `"58.0.0"` |
| `v57900_tests::cargo_toml_version_is_57_9_0` | `"57.9.0"` | `"58.0.0"` |

### Step 7: ★クリーンアップ（テスト全通過後）

```bash
cargo clean
```

`fav/tmp/hello.fav` は `cargo clean` で削除されないことを確認する。

---

## 実装順序の厳守

```
Step 1 (Cargo.toml)
→ Step 2 (MILESTONE.md)
→ Step 3 (README.md)
→ Step 4 (CHANGELOG.md)  ← changelog_has_v58_0_0 テストの前提
→ Step 5 (v58000_tests 追加)
→ Step 6 (rolling 更新)
→ cargo build → cargo test → cargo clippy
→ Step 7 (cargo clean)
```

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3276 tests passed, 0 failed`（ベース 3272 + 4）

```bash
cargo clippy -- -D warnings
```

期待: 警告ゼロ

---

## ポスト処理

1. `versions/current.md` を v58.0.0 / 3276 tests に更新
2. `versions/roadmap/roadmap-v57.1-v58.0.md` の v58.0.0 実績を COMPLETE に更新
3. `versions/roadmap/roadmap-v55.1-v60.0.md` のテスト数推移テーブルに v58.0.0 行を追加（v57.9.0 行の直後）
