# v33.3.0 — Plan: ストリーミング評価 確認・テスト補強

## 実装方針

`#[streaming]` アノテーション・`StreamingAnnotation` 構造体・ストリーミングパイプライン実行は
v19.1.0 で完成済み。v33.3.0 は v33.1〜v33.2 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"33.2.0"` → `"33.3.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_33_2_0` スタブ化 + `v333000_tests` 追加 |
| `CHANGELOG.md` | `[v33.3.0]` セクションを先頭に追記 |
| `benchmarks/v33.3.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v33.3.0 に更新 |
| `versions/v30-v35/v33.3.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_33_2_0` をスタブ化

```rust
#[test]
fn cargo_toml_version_is_33_2_0() {
    // Stubbed: version bumped to 33.3.0 in v33.3.0.
}
```

### ② `v333000_tests` を挿入

挿入位置: `v332000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。

```rust
// ── v33.3.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v333000_tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn cargo_toml_version_is_33_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.3.0"), "Cargo.toml must contain '33.3.0'");
    }

    #[test]
    fn benchmark_v33_3_0_exists() {
        let src = include_str!("../../benchmarks/v33.3.0.json");
        assert!(src.contains("33.3.0"), "benchmarks/v33.3.0.json must contain '33.3.0'");
    }

    #[test]
    fn streaming_seq_without_annotation_has_none() {
        // #[streaming] なしの seq は streaming: None（opt-in 設計）
        // v191000_tests はすべて #[streaming] ありのケースのみ確認
        let src = "seq EagerPipeline = StageA |> StageB";
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
            assert!(
                fd.streaming.is_none(),
                "seq without #[streaming] should have streaming: None"
            );
        } else {
            panic!("expected FlwDef");
        }
    }

    #[test]
    fn streaming_chunk_size_boundary_one() {
        // chunk_size = 1（最小境界値）のパース確認
        // v191000_tests::streaming_annotation_parses は chunk_size = 1000 を使用
        let src = "#[streaming(chunk_size = 1)]\nseq MinChunkPipeline = StageA |> StageB";
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
            let s = fd.streaming.as_ref().expect("expected streaming annotation");
            assert_eq!(s.chunk_size, Some(1), "chunk_size = 1 should parse correctly");
        } else {
            panic!("expected FlwDef");
        }
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v33.2.0 完了時点 | — | 2504 |
| `cargo_toml_version_is_33_2_0` スタブ化 | 0 | 2504 |
| `v333000_tests` 追加（4 件） | +4 | **2508** |

---

## CHANGELOG 追記内容

```markdown
## [v33.3.0] — 2026-07-04

### Added
- `v333000_tests`: ストリーミング評価（`#[streaming]`）動作確認テスト 4 件
  - `cargo_toml_version_is_33_3_0` — バージョン確認
  - `benchmark_v33_3_0_exists` — ベンチマークファイル存在確認
  - `streaming_seq_without_annotation_has_none` — アノテーションなし seq が `streaming: None`（opt-in 確認）
  - `streaming_chunk_size_boundary_one` — `chunk_size = 1`（最小境界値）のパース確認

### Notes
- `#[streaming]` / `StreamingAnnotation` / ストリーミングパイプライン実行は v19.1.0 実装済み
- v33.3.0 は Performance & Tooling フェーズの記録としてストリーミング評価動作を確認する
```
