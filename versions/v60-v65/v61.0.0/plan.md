# v61.0.0 Plan — Developer Experience 2.0 宣言 ★クリーンアップ

Date: 2026-07-31
Status: COMPLETE

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `"60.0.0"` → `"61.0.0"` |
| `MILESTONE.md` | 追加 | Developer Experience 2.0 宣言エントリ（先頭に追記） |
| `CHANGELOG.md` | 追加 | v61.0.0 エントリ（先頭に追記） |
| `README.md` | 追加 | v61.0.0 言及（v60.0.0 段落の直後） |
| `fav/src/driver.rs` | 追加 | `v61000_tests` モジュール（`v60900_tests` の直前） |
| `fav/src/driver.rs` | 更新 | 旧 version assertion 9 件を `"61.0.0"` に更新 |

新規ファイルなし。

---

## 実装ステップ

### Step 1: `fav/Cargo.toml` — バージョン更新

```toml
# 変更前
version = "60.0.0"

# 変更後
version = "61.0.0"
```

### Step 2: `MILESTONE.md` — Developer Experience 2.0 宣言エントリ追加

ファイル先頭（`# Favnir Milestones` の直後）に追加。

```markdown
## v61.0.0（2026-07-31）— Developer Experience 2.0

> 「エラーはソース位置を指し、修正候補は即座に現れる。...」

v60.1〜v60.9 達成内容一覧付き。
```

### Step 3: `CHANGELOG.md` — v61.0.0 エントリ追加

ファイル先頭の `---` 直後に追加。v60.1〜v60.9 全 9 機能を ### Added に列挙。

### Step 4: `README.md` — v61.0.0 言及追加

v60.0.0 Enterprise 1.0 段落の直後に Developer Experience 2.0 段落を追加。

### Step 5: `driver.rs` — `v61000_tests` モジュール追加

`v60900_tests` の直前（上側）に挿入。

```rust
// -- v61000_tests (v61.0.0) -- Developer Experience 2.0 宣言 --
#[cfg(test)]
mod v61000_tests {
    #[test]
    fn cargo_toml_version_is_61_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"61.0.0\""), ...);
    }

    #[test]
    fn changelog_has_v61_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v61.0.0"), ...);
    }

    #[test]
    fn milestone_has_dx2() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Developer Experience 2.0"), ...);
    }

    #[test]
    fn readme_mentions_dx2() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Developer Experience 2.0"), ...);
    }
}
```

### Step 6: 旧 version assertion 9 件を更新

driver.rs 内の以下 9 モジュールの `cargo_toml_version_is_*` テストが `"60.0.0"` をアサートしている:
`v56300_tests`, `v56900_tests`, `v57000_tests`, `v57900_tests`, `v58000_tests`,
`v58900_tests`, `v59000_tests`, `v59900_tests`, `v60000_tests`

これらのアサーション文字列（`version = \"60.0.0\"`）とエラーメッセージ（`"should be 60.0.0"`）を
`"61.0.0"` に一括更新（`replace_all`）。

### Step 7: `cargo clean` ★クリーンアップ

テスト全通過後に `cargo clean` を実行。

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `v61000_tests` | `driver.rs` の `v60900_tests` の直前（上側） |
| MILESTONE.md 宣言 | ファイル先頭の `# Favnir Milestones` 直後 |
| CHANGELOG.md エントリ | `---` 直後（最新エントリが先頭） |
| README.md 言及 | v60.0.0 段落の直後 |

---

## 注意点

- `v61000_tests` は `use super::*` 不要（`include_str!` のみ使用）。
- 旧 version assertion のエラーメッセージ文字列も `"61.0.0"` に更新する（code-reviewer 指摘対応）。
- `cargo clean` は `fav/tmp/hello.fav` を削除しないことを確認してから実行（このバージョンでは消えなかった）。
