# Plan: v47.0.0 — Developer Experience 宣言 ★クリーンアップ

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v47000_tests` モジュール追加（4テスト） |
| `fav/Cargo.toml` | version → `"47.0.0"` |
| `CHANGELOG.md` | v47.0.0 マイルストーン宣言エントリ追加 |
| `MILESTONE.md` | v47.0.0 Developer Experience エントリ追加 |
| `README.md` | `"Developer Experience"` への言及を追加 |
| `versions/current.md` | v47.0.0 に更新 |
| `versions/v45-v50/v47.0.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### `fav/src/driver.rs` — `v47000_tests`

`v469000_tests`（47076行付近）の直後に追加する。
v46.x テストモジュール群（v461000〜v469000）の後に配置することで時系列的な順序を保つ。
（`v46000_tests` の直前ではなく末尾が正しい）

```rust
// -- v47000_tests (v47.0.0) -- Developer Experience 宣言 --
#[cfg(test)]
mod v47000_tests {
    #[test]
    fn cargo_toml_version_is_47_0_0() {
        // ../Cargo.toml: fav/src/ → fav/ → fav/Cargo.toml
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"47.0.0\""),
            "Cargo.toml version should be 47.0.0"
        );
    }

    #[test]
    fn changelog_has_v47_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v47.0.0]"),
            "CHANGELOG.md should have v47.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_developer_experience() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Developer Experience"),
            "MILESTONE.md should mention 'Developer Experience'"
        );
    }

    #[test]
    fn readme_mentions_developer_experience() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Developer Experience"),
            "README.md should mention 'Developer Experience'"
        );
    }
}
```

### `MILESTONE.md`

先頭（v46.0.0 エントリの直前）に追加:

```markdown
## v47.0.0 — Developer Experience（2026-07-17）

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>
>  これが Favnir v47.0 — Developer Experience の姿である。」

v47.0.0 をもって、Favnir の **Developer Experience** を正式に宣言する。

### 達成コンポーネント（v46.1〜v46.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `#[test]` ブロック AST + parser | v46.1 | `FnDef.is_test = true`、`#[test] fn` 解析 |
| `fav test` コマンド実装 | v46.2 | `cmd_test`、`#[test]` 収集と VM 実行ループ |
| assertion 拡充 | v46.3 | `assert_ok` / `assert_err` / `assert_ne` VM primitive |
| LSP inlay hints 強化 | v46.4 | `textDocument/inlayHint`、パイプライン推論型表示 |
| LSP クイックフィックス強化 | v46.5 | E0102 did-you-mean / E0101 引数追加提案 |
| `fav explain` 2.0 Phase 1 | v46.6 | dead path（点線）/ error path（赤）Mermaid 可視化 |
| `fav explain --lineage` 2.0 | v46.7 | `is_dead` フラグ + `--show-dead` CLI |
| `fav explain --types` | v46.8 | ステージ宣言型一覧表示 |
| DX ドキュメント + v47.0 前調整 | v46.9 | `fav-test.mdx` / `developer-experience.mdx` |

---
```

### `README.md`

先頭付近のマイルストーン言及箇所（または `---` の直前）に追記:

```markdown
**v47.0** Developer Experience — インラインテスト・LSP クイックフィックス・型情報可視化
```

既存の `v46.0` などへの言及がある場合はその直前に追加する。

---

## 実装順序

1. `MILESTONE.md` に v47.0.0 エントリを追加
2. `README.md` に `"Developer Experience"` を追加
3. `driver.rs` に `v47000_tests` を追加
4. `cargo test` で ≥ 3016 passed 確認（`cargo_toml_version_is_47_0_0` は Cargo.toml 更新前なので一時 FAIL → step 5 後に再確認）
5. `Cargo.toml` version → `"47.0.0"`
6. `CHANGELOG.md` v47.0.0 エントリ追加
7. `cargo test` で 3016 passed, 0 failed を最終確認
8. `cargo clippy -- -D warnings` クリーン確認
9. `versions/current.md` 更新
10. `tasks.md` COMPLETE に更新
11. **`cargo clean`** ★クリーンアップ実施

---

## 注意事項

- **`include_str!` パスまとめ（`fav/src/driver.rs` 起点）**:
  - `../Cargo.toml` → `fav/Cargo.toml` ✓（Cargo.toml は `fav/` 直下）
  - `../../CHANGELOG.md` → `favnir/CHANGELOG.md` ✓
  - `../../MILESTONE.md` → `favnir/MILESTONE.md` ✓
  - `../../README.md` → `favnir/README.md` ✓
- **`cargo_toml_version_is_46_0_0` との差異**: v46000_tests の同名テストは空実装（ボディなし）だが、
  v47000_tests では `include_str!("../Cargo.toml")` で実際に version を検証する実装である。
  上記コードブロックの内容が正しい（空にしないこと）。
- `cargo_toml_version_is_47_0_0` は Cargo.toml の version 更新（step 5）より後に全通過する。
  step 3〜4 で一時 FAIL しても step 5 後の step 7 で確認すれば問題ない。
- `cargo clean` は全テスト・clippy 通過確認後に実施する（クリーン後も `cargo test` で再確認すること）。
