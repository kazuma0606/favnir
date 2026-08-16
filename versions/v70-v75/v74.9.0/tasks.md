# v74.9.0 タスクリスト — 安定化・コードフリーズ（Favnir 2.0 前最終調整）

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.8.0` であることを確認
- [x] `cargo test` が 3686 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v748000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v749000_tests` が未存在であることを確認
- [x] `CHANGELOG.md` に `[v74.1.0]`〜`[v74.8.0]` の全エントリが存在することを確認

---

## T1: `v749000_tests` モジュールを追加

- [x] `v748000_tests` の直後に `// --- v74.9.0: 安定化・コードフリーズ ---` セクションコメントを追加した
- [x] `v749000_tests` モジュールを追加した（`use super::*` 不要）
- [x] `favnir2_full_sprint_all_stable` テストを実装した
  - `CHANGELOG.md` に `[v74.1.0]`〜`[v74.8.0]` の全 8 バージョンが含まれることを assert
- [x] `favnir2_e2e_showcase_runs` テストを実装した
  - `pipeline.fav` に `Result.ok` / `import rune` / `ShowcaseContract` が含まれることを assert
  - `fav.toml` に `schedule` / `tenant` が含まれることを assert
  - `contract.fav` に `ShowcaseInputContract` が含まれることを assert
- [x] `cargo build` でエラーがないことを確認

---

## T2: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.8.0"` → `version = "74.9.0"` に変更した
- [x] `driver.rs` 内の `version = "74.8.0"` 参照を `version = "74.9.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.8.0` を `version should be 74.9.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.8.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.9.0"` を含むことを確認

---

## T2.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v749000` で 2 件 pass することを確認

---

## T3: 全体テスト確認

- [x] `cargo test` 全体で 3688 tests pass（0 failures）であることを確認

---

## T4: `CHANGELOG.md` 更新

- [x] `## [v74.9.0]` エントリを先頭に追加した
  - Tests: 2 件、合計テスト数 3688（+2）

---

## T5: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.9.0)` に更新した
- [x] 「進行中バージョン」を `v74.9.0` に更新した
- [x] 「次に切る版」を `v75.0.0` に更新した

---

## T6: 最終確認（T4・T5 完了後）

- [x] `cargo test v749000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3688 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.9.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.9.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.9.0` であることを確認

---

## スコープ外（明示的除外）

- 新規機能・新規構造体・新規関数の追加（安定化スプリントのため不要）
- `fav run pipeline.fav` の実際の実行（後続フェーズで対応）
- CI 自動実行パイプラインの構築（後続フェーズで対応）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョン v75.0.0 で実施）
