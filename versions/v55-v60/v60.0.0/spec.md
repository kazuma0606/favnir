# v60.0.0 Spec — Enterprise 1.0 宣言 ★クリーンアップ

Date: 2026-07-30
Status: 未着手

---

## 概要

Favnir v60.0.0 は **Enterprise 1.0** の正式宣言バージョン。

v56〜v59 で実装した全エンタープライズ機能（RBAC / Secret 管理 / mTLS / 監査ログ /
コンプライアンス / Blue-Green Deploy / SLA Guarantee / Cost Visibility /
Migration Toolkit / Enterprise Certify）を統合し、
「企業で安心して選ばれるデータパイプライン言語」として Favnir の完成を宣言する。

---

## 宣言文

> 「ストリームはウィンドウで区切られ、型システムは制約で守られる。
>  アクセスはロールで制御され、シークレットはコードに現れない。
>  デプロイは無停止で切り替わり、ポリシーはコードで記述される。
>  コストは可視化され、SLA は保証され、コンプライアンスは証明される。
>
>  Favnir はデータエンジニアリングのエンタープライズ標準になった。
>
>  これが Favnir v60.0 — Enterprise 1.0 の姿である。」

---

## 実装スコープ

### 1. バージョン更新

- `fav/Cargo.toml`: `59.9.0` → `60.0.0`

### 2. CHANGELOG.md — v60.0.0 エントリ追加

v60.0.0 の宣言内容を記録する。

### 3. MILESTONE.md — Enterprise 1.0 宣言文エントリ

`## v60.0.0（2026-07-30）— Enterprise 1.0` エントリを追加（宣言文 + v56〜v59 達成内容）。

現行の `## v60.0.0（予定）— Enterprise 1.0` エントリを正式版に置き換える。

`milestone_has_enterprise1` テストは `"Enterprise 1.0"` を検索する。

### 4. README.md — Enterprise 1.0 リリース言及更新

`v60.0.0 — Enterprise 1.0 として宣言予定です。` → 正式リリース文に更新。

`readme_mentions_enterprise1` テストは `"Enterprise 1.0"` を検索する。

### 5. driver.rs — v60000_tests 追加（4 件）

```rust
// -- v60000_tests (v60.0.0) -- Enterprise 1.0 宣言 --
#[cfg(test)]
mod v60000_tests {
    #[test]
    fn cargo_toml_version_is_60_0_0() { ... }
    #[test]
    fn changelog_has_v60_0_0() { ... }
    #[test]
    fn milestone_has_enterprise1() { ... }
    #[test]
    fn readme_mentions_enterprise1() { ... }
}
```

- `cargo_toml_version_is_60_0_0`: `Cargo.toml` に `version = "60.0.0"` を確認
- `changelog_has_v60_0_0`: `CHANGELOG.md` に `v60.0.0` を確認
- `milestone_has_enterprise1`: `MILESTONE.md` に `Enterprise 1.0` を確認
- `readme_mentions_enterprise1`: `README.md` に `Enterprise 1.0` を確認

`use super::*;` は不要（`include_str!` のみ使用）。

### 6. driver.rs — ローリングチェック更新（8 件）

v59.9.0 時点のローリングチェックプール（8 件）を全件 `60.0.0` に更新する。

| モジュール | rolling check 関数名 |
|---|---|
| v59000_tests | cargo_toml_version_is_59_0_0 |
| v58900_tests | cargo_toml_version_is_58_9_0 |
| v58000_tests | cargo_toml_version_is_58_0_0 |
| v57900_tests | cargo_toml_version_is_57_9_0 |
| v57000_tests | cargo_toml_version_is_57_0_0 |
| v56900_tests | cargo_toml_version_is_56_9_0 |
| v56300_tests | cargo_toml_version_is_56_3_0 |
| v59900_tests | cargo_toml_version_is_59_9_0 |

更新内容:
- `version = \"59.9.0\"` → `\"60.0.0\"` （8 件）
- `"Cargo.toml version should be 59.9.0"` → `"Cargo.toml version should be 60.0.0"` （8 件）

**注意**: `// -- vXXXXX_tests (vX.Y.Z) --` コメント行は更新しない。

### 7. ★クリーンアップ（cargo clean）

v60.0.0 は `★クリーンアップ` バージョン。`cargo test` 全通過確認後に `cargo clean` を実行する。

---

## 完了条件

- `cargo test` 全通過（failures=0、テスト数 ≥ **3330**）
- `v60000_tests` 4 件 pass（ベース 3326 + 4 = 3330 tests passed, 0 failed）
- `MILESTONE.md` に Enterprise 1.0 宣言文エントリが正式追加されている（`（予定）` エントリを置き換え）
- `README.md` が Enterprise 1.0 正式リリース文に更新されている
- `cargo clean` 完了

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v59.1-v60.0.md`
- マスター: `versions/roadmap/roadmap-v55.1-v60.0.md`
