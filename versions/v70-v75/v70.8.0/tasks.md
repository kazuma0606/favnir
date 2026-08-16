# v70.8.0 タスクリスト — `fav doctor` 強化

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.7.0` であることを確認
- [x] driver.rs の `cargo_toml_version_is_70_7_0` テストが存在することを確認
- [x] `cargo test` が全 pass（3576 tests）であることを確認
- [x] driver.rs に `cmd_doctor_run` が存在することを確認（line ~48966）
- [x] `DoctorCheck` 構造体と `DoctorStatus` enum が driver.rs に定義されていることを確認
- [x] driver.rs の `DoctorStatus` enum の variant が `Ok / Warn / Fail`（`Error` は存在しない）であることを確認（line ~48855）
- [x] driver.rs に `doctor_check_paper_rune` / `doctor_check_changelog_entry` が未存在であることを確認

---

## T1: driver.rs に `doctor_check_paper_rune` を追加

- [x] `cmd_doctor_run` の直後に `doctor_check_paper_rune(rune_dir: &str) -> DoctorCheck` を追加する
  - `rune.toml` が存在しない場合 → `DoctorStatus::Ok`
  - `rune.toml` が存在し `<name>.fav` が空または非存在 → `DoctorStatus::Fail`
  - `rune.toml` が存在し `<name>.fav` が非空 → `DoctorStatus::Ok`
- [x] `cargo test` で既存テスト（3576 件）が全 pass することを確認

---

## T2: driver.rs に `doctor_check_changelog_entry` を追加

- [x] `doctor_check_paper_rune` の直後に `doctor_check_changelog_entry(changelog_content: &str, version: &str) -> DoctorCheck` を追加する
  - `changelog_content.contains("[{version}]")` が true → `DoctorStatus::Ok`
  - false → `DoctorStatus::Fail`
- [x] `cargo test` で既存テスト（3576 件）が全 pass することを確認

---

## T3: `v708000_tests` モジュールを driver.rs 末尾に追加

- [x] `v707000_tests` の直後（driver.rs 末尾）に `v708000_tests` モジュールを追加する
- [x] `doctor_detects_paper_rune` テストを実装する:
  - tempdir に `rune.toml` + 空の `test.fav` を作成
  - `doctor_check_paper_rune` → `DoctorStatus::Fail` を assert
  - tempdir を削除
- [x] `doctor_detects_missing_changelog_entry` テストを実装する:
  - `[v70.6.0]` のみ含む changelog 文字列を用意
  - `doctor_check_changelog_entry(changelog, "v70.8.0")` → `DoctorStatus::Fail` を assert
  - `doctor_check_changelog_entry(changelog, "v70.6.0")` → `DoctorStatus::Ok` を assert
- [x] `cargo test v708000` で 2 件 pass することを確認

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.7.0"` → `"70.8.0"` に変更する
- [x] driver.rs 内の `"70.7.0"` 文字列を sed で `"70.8.0"` に一括更新
  - 対象: `cargo_toml_version_is_70_7_0` テスト関数内の `"70.7.0"` 文字列
  - 注: テスト関数名 `cargo_toml_version_is_70_7_0` 自体はリネームしない（識別子として残す）

---

## T5: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.7.0 エントリの直前）に v70.8.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `v708000_tests` 2 件（3576 → 3578 tests）
  - Added: `doctor_check_paper_rune` — Paper Rune 検出
  - Added: `doctor_check_changelog_entry` — CHANGELOG 整合性チェック

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v70.8.0`（`fav doctor` 強化）に更新する
- [x] 「次に切る版」を `v70.9.0` に更新する

---

## T7: 最終確認

- [x] `cargo test v708000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3578 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.8.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### code-reviewer 指摘（実装後）
- **[HIGH] `dir.file_name()` ベースの .fav 推定が実際の rune 構造と不一致**: `read_dir` で全 `*.fav`（`*.test.fav` 除く）を列挙する方式に変更
- **[HIGH] `rune.toml` 非存在時に `Ok` を返す設計が不正確**: `DoctorStatus::Warn` + `"rune.toml not found"` に変更
- **[MED] テストが別理由（ファイル未発見）で通過していた**: 空 `.fav` / 非空 `.fav` 両ケースを明示的にテスト、`detail` フィールド確認も追加
- **[MED] `detail` フィールド未検証**: `assert_eq!(check_ok.detail, "v70.6.0 entry found")` を追加
- **[MED] `read_to_string` 失敗時の設計**: `read_dir` 方式で `.unwrap_or(false)` とし実装なし扱い（誤検知 Fail を回避）
- **[LOW] tempdir 競合**: サフィックス変更（`_empty` / `_impl`）で区別
- **[LOW] `cmd_doctor_run` への組み込み漏れ**: spec でスコープ外と明記済みのため対応なし

### spec-reviewer 指摘（実装前）
- **[HIGH] `DoctorStatus::Error` は存在しない**: `DoctorStatus::Fail` に全置換（spec/plan/tasks）
- **[HIGH] `--fix` フラグのロードマップ不整合**: spec Background に延期理由を明記
- **[HIGH] self-hosting coverage チェック除外の根拠なし**: spec に v70.7.0 で既実装済みと明記
- **[MED] ロードマップのテスト数 3575 が誤り**: roadmap を 3578 / v70.9.0 を 3580 に修正
- **[MED] T4 にテスト関数名リネーム注記なし**: 注記追加
- **[MED] spec 出力例が `✓/⚠/✗`**: `[OK]/[WARN]/[FAIL]` に修正
- **[LOW] 完了チェックリストに current.md 確認なし**: 項目追加

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `doctor_detects_paper_rune` が pass
- [x] `doctor_detects_missing_changelog_entry` が pass
- [x] テスト総数: 3578（+2）
- [x] `versions/current.md` が v70.8.0 に更新されていることを確認
