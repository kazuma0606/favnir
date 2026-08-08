# v65.0.0 Plan — Performance 1.0 宣言 ★クリーンアップ

Version: 65.0.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

- ベーステスト数 3449 の確認
- `Cargo.toml` version が `"64.0.0"` であることを確認
- `MILESTONE.md` / `README.md` に `"Performance 1.0"` がないことを確認
- `driver.rs` に `v64900_tests` が存在し `v65000_tests` がないことを確認

### Step 2: `fav/Cargo.toml` — バージョン更新

```toml
version = "65.0.0"
```

（`version = "64.0.0"` を `version = "65.0.0"` に変更）

### Step 3: `MILESTONE.md` — 宣言エントリ追加

ファイル先頭の `## v64.0.0` 行の直前に以下を挿入:

```markdown
## v65.0.0（2026-08-02）— Performance 1.0

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデー���パイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

**Performance 1.0** の宣言バージョン。v64.1〜v64.9 で実装した全機���を統合し、
AOT ネイティブコン��イル・差分ビルド・flamegraph プロファイリング・外部ベンチマーク比較・
パフォーマンス lint・WASM ビルドの完成を宣言した。

**v64.1〜v64.9 達成内容:**
- v64.1（CI 統合）: `cmd_build_ci` / GitHub Actions テンプレート
- v64.2（リグレッション検出）: `BenchTomlConfig` / `[bench] regression_threshold_pct`
- v64.3（パフォーマンスガイド）: `site/content/docs/runtime/performance.mdx`
- v64.4（flamegraph AOT）: `cmd_profile_flamegraph_aot` / IR fns → SVG
- v64.5（外部ベンチ比較）: `site/content/docs/runtime/benchmarks.mdx` / `run_comparison.sh`
- v64.6（lint --perf）: `LintTomlConfig.perf` / `[lint] perf = true`
- v64.7（WASM ビルド）: `cmd_build_wasm` / `wasm_codegen_program`
- v64.8（総括記事）: `site/content/docs/performance/performance1-overview.mdx`
- v64.9（安定化）: `scale_all_v64_features_stable` / `performance1_overview_doc_complete`

**テスト数**: 3453

---
```

### Step 4: `README.md` — v65.0.0 宣言追記

`v64.0.0` の宣言行（`**v64.0.0 — Incremental & Scale を宣言しました`）の直前に追加:

```markdown
**v65.0.0 — Performance 1.0 を宣言しました（2026-08-02）。**
v64.1〜v64.9 で実装した AOT ネイティブコンパイル・差分ビルド・flamegraph プロファイリング・
外部ベンチマーク比較・パフォーマンス lint・WASM ビルドを統合し、
「型安全」と「高速」を両立したデー��パイプライン言語と���ての完成を宣言した。

```

### Step 5: `driver.rs` — `v65000_tests` 追加

`// -- v64900_tests` コメント行の直前に以下を挿入:

```rust
// -- v65000_tests (v65.0.0) -- Performance 1.0 宣言 --
#[cfg(test)]
mod v65000_tests {
    #[test]
    fn cargo_toml_version_is_65_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"65.0.0\""),
            "Cargo.toml should have version 65.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v65_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v65.0.0"), "CHANGELOG.md should mention v65.0.0");
    }

    #[test]
    fn milestone_has_performance1() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Performance 1.0"),
            "MILESTONE.md should contain 'Performance 1.0'"
        );
    }

    #[test]
    fn readme_mentions_performance1() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Performance 1.0") || readme.contains("v65.0"),
            "README.md should mention Performance 1.0 or v65.0"
        );
    }
}
```

### Step 6: ビルド・テスト（クリーンアップ前）

```bash
cargo build 2>&1 | tail -5
cargo test --bin fav v65000_tests 2>&1 | tail -10
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

### Step 7: ★クリーンアップ

```bash
cargo clean
# fav/tmp/hello.fav が消えていれば復元
# 内容: fn add(a: Int, b: Int) -> Int { a + b }
#        fn main() -> Bool { add(1, 2) == 3 }
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

### Step 8: ドキュメント更新（T5）

- `CHANGELOG.md` 先頭に v65.0.0 エントリ追加
- `roadmap-v64.1-v65.0.md` v65.0 セクションに実績追記
- `versions/current.md` を v65.0.0（3453 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `v65000_tests` は `use super::*` 不要（`include_str!` のみ使用）
- `cargo_toml_version_is_65_0_0` は `../Cargo.toml`（`fav/src/` から `../` = `fav/`）
- `changelog_has_v65_0_0` / `milestone_has_performance1` / `readme_mentions_performance1` は `../../` = `favnir/`
- `★クリーンアップ` 後の `hello.fav` 復元を忘れないこと（MEMORY.md に既知問題として記録済み）
- README.md の挿入位置は `**v64.0.0 — Incremental & Scale` で grep して特定する
