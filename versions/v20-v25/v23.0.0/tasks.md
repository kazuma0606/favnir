# v23.0.0 — Distributed Scale マイルストーン宣言 タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1886 passed / 0 failed（v230000_tests 5/5 PASS）

---

## タスク一覧

### T1: `benchmarks/v23.0.0.json` 作成

- [x] **事前確認**: `cat benchmarks/v22.8.0.json` で既存フォーマットを確認（リポジトリルート `C:\Users\yoshi\favnir` から実行）
- [x] `benchmarks/v23.0.0.json` を作成（plan.md T1 の JSON に従う）
  - トップレベルキー `"metrics"` に分散機能メタデータを記録
  - `"milestone_checklist"` に 5 完了条件の達成状況を記録
- [x] valid JSON であることを確認（`uv run python -m json.tool benchmarks/v23.0.0.json` 等）
- [x] `"metrics"` フィールドが存在することを確認

---

### T2: `fav/Cargo.toml` バージョン更新

> 注意: T6-ignore（`version_is_22_8_0` への `#[ignore]` 追加）が完了してから実施すること。バージョンを先に変更すると `version_is_22_8_0` テストが失敗する。

- [x] T6-ignore が完了済みであることを確認
- [x] `version = "22.8.0"` → `"23.0.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `CHANGELOG.md` 更新

- [x] **事前確認**: `grep "\[v22\." CHANGELOG.md | head -10` で v22.1.0〜v22.8.0 の全エントリが存在することを確認
- [x] v23.0.0 エントリを CHANGELOG.md の先頭（現在の v22.8.0 エントリの上）に追加（plan.md T3 に従う）
- [x] 追加後に `grep "\[v23.0.0\]" CHANGELOG.md` で存在確認

---

### T4: `README.md` 更新

- [x] **事前確認**: `grep "現在のバージョン\|v22\.0\b" README.md` でバージョン記載箇所を特定
- [x] バージョン表記を v23.0.0 に更新
- [x] Distributed Scale セクションを Features 一覧に追加（plan.md T4 に従う）
  - Developer Tooling セクションの直下に追加
  - Checkpoint / Distributed par / State Rune / Event-driven / Orchestration / SLA / OTel / deploy の 8 項目
- [x] バージョン履歴表に v22.1.0〜v23.0.0 のエントリを追加
- [x] `grep "OpenTelemetry\|OTel" README.md` で OTel 記載確認
- [x] `grep "orchestrate\|DAG" README.md` で orchestrate 記載確認

---

### T5: `site/content/docs/tools/distributed-scale.mdx` 新規作成

- [x] **事前確認**: `ls site/content/docs/cli/` および `ls site/content/docs/runes/` でリンク先 MDX が存在することを確認（リポジトリルート `C:\Users\yoshi\favnir` から実行）
  - `cli/`: `checkpoint.mdx` / `par-distributed.mdx` / `trigger.mdx` / `orchestrate.mdx` / `sla.mdx` / `otel.mdx` / `deploy.mdx`
  - `runes/`: `state.mdx`
- [x] `site/content/docs/tools/distributed-scale.mdx` を新規作成（plan.md T5 の MDX に従う）
  - frontmatter（title / description）
  - 5 完了条件の達成表
  - 各機能ページへのリンク（8 件）
- [x] ファイルが存在することを確認

---

### T6: `fav/src/driver.rs` — `v230000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_22_8_0" fav/src/driver.rs` で `v228000_tests` 内のテストを確認
- [x] **T2 より前に実施**: `v228000_tests::version_is_22_8_0` に `#[ignore]` を追加
- [x] `v230000_tests` モジュールを `v228000_tests` の直後に追加（5 件、plan.md T6 のコードに従う）
  - `version_is_23_0_0`
  - `changelog_has_v22x_entries`
  - `readme_mentions_otel`
  - `readme_mentions_orchestrate`
  - `bench_v23_baseline_exists`
- [x] `cargo test v230000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1882 件以上合格）を確認

---

## テスト一覧（v230000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_0_0` | Cargo.toml に `"23.0.0"` が含まれる |
| `changelog_has_v22x_entries` | CHANGELOG に v22.1.0〜v22.8.0 + v23.0.0 全エントリが含まれる |
| `readme_mentions_otel` | README に `"OpenTelemetry"` または `"OTel"` が含まれる |
| `readme_mentions_orchestrate` | README に `"orchestrate"` または `"DAG"` が含まれる |
| `bench_v23_baseline_exists` | `benchmarks/v23.0.0.json` が存在し `"metrics"` フィールドを含む |

---

## 完了条件チェックリスト

- [x] `benchmarks/v23.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON
- [x] `fav/Cargo.toml` version が `23.0.0`
- [x] `CHANGELOG.md` に v22.1.0〜v22.8.0 の全エントリが含まれる（既存確認）
- [x] `CHANGELOG.md` に v23.0.0 エントリが含まれる
- [x] `README.md` に Distributed Scale セクションの記載がある
- [x] `README.md` に OpenTelemetry の記載がある
- [x] `README.md` に orchestrate の記載がある
- [x] `site/content/docs/tools/distributed-scale.mdx` が存在する
- [x] `cargo test v230000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1882 件以上合格）

---

## 優先度

```
T1（benchmarks/v23.0.0.json）  ← 最初（T6 の include_str! によるコンパイル時依存あり）
T2（Cargo.toml）               ← T1 と並列可、ただし T6-ignore より後
T3（CHANGELOG.md）             ← T1 と並列可
T4（README.md）                ← T1 と並列可
T5（distributed-scale.mdx）   ← T1 と並列可
T6（driver.rs テスト）         ← T1〜T5 完了後（T1 前に cargo check 不可）
```

Rust コードへの変更は T2（バージョン）と T6（#[ignore] + テスト）のみ。

---

## コードレビュー指摘と対応

| # | ラベル | 内容 | 対応 |
|---|--------|------|------|
| 1 | [MED] | `version_is_23_0_0` の assert 文字列が `"23.0.0"` のみで既存パターン（`version = "X.X.X"`）と不一致 | `cargo.contains("version = \"23.0.0\"")` に修正 |
| 2 | [LOW] | `benchmarks/v23.0.0.json` の `_metrics_notes.test_count` コメントと実測値が一致（問題なし） | 変更なし |
| 3 | [LOW] | `distributed-scale.mdx` の機能説明が最小限（他マイルストーン MDX と同水準） | 変更なし |
