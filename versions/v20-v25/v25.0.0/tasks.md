# v25.0.0 — Practical Self-Hosting マイルストーン宣言 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.8.0"` であること
- [x] `cargo test --bin fav 2>&1 | grep "test result: ok"` — 1969 件であること
- [x] `grep -n "mod v248000_tests" fav/src/driver.rs | head -3` — 存在すること
- [x] `grep -n "mod v250000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "v1.x" STABILITY.md | head -3` — `"v1.x"` が含まれること

---

### T1: `MILESTONE.md` 作成（リポジトリルート）

- [x] `MILESTONE.md` を新規作成（`"Practical Self-Hosting"` + `"compiler.fav"` 含む）
- [x] 達成済みコンポーネント表（compiler.fav / checker.fav / cli.fav / vm.fav）
- [x] 各コンポーネントの達成バージョン（v8.1.0〜v24.0.0）
- [x] VM エンジンは Rust で永続維持する旨の説明
- [x] ロードマップ最終テスト：項目 1 のみ達成済み、項目 2〜5 は v25.x 延期と明記
- [x] **事後確認**: `grep "Practical Self-Hosting" MILESTONE.md` — 存在すること
- [x] **事後確認**: `grep "compiler.fav" MILESTONE.md` — 存在すること

---

### T2: `README.md` 更新

- [x] `README.md` に `"v25.0"` を追記（テスト要件。偽陽性防止のため `v25.0` を必須とする）
- [x] マイルストーン達成バッジ / セクション追加
- [x] **事後確認**: `grep "v25.0" README.md` — 存在すること

---

### T3: `site/content/docs/v1-release.mdx` 作成

- [x] `site/content/docs/v1-release.mdx` を新規作成（`"v25.0"` 含む）
- [x] v24.1〜v24.8 の各バージョン機能一覧を記載
- [x] STABILITY.md への参照リンクを含む
- [x] **事後確認**: `grep "v25.0" site/content/docs/v1-release.mdx` — 存在すること

---

### T4: `versions/roadmap-v20.1-v25.0.md` 更新

（`roadmap-master.md` は v17〜v20 用のため対象外）

- [x] v24.1〜v24.8 を「完了」に更新
- [x] v25.0.0 を「宣言済み」に更新
- [x] **事後確認**: `grep "v25.0" versions/roadmap-v20.1-v25.0.md` — 存在すること

---

### T5: `fav/src/driver.rs` — v250000_tests 追加

- [x] **T5-1**: `v250000_tests` モジュールを `v248000_tests` の直後に追加（5 件）
  - `milestone_md_has_selfhost_declaration`
  - `readme_mentions_v1_release`
  - `stability_md_exists`
  - `site_v1_release_page_exists`
  - `changelog_has_v25_0_0`
- [x] `cargo test v250000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1974 件合格）

---

### T6: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.8.0"` → `"25.0.0"` に変更
- [x] `CHANGELOG.md` 先頭に v25.0.0 エントリを追加（`[v25.0.0]` 含む）
- [x] `benchmarks/v25.0.0.json` を新規作成（test_count: 1974、duration_ms: 17600）
- [x] `cargo test v250000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1974 件合格）

---

## テスト一覧（v250000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `milestone_md_has_selfhost_declaration` | `MILESTONE.md` に `"Practical Self-Hosting"` + `"compiler.fav"` が含まれる | assert |
| `readme_mentions_v1_release` | `README.md` に `"v25.0"` が含まれる | assert |
| `stability_md_exists` | `STABILITY.md` に `"v1.x"` が含まれる | assert |
| `site_v1_release_page_exists` | `site/content/docs/v1-release.mdx` に `"v25.0"` が含まれる | assert |
| `changelog_has_v25_0_0` | `CHANGELOG.md` に `[v25.0.0]` が含まれる | assert |

---

## 完了条件チェックリスト

- [x] `MILESTONE.md` 作成済み（`"Practical Self-Hosting"` + `"compiler.fav"` 含む）
- [x] `README.md` に `"v25.0"` を追記済み
- [x] `site/content/docs/v1-release.mdx` 作成済み（`"v25.0"` 含む）
- [x] `versions/roadmap-v20.1-v25.0.md` の v24.1〜v25.0 を更新済み
- [x] `cargo test v250000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1974 件合格）
- [x] `CHANGELOG.md` に v25.0.0 エントリ
- [x] `benchmarks/v25.0.0.json` 作成済み（test_count: 1974）
