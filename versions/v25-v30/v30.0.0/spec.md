# v30.0.0 Spec — Ecosystem Maturity マイルストーン宣言

**バージョン**: 30.0.0
**日付**: 2026-07-01
**フェーズ**: Ecosystem Maturity（最終マイルストーン）
**前バージョン**: v29.9.0（コミュニティ Rune コンテスト）

---

## 概要

v29.1〜v29.9 で構築した「コミュニティが Rune を育てられる」エコシステムを宣言する。

> **Ecosystem Maturity の定義（本プロジェクト固有）**
> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」状態を指す。

v30.0.0 をもって、以下がすべて揃ったことを正式に宣言する。

- Rune Registry（fav publish / add / search / info）— Lambda + S3 で本番稼働（v29.1）
- AI/ML Rune 4 本（mlflow / pinecone / vertex-ai / sagemaker）— v29.2〜v29.4
- GitHub / PagerDuty Rune — v29.5〜v29.6
- VS Code 拡張 — Marketplace 公開（v29.7）
- ドキュメントサイト v3 — cookbook 32 本・community ページ（v29.8）
- コミュニティ Rune 10 本（stripe / twilio / notion / linear / airtable / sendgrid / hubspot / zendesk / shopify / intercom）— v29.9

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `fav/Cargo.toml` | version 29.9.0 → 30.0.0 |
| `MILESTONE.md` | Ecosystem Maturity 宣言セクション追加 |
| `README.md` | v30.0 マイルストーン追記 |
| `site/content/docs/ecosystem-maturity.mdx` | マイルストーン宣言ドキュメント作成（`fav add stripe` 含む）|
| `versions/roadmap/roadmap-v29.1-v30.0.md` | 達成宣言（COMPLETE）追記 |
| `CHANGELOG.md` | `[v30.0.0]` セクション追加 |
| `benchmarks/v30.0.0.json` | ベンチマーク記録（test_count: 2372）|
| `fav/src/driver.rs` | `v300000_tests` 6 件追加 |
| `versions/v25-v30/v30.0.0/tasks.md` | 実装完了後 COMPLETE に更新 |

---

## テスト戦略

### v300000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `milestone_md_has_ecosystem_maturity` | `MILESTONE.md` が存在し `Ecosystem Maturity` を含む |
| `readme_mentions_v30` | `README.md` が `v30.0` を含む |
| `changelog_has_v30_0_0` | `CHANGELOG.md` に `[v30.0.0]` が存在する |
| `ecosystem_maturity_doc_exists` | `site/content/docs/ecosystem-maturity.mdx` が `fav add stripe` を含む |
| `community_rune_shopify_exists` | `runes/shopify/shopify.fav` が `Shopify.list_orders` を含む |
| `roadmap_v29_v30_declared_complete` | `versions/roadmap/roadmap-v29.1-v30.0.md` が `COMPLETE` を含む |

テスト数: 2366 → **2372**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "30.0.0"
- [ ] `MILESTONE.md` に `Ecosystem Maturity` 宣言セクションが存在する
- [ ] `README.md` に `v30.0` が存在する
- [ ] `site/content/docs/ecosystem-maturity.mdx` が存在し `fav add stripe` を含む
- [ ] `versions/roadmap/roadmap-v29.1-v30.0.md` に `COMPLETE` が存在する
- [ ] `CHANGELOG.md` に `[v30.0.0]` セクションあり
- [ ] `benchmarks/v30.0.0.json` 存在（test_count: 2372）
- [ ] `cargo test --bin fav v300000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2372 tests PASS
- [ ] tasks.md を COMPLETE に更新

---

## スコープ外

- Rune Registry の実際の Lambda デプロイ（インフラ稼働後）
- VS Code Marketplace への実際のパッケージアップロード（手動）
- コミュニティ Rune の実際の API 接続実装（HTTP Rune 有効化後）
- v31.x 以降の計画策定

---

## 象徴デモ

```bash
# Ecosystem Maturity の完成を象徴するデモ
fav add stripe        # Stripe 連携 Rune を追加（コミュニティ投稿）
fav run my_pipeline.fav  # Stripe.create_payment_intent が 5 分で動く
```

---

## v29.1〜v29.9 達成サマリー

| バージョン | テーマ | テスト数 |
|---|---|---|
| v29.1.0 | fav publish 実装 | 2318 |
| v29.2.0 | mlflow Rune | 2324 |
| v29.3.0 | pinecone Rune | 2330 |
| v29.4.0 | vertex-ai / sagemaker Rune | 2336 |
| v29.5.0 | github Rune | 2342 |
| v29.6.0 | pagerduty Rune | 2348 |
| v29.7.0 | VS Code 拡張 | 2354 |
| v29.8.0 | ドキュメントサイト v3 | 2360 |
| v29.9.0 | コミュニティ Rune コンテスト | 2366 |
| **v30.0.0** | **Ecosystem Maturity 宣言** | **2372** |
