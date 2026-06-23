# v22.0.0 実装計画 — Developer Tooling Complete マイルストーン宣言

## 実装方針

v21.0.0（Runtime Excellence 宣言）と同じパターン。
新機能・Rust コード変更は最小限。主な作業はドキュメント整備とバージョン更新。

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `benchmarks/v22.0.0.json` 作成 | なし |
| T2 | `fav/Cargo.toml` バージョン更新（21.8.0 → 22.0.0） | なし |
| T3 | `CHANGELOG.md` 更新（v22.0.0 エントリ追加） | なし |
| T4 | `README.md` 更新（Developer Tooling セクション追加） | なし |
| T5 | `site/content/docs/tools/developer-tooling.mdx` 新規作成 | なし |
| T6 | `fav/src/driver.rs` — `v220000_tests` 追加（5 件） | T1, T2, T3, T4, T5 |

**Rust コードへの変更は T2（バージョン）と T6（テスト）のみ。**

---

## T1: `benchmarks/v22.0.0.json` — マイルストーンスナップショット

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
    "lint_rules": "W001-W005(v9.3.0) + W006-W009(v13.1.0等) + W010-W019(v21.4.0) = 19件実装済み",
    "lsp_features": "completion/hover/diagnostics/goto_def/code_action/rename/references/signature_help の 8機能",
    "test_count": "v21.8.0完了直後の実測値。v220000_tests 5件追加後は増加する"
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

> `metrics` キーは `compare.fav` 形式と互換が必要な `"metrics"` 文字列を含むことがテストで確認される。

---

## T2: `fav/Cargo.toml` バージョン更新

```toml
version = "22.0.0"
```

---

## T3: `CHANGELOG.md` 更新

v21.1.0〜v21.8.0 のエントリはすでに存在する（要確認）。先頭に v22.0.0 エントリを追加:

```markdown
## [v22.0.0] — 2026-06-21 — Developer Tooling Complete マイルストーン宣言

v21.1.0〜v21.8.0 で達成した開発者ツール整備の集大成。
全 5 完了条件（DAP ステップ実行 / カバレッジ HTML / Mermaid 出力 /
LSP rename / Playground 共有 URL）を達成。

```

既存の v21.8.0 エントリの直上に挿入する。

---

## T4: `README.md` 更新

### 変更箇所

1. バージョンバッジ / 「現在のバージョン」を v22.0.0 に更新
2. **Developer Tooling** セクションを Features 一覧に追加（Runtime Excellence セクションの直下）:

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

3. バージョン履歴表に v21.1.0〜v22.0.0 のエントリを追加:

```markdown
| v21.1.0 | DAP デバッガー |
| v21.2.0 | `fav explain` Mermaid / D2 出力 |
| v21.3.0 | `fav test --coverage` HTML / LCOV |
| v21.4.0 | `fav lint` W010〜W019 |
| v21.5.0 | LSP コードアクション / rename / references |
| v21.6.0 | Playground v2（共有 URL）|
| v21.7.0 | `fav doc` サイト生成 |
| v21.8.0 | `fav migrate` 強化 |
| v22.0.0 | Developer Tooling Complete マイルストーン宣言 |
```

---

## T5: `site/content/docs/tools/developer-tooling.mdx`

```mdx
# Developer Tooling

v21.x シリーズ（v21.1.0〜v21.8.0）で達成した開発者ツール整備の全体像。

> **Developer Tooling Complete マイルストーン（v22.0.0）**: 全 5 完了条件達成

## 達成した完了条件

| 完了条件 | 達成バージョン |
|---|---|
| VS Code でブレークポイントを置いてステップ実行できる | v21.1.0 ✅ |
| `fav test --coverage` で HTML カバレッジレポートが生成される | v21.3.0 ✅ |
| `fav explain --format mermaid` が動作する | v21.2.0 ✅ |
| LSP の `rename` が全参照を追跡してリネームできる | v21.5.0 ✅ |
| Playground でコードの共有 URL が生成できる | v21.6.0 ✅ |

## ツール一覧

- [DAP デバッガー](./dap) — v21.1.0
- [`fav explain` 可視化](../cli/explain) — v21.2.0
- [テストカバレッジ](./coverage) — v21.3.0
- [`fav lint`](./lint) — v21.4.0
- [LSP](./lsp) — v21.5.0
- [Playground](./playground) — v21.6.0
- [`fav doc` サイト](./doc-site) — v21.7.0
- [`fav migrate`](../cli/migrate) — v21.8.0
```

---

## T6: `fav/src/driver.rs` — `v220000_tests` 追加

### 事前: `v218000_tests::version_is_21_8_0` に `#[ignore]` を追加

```rust
#[test]
#[ignore]
fn version_is_21_8_0() { ... }
```

### テストコード

```rust
#[cfg(test)]
mod v220000_tests {
    use super::*;

    fn repo_path(rel: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(rel)
    }

    #[test]
    fn version_is_22_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"22.0.0\""), "Cargo.toml should have version 22.0.0");
    }

    #[test]
    fn changelog_has_v21x_entries() {
        let cl = include_str!("../../CHANGELOG.md");
        for v in &["v21.1.0", "v21.2.0", "v21.3.0", "v21.4.0",
                   "v21.5.0", "v21.6.0", "v21.7.0", "v21.8.0", "v22.0.0"] {
            assert!(cl.contains(v), "CHANGELOG should have {} entry", v);
        }
    }

    #[test]
    fn readme_mentions_dap() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("DAP") || readme.contains("デバッガー"),
            "README should mention DAP debugger"
        );
    }

    #[test]
    fn readme_mentions_coverage() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("coverage") || readme.contains("カバレッジ"),
            "README should mention coverage"
        );
    }

    #[test]
    fn bench_v22_baseline_exists() {
        let content = include_str!("../../benchmarks/v22.0.0.json");
        assert!(content.contains("\"metrics\""),
            "v22.0.0.json should contain metrics field");
    }
}
```

---

## 実装上の注意点

### T6 は T1 完了前に `cargo check` を実行しないこと

`bench_v22_baseline_exists` テストは `include_str!("../../benchmarks/v22.0.0.json")` を使用する。
`include_str!` はコンパイル時にファイルが存在しないとビルドエラーになる。
**T1（benchmarks/v22.0.0.json 作成）を完了させてから T6 の実装・`cargo check` を実行すること。**

### CHANGELOG の既存確認

v21.1.0〜v21.8.0 のエントリはすでに存在している（事前確認済み）。
T3 では v22.0.0 エントリの追加のみ行う。ダブルエントリに注意。

### benchmarks/v22.0.0.json のフォーマット

v21.0.0.json は Runtime Excellence の実行時パフォーマンス指標を `"metrics"` に格納した。
v22.0.0.json は開発者ツールのメタデータ（テスト件数・lint ルール数等）を `"metrics"` に格納する。
`compare.fav` との互換性は `"metrics"` キーの存在のみで確認する（数値型は任意）。

### T5 の MDX リンク

各ツールの個別 MDX（`dap.mdx` / `coverage.mdx` / `lint.mdx` / `lsp.mdx` / `playground.mdx` / `doc-site.mdx`）はすでに存在するため、
`developer-tooling.mdx` からのリンクはすべて有効なリンクになる。
`fav explain` と `fav migrate` は `site/content/docs/cli/` 配下への相対パスで参照する。

---

## リスクと対策

| リスク | 対策 |
|---|---|
| CHANGELOG に v21.x エントリが欠けている | T6 の `changelog_has_v21x_entries` でコンパイル時に全バージョン確認 |
| README に DAP / coverage が未記載 | T6 の `readme_mentions_*` テストで確認 |
| `benchmarks/v22.0.0.json` の JSON 形式が不正 | `include_str!` + `contains("\"metrics\"")` でシンプルに確認 |
| `developer-tooling.mdx` 内リンクが 404 | 各ツール MDX の存在は v21.x で確認済み。新規リンク先なし |
