# Plan: v86.0.0 — SAP Foundation 1.0 宣言 ★クリーンアップ

## Step 1: 前提確認

- `cargo test` を実行し、3,949 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85900_tests` が存在することを確認する（v85.9.0 完了済みの証拠）

## Step 2: CHANGELOG.md に v86.0.0 エントリを追加

先頭（v85.9.0 エントリの前）に追加する。
テストモジュール追加より先に行う（`changelog_has_v86_0_0` テストが先に通る必要があるため）。

```markdown
## [v86.0.0] — 2026-08-23 — SAP Foundation 1.0 宣言 ★クリーンアップ

### Added
- SAP Foundation 1.0 を宣言:「SAP に、型安全に接続できるようになった。`fav.toml [sap]` を書けば、Favnir が SAP OData v4 と話せる。」
- `fav/src/driver.rs` — `mod v86000_tests`（テスト 4 件）を追加
- 合計テスト数: **3,953**（+4）

### Changed
- `fav/Cargo.toml` — バージョンを `86.0.0` に更新
- `MILESTONE.md` — SAP Foundation 1.0 エントリを追加
- `README.md` — v86.0 SAP Integration セクションを追加
- `versions/current.md` — v86.0.0 に更新
```

## Step 3: MILESTONE.md に SAP Foundation 1.0 エントリを追加

先頭（v85.0.0 エントリの前）に追加する。

```markdown
## v86.0.0（2026-08-23）— SAP Foundation 1.0 宣言

> 「SAP に、型安全に接続できるようになった。
>  `fav.toml [sap]` を書けば、Favnir が SAP OData v4 と話せる。」

**SAP Foundation 1.0** の宣言バージョン。v85.1.0〜v85.9.0 で実装した
SAP Integration Era の第 1 スプリントの完成を宣言した。テスト数: 3,953。

**SAP Foundation 1.0（v85.1〜v85.9）達成内容:**
- **Rust 基盤**: `SapTomlConfig` / `inject_sap_config()` / `fav.toml [sap]` 解析
- **Favnir 型**: `SapConfig` / `SapError` / `SapErrorCode` / `ODataParams`
- **Rune**: `sap-odata`（`odata_get` / `odata_list` / `sap_config_from_env`）
- **インフラ**: Docker Compose モックサーバー + SSM Parameter Store Terraform
- **テンプレート**: `fav new` が `[sap]` コメントブロックを生成
```

## Step 4: README.md を更新

v86.0 セクションを先頭付近に追加し、SAP Integration に言及する。

```markdown
## v86.0 — SAP Foundation 1.0 宣言（2026-08-23）

Favnir v86.0 で **SAP Foundation 1.0** を宣言しました。

SAP S/4HANA OData v4 に型安全に接続できるようになりました。
`fav.toml [sap]` セクションを設定するだけで、`sap_odata.odata_get()` / `odata_list()` で
SAP データをパイプラインに取り込めます。

**SAP Integration への第一歩 — `fav.toml [sap]` を書けば SAP と話せる。**
```

## Step 5: `versions/current.md` を更新

- 「最終更新」を `2026-08-23 (v86.0.0)` に変更
- 「最新安定版」を `v86.0.0 — SAP Foundation 1.0 宣言 — 3953 tests` に変更
- 「進行中バージョン」を `v86.1.0〜v90.0.0` に変更
- 「次に切る版」を `v86.1.0` に変更
- マイルストーン進捗に `v86.0 — SAP Foundation 1.0 | **完了**` を追加

## Step 6: `fav/Cargo.toml` のバージョンを更新

`version = "85.0.0"` → `version = "86.0.0"`

## Step 7: `fav/src/driver.rs` のアサーションを一括更新

`version = \"85.0.0\"` → `version = \"86.0.0\"` を `replace_all: true` で一括置換する。
（35 件が対象）

## Step 8: `mod v86000_tests` を追加

`mod v85900_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v86000_tests {
    #[test]
    fn cargo_toml_version_is_86_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("version = \"86.0.0\""),
            "Cargo.toml should have version = \"86.0.0\""
        );
    }

    #[test]
    fn changelog_has_v86_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v86.0.0"), "CHANGELOG.md should mention v86.0.0");
    }

    #[test]
    fn milestone_has_sap_foundation() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("SAP Foundation"),
            "MILESTONE.md should mention SAP Foundation"
        );
    }

    #[test]
    fn readme_mentions_sap_integration() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("SAP"),
            "README.md should mention SAP integration"
        );
    }
}
```

## Step 9: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3953 tests, 0 failures
```

## Step 10: `cargo clean` 実施

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
```

`fav/tmp/hello.fav` が残っていることを確認する（target/ のみ削除）。

## Step 11: CI 事前確認

`cargo clean` 後はリビルドが必要なため、まず `cargo build` してから確認する。

```bash
cargo build 2>&1 | tail -3
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
