# v75.0.0 実装計画 — Favnir 2.0 宣言 ★クリーンアップ

Date: 2026-08-14

---

## 実装ステップ

### Step 1: MILESTONE.md / README.md を更新

テストモジュール追加より先に実施する（`milestone_has_favnir_2` / `readme_mentions_favnir_2` テストのアサートが通る状態にするため）。

- `MILESTONE.md` に「Favnir 2.0 宣言」セクションを追記
- `README.md` に v75.0 / Favnir 2.0 達成を追記

### Step 1.5: CHANGELOG.md 更新

`changelog_has_v75_0_0` テストのアサートが通る状態にするため、テストモジュール追加より前に実施する。

- v75.0.0 エントリを CHANGELOG 先頭に追加（Tests: 4 件、合計 3692）

### Step 2: `v75000_tests` モジュールを `driver.rs` に追加

`v749000_tests` の直後（ファイル末尾）に追加する。
`include_str!` のみ使用・外部シンボル不使用のため `use super::*` は不要。

```rust
// --- v75.0.0: Favnir 2.0 宣言 ★クリーンアップ ---

#[cfg(test)]
mod v75000_tests {
    // include_str! のみ使用・外部シンボル不使用のため use super は不要

    #[test]
    fn cargo_toml_version_is_75_0_0() {
        // NOTE: この関数名は「v75.0.0 スプリントで追加されたテスト」を示す。
        // アサート値は新バージョンリリース時に replace_all で常に最新バージョンに更新される設計。
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(cargo_toml.contains("version = \"75.0.0\""),
            "Cargo.toml version should be 75.0.0");
    }

    #[test]
    fn changelog_has_v75_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(changelog.contains("[v75.0.0]"),
            "CHANGELOG.md should have v75.0.0 entry");
    }

    #[test]
    fn milestone_has_favnir_2() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(milestone.contains("Favnir 2.0"),
            "MILESTONE.md should mention Favnir 2.0");
    }

    #[test]
    fn readme_mentions_favnir_2() {
        let readme = include_str!("../../README.md");
        assert!(readme.contains("v75.0") || readme.contains("Favnir 2.0"),
            "README.md should mention v75.0 or Favnir 2.0");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.9.0"` → `version = "75.0.0"`
- `driver.rs` 内の `version = \"74.9.0\"` を `version = \"75.0.0\"` に replace_all（コメント行 `// ---` は置換対象外）
- `version should be 74.9.0` を `version should be 75.0.0` に replace_all（アサートメッセージのみ）
- `cargo build` で `Cargo.lock` 自動更新

### Step 4: 部分テスト確認

```bash
cargo test v75000 -- --test-threads=8
```
4 件 pass を確認（CHANGELOG は Step 1.5 で更新済みのため `changelog_has_v75_0_0` も通る）。

### Step 5: cargo clean クリーンアップ

```bash
cargo clean
```

- 実施後、`fav/tmp/hello.fav` の存在を確認する
- 消えていた場合は以下の内容で復元する:
  ```
  fn add(a: Int, b: Int) -> Int { a + b }
  fn main() -> Bool { add(1, 2) == 3 }
  ```

### Step 6: 全体テスト確認（cargo clean 後）

```bash
cargo test -j 8 -- --test-threads=8
```
3692 tests pass（0 failures）を確認。

### Step 7: versions/current.md 更新

- 最終更新: `2026-08-14 (v75.0.0)`
- 最新安定版: `v75.0.0`
- 進行中: 完了（次フェーズ未計画）

### Step 8: ロードマップ更新

- `versions/roadmap/roadmap-v74.1-v75.0.md` の Status を「完了」に更新
