# v33.3.0 — Spec: ストリーミング評価（#[streaming]）確認・テスト補強

## 概要

v33.3.0 は **ストリーミング評価（`#[streaming]` アノテーション）** の確認・テスト補強バージョン。

ロードマップ v33.3 のテーマ「`#[streaming]` でパイプラインをストリーミング評価に切り替える」は
v19.1.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `StreamingAnnotation` 構造体（`chunk_size: Option<i64>`）| ✓ | v19.1.0 |
| `#[streaming(chunk_size = N)]` / `#[streaming]` パース | ✓ | v19.1.0 |
| `FlwDef.streaming: Option<StreamingAnnotation>` フィールド | ✓ | v19.1.0 |
| `TrfDef.stateful: bool`（`#[stateful]` アノテーション）| ✓ | v19.1.0 |
| ストリーミングパイプライン実行（chunk 単位評価）| ✓ | v19.1.0 |
| `v191000_tests` — 4 件（`streaming_annotation_parses` 等）| ✓ | v19.1.0 |

v33.3.0 では新規実装は行わず、`v333000_tests` で動作を確認・記録するにとどまる
（v33.1〜v33.2 と同じ「確認・記録」パターン）。

---

## ストリーミング評価 仕様確認

### 構文

```favnir
// chunk_size 指定あり
#[streaming(chunk_size = 1000)]
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb

// chunk_size 省略（デフォルト）
#[streaming]
seq SimplePipeline = StageA |> StageB

// アノテーションなし = eager（通常評価）
seq EagerPipeline = StageA |> StageB
```

### AST 構造

```
FlwDef.streaming: Option<StreamingAnnotation>
  Some(StreamingAnnotation { chunk_size: Some(1000) })  // chunk_size 指定
  Some(StreamingAnnotation { chunk_size: None })         // #[streaming] のみ
  None                                                   // アノテーションなし
```

### opt-in 設計

`#[streaming]` は **opt-in**。アノテーションなしの `seq` は通常の eager 評価。

---

## 追加するテスト（v333000_tests — 4 件）

`v333000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- `use crate::frontend::parser::Parser;` を明示 import

テスト名は v191000_tests（`streaming_annotation_parses` / `streaming_default_chunk_size_parses` /
`streaming_pipeline_executes` / `streaming_stateful_annotation_parses`）と被らないよう
`streaming_` プレフィックスを使用。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_3_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.3.0"), "Cargo.toml must contain '33.3.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_3_0_exists() {
    let src = include_str!("../../benchmarks/v33.3.0.json");
    assert!(src.contains("33.3.0"), "benchmarks/v33.3.0.json must contain '33.3.0'");
}
```

### テスト 3: アノテーションなし seq → streaming: None（opt-in 確認）

v191000_tests はすべて `#[streaming]` ありのケースをテストしている。
v33.3.0 ではアノテーションなしの `seq` が `streaming: None` になることを確認し、
opt-in 設計を記録する。

```rust
fn streaming_seq_without_annotation_has_none() {
    // #[streaming] なしの seq は streaming: None（opt-in 設計）
    // v191000_tests はすべて #[streaming] ありのケースのみ確認
    use crate::frontend::parser::Parser;
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
```

### テスト 4: chunk_size = 1（境界値）

v191000_tests の `streaming_annotation_parses` は `chunk_size = 1000` を使用。
v33.3.0 では `chunk_size = 1`（最小境界値）でのパースを確認する。

```rust
fn streaming_chunk_size_boundary_one() {
    // chunk_size = 1（最小境界値）のパース確認
    // v191000_tests::streaming_annotation_parses は chunk_size = 1000 を使用
    use crate::frontend::parser::Parser;
    let src = "#[streaming(chunk_size = 1)]\nseq MinChunkPipeline = StageA |> StageB";
    let prog = Parser::parse_str(src, "test.fav").expect("parse");
    if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
        let s = fd.streaming.as_ref().expect("expected streaming annotation");
        assert_eq!(s.chunk_size, Some(1), "chunk_size = 1 should parse correctly");
    } else {
        panic!("expected FlwDef");
    }
}
```

---

## テストモジュールの配置

`v333000_tests` は `v332000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.3.0"`
- `cargo_toml_version_is_33_2_0` が空スタブになっていること
- `cargo test --bin fav v333000` — 4/4 PASS
- `cargo test` — 全件 PASS（2508 件、0 failures）
- `CHANGELOG.md` に `[v33.3.0]` セクション
- `benchmarks/v33.3.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.3.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.3.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
