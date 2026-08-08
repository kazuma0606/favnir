# Tasks — v57.4.0 — 依存関係セキュリティスキャン（`fav audit --security`）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.4.0 セクションを確認
- [x] ベーステスト数 3259（v57.3.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `57.3.0` であることを確認（更新前）
- [x] `v57400_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v57300_tests` が `driver.rs` に存在することを確認（`v57400_tests` の挿入位置として使用）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"57.3.0"` を期待していることを確認（更新対象）
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` が `"57.3.0"` を期待していることを確認（更新対象）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` が `"57.3.0"` を期待していることを確認（更新対象・rolling）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `57.4.0` に更新
- [x] T2: `fav/src/driver.rs` — `v57400_tests` モジュールを `v57300_tests` の直前に追加
  - [x] `CveEntry` 構造体定義（rune / version / cve_id / severity / fix_version）
  - [x] `make_cve_db()` ヘルパー関数（kafka@2.1.0 HIGH / redis@1.0.0 MEDIUM の 2 エントリ）
  - [x] `scan_security()` 関数（runes と db を受け取り、一致する `&CveEntry` を返す）
  - [x] `fail_on_high()` 関数（findings に HIGH があれば true）
  - [x] `security_scan_detects_cve` テスト: kafka/redis 検出・postgres スキップを検証
  - [x] `security_scan_fail_on_high` テスト: HIGH あり → true / MEDIUM のみ → false を検証
- [x] T3: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.3.0"` → `"57.4.0"` に更新
  - [x] failure メッセージも `"should be 57.4.0"` に更新
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.3.0"` → `"57.4.0"` に更新
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.3.0"` → `"57.4.0"` に更新
  - [x] モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x] T4: `cargo build` でコンパイルエラーがないことを確認
- [x] T5: `cargo test` 全通過（**3261 tests passed, 0 failed**）
  - [x] `v57400_tests::security_scan_detects_cve` ok
  - [x] `v57400_tests::security_scan_fail_on_high` ok
  - [x] 既存 3259 件全通過
- [x] T6: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T7: `CHANGELOG.md` に v57.4.0 エントリを追加
- [x] T8: `versions/current.md` を v57.4.0 / 3261 tests に更新
- [x] T9: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.4.0 実績を COMPLETE に更新
  - [x] `3259 + 2 = 3261 tests passed, 0 failed（2026-07-28）` を追記
- [x] T10: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.4.0 実績欄も COMPLETE に更新
  - [x] テスト数推移テーブルに v57.4.0 行（3261）を追加

---

## 完了確認

- [x] `security_scan_detects_cve` pass
- [x] `security_scan_fail_on_high` pass
- [x] **3261 tests passed, 0 failed**（ベース 3259 + 2）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に `[v57.4.0]` エントリが追加されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.4.0"` になっている
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.4.0"` になっている（rolling）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.4.0"` になっている（rolling）
- [x] `versions/current.md` が v57.4.0 / 3261 tests を反映
- [x] T9 / T10 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `CveEntry` は `v57400_tests` モジュール内にのみ定義（`toml.rs` や公開 API への追加は不要）
- `scan_security` は `db.iter().filter(...)` パターン — `Vec<&'a CveEntry>` を返すことでクローン不要
- `fail_on_high` は純粋関数（`&[&CveEntry]` を受け取り bool を返す）
- `v57300_tests` / `v57200_tests` / `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
- awk での多行ブロック挿入は過去に失敗実績あり → Python `uv run python` + `str.replace()` を使用
