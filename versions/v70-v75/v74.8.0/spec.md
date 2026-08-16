# v74.8.0 仕様書 — 統合デモ（v70〜v74 の全機能を使ったショーケース）

Date: 2026-08-14

---

## Background

v70〜v74 で実装した全機能（依存型・データコントラクト・SLA・マルチテナント・
パイプラインスケジューリング・Rune マーケットプレイス・セキュリティ監査・Rune 品質基準）を
一本のデモパイプラインにまとめたショーケースを `infra/e2e-demo/favnir2-showcase/` に作成する。

本バージョンはショーケースファイルの作成と、それを検証する Rust テストを追加する。
実際に fav コマンドでショーケースを実行する CI 統合は後続バージョン（v74.9.0 安定化）で対応する。

---

## Goals

1. `infra/e2e-demo/favnir2-showcase/` ディレクトリを作成し、以下の 6 ファイルを追加する
   - `pipeline.fav` — v71〜v74 の全フェーズ機能を網羅するメインパイプライン
   - `fav.toml` — マルチテナント + SLA + スケジュール設定
   - `rune.toml` — カスタム Rune 依存定義
   - `contract.fav` — データコントラクト定義
   - `quality.fav` — 品質スコアリングパイプライン
   - `README.md` — ショーケースの概要・実行手順
2. `v748000_tests` モジュール（2 件）を `driver.rs` に追加する
   - `showcase_demo_structure_complete`
   - `showcase_pipeline_fav_valid`

---

## ファイル内容仕様

### `pipeline.fav`

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

必須キーワード: `ShowcaseContract` / `contract` / `import rune` / `AppCtx` / `bind`

### `fav.toml`

```toml
[project]
name = "favnir2-showcase"
version = "1.0.0"

[schedule]
daily-report = { cron = "0 9 * * *", pipeline = "pipeline.fav" }

[tenant]
isolation = "strict"
```

必須キーワード: `favnir2-showcase` / `schedule` / `tenant`

### `rune.toml`

```toml
[rune]
name = "favnir2-showcase-rune"
version = "1.0.0"

[dependencies]
privacy = "1.0.0"
linalg = "1.0.0"
```

必須キーワード: `privacy` / `linalg`

### `contract.fav`

```favnir
// データコントラクト定義
contract ShowcaseInputContract {
    input:  { text: String, tenant_id: String }
    output: { validated: Bool }
}
```

必須キーワード: `contract` / `ShowcaseInputContract`

### `quality.fav`

```favnir
// 品質スコアリングパイプライン
fn quality_score(data: List<String>) -> Int {
    data.len()
}
```

必須キーワード: `quality_score`

### `README.md`

```markdown
# Favnir 2.0 Showcase

v70〜v74 の全機能を統合したデモパイプライン。

## 実行手順

```bash
fav run pipeline.fav
```

## 機能一覧

- 依存型（Vec<Float>[1536]）
- データコントラクト + SLA
- マルチテナント設定
- パイプラインスケジューリング
```

必須キーワード: `Favnir 2.0 Showcase` / `pipeline.fav`

---

## テスト仕様

**パス:** `include_str!("../../infra/e2e-demo/favnir2-showcase/...")` を使用。
`fav/src/driver.rs` から `../../` で 2 階層上がると `favnir/` に到達し、
既存の `../../CHANGELOG.md` と同一パターン。

### `showcase_demo_structure_complete`

`include_str!` で以下を確認:
- `fav.toml` に `"favnir2-showcase"` / `"schedule"` / `"tenant"` が含まれる
- `README.md` に `"Favnir 2.0 Showcase"` / `"pipeline.fav"` が含まれる

### `showcase_pipeline_fav_valid`

`include_str!` で以下を確認:
- `pipeline.fav` に `"ShowcaseContract"` / `"contract"` / `"import rune"` / `"AppCtx"` / `"bind"` が含まれる

**注:** `contract.fav` / `quality.fav` / `rune.toml` はファイル作成のみ確認し、内容検証テストは行わない（後続バージョンで対応）。

---

## Success Criteria

1. `showcase_demo_structure_complete` テストが pass する
2. `showcase_pipeline_fav_valid` テストが pass する
3. `cargo test` で 3686 tests pass（0 failures）

---

## スコープ外（明示的除外）

- `fav run pipeline.fav` の実際の実行（後続バージョンで対応）
- CI でのショーケース自動実行（v74.9.0 安定化スプリントで対応）
- `infra/` Terraform / AWS 設定（本バージョン対象外）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir2-showcase/pipeline.fav` | 新規作成 |
| `infra/e2e-demo/favnir2-showcase/fav.toml` | 新規作成 |
| `infra/e2e-demo/favnir2-showcase/rune.toml` | 新規作成 |
| `infra/e2e-demo/favnir2-showcase/contract.fav` | 新規作成 |
| `infra/e2e-demo/favnir2-showcase/quality.fav` | 新規作成 |
| `infra/e2e-demo/favnir2-showcase/README.md` | 新規作成 |
| `fav/src/driver.rs` | `v748000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.8.0"` に更新 |
| `CHANGELOG.md` | v74.8.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
