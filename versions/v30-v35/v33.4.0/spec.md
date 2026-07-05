# v33.4.0 — Spec: Arrow 列指向統合 確認・テスト補強

## 概要

v33.4.0 は **Arrow 列指向統合** の確認・テスト補強バージョン。

ロードマップ v33.4 のテーマ「stage の出力を Arrow RecordBatch として格納し、Parquet 書き込みをゼロコピーに」は
v19.5.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `ArrowBatch` 型（VM opaque handle）| ✓ | v19.5.0 |
| `ArrowBatch.from_list` / `ArrowBatch.to_list` | ✓ | v19.5.0 |
| `ArrowBatch.write_parquet` / `ArrowBatch.read_parquet` | ✓ | v19.5.0 |
| `ArrowBatch.from_csv`（mmap CSV → RecordBatch）| ✓ | v19.5.0 / v20.5.0 |
| `#[arrow]` アノテーション → `TrfDef.arrow: bool` | ✓ | v19.5.0 |
| `v195000_tests` — 4 件（`arrow_batch_from_list` 等）| ✓ | v19.5.0 |
| `v205000_tests` — 4 件（`csv_mmap_reads_row_count` 等）| ✓ | v20.5.0 |

v33.4.0 では新規実装は行わず、`v334000_tests` で動作を確認・記録するにとどまる
（v33.1〜v33.3 と同じ「確認・記録」パターン）。

---

## Arrow 列指向統合 仕様確認

### ArrowBatch 型

VM 内で Arrow RecordBatch を opaque handle として保持する。
`VMValue::ArrowBatch(u64)` — ハンドル ID でストアを参照。

### TrfDef.arrow フィールド

```rust
pub struct TrfDef {
    ...
    pub arrow: bool,   // v19.5.0: `#[arrow]` annotation
    pub stateful: bool,// v19.1.0: `#[stateful]` annotation
    ...
}
```

`#[arrow]` と `#[stateful]` は独立したフラグ。片方だけ true にできる。

### 構文例

```favnir
// Arrow RecordBatch を直接操作する stage
#[arrow]
stage AnalyzeData: ArrowBatch -> ArrowBatch = |batch| {
    Result.ok(batch)
}

// アノテーションなし = 通常 stage（arrow: false）
stage Transform: List<Int> -> List<Int> = |rows| {
    Result.ok(rows)
}

// #[stateful] のみ（arrow: false, stateful: true）
#[stateful]
stage Accumulate: Int -> Int = |n| {
    Result.ok(n)
}
```

### Parquet ゼロコピー書き込み

`ArrowBatch.write_parquet(batch, path)` — Arrow RecordBatch を直接 Parquet に書き込む。
`ArrowBatch.from_list(rows) |> write_parquet` でゼロコピーパイプラインを実現。

---

## 追加するテスト（v334000_tests — 4 件）

`v334000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- `use crate::frontend::parser::Parser;` を明示 import
- AST バリアント参照は `crate::ast::Item::TrfDef(...)` の完全パスで記述（`use crate::ast::Item;` 不要）

テスト名は v195000_tests（`arrow_batch_from_list` / `arrow_batch_to_list` /
`arrow_parquet_roundtrip` / `arrow_stage_executes`）と被らないよう設計する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_4_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.4.0"), "Cargo.toml must contain '33.4.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_4_0_exists() {
    let src = include_str!("../../benchmarks/v33.4.0.json");
    assert!(src.contains("33.4.0"), "benchmarks/v33.4.0.json must contain '33.4.0'");
}
```

### テスト 3: `#[arrow]` なし stage は `arrow: false`（逆ケース）

v195000_tests::arrow_stage_executes は `#[arrow]` あり → `trf.arrow == true` を確認する。
v33.4.0 では `#[arrow]` なし stage が `arrow: false` になることを確認し、
デフォルト false の設計を記録する。

```rust
fn arrow_trf_without_annotation_has_false() {
    // #[arrow] なしの stage は arrow: false（v195000_tests の逆ケース）
    let src = "stage Transform: List<Int> -> List<Int> = |rows| { Result.ok(rows) }";
    let prog = Parser::parse_str(src, "test.fav").expect("parse");
    assert_eq!(prog.items.len(), 1, "expected 1 item");
    if let crate::ast::Item::TrfDef(trf) = &prog.items[0] {
        assert!(
            !trf.arrow,
            "stage without #[arrow] should have arrow: false"
        );
    } else {
        panic!("expected TrfDef");
    }
}
```

### テスト 4: `#[arrow]` と `#[stateful]` は独立

`#[stateful]` のみ付いた stage が `trf.arrow == false` かつ `trf.stateful == true` であることを確認。
2 つのアノテーションフラグが独立して管理されていることを記録する。

```rust
fn arrow_trf_arrow_and_stateful_are_independent() {
    // #[stateful] のみ → arrow: false, stateful: true（2フラグ独立）
    let src = "#[stateful]\nstage Accumulate: Int -> Int = |n| { Result.ok(n) }";
    let prog = Parser::parse_str(src, "test.fav").expect("parse");
    assert_eq!(prog.items.len(), 1, "expected 1 item");
    if let crate::ast::Item::TrfDef(trf) = &prog.items[0] {
        assert!(trf.stateful, "stage with #[stateful] should have stateful: true");
        assert!(!trf.arrow, "#[stateful]-only stage should have arrow: false");
    } else {
        panic!("expected TrfDef");
    }
}
```

---

## テストモジュールの配置

`v334000_tests` は `v333000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.4.0"`
- `cargo_toml_version_is_33_3_0` が空スタブになっていること
- `cargo test --bin fav v334000` — 4/4 PASS
- `cargo test` — 全件 PASS（2512 件、0 failures）
- `CHANGELOG.md` に `[v33.4.0]` セクション
- `benchmarks/v33.4.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.4.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.4.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
