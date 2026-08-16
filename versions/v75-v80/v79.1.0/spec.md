# v79.1.0 仕様書 — 統合ショーケース基盤

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.0.0 で Execution Effects 1.0 を宣言した。
v79.1〜v79.9 では v75.1〜v79.0 で実装した全機能（Temporal / Provenance / Verifiable / Execution Effects）を統合するショーケース `infra/e2e-demo/favnir3-showcase/` を構築する。

v79.1.0 は `infra/e2e-demo/favnir2-showcase/` と同構造のショーケース骨格を作成する。

> **Note**: ロードマップ記載のテスト数（3782）は v79.0.0 宣言時点の stale 値。
> 実際のベースは 3787（v79.0.0 完了後の実測値）であり、本バージョン完了後は 3789 が正しい。

---

## Goals

`infra/e2e-demo/favnir3-showcase/` ディレクトリを作成し、以下の 4 ファイルを置く:

| ファイル | 内容 |
|---|---|
| `pipeline.fav` | 4 スプリント全機能統合パイプライン（骨格・各ステージのプレースホルダ） |
| `fav.toml` | `[effects.cached]` / `[effects.adaptive]` 設定 |
| `contract.fav` | `ShowcaseContract3` 宣言 |
| `README.md` | 概要・実行手順 |

---

## ファイル内容仕様

### `pipeline.fav`（骨格）

```favnir
// Favnir 3.0 統合ショーケース — pipeline.fav
// v75.1〜v79.8 全機能統合パイプライン

fn showcase_pipeline(ctx: AppCtx) -> Result<String, String> {
    // Stage 1: Temporal（v75.x）— 後続スプリントで実装
    // Stage 2: Provenance（v76.x）— 後続スプリントで実装
    // Stage 3: Verifiable（v77.x）— 後続スプリントで実装
    // Stage 4: Execution Effects（v78.x）— 後続スプリントで実装
    Result.ok("favnir3-showcase: pipeline skeleton initialized")
}
```

### `fav.toml`

```toml
[project]
name = "favnir3-showcase"
version = "1.0.0"

[schedule]
daily-report = { cron = "0 9 * * *", pipeline = "pipeline.fav" }

[effects.cached]
ttl_seconds = 300
max_entries = 1000

[effects.adaptive]
row_threshold = 100000
latency_target_ms = 500
# [effects.parallel] は v79.5.0 以降で追加予定のため本バージョンでは含まない
```

### `contract.fav`（ShowcaseContract3 宣言）

```favnir
// Favnir 3.0 統合ショーケース — contract.fav
// 全スプリント（Temporal + Provenance + Verifiable + Execution Effects）の統合コントラクト

type ShowcaseContract3 {
    pipeline_name: String,
    temporal_enabled: Bool,
    provenance_enabled: Bool,
    verifiable_enabled: Bool,
    execution_effects_enabled: Bool
}

fn validate_showcase_contract(c: ShowcaseContract3) -> Bool {
    c.temporal_enabled &&
    c.provenance_enabled &&
    c.verifiable_enabled &&
    c.execution_effects_enabled
}
```

### `README.md`

```markdown
# Favnir 3.0 統合ショーケース

v75.1〜v79.8 で実装した全機能（Temporal / Provenance / Verifiable / Execution Effects）を
統合したエンドツーエンドデモ。

## 実行手順

```bash
cd infra/e2e-demo/favnir3-showcase
fav run pipeline.fav
```

## 構成

- `pipeline.fav` — 統合パイプライン
- `fav.toml` — プロジェクト設定
- `contract.fav` — ShowcaseContract3 宣言
````

---

## テストモジュール仕様

```rust
// --- v79.1.0: 統合ショーケース基盤 ---
#[cfg(test)]
mod v791000_tests {
    use super::*;

    #[test]
    fn favnir3_showcase_structure_exists() {
        // infra/e2e-demo/favnir3-showcase/ の 4 ファイルが存在することを確認
        let pipeline = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
        let config   = include_str!("../../infra/e2e-demo/favnir3-showcase/fav.toml");
        let contract = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");
        let readme   = include_str!("../../infra/e2e-demo/favnir3-showcase/README.md");
        assert!(pipeline.contains("showcase_pipeline"), "pipeline.fav must define showcase_pipeline");
        assert!(config.contains("favnir3-showcase"), "fav.toml must contain project name");
        assert!(contract.contains("ShowcaseContract3"), "contract.fav must define ShowcaseContract3");
        assert!(readme.contains("Favnir 3.0"), "README.md must mention Favnir 3.0");
    }

    #[test]
    fn favnir3_showcase_contract_valid() {
        // contract.fav が ShowcaseContract3 と validate_showcase_contract を含むことを確認
        let contract = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");
        assert!(contract.contains("ShowcaseContract3"), "contract must define ShowcaseContract3 type");
        assert!(contract.contains("validate_showcase_contract"), "contract must define validate_showcase_contract fn");
        assert!(contract.contains("temporal_enabled"), "contract must have temporal_enabled field");
        assert!(contract.contains("execution_effects_enabled"), "contract must have execution_effects_enabled field");
    }
}
```

---

## CHANGELOG エントリ形式

```
## [v79.1.0] — 2026-08-16 — 統合ショーケース基盤

### Added
- `infra/e2e-demo/favnir3-showcase/pipeline.fav`: 4 スプリント全機能統合パイプライン骨格
- `infra/e2e-demo/favnir3-showcase/fav.toml`: `[effects.cached]` / `[effects.adaptive]` 設定
- `infra/e2e-demo/favnir3-showcase/contract.fav`: `ShowcaseContract3` 宣言（validate_showcase_contract）
- `infra/e2e-demo/favnir3-showcase/README.md`: 概要・実行手順

### Tests
- `favnir3_showcase_structure_exists`: 4 ファイルの存在と内容を検証
- `favnir3_showcase_contract_valid`: ShowcaseContract3 / validate_showcase_contract を検証
```

---

## Success Criteria

- `cargo test v791000` で 2 件が pass
- `cargo test` で 3789 tests pass（0 failures）
- `infra/e2e-demo/favnir3-showcase/` に 4 ファイルが存在する
- `contract.fav` が `ShowcaseContract3` と `validate_showcase_contract` を含む

---

## Files to modify / create

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir3-showcase/pipeline.fav` | 新規作成（骨格） |
| `infra/e2e-demo/favnir3-showcase/fav.toml` | 新規作成 |
| `infra/e2e-demo/favnir3-showcase/contract.fav` | 新規作成 |
| `infra/e2e-demo/favnir3-showcase/README.md` | 新規作成 |
| `fav/src/driver.rs` | `v791000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.1.0"` に更新 |
| `CHANGELOG.md` | v79.1.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
