# v29.9.0 Tasks — コミュニティ Rune コンテスト

**状態**: COMPLETE
**開始日**: 2026-07-01
**完了日**: 2026-07-01

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.8.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2360 passed` を含むこと
- [x] `driver.rs` に `mod v299000_tests` が存在しないこと
- [x] `CONTRIBUTING.md` に `コミュニティ Rune 開発ガイド` セクション（connect / read / write / error / test）が存在しないこと
- [x] `runes/stripe/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.8.0` → `29.9.0` | [x] |
| T2 | 既存 `CONTRIBUTING.md` に `## コミュニティ Rune 開発ガイド` セクション追記（connect / read / write / error / test）| [x] |
| T3 | コミュニティ Rune スタブ 10 本作成（stripe/twilio/notion/linear/airtable/sendgrid/hubspot/zendesk/shopify/intercom）| [x] |
| T3.1 | `runes/linear/linear.fav` に `Linear.create_issue` を含む（手動確認）| [x] |
| T3.2 | `runes/airtable/airtable.fav` に `Airtable.list_records` を含む（手動確認）| [x] |
| T3.3 | `runes/sendgrid/sendgrid.fav` に `SendGrid.send_email` を含む（手動確認）| [x] |
| T3.4 | `runes/hubspot/hubspot.fav` に `HubSpot.create_contact` を含む（手動確認）| [x] |
| T3.5 | `runes/zendesk/zendesk.fav` に `Zendesk.create_ticket` を含む（手動確認）| [x] |
| T3.6 | `runes/shopify/shopify.fav` に `Shopify.list_orders` を含む（手動確認）| [x] |
| T3.7 | `runes/intercom/intercom.fav` に `Intercom.create_conversation` を含む（手動確認）| [x] |
| T4 | `site/app/community/page.tsx` にコンテスト告知セクション追加（既存 `GitHub Discussions` 維持）| [x] |
| T5 | `CHANGELOG.md` に `[v29.9.0]` セクション追加 | [x] |
| T6 | `benchmarks/v29.9.0.json` 作成（test_count: 2366）| [x] |
| T7 | `driver.rs` に `v299000_tests` 6 件追加 | [x] |
| T8 | `cargo test --bin fav v299000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2366 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## T3 作成ファイル一覧（10 本 × 2 ファイル）

| ディレクトリ | ファイル | 必須含有文字列 |
|---|---|---|
| `runes/stripe/` | `stripe.fav` / `rune.toml` | `Stripe.create_payment_intent` |
| `runes/twilio/` | `twilio.fav` / `rune.toml` | `Twilio.send_sms` |
| `runes/notion/` | `notion.fav` / `rune.toml` | `Notion.query_database` |
| `runes/linear/` | `linear.fav` / `rune.toml` | `Linear.create_issue` |
| `runes/airtable/` | `airtable.fav` / `rune.toml` | `Airtable.list_records` |
| `runes/sendgrid/` | `sendgrid.fav` / `rune.toml` | `SendGrid.send_email` |
| `runes/hubspot/` | `hubspot.fav` / `rune.toml` | `HubSpot.create_contact` |
| `runes/zendesk/` | `zendesk.fav` / `rune.toml` | `Zendesk.create_ticket` |
| `runes/shopify/` | `shopify.fav` / `rune.toml` | `Shopify.list_orders` |
| `runes/intercom/` | `intercom.fav` / `rune.toml` | `Intercom.create_conversation` |

---

## テスト詳細（T7）

```rust
// v299000_tests (v29.9.0) -- コミュニティ Rune コンテスト
#[cfg(test)]
mod v299000_tests {
    #[test]
    fn contributing_md_has_rune_guide() {
        let src = include_str!("../../CONTRIBUTING.md");
        assert!(
            src.contains("connect / read / write / error / test"),
            "CONTRIBUTING.md must contain 'connect / read / write / error / test'"
        );
    }
    #[test]
    fn community_rune_stripe_exists() {
        let src = include_str!("../../runes/stripe/stripe.fav");
        assert!(
            src.contains("Stripe.create_payment_intent"),
            "runes/stripe/stripe.fav must contain 'Stripe.create_payment_intent'"
        );
    }
    #[test]
    fn community_rune_twilio_exists() {
        let src = include_str!("../../runes/twilio/twilio.fav");
        assert!(
            src.contains("Twilio.send_sms"),
            "runes/twilio/twilio.fav must contain 'Twilio.send_sms'"
        );
    }
    #[test]
    fn community_rune_notion_exists() {
        let src = include_str!("../../runes/notion/notion.fav");
        assert!(
            src.contains("Notion.query_database"),
            "runes/notion/notion.fav must contain 'Notion.query_database'"
        );
    }
    #[test]
    fn community_page_has_contest() {
        let src = include_str!("../../site/app/community/page.tsx");
        assert!(
            src.contains("コンテスト"),
            "site/app/community/page.tsx must contain 'コンテスト'"
        );
    }
    #[test]
    fn changelog_has_v29_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.9.0]"),
            "CHANGELOG.md must contain '[v29.9.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.9.0"
- [x] `CONTRIBUTING.md` に `connect / read / write / error / test` を含む Rune 開発ガイドセクションが存在する
- [x] コミュニティ Rune スタブ 10 本が `runes/` 下に存在する
- [x] `site/app/community/page.tsx` にコンテスト告知セクションあり（`コンテスト` を含む）
- [x] `CHANGELOG.md` に `[v29.9.0]` セクションあり
- [x] `benchmarks/v29.9.0.json` 存在（test_count: 2366）
- [x] `cargo test --bin fav v299000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2366 tests PASS
- [x] tasks.md を COMPLETE に更新
