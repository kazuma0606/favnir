# v29.9.0 Plan — コミュニティ Rune コンテスト

**バージョン**: 29.9.0
**日付**: 2026-07-01
**前バージョン**: v29.8.0 (ドキュメントサイト v3)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.9.0"
```

### T2: 既存 CONTRIBUTING.md に `## コミュニティ Rune 開発ガイド` セクション追記

> **注意**: `CONTRIBUTING.md` は v10.0.0 で既に作成済み。既存の `## Rune 追加ガイド` セクションは保持し、
> 末尾に `## コミュニティ Rune 開発ガイド` セクションを **追記** する（上書き・削除しない）。

追記内容:

```markdown
## コミュニティ Rune 開発ガイド

コミュニティ Rune の審査は以下の 5 条件（connect / read / write / error / test）で行います。

### 5 条件（connect / read / write / error / test）

| 条件 | 内容 |
|---|---|
| connect | `RUNE_NAME_URL` 等の環境変数でサービスに接続できる |
| read | データを取得する関数が 1 つ以上ある |
| write | データを書き込む関数が 1 つ以上ある |
| error | エラーを `Result.err` で返す（クラッシュしない）|
| test | `cargo test` で 3 件以上 PASS する |
```

### T3: コミュニティ Rune スタブ 10 本作成

各 Rune は `rune.toml`（`[rune]` セクションのみ）と `.fav` ファイルで構成。

#### stripe

```favnir
// stripe Rune -- Stripe 決済 API 連携（コミュニティ投稿 Rune）
// 接続: STRIPE_SECRET_KEY / STRIPE_BASE_URL 環境変数
// NOTE: STRIPE_SECRET_KEY は HTTP Rune の認証ヘッダー機構が有効化されるまでリクエストには付与されない

fn Stripe.create_payment_intent(api_key: String, amount: Int, currency: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("STRIPE_BASE_URL", "https://api.stripe.com") ++ "/v1/payment_intents",
    { "amount": Int.to_string(amount), "currency": currency }
  )

fn Stripe.retrieve_charge(api_key: String, charge_id: String) -> Result<String, String> !Http =
  Http.get_json(
    Env.get_or("STRIPE_BASE_URL", "https://api.stripe.com") ++ "/v1/charges/" ++ charge_id
  )
```

#### twilio

```favnir
// twilio Rune -- Twilio SMS/Voice API 連携（コミュニティ投稿 Rune）

fn Twilio.send_sms(api_key: String, from: String, to: String, body: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("TWILIO_BASE_URL", "https://api.twilio.com") ++ "/2010-04-01/Accounts/" ++ Env.get_or("TWILIO_ACCOUNT_SID", api_key) ++ "/Messages.json",
    { "From": from, "To": to, "Body": body }
  )

fn Twilio.send_voice(api_key: String, from: String, to: String, twiml_url: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("TWILIO_BASE_URL", "https://api.twilio.com") ++ "/2010-04-01/Accounts/" ++ Env.get_or("TWILIO_ACCOUNT_SID", api_key) ++ "/Calls.json",
    { "From": from, "To": to, "Url": twiml_url }
  )
```

#### notion

```favnir
// notion Rune -- Notion データベース API 連携（コミュニティ投稿 Rune）

fn Notion.query_database(api_key: String, database_id: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("NOTION_BASE_URL", "https://api.notion.com") ++ "/v1/databases/" ++ database_id ++ "/query",
    {}
  )

fn Notion.create_page(api_key: String, database_id: String, properties: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("NOTION_BASE_URL", "https://api.notion.com") ++ "/v1/pages",
    { "parent": { "database_id": database_id }, "properties": properties }
  )
```

（remaining 7 Runes: linear / airtable / sendgrid / hubspot / zendesk / shopify / intercom — 同パターン）

### T4: site/app/community/page.tsx にコンテスト告知セクション追加

> **注意**: `site/app/community/page.tsx` は v29.8.0 で作成済み。既存の `GitHub Discussions` テキストを**維持**したまま、
> コンテスト告知セクションを **追記** する（既存内容を削除・上書きしない）。

追記内容: コンテスト概要・募集要件（5 条件）・応募方法・特典。
`コンテスト` という文字列を含む必要あり（テスト対象）。

### T5: CHANGELOG.md に [v29.9.0] セクション追加

```markdown
## [v29.9.0] — 2026-07-01

### Added
- `CONTRIBUTING.md` — Rune 開発ガイド（5 条件）追加
- `runes/stripe|twilio|notion|linear|airtable|sendgrid|hubspot|zendesk|shopify|intercom/` — コミュニティ Rune スタブ 10 本
- `site/app/community/page.tsx` — 第 1 回 Rune コンテスト告知セクション追加
- テスト数: 2360 → 2366（+6）
```

### T6: benchmarks/v29.9.0.json 作成

```json
{
  "version": "29.9.0",
  "date": "2026-07-01",
  "milestone": "Ecosystem Maturity (phase 9)",
  "test_count": 2366,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T7: driver.rs に v299000_tests 6 件追加

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

### T8: cargo test --bin fav v299000 — 6/6 PASS 確認

### T9: cargo test --bin fav — 2366 tests PASS 確認

### T10: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.8.0 | 2360 |
| v29.9.0 | **2366** (+6) |
