# v74.8.0 実装計画 — 統合デモ（v70〜v74 の全機能を使ったショーケース）

Date: 2026-08-14

---

## 実装ステップ

### Step 1: ショーケースディレクトリとファイルを作成

`infra/e2e-demo/favnir2-showcase/` を作成し、以下の 6 ファイルを配置する。

**pipeline.fav:**
```favnir
// v74.8.0 Showcase: Favnir 2.0 統合デモ
// 使用機能: v71 依存型 / v72 データコントラクト・SLA / v73 AI / v74 マルチテナント

import rune "privacy"
import rune "linalg"

contract ShowcaseContract {
    input:  { text: NonEmptyStr, tenant_id: String }
    output: { vector: Vec<Float>[1536], score: Float where self >= 0.0 }
    sla:    { max_latency_ms: 3000 }
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind rows  <- ctx.io.read_file_raw("data/input.csv")
    bind clean <- Rune.privacy.mask(rows, fields: ["email"])
    bind embed <- OpenAI.embed_batch(clean)
    bind score <- Rune.linalg.cosine_sim(embed, ctx.tenant.ref_vector)
    ctx.io.println(f"Done. mean_score={Float.mean(score)}")
}
```

**fav.toml:**
```toml
[project]
name = "favnir2-showcase"
version = "1.0.0"

[schedule]
daily-report = { cron = "0 9 * * *", pipeline = "pipeline.fav" }

[tenant]
isolation = "strict"
```

**rune.toml:**
```toml
[rune]
name = "favnir2-showcase-rune"
version = "1.0.0"

[dependencies]
privacy = "1.0.0"
linalg = "1.0.0"
```

**contract.fav:**
```favnir
// データコントラクト定義
contract ShowcaseInputContract {
    input:  { text: String, tenant_id: String }
    output: { validated: Bool }
}
```

**quality.fav:**
```favnir
// 品質スコアリングパイプライン
fn quality_score(data: List<String>) -> Int {
    data.len()
}
```

**README.md:**
```markdown
# Favnir 2.0 Showcase

v70〜v74 の全機能を統合したデモパイプライン。

## 実行手順

```bash
fav run pipeline.fav
```

## 機能一覧

- 依存型（Vec<Float>[1536]）: v71.x
- データコントラクト + SLA: v72.x
- マルチテナント設定: v74.2.0
- パイプラインスケジューリング: v74.5.0
- Rune マーケットプレイス: v74.1.0
- セキュリティ監査: v74.6.0
- Rune 品質基準: v74.7.0
```

### Step 2: `v748000_tests` モジュールを `driver.rs` に追加

`v747000_tests` の直後に追加する。
本テストは `include_str!` のみ使用し、外部シンボル不使用のため `use super::*` は不要。

```rust
#[cfg(test)]
mod v748000_tests {
    #[test]
    fn showcase_demo_structure_complete() {
        let fav_toml = include_str!("../../infra/e2e-demo/favnir2-showcase/fav.toml");
        assert!(fav_toml.contains("favnir2-showcase"), "project name missing");
        assert!(fav_toml.contains("schedule"), "schedule section missing");
        assert!(fav_toml.contains("tenant"), "tenant section missing");

        let readme = include_str!("../../infra/e2e-demo/favnir2-showcase/README.md");
        assert!(readme.contains("Favnir 2.0 Showcase"), "README title missing");
        assert!(readme.contains("pipeline.fav"), "pipeline.fav reference missing");
    }

    #[test]
    fn showcase_pipeline_fav_valid() {
        let pipeline = include_str!("../../infra/e2e-demo/favnir2-showcase/pipeline.fav");
        assert!(pipeline.contains("ShowcaseContract"), "contract name missing");
        assert!(pipeline.contains("contract"), "contract keyword missing");
        assert!(pipeline.contains("import rune"), "rune import missing");
        assert!(pipeline.contains("AppCtx"), "AppCtx missing");
        assert!(pipeline.contains("bind"), "bind keyword missing");
    }
}
```

**注意:** `include_str!("../../infra/e2e-demo/favnir2-showcase/...")` は
`fav/src/driver.rs` からの相対パス = `favnir/infra/e2e-demo/favnir2-showcase/`

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.7.0"` → `version = "74.8.0"`
- `driver.rs` 内の `version = \"74.7.0\"` を `version = \"74.8.0\"` に replace_all
- `version should be 74.7.0` を `version should be 74.8.0` に replace_all
- `cargo build` で `Cargo.lock` が自動更新される

### Step 4: テスト確認

- `cargo test v748000` で 2 件 pass を確認
- `cargo test` 全体で 3686 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.8.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.8.0)`
- 進行中: `v74.8.0`
- 次: `v74.9.0`
