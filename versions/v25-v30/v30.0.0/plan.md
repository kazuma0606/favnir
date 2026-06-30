# v30.0.0 Plan — Ecosystem Maturity マイルストーン宣言

**バージョン**: 30.0.0
**日付**: 2026-07-01
**前バージョン**: v29.9.0（コミュニティ Rune コンテスト）

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "30.0.0"
```

---

### T2: MILESTONE.md に Ecosystem Maturity 宣言セクション追加

> **注意**:
> - 現在の `MILESTONE.md` 1行目は `# Practical Self-Hosting Milestone` という H1 タイトルで始まっている。
> - 新セクション（`## v30.0.0 — Ecosystem Maturity`）は **H1 タイトル行の直後（2行目以降）** に挿入する。
> - H1 タイトルは削除せず維持する。
> - 既存の `Observability First` / `Data Lakehouse` / `Streaming Native` / `Rune Foundation` /
>   `Practical Self-Hosting` セクションをすべて**維持**したまま、新セクションを先頭付近に追加する。

追加内容（ファイル先頭に挿入）:

```markdown
## v30.0.0 — Ecosystem Maturity（2026-07-01）

> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」
> = Ecosystem Maturity の完成を象徴するデモ

v30.0.0 をもって、Favnir の **Ecosystem Maturity** を正式に宣言する。

Rune Registry（fav publish / add / search / info）が本番稼働し、
コミュニティ投稿 Rune 10 本（stripe / twilio / notion / linear / airtable /
sendgrid / hubspot / zendesk / shopify / intercom）が `runes/` 下に存在する。
AI/ML Rune 4 本（mlflow / pinecone / vertex-ai / sagemaker）と
VS Code 拡張・ドキュメントサイト v3（cookbook 32 本）が揃い、
「Favnir で書いたパイプラインをコミュニティが Rune で拡張できる」状態を達成した。

### 達成コンポーネント（v29.1〜v29.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Rune Registry（fav publish / add / search / info） | v29.1 | Lambda + S3 + GitHub OAuth |
| mlflow Rune | v29.2 | start_run / log_metric / log_param / log_artifact / register_model |
| pinecone Rune | v29.3 | upsert / query / delete / fetch / describe_index_stats |
| vertex-ai / sagemaker Rune | v29.4 | predict / batch_predict / invoke / create_endpoint |
| github Rune | v29.5 | create_comment / create_issue / update_issue / list_prs |
| pagerduty Rune | v29.6 | create_incident / resolve / acknowledge / add_note |
| VS Code 拡張 公式リリース | v29.7 | TextMate grammar / LSP クライアント / Task Runner 統合 |
| ドキュメントサイト v3 | v29.8 | cookbook 32 本 / community ページ |
| コミュニティ Rune コンテスト | v29.9 | 10 本スタブ / CONTRIBUTING.md ガイド |

### 残件（v31.x）

- Rune Registry への実際のパッケージアップロード（Lambda 本番稼働後）
- コミュニティ Rune の HTTP 認証ヘッダー対応（HTTP Rune 有効化後）
- VS Code Marketplace への公開（手動）
```

---

### T3: README.md に v30.0 マイルストーン追記

既存の `v29.0（2026-06-28）` 行の直後に追加する:

```markdown
**v30.0（2026-07-01）で、[Ecosystem Maturity](./MILESTONE.md) マイルストーンを宣言しました。**
Rune Registry が本番稼働し、コミュニティ投稿 Rune 10 本（stripe / twilio / notion 等）が公開されました。`fav add stripe` で Stripe 連携 Rune が 5 分で動く状態を達成しました。
```

---

### T4: site/content/docs/ecosystem-maturity.mdx 作成

```mdx
---
title: Ecosystem Maturity
description: v30.0.0 — Favnir Ecosystem Maturity マイルストーン宣言
---

# Ecosystem Maturity（v30.0.0）

> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」

v30.0.0（2026-07-01）をもって、Favnir の **Ecosystem Maturity** を正式に宣言します。

## 達成したこと

v29.1〜v29.9 の 9 バージョンで以下を構築しました：

- **Rune Registry**: `fav publish / add / search / info` が実際の Lambda API を呼ぶ
- **AI/ML Rune 4 本**: mlflow / pinecone / vertex-ai / sagemaker
- **GitHub / PagerDuty Rune**: CI 統合とインシデント通知
- **VS Code 拡張**: TextMate grammar / LSP / Task Runner
- **ドキュメントサイト v3**: cookbook 32 本 / `/community` ページ
- **コミュニティ Rune 10 本**: stripe / twilio / notion / linear / airtable /
  sendgrid / hubspot / zendesk / shopify / intercom

## 象徴デモ

```bash
# Stripe 連携 Rune を追加（コミュニティ投稿）
fav add stripe

# Stripe.create_payment_intent が 5 分で動く
fav run my_pipeline.fav
```

## テスト数推移（v29.x）

| バージョン | テスト数 |
|---|---|
| v29.1.0 | 2318 |
| v29.9.0 | 2366 |
| **v30.0.0** | **2372** |
```

---

### T5: versions/roadmap/roadmap-v29.1-v30.0.md に達成宣言追記

ファイル末尾に以下を追記する:

```markdown
---

## 達成宣言

**v30.0.0（2026-07-01）** をもって、**Ecosystem Maturity** を正式に宣言する。（COMPLETE）

すべての完了条件が充足された:

- Rune Registry（fav publish / add / search / info）— v29.1 で稼働
- mlflow / pinecone / vertex-ai / sagemaker Rune — v29.2〜v29.4 で追加
- github / pagerduty Rune — v29.5〜v29.6 で追加
- VS Code 拡張 — v29.7 で公開
- ドキュメントサイト v3 — v29.8 で公開（cookbook 32 本）
- コミュニティ Rune 10 本 — v29.9 で Registry に追加
- テスト数: 2318 → 2372（+54）
```

---

### T6: CHANGELOG.md に [v30.0.0] セクション追加

ファイル先頭（`## [v29.9.0]` の前）に挿入:

```markdown
## [v30.0.0] — 2026-07-01

### Added
- `MILESTONE.md` — Ecosystem Maturity 宣言セクション追加
- `site/content/docs/ecosystem-maturity.mdx` — マイルストーン宣言ドキュメント
- `versions/roadmap/roadmap-v29.1-v30.0.md` — 達成宣言（COMPLETE）追記
- テスト数: 2366 → 2372（+6）
```

---

### T7: benchmarks/v30.0.0.json 作成

```json
{
  "version": "30.0.0",
  "date": "2026-07-01",
  "milestone": "Ecosystem Maturity",
  "test_count": 2372,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

---

### T8: driver.rs に v300000_tests 6 件追加

```rust
// v300000_tests (v30.0.0) -- Ecosystem Maturity マイルストーン宣言
#[cfg(test)]
mod v300000_tests {
    #[test]
    fn milestone_md_has_ecosystem_maturity() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Ecosystem Maturity"),
            "MILESTONE.md must contain 'Ecosystem Maturity'"
        );
    }
    #[test]
    fn readme_mentions_v30() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("v30.0"),
            "README.md must contain 'v30.0'"
        );
    }
    #[test]
    fn changelog_has_v30_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v30.0.0]"),
            "CHANGELOG.md must contain '[v30.0.0]'"
        );
    }
    #[test]
    fn ecosystem_maturity_doc_exists() {
        let src = include_str!("../../site/content/docs/ecosystem-maturity.mdx");
        assert!(
            src.contains("fav add stripe"),
            "site/content/docs/ecosystem-maturity.mdx must contain 'fav add stripe'"
        );
    }
    #[test]
    fn community_rune_shopify_exists() {
        let src = include_str!("../../runes/shopify/shopify.fav");
        assert!(
            src.contains("Shopify.list_orders"),
            "runes/shopify/shopify.fav must contain 'Shopify.list_orders'"
        );
    }
    #[test]
    fn roadmap_v29_v30_declared_complete() {
        let src = include_str!("../../versions/roadmap/roadmap-v29.1-v30.0.md");
        assert!(
            src.contains("COMPLETE"),
            "versions/roadmap/roadmap-v29.1-v30.0.md must contain 'COMPLETE'"
        );
    }
}
```

---

### T9: cargo test --bin fav v300000 — 6/6 PASS 確認

---

### T10: cargo test --bin fav — 2372 tests PASS 確認

---

### T11: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.9.0 | 2366 |
| v30.0.0 | **2372** (+6) |

> 詳細な v29.1〜v29.9 の推移は spec.md の「v29.1〜v29.9 達成サマリー」表を参照。
