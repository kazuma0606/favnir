# v22.0.0 Spec — Developer Tooling Complete マイルストーン宣言

## 概要

v21.x シリーズ（v21.1〜v21.8）で構築した開発者ツール整備の集大成を宣言するマイルストーンリリース。
新しい言語機能の追加はなく、ロードマップ完了条件の確認・CHANGELOG 更新・README 更新・バージョン番号の更新が主な作業。

**テーマ**: 「VS Code で Favnir を書くなら Python より快適」への到達宣言

---

## v21.x で達成した Developer Tooling 機能

| バージョン | 機能 | 達成内容 |
|---|---|---|
| v21.1.0 | DAP デバッガー | `fav dap`（ポート 5678）、ブレークポイント・ステップ実行・変数インスペクション |
| v21.2.0 | `fav explain` 可視化強化 | Mermaid / D2 / JSON 形式のリネージ図出力 |
| v21.3.0 | テストカバレッジ | `fav test --coverage`、HTML レポート + LCOV 出力 |
| v21.4.0 | `fav lint` 強化 | W010〜W019（10 ルール追加）※ロードマップ記載の W006〜W015 から変更 |
| v21.5.0 | LSP コードアクション強化 | codeAction（4種）/ rename / references |
| v21.6.0 | Playground v2 | 共有 URL・テンプレートギャラリー・実行統計 |
| v21.7.0 | `fav doc` サイト生成 | `--format site` / `--serve`、ダークテーマ自己完結 HTML |
| v21.8.0 | `fav migrate` 強化 | `--from/--to`・`--config fav.toml`・移行サマリー |

---

## v22.0.0 実装内容

### 1. バージョン番号更新

- `fav/Cargo.toml`: `21.8.0` → `22.0.0`

### 2. CHANGELOG.md 更新

v21.1.0〜v21.8.0 のエントリはすでに記載済み。v22.0.0 エントリを先頭に追加:

```markdown
## [v22.0.0] — 2026-06-21 — Developer Tooling Complete マイルストーン宣言

v21.1.0〜v21.8.0 で達成した開発者ツール整備の集大成。
全 5 完了条件（DAP ステップ実行 / カバレッジ HTML / Mermaid 出力 /
LSP rename / Playground 共有 URL）を達成。
```

### 3. README.md 更新

- 「現在のバージョン」を v22.0.0 に更新
- **Developer Tooling** セクションを Features 一覧に追加
- バージョン履歴表に v21.1.0〜v22.0.0 のエントリを追加

```markdown
### Developer Tooling（v21.x）
- **DAP デバッガー**: VS Code / Neovim からブレークポイント・ステップ実行（`fav dap`）
- **テストカバレッジ**: HTML レポート + LCOV 出力（`fav test --coverage`）
- **リネージ可視化**: Mermaid / D2 形式のパイプライン図（`fav explain --format mermaid`）
- **LSP 強化**: コードアクション・rename・references（全参照追跡）
- **Playground v2**: 共有 URL・テンプレートギャラリー・実行統計
- **`fav doc` サイト**: `///` コメントから静的 HTML を自動生成（`--serve` でローカルプレビュー）
- **`fav migrate`**: `--from v13 --to v14`・`--config fav.toml`・移行サマリー
- **lint W010〜W019**: ネスト深度・magic number・文字列連鎖など 10 ルール追加
```

### 4. ベンチマーク記録（`benchmarks/v22.0.0.json`）

v21.8.0 時点でのテスト件数・ツール数をスナップショットとして記録する。

```json
{
  "version": "22.0.0",
  "timestamp": "2026-06-21T00:00:00Z",
  "_note": "Developer Tooling Complete milestone snapshot.",
  "metrics": {
    "test_count": 1831,
    "lint_rules": 19,
    "lsp_features": 8,
    "dap_port": 5678,
    "coverage_formats": 2
  },
  "_metrics_notes": {
    "lint_rules": "W001-W005(v9.3.0) + W006-W009(v13.1.0等) + W010-W019(v21.4.0) = 19件実装済み。実測: grep -c 'W0' fav/src/lint.rs で確認",
    "lsp_features": "completion/hover/diagnostics/goto_def/code_action/rename/references/signature_help の 8機能",
    "test_count": "v21.8.0完了直後の実測値。v220000_tests 5件追加後は1836件以上になる"
  },
  "milestone_checklist": {
    "dap_step_execution":      { "achieved": true, "version": "v21.1.0" },
    "coverage_html_report":    { "achieved": true, "version": "v21.3.0" },
    "explain_mermaid_output":  { "achieved": true, "version": "v21.2.0" },
    "lsp_rename_all_refs":     { "achieved": true, "version": "v21.5.0" },
    "playground_share_url":    { "achieved": true, "version": "v21.6.0" }
  }
}
```

### 5. site/ MDX 更新

v22.0.0 ではツール群の概要ページを新規作成する:

- `site/content/docs/tools/developer-tooling.mdx` **新規** — Developer Tooling マイルストーン概要ページ

各ツールの個別 MDX（`dap.mdx` / `coverage.mdx` / `lint.mdx` / `lsp.mdx` / `playground.mdx` / `doc-site.mdx`）はすでに作成済みのため追加不要。

### 6. テスト（v220000_tests、5 件）

```rust
fn version_is_22_0_0()              // Cargo.toml に "22.0.0" が含まれる
fn changelog_has_v21x_entries()     // CHANGELOG に v21.1.0〜v21.8.0 の全エントリが含まれる
fn readme_mentions_dap()            // README に "DAP" または "デバッガー" が含まれる
fn readme_mentions_coverage()       // README に "coverage" または "カバレッジ" が含まれる
fn bench_v22_baseline_exists()      // benchmarks/v22.0.0.json が存在し "metrics" を含む
```

---

## ロードマップ完了条件との対応

| ロードマップ完了条件 | 達成バージョン | 検証テスト |
|---|---|---|
| VS Code でブレークポイントを置いてステップ実行できる | v21.1.0 | `readme_mentions_dap` |
| `fav test --coverage` で HTML レポートが生成される | v21.3.0 | `readme_mentions_coverage` |
| `fav explain --format mermaid` が動作する | v21.2.0 | `changelog_has_v21x_entries` |
| LSP の `rename` が全参照を追跡してリネームできる | v21.5.0 | `changelog_has_v21x_entries` |
| Playground でコードの共有 URL が生成できる | v21.6.0 | `changelog_has_v21x_entries` |

**テスト検証方針の注記:**
DAP と coverage は README 記載を直接確認するテストを設ける。Mermaid / LSP rename / Playground 共有 URL は CHANGELOG エントリの存在をもって実装達成とみなし、個別の README テストは設けない。より厳密な検証が必要な場合は `readme_mentions_mermaid` 等を追加できる。

**lint ルール番号の変更について:**
ロードマップ（v21.1-v22.0.md）は lint ルールを **W006〜W015** と記載していた。実装時に既存ルール番号との整合を取り **W010〜W019** として実装した（W006〜W009 は v21.3.0 以前に別途実装済み）。ロードマップとの乖離は意図的な設計変更である。

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml` に `"22.0.0"` が含まれる | [ ] |
| `CHANGELOG.md` に v21.1.0〜v21.8.0 の全エントリが含まれる（既存確認） | [ ] |
| `CHANGELOG.md` に v22.0.0 エントリが含まれる | [ ] |
| `README.md` に Developer Tooling セクションの記載がある | [ ] |
| `README.md` に DAP の記載がある | [ ] |
| `README.md` に coverage の記載がある | [ ] |
| `benchmarks/v22.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON | [ ] |
| `site/content/docs/tools/developer-tooling.mdx` が存在する | [ ] |
| `cargo test v220000 --bin fav` — 5/5 PASS | [ ] |
| `cargo test --bin fav` — リグレッションなし（実装前の実測値以上合格） | [ ] |
