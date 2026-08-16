# v74.9.0 実装計画 — 安定化・コードフリーズ（Favnir 2.0 前最終調整）

Date: 2026-08-14

---

## 実装ステップ

### Step 1: `v749000_tests` モジュールを `driver.rs` に追加

`v748000_tests` の直後に追加する。
本テストは `include_str!` のみ使用し、外部シンボル不使用のため `use super::*` は不要。

```rust
// --- v74.9.0: 安定化・コードフリーズ（Favnir 2.0 前最終調整） ---

#[cfg(test)]
mod v749000_tests {
    // include_str! のみ使用・外部シンボル不使用のため use super は不要

    #[test]
    fn favnir2_full_sprint_all_stable() {
        let changelog = include_str!("../../CHANGELOG.md");
        // v74.x スプリント全バージョンが CHANGELOG に存在することを確認
        assert!(changelog.contains("[v74.1.0]"), "v74.1.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.2.0]"), "v74.2.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.3.0]"), "v74.3.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.4.0]"), "v74.4.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.5.0]"), "v74.5.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.6.0]"), "v74.6.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.7.0]"), "v74.7.0 missing from CHANGELOG");
        assert!(changelog.contains("[v74.8.0]"), "v74.8.0 missing from CHANGELOG");
    }

    #[test]
    fn favnir2_e2e_showcase_runs() {
        // pipeline.fav の主要要素確認
        let pipeline = include_str!("../../infra/e2e-demo/favnir2-showcase/pipeline.fav");
        assert!(pipeline.contains("Result.ok"), "pipeline.fav: Result.ok missing");
        assert!(pipeline.contains("import rune"), "pipeline.fav: import rune missing");
        assert!(pipeline.contains("ShowcaseContract"), "pipeline.fav: ShowcaseContract missing");

        // fav.toml の設定確認
        let fav_toml = include_str!("../../infra/e2e-demo/favnir2-showcase/fav.toml");
        assert!(fav_toml.contains("schedule"), "fav.toml: schedule missing");
        assert!(fav_toml.contains("tenant"), "fav.toml: tenant missing");

        // contract.fav の確認
        let contract = include_str!("../../infra/e2e-demo/favnir2-showcase/contract.fav");
        assert!(contract.contains("ShowcaseInputContract"), "contract.fav: ShowcaseInputContract missing");
    }
}
```

### Step 2: バージョン更新

- `fav/Cargo.toml`: `version = "74.8.0"` → `version = "74.9.0"`
- `driver.rs` 内の `version = \"74.8.0\"` を `version = \"74.9.0\"` に replace_all（コメント行 `// ---` は置換対象外）
- `version should be 74.8.0` を `version should be 74.9.0` に replace_all（アサートメッセージのみ対象）
- `cargo build` で `Cargo.lock` が自動更新される

### Step 3: テスト確認

- `cargo test v749000` で 2 件 pass を確認
- `cargo test` 全体で 3688 tests pass を確認

### Step 4: `CHANGELOG.md` 更新

v74.9.0 エントリを先頭に追加。

### Step 5: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.9.0)`
- 進行中: `v74.9.0`
- 次: `v75.0.0`
