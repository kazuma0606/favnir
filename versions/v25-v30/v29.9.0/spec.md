# v29.9.0 Spec — コミュニティ Rune コンテスト

**バージョン**: 29.9.0
**日付**: 2026-07-01
**フェーズ**: Ecosystem Maturity (phase 9)
**前バージョン**: v29.8.0 (ドキュメントサイト v3)

---

## 概要

Rune Registry（v29.1）を起点に、コミュニティが Rune を公開・共有する文化を育てる。
第 1 回 Favnir Rune コンテストを告知し、コミュニティ投稿 Rune 10 本のスタブを追加する。

> **ポジショニング**: ドキュメントサイト v3（v29.8）で「Favnir で何ができるか」が揃った。
> 次は「コミュニティが Rune を作って公開できる」仕組みを整える。
> `CONTRIBUTING.md` に Rune 開発ガイドを追記し、コンテスト告知で外部貢献を促進する。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `CONTRIBUTING.md` | Rune 開発ガイド追加（5 条件 / rune.toml / テスト方法）|
| `runes/stripe/` | Stripe 決済 Rune スタブ（コミュニティ貢献 Rune 代表）|
| `runes/twilio/` | Twilio SMS/Voice Rune スタブ |
| `runes/notion/` | Notion データベース Rune スタブ |
| `runes/linear/` | Linear Issue 管理 Rune スタブ |
| `runes/airtable/` | Airtable レコード操作 Rune スタブ |
| `runes/sendgrid/` | SendGrid メール送信 Rune スタブ |
| `runes/hubspot/` | HubSpot CRM Rune スタブ |
| `runes/zendesk/` | Zendesk チケット管理 Rune スタブ |
| `runes/shopify/` | Shopify 注文管理 Rune スタブ |
| `runes/intercom/` | Intercom メッセージ Rune スタブ |
| `site/app/community/page.tsx` | コンテスト告知セクション追加 |
| `fav/Cargo.toml` | version 29.8.0 → 29.9.0 |
| `CHANGELOG.md` | `[v29.9.0]` セクション追加 |
| `benchmarks/v29.9.0.json` | ベンチマーク記録 |
| `fav/src/driver.rs` | `v299000_tests` 6 件追加 |
| `versions/v25-v30/v29.9.0/tasks.md` | 実装完了後 COMPLETE に更新 |

---

## コミュニティ Rune コンテスト概要

### 募集要件（CONTRIBUTING.md に掲載）

| 条件 | 内容 |
|---|---|
| connect | サービスに接続できる（認証・エンドポイント設定）|
| read | データを取得できる（少なくとも 1 つの read 関数）|
| write | データを書き込める（少なくとも 1 つの write 関数）|
| error | エラーを `Result.err` で返す（クラッシュしない）|
| test | `cargo test` で 3 件以上 PASS する |

### コミュニティ投稿 Rune 10 本

| ディレクトリ | サービス | 代表関数 |
|---|---|---|
| `runes/stripe/` | Stripe 決済 | `Stripe.create_payment_intent` / `Stripe.retrieve_charge` |
| `runes/twilio/` | Twilio SMS | `Twilio.send_sms` / `Twilio.send_voice` |
| `runes/notion/` | Notion DB | `Notion.query_database` / `Notion.create_page` |
| `runes/linear/` | Linear Issue | `Linear.create_issue` / `Linear.update_issue` |
| `runes/airtable/` | Airtable | `Airtable.list_records` / `Airtable.create_record` |
| `runes/sendgrid/` | SendGrid | `SendGrid.send_email` / `SendGrid.send_template` |
| `runes/hubspot/` | HubSpot CRM | `HubSpot.create_contact` / `HubSpot.update_deal` |
| `runes/zendesk/` | Zendesk | `Zendesk.create_ticket` / `Zendesk.update_ticket` |
| `runes/shopify/` | Shopify | `Shopify.list_orders` / `Shopify.create_order` |
| `runes/intercom/` | Intercom | `Intercom.create_conversation` / `Intercom.send_message` |

---

## テスト戦略

### v299000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `contributing_md_has_rune_guide` | `CONTRIBUTING.md` が存在し `connect / read / write / error / test` を含む |
| `community_rune_stripe_exists` | `runes/stripe/stripe.fav` が存在し `Stripe.create_payment_intent` を含む |
| `community_rune_twilio_exists` | `runes/twilio/twilio.fav` が存在し `Twilio.send_sms` を含む |
| `community_rune_notion_exists` | `runes/notion/notion.fav` が存在し `Notion.query_database` を含む |
| `community_page_has_contest` | `site/app/community/page.tsx` が存在し `コンテスト` を含む |
| `changelog_has_v29_9_0` | `CHANGELOG.md` に `[v29.9.0]` が存在する |

テスト数: 2360 → **2366**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "29.9.0"
- [ ] `CONTRIBUTING.md` に `connect / read / write / error / test` を含む Rune 開発ガイドセクションが存在する
- [ ] コミュニティ Rune スタブ 10 本が `runes/` 下に存在する（各ディレクトリに `.fav` + `rune.toml` の 2 ファイル）
- [ ] `site/app/community/page.tsx` にコンテスト告知セクションあり
- [ ] `CHANGELOG.md` に `[v29.9.0]` セクションあり
- [ ] `benchmarks/v29.9.0.json` 存在（test_count: 2366）
- [ ] `cargo test --bin fav v299000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2366 tests PASS

---

## スコープ外

- 実際の Stripe / Twilio / Notion 等の API への接続実装 — HTTP 有効化後に対応
- Rune Registry（Lambda）への実際のアップロード — インフラ稼働後
- コンテスト審査・採点の自動化 — 手動レビュー
- コミュニティメンバーによる実際の Rune 投稿 — コンテスト期間中に受付
