# v79.8.0 仕様書 — ドキュメント完全化（v3 リファレンス）

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.7.0 で OSS コミュニティ整備を完了した。
v79.8.0 では v79.1〜v79.5 の 4 スプリント（Temporal / Provenance / Verifiable / Execution Effects）で追加した全機能のドキュメントを `site/content/docs/v3/` に作成する。

ロードマップの `**実装内容:**` にある 2 ファイルのみを本バージョンで実装する:
- `site/content/docs/v3/temporal.mdx`
- `site/content/docs/v3/migration-v2-v3.mdx`

> **スコープ外**: `provenance.mdx` / `verifiable.mdx` / `execution-effects.mdx` はロードマップ上「後続コミットで追加」とされているため本バージョンでは対象外とする。

> **Note**: テスト数はベース 3801（v79.7.0 完了後の実測値）。完了後は 3803。

---

## Goals

- `site/content/docs/v3/temporal.mdx` を新規作成し、FreshnessPolicy / AsOfQuery / SCD2 の概要を記述する
- `site/content/docs/v3/migration-v2-v3.mdx` を新規作成し、v2（v75.0）→ v3（v80.0）移行ガイドを記述する
- Rust テスト 2 件でファイル内容を検証する

---

## `temporal.mdx` 内容

```mdx
# Temporal（時間型データ処理）

Favnir 3.0 の Temporal 機能は、時刻・鮮度・履歴を型として表現します。

## FreshnessPolicy

データの鮮度を `max_age` で制約します:

```favnir
bind _ <- FreshnessPolicy.check(snapshot, max_age: Duration.hours(1))
```

## AsOfQuery

特定時刻時点のスナップショットを取得します:

```favnir
bind snapshot <- AsOfQuery { table: "orders", as_of_ts: ctx.run_ts }
```

## SCD2（緩やかに変化するディメンジョン）

履歴を保持しながらレコードを更新します:

```favnir
bind history <- apply_scd2_update(existing_customers, new_data, ctx.run_ts)
```
```

---

## `migration-v2-v3.mdx` 内容

```mdx
# v2 → v3 移行ガイド

Favnir v2（v75.0）から v3（v80.0）への移行手順を説明します。

## 主な変更点

| 機能 | v2 | v3 |
|---|---|---|
| 時間処理 | 手動 | Temporal（FreshnessPolicy / AsOfQuery）|
| データ来歴 | なし | Provenance（TracedData / DataSource）|
| 不変条件 | なし | Verifiable（contract / invariant）|
| 実行戦略 | 固定 | Execution Effects（!Adaptive / !Cached）|

## 移行手順

1. `ctx.run_ts` を使って時刻依存処理を Temporal に移行する
2. `TracedData.wrap` でデータ来歴を記録する
3. `contract` ブロックで不変条件を定義する
4. `!Adaptive !Cached` エフェクトで実行戦略を指定する
```

---

## テストモジュール仕様

```rust
// --- v79.8.0: ドキュメント完全化（v3 リファレンス）---
#[cfg(test)]
mod v798000_tests {
    const TEMPORAL:   &str = include_str!("../../site/content/docs/v3/temporal.mdx");
    const MIGRATION:  &str = include_str!("../../site/content/docs/v3/migration-v2-v3.mdx");

    #[test]
    fn docs_v3_temporal_exists() {
        assert!(TEMPORAL.contains("FreshnessPolicy"), "temporal.mdx must document FreshnessPolicy");
        assert!(TEMPORAL.contains("AsOfQuery"), "temporal.mdx must document AsOfQuery");
        assert!(TEMPORAL.contains("SCD"), "temporal.mdx must document SCD");
    }

    #[test]
    fn docs_v3_migration_guide_exists() {
        assert!(MIGRATION.contains("v2"), "migration-v2-v3.mdx must reference v2");
        assert!(MIGRATION.contains("v3"), "migration-v2-v3.mdx must reference v3");
        assert!(MIGRATION.contains("Temporal"), "migration-v2-v3.mdx must mention Temporal");
    }
}
```

注意:
- `include_str!` のパスは driver.rs から見た相対パス（`fav/src/` → `../../site/content/docs/v3/`）
- `use super::*` 不要（`include_str!` + `assert!` のみ）
- `const TEMPORAL` / `const MIGRATION` パターンを採用

---

## CHANGELOG エントリ形式

```
## [v79.8.0] — 2026-08-16 — ドキュメント完全化（v3 リファレンス）

### Added
- `site/content/docs/v3/temporal.mdx`: Temporal 機能リファレンス（FreshnessPolicy / AsOfQuery / SCD2）
- `site/content/docs/v3/migration-v2-v3.mdx`: v2 → v3 移行ガイド

### Tests
- `docs_v3_temporal_exists`: temporal.mdx に FreshnessPolicy / AsOfQuery / SCD が含まれることを検証
- `docs_v3_migration_guide_exists`: migration-v2-v3.mdx に v2 / v3 / Temporal が含まれることを検証
```

---

## Success Criteria

- `cargo test v798000` で 2 件が pass
- `cargo test` で 3803 tests pass（0 failures）
- `site/content/docs/v3/temporal.mdx` に `FreshnessPolicy` / `AsOfQuery` / `SCD` が存在する
- `site/content/docs/v3/migration-v2-v3.mdx` に `v2` / `v3` / `Temporal` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/v3/` | ディレクトリ新規作成 |
| `site/content/docs/v3/temporal.mdx` | 新規作成（Temporal リファレンス）|
| `site/content/docs/v3/migration-v2-v3.mdx` | 新規作成（v2→v3 移行ガイド）|
| `fav/src/driver.rs` | `v798000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.8.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.8.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
