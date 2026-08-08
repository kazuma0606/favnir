# v67.0.0 実装計画 — AI-Native Stage Layer 宣言 ★クリーンアップ

Version: 67.0.0
Status: 未着手
Base tests: 3493
Target tests: 3497

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: ファイル更新（4件）

以下を順番に実施する:

1. `fav/Cargo.toml` — `version = "66.0.0"` → `version = "67.0.0"` に変更
2. `MILESTONE.md` — `## v66.0.0` の直前に v67.0.0 エントリを挿入
3. `README.md` — v67.0.0 宣言（`"AI-Native"` または `"v67.0"` を含む）を追加
4. `CHANGELOG.md` — `## [v66.0.0]` の直前に v67.0.0 エントリを挿入

### Step 2: `driver.rs` テスト追加

`// -- v66900_tests (v66.9.0)` コメントの直前に `v67000_tests` を挿入。

4 テスト関数:
- `cargo_toml_version_is_67_0_0` — `include_str!("../Cargo.toml")` に `version = "67.0.0"` を含む
- `changelog_has_v67_0_0` — `include_str!("../../CHANGELOG.md")` に `"v67.0.0"` を含む
- `milestone_has_ai_native_stage` — `include_str!("../../MILESTONE.md")` に `"AI-Native Stage Layer"` を含む
- `readme_mentions_ai_native` — `include_str!("../../README.md")` に `"AI-Native"` または `"v67.0"` を含む

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67000_tests
```

### Step 4: `cargo clean` + `fav/tmp/hello.fav` 復元

```bash
# fav/ ディレクトリで実行
cargo clean
# hello.fav を復元（内容は spec.md 参照）
```

### Step 5: フルテスト確認

```bash
cargo test -j 8 -- --test-threads=8
```

3497 tests passed, 0 failed を確認。

---

## `driver.rs` 挿入コード

```rust
// -- v67000_tests (v67.0.0) -- AI-Native Stage Layer 宣言 --
#[cfg(test)]
mod v67000_tests {
    #[test]
    fn cargo_toml_version_is_67_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"67.0.0\""),
            "Cargo.toml should have version 67.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v67_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v67.0.0"), "CHANGELOG.md should mention v67.0.0");
    }

    #[test]
    fn milestone_has_ai_native_stage() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("AI-Native Stage Layer"),
            "MILESTONE.md should contain 'AI-Native Stage Layer'"
        );
    }

    #[test]
    fn readme_mentions_ai_native() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("AI-Native") || readme.contains("v67.0"),
            "README.md should mention AI-Native Stage Layer or v67.0"
        );
    }
}
```

---

## リスク・注意点

- `cargo clean` は ★クリーンアップとして必須。ただし過去実績として clean 後に `fav/tmp/hello.fav` が消えるケースが報告されているため、必ず復元すること（`cargo clean` 自体は `fav/tmp/` を直接削除しないが念のため）
- `MILESTONE.md` / `CHANGELOG.md` / `README.md` の挿入位置を間違えると `include_str!` テストが FAIL する
- CHANGELOG.md は v66.0.0 エントリの「直前」に挿入（v66.0.0 の後ではなく前）
- `readme_mentions_ai_native` は OR 条件（`"AI-Native"` または `"v67.0"`）なので、どちらかが README に存在すれば OK
- **[重要] Cargo.toml の version を `"67.0.0"` に変更すると、既存の `v66000_tests::cargo_toml_version_is_66_0_0` テストが FAIL する**。過去に同様の問題が発生した実績あり（v66.0.0 実装時）。T3 のフルテスト実行前に `v66000_tests` の当該テストを修正する必要がある（`"66.0.0"` → `"67.0.0"` へのアサート更新、またはテストを削除・コメントアウトするのではなく、v66000_tests 全体が Cargo.toml version 変更後も動作するよう確認すること）

## 非スコープ

- W055〜W059 の実際の検出ロジック実装 — 将来フェーズ
- v68.x スプリント計画 — 別途策定
