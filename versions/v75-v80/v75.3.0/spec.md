# v75.3.0 仕様書 — SCD Type 1 / Type 2 ネイティブ型

Date: 2026-08-14
Status: 計画中

---

## Background

データウェアハウスでは「緩やかに変化するディメンション（Slowly Changing Dimensions, SCD）」が定番パターンである。

- **SCD Type 1**: 旧レコードを上書き（履歴なし）
- **SCD Type 2**: 旧レコードを有効期限付きで保持し、新レコードを追加（履歴あり）

v75.2.0 で `AsOfQuery`（時点クエリ）を実装した。v75.3.0 では SCD の行管理ロジック自体を Favnir のファーストクラス型として実装し、データウェアハウスのディメンション管理を型安全に行えるようにする。

---

## Goals

1. `ScdType` enum（Type1 / Type2）を追加する
2. `ScdRow` 構造体（SCD Type 2 の一行を表す）を追加する
3. `apply_scd2_update` 関数（SCD Type 2 のアップサート処理）を追加する
4. Rust テスト 2 件を追加し 3698 tests に到達する

---

## 型・関数仕様

### `ScdType` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ScdType {
    Type1,
    Type2,
}
```

SCD の種別を表す enum。現バージョンでは Type2 のロジックのみ実装する。

---

### `ScdRow` 構造体

```rust
#[derive(Debug, Clone)]
pub struct ScdRow {
    pub valid_from: i64,          // 有効開始タイムスタンプ（UNIX 秒）
    pub valid_to: Option<i64>,    // 有効終了タイムスタンプ（None = 現在有効）
    pub is_current: bool,         // 現在有効なレコードか
    pub data: String,             // JSON シリアライズ済みレコードデータ
}
```

`data` フィールドは Favnir の型システムに依存せず汎用的に使えるよう `String` とする。

---

### `apply_scd2_update` 関数

```rust
pub fn apply_scd2_update(existing: &[ScdRow], new_data: &str, new_ts: i64) -> Vec<ScdRow>
```

**動作:**
1. `existing` から `is_current == true` かつ `data != new_data` のレコードを探す
2. 該当レコードの `is_current` を `false`、`valid_to` を `new_ts - 1` に設定（閉じる）
3. 新レコード（`valid_from = new_ts, valid_to = None, is_current = true, data = new_data`）を末尾に追加
4. `data` が変化していない場合（no-op）は新レコードを追加しない
5. `existing` が空の場合は無条件で新レコードを追加する（初回 upsert）

---

## Favnir コード例

```favnir
// SCD Type 2: 変更履歴を保持
fn upsert_customer(existing: List<ScdRow>, new_data: String, now: Int) -> List<ScdRow> {
    apply_scd2_update(existing, new_data, now)
    // → 旧レコードを valid_to で閉じ、新レコードを is_current=true で追加
}
```

---

## Success Criteria

- `ScdType` enum が定義されている（Type1 / Type2）
- `ScdRow` 構造体が定義されている（4フィールド）
- `apply_scd2_update` が正しく動作する（下記 2 テスト）
- `cargo test` が 3698 tests all pass

---

## テスト仕様

### `scd2_creates_history_row`

- `existing` に `is_current=true, data="旧"` のレコード 1 件
- `new_data = "新"` で `apply_scd2_update` を呼ぶ
- 結果が 2 件（旧レコード + 新レコード）
- 新レコードの `is_current = true`, `data = "新"`, `valid_from = new_ts`, `valid_to = None`

### `scd2_marks_previous_expired`

- 旧レコードの `is_current = false`, `valid_to = Some(new_ts - 1)` を確認
- no-op ケース（data が同一）では新レコードが追加されないことを確認

---

## 変更ファイル

- `fav/src/driver.rs` — `ScdType`, `ScdRow`, `apply_scd2_update`, `v753000_tests` を追加
- `CHANGELOG.md` — v75.3.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.2.0` → `75.3.0` に更新

---

## 対象外

- SCD Type 1（上書き）のロジック実装（型定義のみ）
- SCD Type 3（前回値保持）
- Favnir 言語レベルでの `ScdRow` 型統合（VM primitive 接続は将来バージョンで対応予定）
