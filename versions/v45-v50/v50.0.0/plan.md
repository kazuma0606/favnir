# Plan: v50.0.0 — Production 2.0 宣言 ★クリーンアップ

Date: 2026-07-18

---

## 実装方針

### Step 1: `README.md` に Language Maturity 言及を追加

`README.md` の適切な箇所（バージョン履歴・マイルストーン付近）に以下を追加:

- `"Language Maturity"` という文字列
- `"v50"` または `"v50.0"` への言及
- Production 2.0 宣言文の一部（1〜2 文程度）

### Step 2: `v50000_tests` モジュール追加

`v499000_tests` の直前に挿入。

```rust
// -- v50000_tests (v50.0.0) -- Production 2.0 宣言 --
#[cfg(test)]
mod v50000_tests {
    #[test]
    fn cargo_toml_version_is_50_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("version = \"50.0.0\""),
            "Cargo.toml version field should be 50.0.0"
        );
    }

    #[test]
    fn changelog_has_v50_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(
            content.contains("v50.0.0"),
            "CHANGELOG.md should contain v50.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_language_maturity() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Language Maturity"),
            "MILESTONE.md should contain 'Language Maturity'"
        );
        assert!(
            content.contains("v50.0.0"),
            "MILESTONE.md should contain 'v50.0.0' in the Language Maturity entry"
        );
    }

    #[test]
    fn readme_mentions_language_maturity() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("Language Maturity"),
            "README.md should mention 'Language Maturity'"
        );
    }
}
```

### Step 3: バージョン更新・CHANGELOG 追加

1. `Cargo.toml` version → `"50.0.0"`（先に更新）
2. `CHANGELOG.md` に v50.0.0 エントリ追加
3. `cargo test` 3091 passed 確認

### Step 4: ★クリーンアップ（`cargo clean`）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

その後 `fav/tmp/hello.fav` の存在を確認し、消えていた場合は復元する。

**`fav/tmp/hello.fav` の正しい内容:**
```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

### Step 5: クリーンアップ後 `cargo test` 再実行

```bash
cargo test -j 8 -- --test-threads=8
```

3091 passed, 0 failed を確認する。

### Step 6: 残ファイル更新

- `versions/current.md` を v50.0.0 に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v50.0.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `cargo clean` は `fav/tmp/hello.fav` を削除する可能性がある（過去実績あり）→ **必ず復元確認**
- `milestone_has_language_maturity` は v49.8.0 の `milestone_has_language_maturity`（`v498000_tests`）と同名だが、別モジュール（`v50000_tests`）内のため Rust 上は重複しない
- `changelog_has_v50_0_0` テストは CHANGELOG に v50.0.0 エントリを追加した後に pass する — テスト追加と CHANGELOG 追加の順序が問題にならないよう、CHANGELOG は Step 3 でまとめて行う
- **Step 2 完了直後には `cargo test` を走らせないこと** — `changelog_has_v50_0_0` / `readme_mentions_language_maturity` / `cargo_toml_version_is_50_0_0` がまだ fail する。全通過確認は Step 3 完了後に行うこと
