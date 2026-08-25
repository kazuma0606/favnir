# v84.0.0 実装計画 — Observability 2.0 宣言 ★クリーンアップ

## 実装ステップ

### Step 1: `cargo clean`

```bash
cd fav && cargo clean
```

`cargo clean` 後、`fav/tmp/hello.fav` が消えている場合は以下の内容で復元する
（`bootstrap_c2_artifact_roundtrip` テストが依存するため）:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

### Step 2: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "83.0.0"` を `version = "84.0.0"` に変更する。

### Step 3: `CHANGELOG.md` 更新

先頭に v84.0.0 エントリを追加する。

```markdown
## [v84.0.0] — 2026-08-21 — Observability 2.0 宣言

> 「メトリクスが型になり、アラートが型になり、SLO が型になった。
>  Favnir のパイプラインは壊れる前に教えてくれる。」

### Changed
- Bump version to 84.0.0
...
```

### Step 4: `MILESTONE.md` 更新

v84.0 の達成内容を先頭に追記する。

### Step 5: `README.md` 更新

`fav observe` コマンドの言及を追加する（例: コマンド一覧または特徴説明セクション）。

### Step 6: `driver.rs` に `v84000_tests` を追加

`v83900_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v84000_tests {
    #[test]
    fn cargo_toml_version_is_84_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"84.0.0\""), "Cargo.toml should have version 84.0.0");
    }

    #[test]
    fn changelog_has_v84_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v84.0.0"), "CHANGELOG.md should mention v84.0.0");
    }

    #[test]
    fn milestone_has_observability_2() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Observability 2.0"), "MILESTONE.md should mention Observability 2.0");
    }

    #[test]
    fn readme_mentions_fav_observe() {
        let content = include_str!("../../README.md");
        assert!(content.contains("fav observe"), "README.md should mention fav observe");
    }
}
```

### Step 7: `versions/current.md` 更新

```
最終更新: 2026-08-21 (v84.0.0)
最新安定版: v84.0.0 — Observability 2.0 宣言 — 3909 tests
```

### Step 8: `roadmap-v80.1-v85.0.md` Sprint 4 テーブル更新

Sprint 4（v83.1〜v84.0）の各行を「完了」に更新し、テスト数を drift 補正後の実際値に修正する。

### Step 9: `cargo test` で全テスト通過を確認

期待: 3909 tests pass（+4）、0 failures

### Step 10: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
