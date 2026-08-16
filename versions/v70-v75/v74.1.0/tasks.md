# v74.1.0 タスクリスト — Rune マーケットプレイス（バージョン管理・依存解決）

Date: 2026-08-13
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.0.0` であることを確認
- [x] `cargo test` が 3669 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v74000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v741000_tests` が未存在であることを確認

---

## T1: `RunePackage` 構造体 + 関数を `driver.rs` に追加

- [x] `// --- v74.1.0: Rune マーケットプレイス ---` セクションコメントを追加した
- [x] `#[derive(Debug, Clone)] pub struct RunePackage` を追加した（name / version / description / author）
- [x] `pub fn format_rune_publish_manifest(pkg: &RunePackage) -> String` を実装した
  - 既存の `json_escape` ヘルパーを使用して JSON インジェクションを防止
  - 出力: `{"name":"...","version":"...","description":"...","author":"..."}` 形式
- [x] `pub fn parse_rune_dep_entry(entry: &str) -> Result<(String, String), String>` を実装した
  - `rfind('@')` で最後の `@` を区切り文字として使用
  - `@` なし / 先頭 `@` / 末尾 `@` は全て `Err`
- [x] `cargo build` でエラーがないことを確認

---

## T2: `v741000_tests` モジュールを追加

- [x] `v74000_tests` の直後に `v741000_tests` モジュールを追加した
- [x] `use super::{RunePackage, format_rune_publish_manifest, parse_rune_dep_entry}` を追加した
- [x] `rune_marketplace_publish_format` テストを実装した
  - `RunePackage` を構築して `format_rune_publish_manifest` を呼び出す
  - name / version / author が含まれることを assert
  - `{` で始まり `}` で終わることを assert
- [x] `rune_marketplace_add_updates_toml` テストを実装した
  - `"mycompany/crm@^1.0"` → `("mycompany/crm", "^1.0")` を assert
  - `"mycompany/crm"`（バージョンなし）が `Err` を返すことを assert
  - `"@^1.0"`（名前なし）が `Err` を返すことを assert
  - `"mycompany/crm@"`（バージョン空）が `Err` を返すことを assert
- [x] `cargo test v741000` で 2 件 pass することを確認（Step 3 後に実施）

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.0.0"` → `version = "74.1.0"` に変更した
- [x] `driver.rs` 内の `version = "74.0.0"` 参照を `version = "74.1.0"` に replace_all した
- [x] `version should be 74.0.0` を `version should be 74.1.0` に replace_all した（アサートメッセージのみ対象。アサート値 `contains("version = \"74.0.0\"")` は Cargo.toml 更新と同時に `74.1.0` に変わるため個別変更不要）
- [x] 残存 `74.0.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.1.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v741000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3671 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.1.0]` エントリを先頭に追加した
  - Added: `RunePackage` / `format_rune_publish_manifest` / `parse_rune_dep_entry`
  - Tests: 2 件、合計テスト数 3671（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v74.1.0)` に更新した
- [x] 「進行中バージョン」を `v74.1.0` に更新した
- [x] 「次に切る版」を `v74.2.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v741000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3671 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.1.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.1.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.1.0` であることを確認

---

## スコープ外（明示的除外）

- 実際の Rune レジストリサーバーへの通信
- `fav publish rune` / `fav add rune` CLI コマンドの実装
- セマンティックバージョニング互換性チェックの完全実装
