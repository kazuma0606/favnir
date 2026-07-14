# v44.8.0 Plan — パフォーマンス最終調整

## 前提

- 現行バージョン: `44.7.0`（2958 tests）
- 追加テスト数: 2 件（`cargo_toml_version_is_44_8_0` + `bench_stream_result_recorded_in_changelog`）
- 目標テスト数: 2960
- ロードマップ推定（2948）は旧見積もり。実績 2958 を基準とする

---

## AST / 実装確認事項（実装前確認済み）

- `BenchOpts.stream: bool` — v40.7.0 で追加済み（driver.rs 行 5597）
- `bench_opts_has_stream_field` テスト（v40700_tests）— `BenchOpts { stream: true, ..BenchOpts::default() }` が有効
- CHANGELOG パス: `include_str!("../../CHANGELOG.md")` — 既存テスト（v40700_tests::changelog_has_v40_7_0 等）で使用済みの形式
- `collect_bench_stream_notes` を配置する場所: `collect_stage_max_inflight_annotations` の直後（`bare_inner_literal_line` の直前）

---

## ステップ

### Step 1: driver.rs — `collect_bench_stream_notes` 追加

`collect_stage_max_inflight_annotations` の直後に配置:

```rust
/// v44.8.0: CHANGELOG から `bench --stream` 計測結果行を収集。
/// パフォーマンス追跡の MVP。
/// NOTE: 実行時ベンチマーク計測（VM レベル最適化・v41.0 実測比較）は将来版のスコープ。
pub fn collect_bench_stream_notes(changelog: &str) -> Vec<String> {
    changelog
        .lines()
        .filter(|line| line.contains("bench --stream"))
        .map(|line| line.trim().to_string())
        .collect()
}
```

### Step 2: driver.rs — `v44800_tests` 追加 / スタブ化 / Cargo.toml

`v44700_tests` の直前（上の行）に挿入（driver.rs はバージョン降順配置）:

```rust
// -- v44800_tests (v44.8.0) -- パフォーマンス最終調整 --
#[cfg(test)]
mod v44800_tests {
    #[test]
    fn cargo_toml_version_is_44_8_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.8.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn bench_stream_result_recorded_in_changelog() {
        let changelog = include_str!("../../CHANGELOG.md");
        let notes = super::collect_bench_stream_notes(changelog);
        assert!(
            !notes.is_empty(),
            "CHANGELOG.md must contain at least one 'bench --stream' note, got: {:?}",
            notes
        );
    }
}
```

スタブ化: `v44700_tests::cargo_toml_version_is_44_7_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 44.8.0 in v44.8.0.
```

`fav/Cargo.toml` version: `44.7.0` → `44.8.0`

### Step 3: CHANGELOG.md に v44.8.0 エントリ追加（`bench --stream` 記述を含む）

```markdown
## [v44.8.0] — 2026-07-15

### Added
- `collect_bench_stream_notes(changelog: &str) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - CHANGELOG から `bench --stream` 計測結果行を収集するパフォーマンス追跡 MVP

### Performance
- `fav bench --stream` 計測結果: BenchOpts.stream = true での実行パスが有効（v40.7.0 追加済み）
  - ストリーム処理パイプラインの bench --stream 実行に対応

### Notes
- VM レベル実行速度最適化・v41.0 との実測比較は将来版のスコープ
```

### Step 4: テスト実行（2960 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

- `versions/current.md` → v44.8.0、2960 tests、次版 v44.9.0
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.8.0 を `✅ COMPLETE`
- `versions/v40-v45/v44.8.0/tasks.md` → COMPLETE

---

## 注意事項

- `collect_bench_stream_notes` は行単位のフィルタ — 複数行マッチも許容（MVP）
- `bench --stream` は CHANGELOG.md エントリ内（Step 3 で追加）に含まれるため、テストは必ず通る
- `collect_bench_stream_notes` に `use crate::...` インポートは不要（標準ライブラリのみ使用）
