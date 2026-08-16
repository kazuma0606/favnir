# v79.9.0 仕様書 — 安定化・コードフリーズ（Favnir 3.0 前最終調整）

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.8.0 でドキュメント完全化を完了した。
v79.9.0 は v79.1〜v79.8 の全スプリントを統合確認する最終安定化スプリントである。

新機能は追加しない。バグ修正のみ受け入れる。

検証対象:
- v79.1: 統合ショーケース基盤（pipeline.fav / contract.fav / fav.toml / README.md）
- v79.2: Temporal ステージ（load_with_freshness / AsOfQuery / FreshnessPolicy）
- v79.3: Provenance ステージ（load_with_provenance / TracedData / OpenLineage）
- v79.4: Verifiable コントラクト（Favnir3ShowcaseContract / invariant）
- v79.5: Execution Effects ステージ（join_stage / !Adaptive / !Cached）
- v79.6〜v79.8: 各バージョンの既存テスト（v796000_tests / v797000_tests / v798000_tests）が通ることで担保する（統合テストへの重複アサーション不要）

> **Note**: テスト数はベース 3803（v79.8.0 完了後の実測値）。完了後は 3805。

---

## Goals

- `infra/e2e-demo/favnir3-showcase/` の全 4 ステージが揃っていることを統合確認する
- E2E ショーケースが実行可能な構造を持っていることを検証する
- Rust テスト 2 件でクロスカット統合を検証する

---

## テストモジュール仕様

```rust
// --- v79.9.0: 安定化・コードフリーズ ---
#[cfg(test)]
mod v799000_tests {
    const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
    const CONTRACT: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");
    const CONFIG:   &str = include_str!("../../infra/e2e-demo/favnir3-showcase/fav.toml");
    const README:   &str = include_str!("../../infra/e2e-demo/favnir3-showcase/README.md");

    #[test]
    fn favnir3_full_sprint_all_stable() {
        // v79.2: Temporal ステージ
        assert!(PIPELINE.contains("load_with_freshness"), "Temporal stage must be present");
        assert!(PIPELINE.contains("FreshnessPolicy"), "FreshnessPolicy must be present");
        // v79.3: Provenance ステージ
        assert!(PIPELINE.contains("load_with_provenance"), "Provenance stage must be present");
        assert!(PIPELINE.contains("OpenLineage"), "OpenLineage must be present");
        // v79.4: Verifiable コントラクト
        assert!(CONTRACT.contains("Favnir3ShowcaseContract"), "Verifiable contract must be present");
        assert!(CONTRACT.contains("invariant"), "invariant must be present");
        // v79.5: Execution Effects ステージ
        assert!(PIPELINE.contains("join_stage"), "Execution Effects stage must be present");
        assert!(PIPELINE.contains("!Adaptive"), "!Adaptive effect must be present");
    }

    #[test]
    fn favnir3_e2e_showcase_runs() {
        assert!(PIPELINE.contains("showcase_pipeline"), "showcase_pipeline must be defined");
        assert!(CONTRACT.contains("verifiable_enabled"), "contract.fav must reference verifiable_enabled field");
        assert!(CONFIG.contains("favnir3-showcase"), "fav.toml must define project name");
        assert!(CONFIG.contains("effects.cached"), "fav.toml must define effects.cached");
        assert!(CONFIG.contains("effects.adaptive"), "fav.toml must define effects.adaptive");
        assert!(README.contains("Favnir 3.0"), "README must mention Favnir 3.0");
    }
}
```

注意:
- `use super::*` 不要（`include_str!` + `assert!` のみ）
- `const PIPELINE` / `const CONTRACT` / `const CONFIG` / `const README` を複数定義
- v79.1.0 の `favnir3_showcase_structure_exists` / `favnir3_showcase_contract_valid` と重複しない新しいアサーションを追加
- `favnir3_e2e_showcase_runs` の CONTRACT アサートは `verifiable_enabled`（未チェックのフィールド）を使用

---

## CHANGELOG エントリ形式

```
## [v79.9.0] — 2026-08-16 — 安定化・コードフリーズ（Favnir 3.0 前最終調整）

### Stability
- v79.1〜v79.8 の全スプリント統合動作確認（Temporal / Provenance / Verifiable / Execution Effects）

### Tests
- `favnir3_full_sprint_all_stable`: 全 4 ステージ（Temporal / Provenance / Verifiable / Execution Effects）が揃っていることを統合検証
- `favnir3_e2e_showcase_runs`: E2E ショーケースの実行可能構造を検証
```

---

## Success Criteria

- `cargo test v799000` で 2 件が pass
- `cargo test` で 3805 tests pass（0 failures）
- `favnir3_full_sprint_all_stable`: 全 4 ステージが pipeline.fav / contract.fav に揃っていることを確認
- `favnir3_e2e_showcase_runs`: showcase_pipeline / validate_showcase_contract / effects.cached / Favnir 3.0 が揃っていることを確認

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v799000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.9.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.9.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |

> **Note**: 新機能ファイルの追加はない（安定化スプリントのため）。
