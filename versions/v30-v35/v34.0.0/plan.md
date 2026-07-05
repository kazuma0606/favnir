# v34.0.0 — 実装プラン

## 方針

マイルストーン宣言パターン。v33.1〜v33.9 の確認完了を受けて「Performance & Tooling」を宣言する。`cargo clean` 必須（roadmap 規定）。

---

## 実装ステップ

### Step 0: cargo clean（必須）

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build 2>&1 | tail -3
```

`cargo build` が通ることを確認してから以降の作業を進める。
> `fav/tmp/hello.fav` は `target/` 以下にないため `cargo clean` で削除されない（事前確認済み）。復元不要。

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `33.9.0` → `34.0.0` に変更。

### Step 2: benchmarks/v34.0.0.json 作成

```json
{
  "version": "34.0.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2536,
  "tests_failed": 0,
  "notes": "Performance & Tooling マイルストーン宣言。cargo clean 実施。v340000_tests 4件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

### Step 3: driver.rs 更新

1. `cargo_toml_version_is_33_9_0` を空スタブ化（他3テストは残存・スタブ化しない）
2. `v339000_tests` ブロック末尾の `}` の直後、`// ── v31.7.0 tests` コメントの前に `v340000_tests` を挿入
   > `v339000_tests` は `#[cfg(not(target_arch = "wasm32"))]` でゲートされているが、`v340000_tests` にそのゲートは不要（`include_str!` のみ使用）

```rust
// ── v34.0.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v340000_tests {
    #[test]
    fn cargo_toml_version_is_34_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.0.0"), "Cargo.toml must contain '34.0.0'");
    }

    #[test]
    fn benchmark_v34_0_0_exists() {
        let src = include_str!("../../benchmarks/v34.0.0.json");
        assert!(src.contains("34.0.0"), "benchmarks/v34.0.0.json must contain '34.0.0'");
    }

    #[test]
    fn milestone_performance_tooling_declared() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Performance & Tooling"),
            "MILESTONE.md must contain 'Performance & Tooling'"
        );
    }

    #[test]
    fn readme_mentions_v34() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v34"), "README.md must mention 'v34'");
    }
}
```

### Step 4: MILESTONE.md 更新

先頭（最初の `##` セクションの前）に v34.0.0 セクションを追加:

```markdown
## v34.0.0 — Performance & Tooling（2026-07-04）

> 「`fav build --target native` でネイティブバイナリが生成でき、
>  10GB CSV を定常メモリで処理でき、
>  Lambda コールドスタートが 100ms 以下になること」
> = Performance & Tooling の完成を象徴する定義

v34.0.0 をもって、Favnir の **Performance & Tooling** を正式に宣言する。

...（spec.md 参照）
```

### Step 5: README.md 更新

v33.0 行の直後に v34.0 行を追加:

```markdown
**v34.0（2026-07-04）で、[Performance & Tooling](./MILESTONE.md) マイルストーンを宣言しました。**
AOT ネイティブバイナリ / インクリメンタルコンパイル / ストリーミング評価 / Arrow 統合 / WASM 最適化 / 並列コンパイルが揃い、「本番で速い」データパイプラインが実現しました。
```

### Step 6: CHANGELOG.md 更新

先頭に `[v34.0.0]` セクションを追加。

### Step 7: versions/current.md 更新

最新安定版を v34.0.0 に変更。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v340000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.0.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
