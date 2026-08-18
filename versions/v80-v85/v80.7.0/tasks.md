# Tasks: v80.7.0 — スキーマスナップショットテスト（`SchemaSnapshot`）

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3823）と実際のベース（3828）が 7 件ずれているが、
> v80.2.0〜v80.6.0 の各 code-reviewer 対応で累積 7 件追加されたことが原因。完了時の目標は **3830**。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3828 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する（本スプリントでは v81.0.0 クリーンアップ時に更新する慣例）
- [x] `fav/src/driver.rs` に `mod v80600_tests` が存在することを確認する（v80.6.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.6.0 の `compute_test_coverage` / `coverage_pct` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `ColumnSnapshot` 構造体（`#[derive(Debug, Clone, PartialEq)]`、`name: String`, `type_name: String`, `nullable: bool`）を追加する
- [x] `SchemaSnapshot` 構造体（`#[derive(Debug, Clone)]`、`pipeline_name: String`, `columns: Vec<ColumnSnapshot>`）を追加する
- [x] `SchemaSnapshotDiff` 構造体（`#[derive(Debug)]`、`added: Vec<String>`, `removed: Vec<String>`, `changed: Vec<String>`）を追加する
- [x] `compare_schema_snapshots(current: &SchemaSnapshot, baseline: &SchemaSnapshot) -> SchemaSnapshotDiff` を実装する
  - `HashMap` で current / baseline の列を名前でインデックス化する
  - baseline の各列: current になければ `removed`、型/nullable が異なれば `changed`
  - current の各列: baseline になければ `added`
  - 出力の安定性のため `added` / `removed` / `changed` をそれぞれソートする
- [x] `format_schema_diff(diff: &SchemaSnapshotDiff) -> String` を実装する
  - 全空: `"OK: schema unchanged"`
  - 差分あり: `"added=[...], removed=[...], changed=[...]"`
- [x] `schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool` を実装する
  - `removed` または `changed` が非空なら `true`
  - `added` のみなら `false`

## T2: `fav/src/driver.rs` に `mod v80700_tests` を追加

- [x] `mod v80600_tests { ... }` の直後に `#[cfg(test)] mod v80700_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `baseline()` ヘルパー関数（id: Int / amount: Float の 2 列）を定義する
- [x] `schema_snapshot_no_diff_when_equal` テストを実装する
  - 同一スキーマ → 全フィールドが空
  - `format_schema_diff` が `"OK: schema unchanged"` を返すことを確認する
  - `schema_diff_is_breaking` が `false` を返すことを確認する
- [x] `schema_snapshot_detects_removed_column` テストを実装する
  - baseline: id + amount / current: id + note
  - `diff.removed == ["amount"]`、`diff.added == ["note"]`、`diff.changed` が空
  - `schema_diff_is_breaking` が `true` を返すことを確認する（removed があるため）
  - `format_schema_diff` が `"added=[note], removed=[amount], changed=[]"` を返すことを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3830 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

> 注意: テストモジュールに `changelog_has_vXX` テストが含まれるバージョンでは、
> T3（cargo test）より **前** に CHANGELOG を更新すること。
> 本バージョンの `v80700_tests` には CHANGELOG チェックテストは含まれないため順序は問わない。

- [x] `CHANGELOG.md` の先頭に v80.7.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
