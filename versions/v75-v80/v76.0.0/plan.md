# v76.0.0 実装計画 — Temporal Data Native 宣言 ★クリーンアップ

Date: 2026-08-15

---

## Step 1: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v76.0.0 エントリを追加する。

```markdown
## [v76.0.0] — 2026-08-15 — Temporal Data Native 宣言

Temporal Data Native スプリント（v75.1〜v75.9）の完成を宣言。
鮮度が型となり、SCD が構造となり、タイムトラベルが API となった。
Favnir のパイプラインは今、時間軸を型で保証する。

### Milestone
- Temporal Data Native 宣言（v75.1〜v75.9 完成）
- `FreshnessPolicy` / `TemporalRange` / `AsOfQuery` / `ScdRow` / `TemporalJoinConfig`
  / `RetentionPolicy` / `StreamFreshnessMonitor` / `TemporalContract` / `TimeTravelQuery` が揃った

### Tests
- `cargo_toml_version_is_76_0_0`
- `changelog_has_v76_0_0`
- `milestone_has_temporal_data_native`
- `readme_mentions_temporal`
```

---

## Step 2: MILESTONE.md 更新

`MILESTONE.md` の先頭（`## v75.0.0` の前）に v76.0.0 エントリを追加する。

```markdown
## v76.0.0（2026-08-15）— Temporal Data Native 宣言

> 「鮮度が型となり、SCD が構造となり、タイムトラベルが API となった。
>  Favnir のパイプラインは今、時間軸を型で保証する。」

**Temporal Data Native** の宣言バージョン。v75.1〜v75.9 で実装した
Temporal Data Native 基盤の完成を宣言した。

**v75.1〜v75.9 達成内容:**
- `FreshnessPolicy`（鮮度ポリシー型・Fail/Warn 戦略）— v75.1.0
- `TemporalRange` / `AsOfQuery` / `unix_secs_to_utc` / `is_leap`（時点型・UTC変換）— v75.2.0
- `ScdRow` / `apply_scd2_update` / `apply_scd1_update`（SCD 2.0 型安全更新）— v75.3.0
- `TemporalJoinConfig` / `format_temporal_join_sql`（時点結合SQL生成）— v75.4.0
- `RetentionPolicy` / `apply_retention_check`（データ保持ポリシー）— v75.5.0
- `StreamFreshnessMonitor` / `check_stream_lag`（ストリーム遅延監視）— v75.6.0
- `TemporalContract` / `validate_temporal_contract`（統合コントラクト検証）— v75.7.0
- `TimeTravelQuery` / `cmd_time_travel` / `parse_time_travel_timestamp`（タイムトラベルSQL）— v75.8.0
- 安定化・E2E テスト（`temporal_full_sprint_all_stable` / `temporal_e2e_pipeline_valid`）— v75.9.0
```

---

## Step 3: README.md 更新

`README.md` の `## v75.0 — Favnir 2.0 宣言` セクションの前に v76.0 セクションを追加する。

```markdown
## v76.0 — Temporal Data Native 宣言（2026-08-15）

Favnir v76.0 で **Temporal Data Native** を宣言しました。
鮮度が型となり、SCD が構造となり、タイムトラベルが API となりました。
`FreshnessPolicy` がデータの陳腐化をコンパイル時に検出し、
`TemporalContract` がパイプライン全体の時間的整合性を保証します。
`cmd_time_travel` が Snowflake・Delta・Generic の SQL 方言を型安全に生成します。
```

---

## Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml`: `75.9.0` → `76.0.0`
`driver.rs` 内の `75.9.0` バージョン文字列アサーションを `76.0.0` に一括更新（replace_all）。
注: `v76000_tests` モジュール内にはバージョン文字列アサーションが存在しないため、Cargo.toml の version 行と driver.rs 内の `cargo_toml_version_is_X` テストのアサーション文字列が対象となる。
注: `fav/Cargo.lock` は次の `cargo test` 実行時に自動更新される。

---

## Step 5: テストモジュール v76000_tests 追加

Cargo.toml 更新後にテストを追加する（`cargo_toml_version_is_76_0_0` が Cargo.toml を読むため、先にバージョンを更新しておく必要がある）。

`fav/src/driver.rs` の末尾に追加する：

```rust
#[cfg(test)]
mod v76000_tests {
    #[test]
    fn cargo_toml_version_is_76_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"76.0.0\""));
    }

    #[test]
    fn changelog_has_v76_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v76.0.0]"));
    }

    #[test]
    fn milestone_has_temporal_data_native() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Temporal Data Native"));
    }

    #[test]
    fn readme_mentions_temporal() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Temporal"));
    }
}
```

---

## Step 6: cargo check

`cargo check` でコンパイルエラーがないことを確認する。

---

## Step 7: versions/current.md 更新

- 進行中バージョン: v76.0.0
- 次に切る版: v76.1.0

---

## Step 8: ★cargo clean + hello.fav 復元

1. `cargo clean` を実行してビルドキャッシュをリセット
2. `fav/tmp/hello.fav` を復元（bootstrap テスト要件）:
   ```
   fn add(a: Int, b: Int) -> Int { a + b }
   fn main() -> Bool { add(1, 2) == 3 }
   ```

---

## Step 9: 最終確認

`cargo test` が 3714 tests all pass であることを確認。
