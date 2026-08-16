# v79.8.0 実装計画 — ドキュメント完全化（v3 リファレンス）

Date: 2026-08-16

---

## 実装順序

### Step 1: `site/content/docs/v3/` ディレクトリ作成 + `temporal.mdx` 作成

`site/content/docs/v3/temporal.mdx` を新規作成:

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

### Step 2: `migration-v2-v3.mdx` 作成

`site/content/docs/v3/migration-v2-v3.mdx` を新規作成:

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

### Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.8.0 エントリを追加。

---

### Step 4: driver.rs — v798000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.8.0: ドキュメント完全化（v3 リファレンス）---
#[cfg(test)]
mod v798000_tests {
    const TEMPORAL:  &str = include_str!("../../site/content/docs/v3/temporal.mdx");
    const MIGRATION: &str = include_str!("../../site/content/docs/v3/migration-v2-v3.mdx");

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

注意: `use super::*` 不要。`const TEMPORAL` / `const MIGRATION` パターンを採用。

---

### Step 5: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.7.0"` → `"79.8.0"` に更新。

driver.rs 内の escaped `\"79.7.0\"` を `\"79.8.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.7.0` も `79.8.0` に更新。

更新後に `grep -c "79\.7\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.7.0: OSS 公開強化・コミュニティ整備 ---` コメント行の 1 件のみ）

---

### Step 6: versions/current.md 更新

- `## 進行中バージョン` → `**v79.8.0**（ドキュメント完全化 v3 リファレンス）`
- `## 次に切る版` → `**v79.9.0**（安定化・コードフリーズ）`

---

### Step 7: 最終確認

```bash
cargo test v798000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3803 tests pass、v798000 2 件 pass を確認。

---

## 依存順序サマリ

```
site/content/docs/v3/ 作成（Step 1）
  → migration-v2-v3.mdx 作成（Step 2）
  → CHANGELOG 更新（Step 3）
  → driver.rs テスト追加（Step 4）← 両 mdx が先に作成されていること
  → Cargo.toml + エラーメッセージ更新（Step 5）
  → current.md 更新（Step 6）
  → 最終確認（Step 7）
```
