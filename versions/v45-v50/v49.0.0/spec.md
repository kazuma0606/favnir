# Spec: v49.0.0 — Module & Package 2.0 宣言 ★クリーンアップ

## 宣言文

> 「��ッケージ import ��ローカル import ��構文で明確に分離され、
>  `fav.toml` が依存関係の唯一の真実となった。
>
>  これが Favnir v49.0 — Module & Package 2.0 の姿である。」

---

## 概要

v48.1〜v48.9 で実装した Module & Package 2.0 の全機能を宣言し、
`MILESTONE.md` と `README.md` にマイルストーンを記録する。
`driver.rs` に `v49000_tests`（4テスト）を追加し、`cargo clean` でビルドアーティファクトを除去する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v49.0.0 Module & Package 2.0 エントリを先頭に追加 |
| `README.md` | `"Module & Package 2.0"` への言及を追加（v48.0 エントリの直後）|
| `fav/src/driver.rs` | `v49000_tests` 追加（4テスト）|
| `fav/Cargo.toml` | version → `"49.0.0"` |
| `CHANGELOG.md` | v49.0.0 エントリ追加 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | v49.0.0 実績記入 |
| `versions/roadmap/roadmap-v45.1-v50.0.md` | v49.0 完了を反映（実績 tests 数） |

---

## MILESTONE.md 追加内容

```markdown
## v49.0.0 — Module & Package 2.0（2026-07-18）

> 「パッケージ import とローカル import が構文で明確に分離され、
>  `fav.toml` が依存関係の唯一の真実となった。
>
>  これが Favnir v49.0 — Module & Package 2.0 の姿である。」

v49.0.0 をもって、Favnir の **Module & Package 2.0** を正式に宣言する。

### 達成コンポーネント（v48.1〜v48.9）

| ���ンポーネント | バージョン | 内容 |
|---|---|---|
| `ImportKind::Package` / `ImportKind::Local` AST + parser | v48.1.0 | パッケージ import 構文刷新 |
| ロ��カル import `"./"` プレフ��ックス対応 | v48.2.0 | ローカル���ァイル import 構文 |
| `fav.toml [runes]` 解決ロジック + E0417 | v48.3.0 | 依存関係の一��管理 |
| `fav install`（`runes/` スタブ展開）| v48.4.0 | ��ッケージインストールコマンド |
| W035 `legacy_import_rune` lint ルール + E0417 実発行 | v48.5.0 | 旧構文の非推奨化 + checker.rs E0417 実発行 |
| 循環 import 検出 + E0418 | v48.6.0 | import ���ラフ循環検出 |
| `rune.toml` 標準化（`validate_rune_toml`）| v48.7.0 | Rune 仕様の統一 |
| `list_installed_runes` / `get_rune_version` ヘルパー | v48.8.0 | runes/ ディレクトリ管理 |
| Module ドキュメント + migration guide | v48.9.0 | ユーザー向け移行ガイド |
```

---

## テスト（+4）

`v49000_tests` を `v489000_tests` の直前に追加:

```rust
#[cfg(test)]
mod v49000_tests {
    #[test]
    fn cargo_toml_version_is_49_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"49.0.0\""),
            "Cargo.toml should have version = \"49.0.0\"");
    }

    #[test]
    fn changelog_has_v49_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v49.0.0]"),
            "CHANGELOG.md should contain [v49.0.0]");
    }

    #[test]
    fn milestone_has_module_package_v2() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Module & Package 2.0"),
            "MILESTONE.md should contain 'Module & Package 2.0'");
    }

    #[test]
    fn readme_mentions_module_package_v2() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Module & Package 2.0"),
            "README.md should mention 'Module & Package 2.0'");
    }
}
```

テスト数: 3065 → **3069**（+4）

---

## ★クリーンアップ

`cargo clean` を実施してビルドアーティファクトを除去する。

- `cargo clean` 後、`fav/tmp/hello.fav` が残��している���とを確認
- `cargo test` を��実行し 3069 passed を確認

---

## 完了条件

- `MILESTONE.md` に `"Module & Package 2.0"` が含まれる
- `README.md` に `"Module & Package 2.0"` が含まれる
- `cargo test` 3069 passed, 0 failed（3065 + 4 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"49.0.0"`
- `CHANGELOG.md` に v49.0.0 エントリ追加
- `versions/current.md` を v49.0.0（3069 tests）に更新、進行中���ージョンを `v49.1.0` に更新
- `versions/roadmap/roadmap-v48.1-v49.0.md` に v49.0.0 実績を記入
- `versions/roadmap/roadmap-v45.1-v50.0.md` に v49.0 完了を反映（実績 3069 tests）
- `cargo clean` 完了・`fav/tmp/hello.fav` 存在確認・クリーン後 `cargo test` 再通過
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
