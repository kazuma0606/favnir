# v22.0.0 — Developer Tooling Complete マイルストーン宣言 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `benchmarks/v22.0.0.json` 作成

- [x] `benchmarks/v22.0.0.json` を作成（plan.md T1 の JSON に従う）
  - トップレベルキー `"metrics"` に開発者ツールメタデータを記録
  - `"milestone_checklist"` に 5 完了条件の達成状況を記録
- [x] valid JSON であることを確認（`uv run python -m json.tool benchmarks/v22.0.0.json` 等）
- [x] `"metrics"` フィールドが存在することを確認

---

### T2: `fav/Cargo.toml` バージョン更新

- [x] `version = "21.8.0"` → `"22.0.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `CHANGELOG.md` 更新

- [x] **事前確認**: `grep "\[v21\." CHANGELOG.md` で v21.1.0〜v21.8.0 の全エントリが存在することを確認
- [x] v22.0.0 エントリを CHANGELOG.md の先頭（現在の v21.8.0 エントリの上）に追加（plan.md T3 に従う）
- [x] 追加後に `grep "\[v22.0.0\]" CHANGELOG.md` で存在確認

---

### T4: `README.md` 更新

- [x] **事前確認**: `grep "現在のバージョン\|Current Version" README.md` でバージョン記載箇所を特定
- [x] バージョン表記を v22.0.0 に更新
- [x] Developer Tooling セクションを Features 一覧に追加（plan.md T4 に従う）
  - Runtime Excellence セクションの直下に追加
  - DAP / coverage / Mermaid / LSP / Playground / doc / migrate / lint W010〜W019 の 8 項目
- [x] バージョン履歴表に v21.1.0〜v22.0.0 のエントリを追加
- [x] `grep "DAP\|デバッガー" README.md` で DAP 記載確認
- [x] `grep "coverage\|カバレッジ" README.md` で coverage 記載確認

---

### T5: `site/content/docs/tools/developer-tooling.mdx` 新規作成

- [x] **事前確認**: `ls site/content/docs/tools/` でリンク先の MDX が存在することを確認
  - `coverage.mdx` / `dap.mdx` / `lint.mdx` / `lsp.mdx` / `playground.mdx` / `doc-site.mdx` の 6 件
- [x] `site/content/docs/tools/developer-tooling.mdx` を新規作成（plan.md T5 の MDX に従う）
  - 5 完了条件の達成表
  - 各ツールへのリンク（8 件）
  - `fav explain` → `../cli/explain`、`fav migrate` → `../cli/migrate` の相対パスを使用
- [x] ファイルが存在することを確認

---

### T6: `fav/src/driver.rs` — `v220000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_21_8_0" fav/src/driver.rs` で `v218000_tests` 内のテストを確認（コマンドはリポジトリルート `C:\Users\yoshi\favnir` から実行する）
- [x] `v218000_tests::version_is_21_8_0` に `#[ignore]` を追加
- [x] `v220000_tests` モジュールを追加（5 件、plan.md T6 のコードに従う）
  - `version_is_22_0_0`
  - `changelog_has_v21x_entries`
  - `readme_mentions_dap`
  - `readme_mentions_coverage`
  - `bench_v22_baseline_exists`
- [x] `cargo test v220000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（実装前の実測値以上）を確認（実装前に `cargo test --bin fav 2>&1 | tail -1` で件数を記録しておくこと）

---

## テスト一覧（v220000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_0_0` | Cargo.toml に `"22.0.0"` が含まれる |
| `changelog_has_v21x_entries` | CHANGELOG に v21.1.0〜v21.8.0 + v22.0.0 全エントリが含まれる |
| `readme_mentions_dap` | README に `"DAP"` または `"デバッガー"` が含まれる |
| `readme_mentions_coverage` | README に `"coverage"` または `"カバレッジ"` が含まれる |
| `bench_v22_baseline_exists` | `benchmarks/v22.0.0.json` が存在し `"metrics"` フィールドを含む |

---

## 完了条件チェックリスト

- [x] `benchmarks/v22.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON
- [x] `fav/Cargo.toml` version が `22.0.0`
- [x] `CHANGELOG.md` に v21.1.0〜v21.8.0 の全エントリが含まれる（既存確認）
- [x] `CHANGELOG.md` に v22.0.0 エントリが含まれる
- [x] `README.md` に Developer Tooling / DAP / coverage の記載がある
- [x] `site/content/docs/tools/developer-tooling.mdx` が存在する
- [x] `cargo test v220000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（実装前の実測値以上合格）

---

## 優先度

```
T1（benchmarks/v22.0.0.json）  ← 最初（T6 の include_str! によるコンパイル時依存あり）
T2（Cargo.toml）               ← T1 と並列可
T3（CHANGELOG.md）             ← T1 と並列可
T4（README.md）                ← T1 と並列可
T5（developer-tooling.mdx）    ← T1 と並列可
T6（driver.rs テスト）         ← T1〜T5 完了後（T1 前に cargo check 不可）
```

Rust コードへの変更は T2（バージョン）と T6（テスト）のみ。
