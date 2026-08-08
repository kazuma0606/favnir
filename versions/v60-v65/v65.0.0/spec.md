# v65.0.0 Spec — Performance 1.0 宣言 ★クリーンアップ

Version: 65.0.0
Status: 未着手
Base tests: 3449
Target tests: 3453

---

## 概要

v64.1〜v64.9 で実装した「Performance 1.0」機能群を統合し、マイルストーン宣言を行う。

宣言文（MILESTONE.md に記載）:

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデータパイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

ロードマップ `roadmap-v64.1-v65.0.md` の v65.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3449 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"64.0.0"` であることを確認（`"65.0.0"` への更新対象）
- `MILESTONE.md` に `"Performance 1.0"` が含まれないことを確認（新規追加対象）
- `README.md` に `"Performance 1.0"` が含まれないことを確���（追記対象）
- `driver.rs` に `v64900_tests` が存在することを確認（`v65000_tests` の挿入位置）
- `driver.rs` に `v65000_tests` が存在しないことを確認（新規追加）

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新

```toml
version = "65.0.0"
```

### 2. `MILESTONE.md` — 宣言エントリ追加

ファイル先頭の `## v64.0.0` エントリの前に挿入:

```markdown
## v65.0.0（2026-08-02）— Performance 1.0

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデータパイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

**Performance 1.0** の宣言バージョン。v64.1〜v64.9 で実装した全機能を統合し、
AOT ネイティブコンパイル・差分ビルド・flamegraph プロファイリング・外部ベンチマーク比較・
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

### 3. `README.md` — v65.0.0 宣言追記

`**v64.0.0 — Incremental & Scale を宣言しました` で始まる行の直前に追加:

```markdown
**v65.0.0 — Performance 1.0 を宣言しました（2026-08-02）。**
v64.1〜v64.9 で実装した AOT ネイティブコンパイル・差分ビルド・flamegraph プロファイリング・
外部ベンチマーク比較・パフォーマンス lint・WASM ビルドを統合し、
「型安全」と「高速」を両立したデー���パイプライン言語としての完成を宣言した。
```

### 4. `CHANGELOG.md` — v65.0.0 エントリ追加

ファイル先頭（`## [v64.9.0]` の前）に挿入:

```markdown
## [v65.0.0] — 2026-08-02 — Performance 1.0 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に `"Performance 1.0"` 宣言文エントリを追加
- `v65000_tests`: 4 件追加（3449 → 3453 tests）
  - `cargo_toml_version_is_65_0_0`
  - `changelog_has_v65_0_0`
  - `milestone_has_performance1`
  - `readme_mentions_performance1`

### Changed
- `fav/Cargo.toml` version `"64.0.0"` → `"65.0.0"`
- `README.md` に Performance 1.0 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `fav build --target wasm32` CLI dispatch 統合・`fav lint --perf` CLI フラグは後送り（v66 以降）
```

### 5. `driver.rs` — `v65000_tests` 追加

`v64900_tests` の直前に挿入:

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

### 6. ★クリーンアップ（`cargo clean`）

テスト全通過後に `cargo clean` を実行し、クリーンビルドで全テストが通過することを確認する。

---

## 完了条件

- `fav/Cargo.toml` の version が `"65.0.0"`
- `MILESTONE.md` に `"Performance 1.0"` 宣言文エントリあり
- `README.md` に `"Performance 1.0"` または `"v65.0"` の言及あり
- `cargo test --bin fav v65000_tests` で 4 件 PASS
  - `cargo_toml_version_is_65_0_0` PASS
  - `changelog_has_v65_0_0` PASS
  - `milestone_has_performance1` PASS
  - `readme_mentions_performance1` PASS
- `cargo test -j 8 -- --test-threads=8` で 3453 tests passed, 0 failed
- `cargo clean` → `cargo test -j 8 -- --test-threads=8` で全通過

---

## 非スコープ

- `fav build --target wasm32` の `main.rs` CLI dispatch 統合（v66 以降）
- `fav lint --perf` CLI フラグ（`main.rs`）の統合（v66 以降）
- ドキュメントサイトのデプロイ（別途 `/deploy-site` で実施）

---

## 技術ノート

### `include_str!` パス

- `"../Cargo.toml"` → `fav/Cargo.toml`（`fav/src/driver.rs` から `../` = `fav/`）
- `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md`
- `"../../MILESTONE.md"` → `favnir/MILESTONE.md`
- `"../../README.md"` → `favnir/README.md`

### ★クリーンアップ注意事項

`cargo clean` で `fav/tmp/hello.fav` が削除される場合がある（既知の問題）。
`bootstrap_c2_artifact_roundtrip` テストが依存しているため、`cargo clean` 後に
`fav/tmp/hello.fav` が存在しない場合は復元すること。

内容: `fn add(a: Int, b: Int) -> Int { a + b }` + 改行 + `fn main() -> Bool { add(1, 2) == 3 }`
