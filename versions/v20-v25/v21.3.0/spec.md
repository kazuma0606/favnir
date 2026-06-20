# v21.3.0 Spec — テストカバレッジ HTML / LCOV 出力

## 概要

`fav test --coverage` の出力形式を **HTML レポート / LCOV** に拡張し、
どのパイプライン経路がテストされているかをブラウザと CI で可視化できるようにする。

**テーマ**: Developer Tooling Complete シリーズ第3弾 — 「カバレッジが見えると改善できる」

---

## 動機

v17.6.0 で `fav test --coverage` を実装済みだが、出力は `coverage.txt`（テキスト）のみ。
HTML レポートがないため、ブラウザで色付き行表示ができない。
LCOV がないため、GitHub Actions の coveralls / codecov 連携ができない。

---

## 現状（実装済み）

| 機能 | 状態 |
|---|---|
| `fav test --coverage` | ✅ 実装済み — テキスト形式でコンソール出力 |
| `--coverage-report <dir>` → `coverage.txt` | ✅ 実装済み |
| 関数別カバレッジ（`format_coverage_report_by_fn`） | ✅ 実装済み |
| HTML レポート（`coverage/index.html`） | ❌ 未実装 |
| LCOV 形式（`coverage/lcov.info`） | ❌ 未実装 |
| コンソールサマリー改善（ファイル別 ✓/✗ 行） | ❌ 未実装 |

---

## 成果物一覧

| 成果物 | 役割 |
|---|---|
| `fav/src/coverage/mod.rs` | HTML レポート / LCOV 生成ロジック |
| `fav/src/driver.rs` | `cmd_test` の coverage 出力部分を更新 / `--html` / `--lcov` フラグ追加 |
| `fav/src/main.rs` | `fav test` の引数パース更新（`--html` / `--lcov`） |
| `site/content/docs/tools/coverage.mdx` | 使い方ドキュメント（HTML / LCOV 例含む） |

---

## 機能仕様

### CLI

```bash
# テキストサマリー（既存動作は変更なし）
fav test --coverage src/

# HTML レポートを coverage/ ディレクトリに生成（--coverage-report で出力先指定）
fav test --coverage --html --coverage-report coverage/ src/
# → coverage/index.html（ファイル一覧 + 行ハイライト）

# LCOV 形式を coverage/ ディレクトリに生成
fav test --coverage --lcov --coverage-report coverage/ src/
# → coverage/lcov.info（CI 連携用）

# HTML + LCOV 同時生成
fav test --coverage --html --lcov --coverage-report coverage/ src/
```

**注意**: `--html` / `--lcov` は `--coverage` と `--coverage-report <dir>` を組み合わせて使う。
`--html` / `--lcov` 単独では警告を表示して終了する（ファイル出力先が不明のため）。
v21.3.0 では `--coverage-report` のデフォルト値は設けない（明示指定を要求する）。

**スコープ外（将来版）**: branch カバレッジ（match の各 arm）は v21.3.0 では対象外。行カバレッジのみ実装。

### コンソールサマリー改善

`--coverage` 時のコンソール出力をファイル別の ✓/✗ 形式に変更:

```
Coverage: 78.4% (234/298 lines)
  ✓ src/pipeline.fav      95.2%  (40/42)
  ✓ src/transform.fav     88.1%  (59/67)
  ✗ src/loader.fav        51.3%  (81/158)  ← 要改善
```

しきい値（デフォルト 80%）以下のファイルは `✗` で表示。

### HTML レポート仕様

`coverage/index.html` の構成:

1. **サマリーヘッダー**: プロジェクト全体のカバレッジ率・カバー行 / 全行
2. **ファイル一覧テーブル**: ファイル名・行カバレッジ率・関数カバレッジ率
3. **ファイル詳細ページ（インライン）**: 各行を緑（covered）/ 赤（uncovered）/ 灰（non-executable）でハイライト

HTML は外部 CSS/JS 依存なし（スタイルをインライン `<style>` に記述）。

### LCOV 形式仕様

```
TN:
SF:src/pipeline.fav
FN:10,LoadCsv
FN:25,Transform
FNDA:1,LoadCsv
FNDA:0,Transform
FNF:2
FNH:1
DA:11,1
DA:12,1
DA:26,0
LF:3
LH:2
end_of_record
```

---

## テスト（v213000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_3_0` | Cargo.toml に `"21.3.0"` が含まれる |
| `coverage_html_contains_structure` | HTML 出力に `<html>`・`<table>`・カバレッジ率が含まれる |
| `coverage_html_highlights_covered_lines` | カバー済み行が `covered` クラス、未カバー行が `uncovered` クラス |
| `coverage_lcov_format` | LCOV 出力に `SF:` / `DA:` / `end_of_record` が含まれる |
| `coverage_summary_line_format` | コンソールサマリーに `Coverage:` と `✓`/`✗` が含まれる |

---

## 完了条件

- [ ] `fav test --coverage --html src/` で `coverage/index.html` が生成される
- [ ] `fav test --coverage --lcov src/` で `coverage/lcov.info` が生成される
- [ ] コンソールに `Coverage: XX.X% (N/M lines)` のサマリー行が表示される
- [ ] HTML にファイル名・カバレッジ率・行ハイライトが含まれる
- [ ] LCOV に `SF:` / `FN:` / `DA:` / `end_of_record` が含まれる
- [ ] 既存の `--coverage` / `--coverage-report` の動作がリグレッションしない
- [ ] `cargo test v213000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `CHANGELOG.md` に v21.3.0 エントリが追加されている
- [ ] `fav/Cargo.toml` version が `21.3.0`
- [ ] `site/content/docs/tools/coverage.mdx` が存在する
