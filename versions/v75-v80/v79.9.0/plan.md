# v79.9.0 実装計画 — 安定化・コードフリーズ（Favnir 3.0 前最終調整）

Date: 2026-08-16

---

## 実装順序

### Step 1: E2E ショーケース統合確認（手動チェック）

`infra/e2e-demo/favnir3-showcase/` の全ファイルを確認する:

- `pipeline.fav`: 全 4 ステージが揃っているか
  - `load_with_freshness`（Temporal v79.2）
  - `load_with_provenance`（Provenance v79.3）
  - `join_stage`（Execution Effects v79.5）
  - `showcase_pipeline`（基盤 v79.1）
- `contract.fav`: `Favnir3ShowcaseContract` / `invariant`（Verifiable v79.4）
- `fav.toml`: `[effects.cached]` / `[effects.adaptive]`（v79.1）
- `README.md`: `Favnir 3.0` 記述

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.9.0 エントリを追加。

---

### Step 3: driver.rs — v799000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

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

注意: `use super::*` 不要。4 つの `const` を定義するパターン。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.8.0"` → `"79.9.0"` に更新。

driver.rs 内の escaped `\"79.8.0\"` を `\"79.9.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.8.0` も `79.9.0` に更新。

更新後に `grep -c "79\.8\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.8.0: ドキュメント完全化（v3 リファレンス）---` コメント行の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.9.0**（安定化・コードフリーズ）`
- `## 次に切る版` → `**v80.0.0**（Favnir 3.0 宣言 ★クリーンアップ）`

---

### Step 6: 最終確認

```bash
cargo test v799000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3805 tests pass、v799000 2 件 pass を確認。

---

## 依存順序サマリ

```
E2E 確認（Step 1）— 手動確認のみ
  → CHANGELOG 更新（Step 2）
  → driver.rs テスト追加（Step 3）
  → Cargo.toml + エラーメッセージ更新（Step 4）
  → current.md 更新（Step 5）
  → 最終確認（Step 6）
```
