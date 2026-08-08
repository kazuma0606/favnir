# v59.8.0 Tasks — ドキュメントサイト Enterprise 1.0 総括記事

Date: 2026-07-30
Status: COMPLETE（2026-07-30）— 3324 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3322 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.7.0"` であることを確認
- [x] `fav/src/driver.rs` に `v59800_tests` がまだ存在しないことを確認
- [x] `site/content/docs/enterprise/index.mdx` がまだ存在しないことを確認
- [x] `site/content/cookbook/enterprise-checklist.mdx` がまだ存在しないことを確認
- [x] `grep -o '"version = \\"59\.7\.0\\""' fav/src/driver.rs | wc -l` でローリング文字列件数を確認（7 件の assertion）
- [x] または `grep -c 'Cargo.toml version should be 59\.7\.0' fav/src/driver.rs` が 7 件であることを確認（failure メッセージ）
  - **注意**: `v59700_tests` コメント行にも `59.7.0` が含まれるため `grep -o '59\.7\.0'` は 15 件になるが、それは正常（コメント行は置換対象外）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.7.0"` → `"59.8.0"`

---

## T2: docs/enterprise/index.mdx 作成

- [x] `site/content/docs/enterprise/index.mdx` を新規作成
  - フロントマターなし・H1 見出しで開始（既存 enterprise docs と同形式）
  - `"Enterprise 1.0"` を含む（`docs_enterprise_index_exists` テストの要件）
  - Enterprise 1.0 機能一覧テーブル・認定要件・移行ガイドを記載
  - `fav certify --level enterprise` への言及を記載

---

## T3: cookbook/enterprise-checklist.mdx 作成

- [x] `site/content/cookbook/enterprise-checklist.mdx` を新規作成
  - フロントマター（`title` / `description`）付き（既存 cookbook ファイルと同形式）
  - `"Enterprise"` を含む（`cookbook_enterprise_checklist_exists` テストの要件）
  - fav.toml チェックリスト・CI チェックリスト・移行チェックリストを記載

---

## T4: driver.rs — v59800_tests 追加

- [x] **注意**: T2〜T3（MDX 作成）を先に行うこと（`include_str!` はコンパイル時に読み込む）
- [x] `v59800_tests` モジュールを `v59700_tests` の直前（既存セパレータ行の前）に挿入
  - [x] `docs_enterprise_index_exists` テスト: `include_str!("../../site/content/docs/enterprise/index.mdx").contains("Enterprise 1.0")` を検証
  - [x] `cookbook_enterprise_checklist_exists` テスト: `include_str!("../../site/content/cookbook/enterprise-checklist.mdx").contains("Enterprise")` を検証
  - [x] `use super::*;` は不要（`include_str!` のみ使用）

---

## T5: driver.rs — ローリングチェック更新

- [x] `version = \"59.7.0\"` → `\"59.8.0\"` に一括更新（7 件）
- [x] failure メッセージ 7 件を `"59.8.0"` に更新（全 7 件とも同一パターン）:
  - `"Cargo.toml version should be 59.7.0"` → `"Cargo.toml version should be 59.8.0"`
  - 対象: `v59000_tests` / `v58900_tests` / `v58000_tests` / `v57900_tests` / `v57000_tests` / `v56900_tests` / `v56300_tests`
  - **注意**: `// -- v59700_tests (v59.7.0) --` コメント行の `59.7.0` は置換しないこと（ヒストリコメントは保持）
  - **注意**: `rolling check from` サフィックスは driver.rs に存在しない（特殊書式なし）
  - **注意**: `v59100_tests`〜`v59700_tests` は rolling check なし（対象外）

---

## T6: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v59800_tests::docs_enterprise_index_exists` pass を確認
- [x] `v59800_tests::cookbook_enterprise_checklist_exists` pass を確認
- [x] 総テスト数 **3324** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T7: 事後処理

- [x] `CHANGELOG.md` に v59.8.0 エントリを追加
- [x] `versions/current.md` を v59.8.0 / 3324 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.8.0 実績欄を更新
- [x] `roadmap-v59.1-v60.0.md` の v59.9.0 ベース数を「着手時に更新」→ `3324` に確定（T6 でテスト数確認後に実施）
- [x] **参考**: roadmap の v60.0.0 セクションのベース数（現在 `3316` と古い値）は v59.9.0 完了後に更新予定
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

Status: COMPLETE（2026-07-30）— 3324 tests passed, 0 failed
