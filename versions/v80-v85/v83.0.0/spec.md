# v83.0.0 — Pipeline Contracts 1.0 宣言 ★クリーンアップ

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリント（v82.1.0〜v82.9.0）の完成宣言バージョン。

**宣言文:**
> 「パイプライン間の約束が型になった。
>  `IoContract` がインターフェースを定義し、`SlaContract` が応答時間を保証し、
>  `ContractRegistry` がチームを繋ぐ。
>  Favnir のパイプラインは今、契約で安全に接続できる。」

本バージョンでは以下のクリーンアップ作業を行い、4 件のテストで完成を検証する。

---

## Goals

1. `cargo clean` を実施して build artifacts をクリアする
2. `fav/Cargo.toml` のバージョンを `83.0.0` に更新する
3. `CHANGELOG.md` に v83.0.0 エントリを追加する
4. `MILESTONE.md` に Pipeline Contracts 1.0 達成の宣言文を追加する
5. `README.md` に `ContractRegistry` への言及を追加する
6. `versions/current.md` を更新する（現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認してから）
7. `versions/roadmap/roadmap-v80.1-v85.0.md` の Sprint 3 バージョン一覧テーブルを全行「完了」に更新する
8. `v83000_tests` モジュールを `driver.rs` に追加する（4 件）

---

## Success Criteria

- `cargo test` 全 pass（3,887 tests = 3,883 + 4）※ drift 補正後
- 新規テスト 4 件（`v83000_tests` モジュール）:
  - `cargo_toml_version_is_83_0_0`
  - `changelog_has_v83_0_0`
  - `milestone_has_pipeline_contracts`
  - `readme_mentions_contract_registry`

---

## テストの実装例

```rust
#[test]
fn cargo_toml_version_is_83_0_0() {
    let content = include_str!("../Cargo.toml");
    assert!(content.contains("version = \"83.0.0\""));
}

#[test]
fn changelog_has_v83_0_0() {
    let content = include_str!("../../CHANGELOG.md");
    assert!(content.contains("v83.0.0"));
}

#[test]
fn milestone_has_pipeline_contracts() {
    let content = include_str!("../../MILESTONE.md");
    assert!(content.contains("Pipeline Contracts"));
}

#[test]
fn readme_mentions_contract_registry() {
    let content = include_str!("../../README.md");
    assert!(content.contains("ContractRegistry"));
}
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version = "82.0.0"` → `version = "83.0.0"`（Cargo.toml は宣言バージョン時のみ更新される慣習のため、v82.1〜v82.9 では更新されず現在値は `"82.0.0"`） |
| `fav/Cargo.lock` | `Cargo.toml` バージョン更新時に自動更新される |
| `fav/src/driver.rs` | `#[cfg(test)] mod v83000_tests` を追加（4 件） |
| `CHANGELOG.md` | v83.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | Pipeline Contracts 1.0 達成宣言を追加 |
| `README.md` | `ContractRegistry` への言及を追加 |
| `versions/current.md` | 現行バージョン・次バージョン欄を更新 |
| `versions/roadmap/roadmap-v80.1-v85.0.md` | Sprint 3 テーブルを全行「完了」に更新 |
