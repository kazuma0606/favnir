# v33.2.0 — Plan: インクリメンタルコンパイル 確認・テスト補強

## 実装方針

インクリメンタルコンパイル（`IncrementalCache` / `fingerprint` / `dep_graph`）は
v19.3.0 で完成済み。v33.2.0 は v33.1.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"33.1.0"` → `"33.2.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_33_1_0` スタブ化 + `v332000_tests` 追加 |
| `CHANGELOG.md` | `[v33.2.0]` セクションを先頭に追記 |
| `benchmarks/v33.2.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v33.2.0 に更新 |
| `versions/v30-v35/v33.2.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_33_1_0` をスタブ化

```rust
// v331000_tests 内（既存の #[test] fn を空スタブに置き換える）
#[test]
fn cargo_toml_version_is_33_1_0() {
    // Stubbed: version bumped to 33.2.0 in v33.2.0.
}
```

### ② `v332000_tests` を挿入

挿入位置: `v331000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。

```rust
// ── v33.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v332000_tests {
    use crate::incremental::{dep_graph, fingerprint};
    use crate::frontend::parser::Parser;

    #[test]
    fn cargo_toml_version_is_33_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.2.0"), "Cargo.toml must contain '33.2.0'");
    }

    #[test]
    fn benchmark_v33_2_0_exists() {
        let src = include_str!("../../benchmarks/v33.2.0.json");
        assert!(src.contains("33.2.0"), "benchmarks/v33.2.0.json must contain '33.2.0'");
    }

    #[test]
    fn incremental_content_hash_deterministic() {
        // 同じコンテンツから同じ SHA-256 ハッシュが生成される（決定性）
        // v193000_tests::cache_invalidates_on_change とは異なる視点（ハッシュ自体の決定性確認）
        let bytes = b"fn main() -> Int { 42 }";
        let h1 = fingerprint::content_hash(bytes);
        let h2 = fingerprint::content_hash(bytes);
        assert_eq!(h1, h2, "content_hash must be deterministic");
        assert_eq!(h1.len(), 64, "SHA-256 hex should be 64 chars");
    }

    #[test]
    fn incremental_dep_graph_no_import_isolated() {
        // use 宣言のないファイルは affected_by に含まれない
        // v193000_tests::dep_graph_propagates の逆ケース（依存なし → 影響なし）
        let src = "fn main() -> Int { 1 }"; // use 宣言なし
        let prog = Parser::parse_str(src, "isolated.fav").expect("parse");
        let graph = dep_graph::build_dep_graph(&prog, "isolated");
        let affected = graph.affected_by("utils");
        assert!(
            !affected.contains(&"isolated".to_string()),
            "file with no imports should not be affected by utils, got: {:?}",
            affected
        );
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v33.1.0 完了時点 | — | 2500 |
| `cargo_toml_version_is_33_1_0` スタブ化 | 0（テストは残る） | 2500 |
| `v332000_tests` 追加（4 件） | +4 | **2504** |

---

## CHANGELOG 追記内容

```markdown
## [v33.2.0] — 2026-07-04

### Added
- `v332000_tests`: インクリメンタルコンパイル動作確認テスト 4 件
  - `cargo_toml_version_is_33_2_0` — バージョン確認
  - `benchmark_v33_2_0_exists` — ベンチマークファイル存在確認
  - `incremental_content_hash_deterministic` — SHA-256 ハッシュの決定性確認
  - `incremental_dep_graph_no_import_isolated` — 依存なしファイルが影響を受けないことを確認

### Notes
- `IncrementalCache` / `fingerprint` / `dep_graph` は v19.3.0 実装済み
- v33.2.0 は Performance & Tooling フェーズの記録としてインクリメンタルコンパイル動作を確認する
```
