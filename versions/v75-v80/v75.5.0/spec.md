# v75.5.0 仕様書 — `RetentionPolicy` 型

Date: 2026-08-15
Status: 計画中

---

## Background

GDPR・CCPA などのデータ保護規制では、個人データの「保存期間」を明示し、期限を過ぎたデータを適切に処理（削除・アーカイブ・匿名化）する義務がある。

v75.x スプリント（Temporal Data Native）では鮮度・SCD・時点結合を型で実装してきた。v75.5.0 では「データをいつまで保持できるか」というポリシーを Rust 型として追加し、保持判定ロジックをテスト可能な形で提供する。

---

## Goals

1. `RetentionAction` enum（Delete, Archive, Anonymize）を追加する
2. `RetentionResult` enum（Keep, Delete, Archive, Anonymize）を追加する
3. `RetentionPolicy` 構造体（max_age_days: u64, action: RetentionAction）を追加する
4. `apply_retention_check(row_ts: i64, now: i64, policy: &RetentionPolicy) -> RetentionResult` を追加する
5. Rust テスト 2 件を追加し 3702 tests に到達する

---

## 型・関数仕様

### `RetentionAction` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionAction {
    Delete,
    Archive,
    Anonymize,
}
```

保持期限を超えたデータに適用するアクション。

---

### `RetentionResult` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionResult {
    Keep,
    Delete,
    Archive,
    Anonymize,
}
```

`apply_retention_check` の判定結果。`Keep` は保持期限内、その他は期限超過に対応するアクション。

---

### `RetentionPolicy` 構造体

```rust
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub max_age_days: u64,
    pub action:       RetentionAction,
}
```

- `max_age_days`: データを保持する最大日数（0 の場合は全行が即時対象）
- `action`: 保持期限超過時に適用するアクション

---

### `apply_retention_check`

```rust
pub fn apply_retention_check(
    row_ts: i64,
    now:    i64,
    policy: &RetentionPolicy,
) -> RetentionResult
```

**判定ロジック:**
- `now - row_ts > policy.max_age_days as i64 * 86400` のとき期限超過
  → `policy.action` に対応する `RetentionResult` を返す（Delete → Delete, Archive → Archive, Anonymize → Anonymize）
- それ以外 → `RetentionResult::Keep`
- `now < row_ts`（未来のタイムスタンプ）も Keep として扱う（差分が負になるため条件不成立）

**境界値:**
- `now - row_ts == max_age_days * 86400`（ちょうど期限当日）→ Keep（`>` による exclusive 判定）
- これは v75.4.0 の `valid_to` と同様の開区間ポリシー

---

## Favnir コード例

```favnir
// 保持ポリシー付きパイプライン定義（将来の Favnir 言語統合を想定）
bind policy <- RetentionPolicy {
    max_age_days: 365,
    action: Anonymize
}
// → 365日超過レコードを匿名化対象として判定
```

---

## Success Criteria

- `RetentionAction` enum が定義されている（Delete / Archive / Anonymize）
- `RetentionResult` enum が定義されている（Keep / Delete / Archive / Anonymize）
- `RetentionPolicy` 構造体が定義されている（max_age_days: u64, action: RetentionAction）
- `apply_retention_check` が正しい `RetentionResult` を返す
- `cargo test` が 3702 tests all pass
- `CHANGELOG.md` の先頭に v75.5.0 エントリが存在する

---

## テスト仕様

### `retention_delete_old_rows`

- `policy = RetentionPolicy { max_age_days: 365, action: RetentionAction::Delete }`
- `row_ts = 0`, `now = 366 * 86400`（366 日後）→ `RetentionResult::Delete`
- `row_ts = 0`, `now = 365 * 86400`（ちょうど 365 日）→ `RetentionResult::Keep`（境界値 exclusive）
- `row_ts = 0`, `now = 100 * 86400`（100 日後）→ `RetentionResult::Keep`

### `retention_anonymize_action`

- `policy = RetentionPolicy { max_age_days: 90, action: RetentionAction::Anonymize }`
- `row_ts = 1_000_000`, `now = 1_000_000 + 91 * 86400`（91 日後）→ `RetentionResult::Anonymize`
- `row_ts = 1_000_000`, `now = 1_000_000 + 90 * 86400`（ちょうど 90 日）→ `RetentionResult::Keep`
- `row_ts = 1_000_000`, `now = 999_999`（過去の now）→ `RetentionResult::Keep`（未来行は Keep）

---

## 変更ファイル

- `fav/src/driver.rs` — `RetentionAction`, `RetentionResult`, `RetentionPolicy`, `apply_retention_check`, `v755000_tests` を追加
- `CHANGELOG.md` — v75.5.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.4.0` → `75.5.0` に更新

---

## 対象外

- Favnir 言語レベルでの `RetentionPolicy` キーワード（将来バージョン予定）
- `exclude_fields` フィールド（ロードマップにある将来拡張、本バージョンは未実装）
- Archive 先のストレージ指定（呼び出し側の責任）
