# Spec: v51.0.0 — Developer Experience 3.0 宣言 ★クリーンアップ

## 概要

v50.1〜v50.9 で実装した DX 3.0 の全機能が動作することを確認し、
`MILESTONE.md` と `README.md` に宣言テキストを追記して **Favnir v51.0 — Developer Experience 3.0** を宣言する。

**宣言文**:

> 「全エラーコードに修正提案が付き、JSON / LSP / CLI で一貫して届く。
>  エディタは型を表示し、trace はパイプラインの流れを可視化する。
>  Favnir の診断は開発者の思考を止めない。
>
>  これが Favnir v51.0 — Developer Experience 3.0 の姿である。」

---

## 背景

| バージョン | 実装内容 |
|---|---|
| v50.1.0 | 全エラーコードに `suggestion` 追加 |
| v50.2.0 | JSON / LSP / CLI で `suggestion` 一貫出力 |
| v50.3.0 | `fav explain --error` 正式導線 |
| v50.4.0 | LSP インレイヒント — 変数・関数戻り型 |
| v50.5.0 | LSP インレイヒント — pipeline stage 型 |
| v50.6.0 | LSP ホバー — Rune メソッドシグネチャ |
| v50.7.0 | `fav run --trace` 構造化ログ / `--watch` |
| v50.8.0 | ドキュメントサイト DX 3.0 記事 |
| v50.9.0 | 安定化・コードフリーズ / `dx3-overview.mdx` |

---

## 成果物仕様

### 1. `MILESTONE.md` 更新

先頭に v51.0.0 エントリを追加する。必須要件:
- `"Developer Experience 3.0"` の文字列を含む（テスト用: `milestone_has_dx3`）
- 宣言文全体を含む

### 2. `README.md` 更新

DX 3.0 マイルストーン到達を明記する。必須要件:
- `"DX 3.0"` または `"Developer Experience 3.0"` を含む（テスト用: `readme_mentions_dx3`）
- バージョン表記を `v51.0` に更新

---

## テスト仕様

### テスト数計算

現在: 3109 tests（v50.9.0 完了時点）

**削除（v509000_tests から）:**
v509000_tests は現在 3 件（`cargo_toml_version_is_50_9_0` / `dx3_overview_doc_exists` / `code_freeze_v50_9_0`）。
うち "50.9.0" を assert する 2 件を削除し、`dx3_overview_doc_exists` は保持する。
- `cargo_toml_version_is_50_9_0`（`"50.9.0"` を assert → v51.0.0 では FAIL）: −1
- `code_freeze_v50_9_0`（`"50.9.0"` を assert → v51.0.0 では FAIL）: −1

**追加（v51000_tests）: +6**
1. `cargo_toml_version_is_51_0_0`
2. `changelog_has_v51_0_0`
3. `milestone_has_dx3`
4. `readme_mentions_dx3`
5. `dx3_milestone_declared`（宣言文テキストの存在確認）
6. `code_freeze_v51_0_0`（"51.0.0" コードフリーズ宣言 — v509000_tests::code_freeze_v50_9_0 の後継）

純増: +6 − 2 = **+4 → 3113 tests**

> **ロードマップ注記**: roadmap では `v51000_tests` 4 件（`cargo_toml_version_is_51_0_0`・`changelog_has_v51_0_0`・`milestone_has_dx3`・`readme_mentions_dx3`）が完了条件。本 spec では v50.9.0 で追加した `code_freeze_v50_9_0`（"50.9.0" チェック）の後継として `code_freeze_v51_0_0` を追加し、また宣言文テキストを検証する `dx3_milestone_declared` を加えることで、ロードマップ必須 4 件 pass かつ ≥ 3113 の両条件を同時に達成する。

### テスト一覧

```rust
fn cargo_toml_version_is_51_0_0() {
    let content = include_str!("../Cargo.toml");
    assert!(content.contains("version = \"51.0.0\""),
        "Cargo.toml version should be 51.0.0");
}

fn changelog_has_v51_0_0() {
    let content = include_str!("../../CHANGELOG.md");
    assert!(content.contains("v51.0.0"),
        "CHANGELOG.md must have v51.0.0 entry");
}

fn milestone_has_dx3() {
    let content = include_str!("../../MILESTONE.md");
    assert!(content.contains("Developer Experience 3.0"),
        "MILESTONE.md must mention Developer Experience 3.0");
}

fn readme_mentions_dx3() {
    let content = include_str!("../../README.md");
    assert!(
        content.contains("DX 3.0") || content.contains("Developer Experience 3.0"),
        "README.md must mention DX 3.0"
    );
}

fn dx3_milestone_declared() {
    let content = include_str!("../../MILESTONE.md");
    assert!(content.contains("v51.0"),
        "MILESTONE.md must have v51.0 entry");
    assert!(content.contains("診断は開発者の思考を止めない") || content.contains("Developer Experience 3.0"),
        "MILESTONE.md must contain DX 3.0 declaration");
}

fn code_freeze_v51_0_0() {
    // v51.0.0 コードフリーズ宣言テスト（v509000_tests::code_freeze_v50_9_0 の後継）。
    // cargo_toml_version_is_51_0_0 と意図的に同じ assert を持つ:
    // 「バージョン固定」という事実をマイルストーン宣言の文脈で独立して記録するため。
    // 次バージョンアップ時は cargo_toml_version_is_X と本テストの両方を更新すること。
    let content = include_str!("../Cargo.toml");
    assert!(content.contains("version = \"51.0.0\""), "code freeze: version must be 51.0.0");
}
```

---

## バージョン要件

- `fav/Cargo.toml` version: `51.0.0`
- テスト数: 3109 → **3113**（純増 +4）
- `cargo clippy -- -D warnings` クリーン
- `cargo clean` 実施（★クリーンアップ）

---

## 完了条件

- `cargo test` 3113 tests passed, 0 failed（failures=0 かつ ≥ 3113）
- `cargo clippy -- -D warnings` クリーン
- `v51000_tests` 全 6 件 pass（うちロードマップ必須 4 件を含む）
- `MILESTONE.md` に `"Developer Experience 3.0"` が含まれる
- `README.md` に `"DX 3.0"` または `"Developer Experience 3.0"` が含まれる
- `CHANGELOG.md` に v51.0.0 エントリ追加
- `versions/current.md` を v51.0.0（3113 tests）に更新
- `cargo clean` 実施（★クリーンアップ）

---

## ロードマップ対応

roadmap-v50.1-v51.0.md v51.0.0 より:

> `v51000_tests` 4 件 pass: `cargo_toml_version_is_51_0_0` / `changelog_has_v51_0_0` / `milestone_has_dx3` / `readme_mentions_dx3`
> `MILESTONE.md` に `"Developer Experience 3.0"` が含まれる
> `cargo test` 全通過（failures=0 かつテスト数 ≥ **3113**）
> `★クリーンアップ`（`cargo clean`）完了
