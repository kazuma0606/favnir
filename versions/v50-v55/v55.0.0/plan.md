# Plan — v55.0.0 — Production 3.0 宣言

## ステップ

### Step 1: v54900_tests の `cargo_toml_version_is_54_9_0` を削除

Cargo.toml を 55.0.0 に更新すると `cargo_toml_version_is_54_9_0` が失敗する。
毎バージョンの規約通り、旧バージョンの `version_is_X` テストを削除してから Cargo.toml を更新する。

`fav/src/driver.rs` の `v54900_tests` から `cargo_toml_version_is_54_9_0` 関数を削除する。

### Step 2: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.0.0` に更新。

```toml
[package]
version = "55.0.0"
```

### Step 3: CHANGELOG.md — v55.0.0 エントリ追加

`CHANGELOG.md` 最上部に v55.0.0 エントリを挿入する。

```markdown
## [v55.0.0] — 2026-07-23 — Production 3.0 宣言

### Production 3.0 宣言

v51〜v54 で積み上げた全機能を最終確認し、Favnir v55.0 — Production 3.0 を宣言する。

- v51: Developer Experience 3.0（診断統一・インレイヒント・trace/watch）
- v52: Performance & Scale（par Tokio・バックプレッシャー・bench 回帰・WASM 最適化）
- v53: Data Quality & Observability 2.0（assert_schema・lineage 強化・audit-log）
- v54: Integration Sprint（explain --error 全網羅・watch-diff・dq-report・doctor・Production 3.0 整備）
- v55: Production 3.0 宣言・★クリーンアップ
```

### Step 4: driver.rs — v55000_tests 追加

`fav/src/driver.rs` の `v54900_tests` モジュールの直前に `v55000_tests` モジュールを挿入。
4 件のテストを追加する。

```rust
// -- v55000_tests (v55.0.0) -- Production 3.0 宣言 --
#[cfg(test)]
mod v55000_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_55_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"55.0.0\""),
            "Cargo.toml version should be 55.0.0"
        );
    }

    #[test]
    fn changelog_has_v55_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v55.0.0]"),
            "CHANGELOG.md should have v55.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_production3() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Production 3.0"),
            "MILESTONE.md should mention Production 3.0"
        );
    }

    #[test]
    fn readme_mentions_production3() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Production 3.0"),
            "README.md should mention Production 3.0"
        );
    }
}
```

### Step 5: テスト実行（クリーンアップ前）

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3206 tests passed, 0 failed`
（3203 - 1 削除 + 4 新規 = 3206。ロードマップ要件 failures=0 かつ ≥ 3201 を満たす。）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン（warnings/errors なし）

### Step 6: ★クリーンアップ

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

クリーンアップ後に再ビルド + 全テスト通過を確認する。

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

### Step 7: ポスト処理

- `versions/current.md` を v55.0.0 / 3206 tests に更新
  - 「最新安定版」→ v55.0.0
  - 「前バージョン」→ v54.9.0 / 3203 tests
  - 「次に切る版」→ 未定（Production 3.0 完成のため）
- `versions/roadmap/roadmap-v54.1-v55.0.md` の v55.0.0 実績を COMPLETE に更新

---

## 注意事項

- `milestone_has_production3` / `readme_mentions_production3` は v54.8.0 / v54.6.0 で実装済みのため
  追加実装不要。テストが pass することを確認するのみ。
- Step 1 で `cargo_toml_version_is_54_9_0` を削除するのは毎バージョンの規約。
  削除後テストカウントは 3202 になるが、v55000_tests の 4 件追加後は 3206 になる。
- `cargo clean` で `target/` を削除する。`fav/tmp/hello.fav` には影響しない。
  クリーンアップ後は `cargo test` を再実行して全通過を確認すること。
- `include_str!("../../CHANGELOG.md")` は `fav/src/driver.rs` からの相対パスで
  `favnir/CHANGELOG.md` を指す（`include_str!("../Cargo.toml")` = `fav/Cargo.toml` と同パターン）。
