# v79.1.0 実装計画 — 統合ショーケース基盤

Date: 2026-08-16

---

## 実装順序

### Step 1: ショーケースディレクトリ作成

```bash
mkdir -p /c/Users/yoshi/favnir/infra/e2e-demo/favnir3-showcase
```

---

### Step 2: pipeline.fav 作成

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

---

### Step 3: fav.toml 作成

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
```

---

### Step 4: contract.fav 作成

```favnir
// Favnir 3.0 統合ショーケース — contract.fav

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

---

### Step 5: README.md 作成

---

### Step 6: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.1.0 エントリを追加。

---

### Step 7: driver.rs — v791000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.1.0: 統合ショーケース基盤 ---
#[cfg(test)]
mod v791000_tests {
    use super::*;

    #[test]
    fn favnir3_showcase_structure_exists() {
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
        let contract = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");
        assert!(contract.contains("ShowcaseContract3"), "contract must define ShowcaseContract3 type");
        assert!(contract.contains("validate_showcase_contract"), "contract must define validate_showcase_contract fn");
        assert!(contract.contains("temporal_enabled"), "contract must have temporal_enabled field");
        assert!(contract.contains("execution_effects_enabled"), "contract must have execution_effects_enabled field");
    }
}
```

---

### Step 8: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.0.0"` → `"79.1.0"` に更新。

driver.rs 内の `79.0.0` バージョン文字列アサーションを `79.1.0` に一括更新。

更新後に `grep -c "79.0.0" driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.0.0: Execution Effects 1.0 宣言 ★クリーンアップ ---` の 1 件のみ）

---

### Step 9: versions/current.md 更新

- `## 進行中バージョン` → `**v79.1.0**（統合ショーケース基盤）`
- `## 次に切る版` → `**v79.2.0**（Temporal showcase パイプライン）`

---

### Step 10: 最終確認

```bash
cargo test v791000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3789 tests pass、v791000 2 件 pass を確認。

---

## 依存順序サマリ

```
ショーケースファイル作成（Step 1〜5）
  → CHANGELOG 更新（Step 6）
  → driver.rs テスト追加（Step 7）  ← ショーケースファイルが先に存在する必要あり
  → Cargo.toml バージョン更新（Step 8）
  → current.md 更新（Step 9）
  → 最終確認（Step 10）
```
