# v73.0.0 実装計画 — Developer Experience 2.0 宣言 ★クリーンアップ

Date: 2026-08-13

---

## 実装ステップ

### T0: 事前確認

1. `fav/Cargo.toml` のバージョンが `72.9.0` であることを確認
2. `cargo test` が 3642 tests pass（0 failures）であることを確認
3. `driver.rs` に `v729000_tests` モジュールが存在することを確認
4. `driver.rs` に `v73000_tests` が未存在であることを確認
5. `driver.rs` 内の `"72.9.0"` 文字列件数を grep で確認しておく

---

### T1: `cargo clean`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

クリーンアップ後に `cargo build -j 8` でクリーンビルドが成功することを確認する。

---

### T2: `MILESTONE.md` 更新

`MILESTONE.md` に「Developer Experience 2.0」マイルストーンを追記する。

追記内容:
```markdown
## v73.0.0 — Developer Experience 2.0（2026-08-13）

VS Code がパイプラインを補完し、AI がエラーを修正し、REPL が型を即座に返し、
Playground がコードを世界と共有する。
自然言語一文が、型安全なパイプラインの雛形になる。

- VS Code 拡張（`editors/vscode/`）— LSP 統合・シンタックスハイライト・型ホバー
- AI エラーアシスタント（`fav ai explain` / `fav ai fix`）
- `fav ai generate` — 自然言語 → Favnir パイプライン生成
- REPL 2.0（`:timing` / TAB 補完ヘルパー）
- Playground 2.0（テンプレートギャラリー・共有リンク）
- `fav init` テンプレートギャラリー（ai-etl / streaming / enterprise / data-quality / distributed）
- `fav watch` 2.0（`--on-change` フラグ）
- `fav learn` — インタラクティブチュートリアル（5 章）
```

---

### T3: `README.md` 更新

`README.md` のマイルストーン一覧に v73.0 の達成を追記する。

追記内容（既存の `v72.0` または `v71.0` 行の後):
```markdown
- **v73.0** — Developer Experience 2.0（VS Code 拡張・AI アシスタント・REPL 2.0・Playground 2.0）
```

---

### T3.5: `CHANGELOG.md` 更新（T6 部分テストの前に実施）

`changelog_has_v73_0_0` テストが T6 で pass できるよう、テストモジュール追加前に CHANGELOG を更新する。

```markdown
## [v73.0.0] — 2026-08-13 — Developer Experience 2.0 宣言 ★クリーンアップ

### Milestone
- **Developer Experience 2.0** 宣言
  VS Code 拡張・AI アシスタント・REPL 2.0・Playground 2.0・`fav learn` が揃い、
  データエンジニアが Favnir を選ぶ開発体験が整った。

### Changed
- `cargo clean` によるビルドキャッシュリセット実施
- `Cargo.toml` バージョンを `73.0.0` に更新

### Docs
- `MILESTONE.md` に Developer Experience 2.0 マイルストーンを追記
- `README.md` に v73.0 達成を追記

### Tests
- `cargo_toml_version_is_73_0_0`
- `changelog_has_v73_0_0`
- `milestone_has_dev_exp2`
- `readme_mentions_dev_exp2`
- 合計テスト数: 3646（+4）
```

---

### T4: `v73000_tests` モジュール追加

`v729000_tests` モジュールの直後に `v73000_tests` を追加する。

```rust
#[cfg(test)]
mod v73000_tests {
    #[test]
    fn cargo_toml_version_is_73_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(cargo_toml.contains("version = \"73.0.0\""),
            "Cargo.toml version should be 73.0.0");
    }

    #[test]
    fn changelog_has_v73_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(changelog.contains("[v73.0.0]"),
            "CHANGELOG.md should have v73.0.0 entry");
    }

    #[test]
    fn milestone_has_dev_exp2() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(milestone.contains("Developer Experience 2.0"),
            "MILESTONE.md should mention Developer Experience 2.0");
    }

    #[test]
    fn readme_mentions_dev_exp2() {
        let readme = include_str!("../../README.md");
        assert!(readme.contains("v73.0") || readme.contains("Developer Experience 2.0"),
            "README.md should mention v73.0 or Developer Experience 2.0");
    }
}
```

確認: `cargo test v73000` で 4 件 pass（CHANGELOG/MILESTONE/README 更新後に実行）

---

### T5: バージョン更新（`fav/Cargo.toml` + `driver.rs`）

- `fav/Cargo.toml`: `version = "72.9.0"` → `version = "73.0.0"`
- `driver.rs` 内のバージョン文字列を一括 replace（`cargo_toml_version_is_X` テスト用）
- replace 後、意図しない `"72.9.0"` 残留がないことを確認

---

### T6: 部分テスト確認

- `cargo test v73000` で 4 件 pass することを確認
  （T2・T3 でファイルを更新しておく必要がある）

---

### T7: 全体テスト確認

- `cargo test` 全体で 3646 tests pass（0 failures）を確認

---

### T8: `versions/current.md` 更新

- 「最終安定版」を `v73.0.0` に更新（v72.0 → v73.0）
- 「進行中バージョン」を `v73.1.0` に更新
- 「次に切る版」を `v73.2.0` に更新
- 「最終更新」を `2026-08-13 (v73.0.0)` に更新
- マイルストーン進捗表の v73.0 行を「完了」に更新
- `roadmap-v72.1-v73.0.md` の v73.0.0 行を「完了（実測 3646）」に更新

---

### T9: 最終確認

- `cargo test v73000` で 4 件 pass
- `cargo test` 全体で 3646 tests pass（0 failures）
- `fav/Cargo.toml` のバージョンが `73.0.0`
- `MILESTONE.md` に「Developer Experience 2.0」が存在する
- `README.md` に `v73.0` が存在する
