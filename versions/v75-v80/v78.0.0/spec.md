# v78.0.0 仕様書 — Verifiable Pipelines 宣言 ★クリーンアップ

Date: 2026-08-16
Status: 計画中

---

## Background

v77.1〜v77.9 で構築した Verifiable Pipelines 基盤（型・関数・CI 統合・反例・確率的契約・安定化）の完成を宣言するマイルストーンバージョン。

**宣言文**:
> 「不変条件が型となり、反例がコンパイラから届く。
>  Favnir のパイプラインは今、その正しさを証明できる。」

クリーンアップ作業（`cargo clean` / ドキュメント更新）と宣言テスト 4 件を追加する。

---

## Goals

1. `cargo clean` を実施してビルドキャッシュを削除する
2. `Cargo.toml` バージョンを `78.0.0` に更新する
3. `CHANGELOG.md` に v78.0.0 エントリを追加する
4. `MILESTONE.md` に「Verifiable Pipelines 宣言」エントリを追記する
5. `README.md` に v78.0 達成を追記する
6. `versions/current.md` を更新する
7. `v78000_tests` モジュール（4 件）を追加し 3760 tests に到達する（現在 3756）

---

## テスト仕様（`v78000_tests`）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_78_0_0` | `Cargo.toml` の version が `"78.0.0"` であること |
| `changelog_has_v78_0_0` | `CHANGELOG.md` に `"[v78.0.0]"` が存在すること |
| `milestone_has_verifiable_pipelines` | `MILESTONE.md` に `"Verifiable Pipelines"` が存在すること |
| `readme_mentions_verifiable_pipelines` | `README.md` に `"Verifiable Pipelines"` / `"v78.0"` が存在すること |

```rust
#[cfg(test)]
mod v78000_tests {
    #[test]
    fn cargo_toml_version_is_78_0_0() {
        // Cargo.toml は fav/Cargo.toml → driver.rs からの相対パスは "../Cargo.toml"
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"78.0.0\""));
    }

    #[test]
    fn changelog_has_v78_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v78.0.0]"));
    }

    #[test]
    fn milestone_has_verifiable_pipelines() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Verifiable Pipelines"));
    }

    #[test]
    fn readme_mentions_verifiable_pipelines() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Verifiable Pipelines"));
        assert!(content.contains("v78.0"));
    }
}
```

---

## MILESTONE.md エントリ仕様

先頭に以下を追加（既存の v77.0.0 エントリの前）:

```markdown
## v78.0.0（2026-08-16）— Verifiable Pipelines 宣言

> 「不変条件が型となり、反例がコンパイラから届く。
>  Favnir のパイプラインは今、その正しさを証明できる。」

**Verifiable Pipelines** の宣言バージョン。v77.1〜v77.9 で実装した
Verifiable Pipelines 基盤の完成を宣言した。

**v77.1〜v77.9 達成内容:**
- `PipelineInvariant` / `InvariantViolation` / `check_count_invariant`（不変条件基盤）— v77.1.0
- `FilterInvariant` / `check_filter_invariant`（フィルター系不変条件）— v77.2.0
- `AggregateInvariant` / `AggregateProperty` / `check_aggregate_invariant`（集約系不変条件）— v77.3.0
- `JoinInvariant` / `JoinType` / `JoinNullPolicy` / `check_join_invariant`（Join 系不変条件）— v77.4.0
- `VerificationReport` / `cmd_verify` / `format_verification_report`（verify コマンド基盤）— v77.5.0
- `CiVerificationConfig` / `CiResult` / `run_ci_verification` / `format_ci_result_summary`（CI 統合）— v77.6.0
- `CounterExampleResult` / `generate_counter_example_values`（反例自動生成）— v77.7.0
- `ProbabilisticContract` / `check_probabilistic_invariant`（確率的契約）— v77.8.0
- 安定化・E2E テスト（`verifiable_full_sprint_all_stable` / `verifiable_e2e_pipeline_verified`）— v77.9.0
```

## README.md エントリ仕様

既存の `## v77.0` セクションの前に追加:

```markdown
## v78.0 — Verifiable Pipelines 宣言（2026-08-16）

Favnir v78.0 で **Verifiable Pipelines** を宣言しました。
不変条件が型となり、反例がコンパイラから届きます。
`PipelineInvariant` がパイプラインの不変条件をファーストクラス型として表現し、
`check_aggregate_invariant` / `check_filter_invariant` / `check_join_invariant` が
集約・フィルター・Join の各レイヤーで型安全な検証を行います。
`generate_counter_example_values` が違反を引き起こす反例を自動生成し、
`check_probabilistic_invariant` がサンプリングベースの確率的契約を検証します。
`run_ci_verification` が CI パイプラインに組み込み可能な検証レポートを生成します。
```

---

## Success Criteria

- `cargo_toml_version_is_78_0_0` が pass
- `changelog_has_v78_0_0` が pass
- `milestone_has_verifiable_pipelines` が pass
- `readme_mentions_verifiable_pipelines` が pass
- `cargo test` が 3760 tests all pass
- `MILESTONE.md` の先頭が `## v78.0.0` エントリで始まる
- `README.md` に `## v78.0 — Verifiable Pipelines 宣言` セクションが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/Cargo.toml` — バージョンを `77.9.0` → `78.0.0` に更新
- `fav/Cargo.lock` — 自動更新（手動編集不要）
- `CHANGELOG.md` — v78.0.0 エントリを追加
- `MILESTONE.md` — Verifiable Pipelines 宣言エントリを追加
- `README.md` — v78.0 達成セクションを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/src/driver.rs` — `v78000_tests` モジュールを追加

---

## 対象外

- 新機能追加: 一切行わない（宣言バージョン）
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `cargo clean` は T0 の前提確認後に実施（`fav/tmp/hello.fav` を先に確認すること）
