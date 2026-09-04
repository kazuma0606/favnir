# Spec: v98.0.0 — SAP Workflow 1.0 宣言

## Background

v97.1.0〜v97.9.0 で SAP Workflow Sprint の全機能が実装・安定化された。
本バージョンは v97.0.0 以来の宣言版として、Cargo.toml のバージョンを `98.0.0` に更新し、
MILESTONE.md / README.md を更新して **SAP Workflow 1.0** を正式宣言する。

## Goals

1. `fav/Cargo.toml` の version を `98.0.0` に更新する
2. `CHANGELOG.md` に v98.0.0 エントリを追加する
3. `fav/src/driver.rs` に `mod v98000_tests`（4 テスト）を追加する
   - `driver.rs` 全体の `"97.0.0"` 文字列を `"98.0.0"` に一括置換する（`cargo_toml_version_is_97_0_0` テスト名も含む）
4. `MILESTONE.md` に v98.0.0 エントリを追加する
5. `README.md` に `## v98.0 — SAP Workflow 1.0` セクションを追加する
6. `cargo clean` を実施する（★クリーンアップ）
7. `cargo test` で 4,235 tests, 0 failures を確認する（cargo clean 後）

## 宣言文

> 「Favnir が、人間の判断を型に閉じ込めた。
>
>  `!Approval` エフェクトが pipeline のシグネチャに現れた時、
>  それはコードが「ここで人間の承認が必要」と語っているのだ。
>
>  承認フローが型になった。それが、Favnir SAP Workflow 1.0 である。」

## テスト（4 件）

```rust
// Cargo.toml のバージョン確認
fn cargo_toml_version_is_98_0_0()

// CHANGELOG に v98.0.0 エントリが存在することを確認
fn changelog_has_v98_0_0()

// MILESTONE.md に SAP Workflow 宣言が含まれることを確認
fn milestone_has_sap_workflow()

// README.md に SAP Workflow への言及があることを確認
fn readme_mentions_sap_workflow()
```

## Success Criteria

- `fav/Cargo.toml` の version が `98.0.0` である
- `CHANGELOG.md` に `[v98.0.0]` エントリが存在する
- `mod v98000_tests` の全テスト（4 件）が pass する
- `MILESTONE.md` に v98.0.0 エントリが存在する
- `README.md` に `v98.0 — SAP Workflow 1.0` セクションが存在する
- `cargo clean` 後の `cargo test` で 4,235 tests, 0 failures
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `97.0.0` → `98.0.0` |
| `fav/src/driver.rs` | 更新・追記 | 既存 `"97.0.0"` テストを `"98.0.0"` に更新 + `mod v98000_tests`（4 テスト） |
| `MILESTONE.md` | 追記 | v98.0.0 エントリ（宣言文 + 達成内容） |
| `README.md` | 追記 | `## v98.0 — SAP Workflow 1.0` セクション |
| `CHANGELOG.md` | 追記 | v98.0.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v98.0.0 に変更、マイルストーン表に追記 |
