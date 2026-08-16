# v74.8.0 タスクリスト — 統合デモ（v70〜v74 の全機能を使ったショーケース）

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.7.0` であることを確認
- [x] `cargo test` が 3684 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v747000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v748000_tests` が未存在であることを確認
- [x] `infra/e2e-demo/favnir2-showcase/` が未存在であることを確認

---

## T1: ショーケースファイルを作成

- [x] `infra/e2e-demo/favnir2-showcase/` ディレクトリを作成した
- [x] `pipeline.fav` を作成した（`ShowcaseContract` / `contract` / `import rune` / `AppCtx` / `bind` を含む）
- [x] `fav.toml` を作成した（`favnir2-showcase` / `schedule` / `tenant` を含む）
- [x] `rune.toml` を作成した（`privacy` / `linalg` を含む）
- [x] `contract.fav` を作成した（`contract` / `ShowcaseInputContract` を含む）
- [x] `quality.fav` を作成した（`quality_score` を含む）
- [x] `README.md` を作成した（`Favnir 2.0 Showcase` / `pipeline.fav` を含む）

---

## T2: `v748000_tests` モジュールを追加

- [x] `v747000_tests` の直後に `v748000_tests` モジュールを追加した
- [x] `showcase_demo_structure_complete` テストを実装した
  - `fav.toml` に `"favnir2-showcase"` / `"schedule"` / `"tenant"` が含まれることを assert
  - `README.md` に `"Favnir 2.0 Showcase"` / `"pipeline.fav"` が含まれることを assert
- [x] `showcase_pipeline_fav_valid` テストを実装した
  - `pipeline.fav` に `"ShowcaseContract"` / `"contract"` / `"import rune"` / `"AppCtx"` / `"bind"` が含まれることを assert
- [x] `include_str!` パスが `../../infra/e2e-demo/favnir2-showcase/...`（driver.rs からの相対パス）であることを確認
- [x] `cargo build` でエラーがないことを確認

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.7.0"` → `version = "74.8.0"` に変更した
- [x] `driver.rs` 内の `version = "74.7.0"` 参照を `version = "74.8.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.7.0` を `version should be 74.8.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.7.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.8.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v748000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3686 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.8.0]` エントリを先頭に追加した
  - Added: `infra/e2e-demo/favnir2-showcase/` 以下 6 ファイル
  - Tests: 2 件、合計テスト数 3686（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.8.0)` に更新した
- [x] 「進行中バージョン」を `v74.8.0` に更新した
- [x] 「次に切る版」を `v74.9.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v748000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3686 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.8.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.8.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.8.0` であることを確認

---

## スコープ外（明示的除外）

- `fav run pipeline.fav` の実際の実行（後続バージョンで対応）
- CI でのショーケース自動実行（v74.9.0 安定化スプリントで対応）
- `infra/` Terraform / AWS 設定（本バージョン対象外）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）
