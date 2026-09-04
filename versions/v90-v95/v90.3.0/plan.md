# Plan: v90.3.0 — `MockSapClient` 実装

## 依存関係

```
Step 1（SapClient 確認）
    ↓
Step 2（mock.fav 作成）
    ↓
Step 3（driver.rs テスト追加）
    ↓
Step 4（cargo test）
    ↓
Step 5（CHANGELOG 更新）
    ↓
Step 6（CI 事前確認）
```

## Steps

### Step 1: 前提確認

- `runes/sap-odata/types.fav` に `interface SapClient` が存在することを確認する
- `runes/ctx/mock_db.fav` の実装パターン（`impl X for Y` 構文）を確認する
- 現テスト数が 4045 であることを確認する

### Step 2: `runes/sap-odata/mock.fav` を新規作成

- `//` コメントスタイルで記述する（`mock_db.fav` と同じ）
- `type MockSapClient = { ... }` レコード型で固定レスポンス 4 フィールドを定義する
- `impl SapClient for MockSapClient` ブロックで 5 メソッドを実装する
  - `business_partners` / `sales_orders` / `materials` / `journal_entries`: 対応 `_result` フィールドをそのまま返す
  - `business_partner_by_id`: `Result.err("not implemented")` を返す

### Step 3: `driver.rs` に `mod v90300_tests` を追加

- `mod v90200_tests` の直後に追加する
- `mock_sap_client_file_exists`: `runes/sap-odata/mock.fav` の存在確認
- `mock_sap_client_implements_sap_client`: `mock.fav` に `impl SapClient for MockSapClient` が含まれることを確認

### Step 4: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4047 tests, 0 failures を確認する

### Step 5: `CHANGELOG.md` に v90.3.0 エントリを追加

- `## [v90.2.0]` の前に v90.3.0 エントリを追加する
- `MockSapClient` / `runes/sap-odata/mock.fav` / `4047` が含まれることを確認する

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
