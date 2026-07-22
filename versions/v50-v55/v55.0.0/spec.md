# Spec — v55.0.0 — Production 3.0 宣言

## 概要

v55.0.0 は Favnir の Production 3.0 マイルストーン宣言バージョン。
v51〜v54 で積み上げた全機能を最終確認し、`★クリーンアップ`（`cargo clean`）を実施して
Production 3.0 を正式宣言する。

---

## 宣言文

> 「型安全なガード節、スケールする並列パイプライン、
>  保証されたデータ品質、そして考えを助ける開発体験。
>  Favnir はデータエンジニアが現場で選ぶ言語になった。
>
>  これが Favnir v55.0 — Production 3.0 の姿である。」

---

## ロードマップ参照

- `versions/roadmap/roadmap-v54.1-v55.0.md` — v55.0.0 セクション
- ベーステスト数: 3203（v54.9.0 完了時点）
- 目標テスト数: 3206（3203 - 1 削除 + 4 新規）
  - 削除: `v54900_tests::cargo_toml_version_is_54_9_0`（Cargo.toml 更新により失敗するため）
  - 追加: `v55000_tests` 4 件

---

## 実装内容

### 1. Cargo.toml バージョン更新

```toml
version = "55.0.0"
```

### 2. CHANGELOG.md — v55.0.0 エントリ追加

```markdown
## [v55.0.0] — 2026-07-23 — Production 3.0 宣言

### Production 3.0 宣言

v51〜v54 で積み上げた全機能を最終確認し、Favnir v55.0 — Production 3.0 を宣言する。

- v51: Developer Experience 3.0（診断統一・インレイヒント・trace/watch）
- v52: Performance & Scale（par Tokio・バックプレッシャー・bench 回帰・WASM 最適化）
- v53: Data Quality & Observability 2.0（assert_schema・lineage 強化・audit-log）
- v54: Integration Sprint（explain --error 全網羅・watch-diff・dq-report・doctor・Production 3.0 整備）
- v55: Production 3.0 宣言・★クリーンアップ
```

### 3. driver.rs — v55000_tests モジュール追加

4 件のテストを `v54900_tests` 直前に挿入する。

```rust
// -- v55000_tests (v55.0.0) -- Production 3.0 宣言 --
#[cfg(test)]
mod v55000_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_55_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"55.0.0\""),
            "Cargo.toml version should be 55.0.0"
        );
    }

    #[test]
    fn changelog_has_v55_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v55.0.0]"),
            "CHANGELOG.md should have v55.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_production3() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Production 3.0"),
            "MILESTONE.md should mention Production 3.0"
        );
    }

    #[test]
    fn readme_mentions_production3() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Production 3.0"),
            "README.md should mention Production 3.0"
        );
    }
}
```

### 4. ★クリーンアップ

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

クリーンアップ後に再度 `cargo test` を実行して全通過を確認する。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_55_0_0` | `fav/Cargo.toml` に `version = "55.0.0"` |
| `changelog_has_v55_0_0` | `CHANGELOG.md` に `[v55.0.0]` エントリ |
| `milestone_has_production3` | `MILESTONE.md` に `Production 3.0` 記載 |
| `readme_mentions_production3` | `README.md` に `Production 3.0` 記載 |

---

## 完了条件

- `cargo test` 全通過（3206 tests passed, 0 failed、かつ failures=0）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_55_0_0` pass
- `changelog_has_v55_0_0` pass
- `milestone_has_production3` pass（v54.8.0 で既追加）
- `readme_mentions_production3` pass（v54.6.0 で既追加）
- `cargo clean` 完了（★クリーンアップ）
- `cargo clean` 後の `cargo test` 全通過確認
- `versions/current.md` が v55.0.0 / 3206 tests を反映（「次に切る版」→ 未定）
- `MILESTONE.md` の `## v55.0.0（予定）` から `（予定）` を除去し宣言日を追記

---

## 備考

- `milestone_has_production3` / `readme_mentions_production3` は v54.8.0 / v54.6.0 で既実装済み。
  v55.0.0 では追加実装不要で、テストが pass することを確認するのみ。
- `★クリーンアップ` は `cargo clean` で `target/` を削除する。
  これにより `fav/tmp/hello.fav` は影響を受けないが、`target/` が再ビルドされる。
  クリーンアップ後は `cargo test` を再実行して全通過を確認すること。
- `v55000_tests` は `v54900_tests` の直前に挿入する（逆時系列配置の規約に従う）。
