# v73.3.0 タスクリスト — PII 検出・マスキング Rune

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.2.0` であることを確認
- [x] `cargo test` が 3650 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v732000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v733000_tests` が未存在であることを確認
- [x] `driver.rs` 内の `"73.2.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: `PiiMaskStrategy` 列挙型追加

- [x] `PiiMaskStrategy { Hash, Redact, Truncate }` を `driver.rs` に追加した
- [x] `pub enum` であることを確認
- [x] `cargo build` でエラーがないことを確認

---

## T2: `mask_pii_fields` 追加

- [x] `pub fn mask_pii_fields(fields: &[(String, String)], strategy: PiiMaskStrategy) -> Vec<(String, String)>` を実装した
  - Hash → 値を `"***"` に置換
  - Redact → 値を `"[REDACTED]"` に置換
  - Truncate → 値を `&value[..2.min(value.len())]` + `"..."` に短縮
- [x] `cargo build` でエラーがないことを確認

---

## T3: `scan_pii_patterns` 追加

- [x] `pub fn scan_pii_patterns(text: &str) -> Vec<String>` を実装した
  - `@` を含む単語 → `"email:<value>"` として検出
  - `-` 含む 7 桁以上の数字列 → `"phone:<value>"` として検出
  - 正規表現ライブラリ不使用（文字列マッチングのみ）
- [x] `cargo build` でエラーがないことを確認

---

## T4: `gdpr_erase_record` 追加

- [x] `pub fn gdpr_erase_record(fields_to_erase: &[&str]) -> Result<usize, String>` を実装した
  - 空配列 → `Err("no fields specified for erasure")`
  - それ以外 → `Ok(fields_to_erase.len())`（スタブ）
- [x] `cargo build` でエラーがないことを確認

---

## T5: `runes/privacy/` スタブ Rune ファイル作成

- [x] `runes/privacy/rune.toml` を作成した（name = "privacy" を含む）
- [x] `runes/privacy/privacy.fav` を作成した（mask / scan / gdpr_erase スタブ）
- [x] ファイルの存在を確認した

---

## T6: `v733000_tests` モジュール追加

- [x] `v732000_tests` モジュールの直後に `v733000_tests` モジュールを追加した
- [x] `use super::{mask_pii_fields, scan_pii_patterns, gdpr_erase_record, PiiMaskStrategy}` を追加した（前バージョンと同様の個別 import 形式）
- [x] `privacy_rune_mask_pii_fields` テストを実装した
  - Hash マスク → 全値が `"***"` になることを assert
  - Redact マスク → 全値が `"[REDACTED]"` になることを assert
  - `scan_pii_patterns` でメール検出 → "email" が含まれることを assert
- [x] `privacy_rune_gdpr_erase` テストを実装した
  - 3 フィールド削除 → `Ok(3)` を assert
  - 空フィールド → `Err` で "no fields" が含まれることを assert
  - `runes/privacy/rune.toml` が "privacy" を含むことを assert
- [x] `cargo test v733000` で 2 件 pass することを確認

---

## T7: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.2.0"` → `version = "73.3.0"` に変更した
- [x] `driver.rs` 内の `version = \"73.2.0\"` を `version = \"73.3.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 73.2.0"` を `"73.3.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 73.2.0"` を `"73.3.0"` に replace_all した
- [x] 残存 `73.2.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `grep "73.2.0" driver.rs` で意図的保持分以外がゼロ件であることを確認
- [x] `cargo build` 後に `fav/Cargo.lock` が `version = "73.3.0"` を含むことを確認

---

## T8: 部分テスト確認

- [x] `cargo test v733000` で 2 件 pass することを確認

---

## T9: 全体テスト確認

- [x] `cargo test` 全体で 3652 tests pass（0 failures）であることを確認

---

## T10: `CHANGELOG.md` 更新

- [x] `## [v73.3.0]` エントリを先頭に追加した
  - Added: `PiiMaskStrategy` / `mask_pii_fields` / `scan_pii_patterns` / `gdpr_erase_record` / `runes/privacy/`
  - Tests: 2 件、合計テスト数 3652（+2）

---

## T11: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.3.0)` に更新した
- [x] 「進行中バージョン」を `v73.3.0` に更新した
- [x] 「次に切る版」を `v73.4.0` に更新した

---

## T12: 最終確認（T10・T11 完了後）

- [x] `cargo test v733000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3652 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.3.0` であることを確認
- [x] `PiiMaskStrategy` / `mask_pii_fields` / `scan_pii_patterns` / `gdpr_erase_record` が pub で存在することを確認
- [x] `runes/privacy/privacy.fav` と `runes/privacy/rune.toml` が存在することを確認
- [x] `CHANGELOG.md` に `[v73.3.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.3.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [BUG] | Truncate で `&value[..2]` がマルチバイト境界でパニック | `value.chars().take(2).collect::<String>()` に変更 |
| [STYLE] | Truncate バリアントのテストなし（空文字・1文字・マルチバイト） | `v733000_truncate_tests::truncate_boundary_values` を追加（+1 test → 合計 3653） |
| [STYLE] | Hash stub に TODO コメントなし | `// TODO: 将来 sha256 ハッシュ化を予定` を実装行に追加 |
| [STYLE] | `rune.toml` アサーションが弱い（"privacy" 含有のみ） | `name = "privacy"` の含有チェックに強化 |

---

## スコープ外（明示的除外）

- `Rune.privacy` の VM primitive 接続（v73.6.0 以降）
- 正規表現ライブラリ（`regex` crate）の導入
- `main.rs` への `privacy` コマンド登録（将来バージョン）
- WASM / サイト MDX 更新（v74.x 以降）
