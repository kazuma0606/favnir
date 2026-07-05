# v32.0.0 仕様書 — Language Polish マイルストーン宣言

## 概要

v31.1〜v31.9 の全コンポーネントが完成したことを受け、
**Language Polish** マイルストーンを正式宣言する。

---

## 背景

ロードマップ v32.0 より:

> **Language Polish の定義（本プロジェクト固有）**
> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」

---

## 達成コンポーネント

| コンポーネント | 完了バージョン | 内容 |
|---|---|---|
| エラーメッセージ v2（rustc スタイル） | v31.1.0 | E0001〜E0021 全件に `hint:` / `note:` 付与、`-->` + `|` 表示 |
| typo 候補 + エラーコード URL | v31.2.0 | Levenshtein ≤ 2 で候補提示、全コードに URL 付与 |
| fav explain コマンド | v31.3.0 | `fav explain E0001〜E0021`（説明・原因・修正例・URL 出力） |
| REPL 品質向上 | v31.4.0 | `:doc` / `:load` / `:history` / `:save` / タブ補完 |
| LSP Inlay Hints | v31.5.0 | `bind` 変数の型推論結果をエディタでインライン表示 |
| fav test --watch | v31.6.0 | ファイル変更を検知してテストを自動再実行（500ms ポーリング） |
| fav check --all | v31.7.0 | プロジェクト全体の .fav ファイルをクロスチェック |
| fav scaffold | v31.8.0 | 既存プロジェクトに `stage` / `seq` スタブを自動追記 |
| ドッグフード修正 vol.2 | v31.9.0 | REPL 空行スキップ / `check --all` 空ディレクトリ警告 |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.9.0` → `32.0.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_9_0` をスタブ化
- `fav/src/driver.rs` — `v320000_tests`（4 件）追加（`use super::*` **なし**、`include_str!` のみ）
- `MILESTONE.md` — v32.0.0「Language Polish」セクションを先頭に追加
- `README.md` — v32.0 マイルストーン宣言の一行を v31.0 行の直後に追加
- `CHANGELOG.md` — `[v32.0.0]` セクション追加
- `benchmarks/v32.0.0.json` 新規作成
- `versions/current.md` — v32.0.0 に更新
- **`cargo clean` + `fav/tmp/hello.fav` 復元 + `cargo build` + `cargo test`**（マイルストーン版の必須クリーンアップ）

### OUT OF SCOPE

- site/ MDX 更新（次フェーズで実施）
- v32.1〜のロードマップ作成（別途作業）

> **cargo clean 注意事項**:
> `cargo clean` を実行すると `fav/tmp/hello.fav` が削除される。
> `bootstrap_c2_artifact_roundtrip` テストはこのファイルに依存するため、
> `cargo clean` 直後に必ず復元すること。
>
> 復元内容:
> ```favnir
> fn add(a: Int, b: Int) -> Int {
>     a + b
> }
>
> fn main() -> Bool {
>     add(1, 2) == 3
> }
> ```

---

## テスト設計（v320000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_32_0_0` | `Cargo.toml` に `"32.0.0"` が含まれること |
| 2 | `milestone_language_polish_declared` | `MILESTONE.md` に `"Language Polish"` が含まれること |
| 3 | `readme_mentions_v32_0` | `README.md` に `"v32.0"` が含まれること |
| 4 | `benchmark_v32_0_0_exists` | `benchmarks/v32.0.0.json` に `"32.0.0"` が含まれること |

> `v320000_tests` は `use super::*` **なし**（`include_str!` のみ使用 — v31.0.0 と同じパターン）。

---

## MILESTONE.md 追記内容（先頭に追加）

```markdown
## v32.0.0 — Language Polish（2026-07-03）

> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」
> = Language Polish の完成を象徴する定義

v32.0.0 をもって、Favnir の **Language Polish** を正式に宣言する。

エラーメッセージが rustc スタイル（`-->` ファイル位置 + `|` ソース行 + `= ヒント:`）に刷新され、
typo 候補（Levenshtein ≤ 2）と全エラーコード URL が付与された。
`fav explain E0001` でエラーの説明・修正例がターミナルで確認できる。
REPL は `:doc` / `:load` / `:history` / `:save` コマンドとタブ補完を備え、
データ探索ツールとして実用レベルに達した。
LSP Inlay Hints により `bind` 変数の型推論結果がエディタでインライン表示される。
`fav test --watch` と `fav check --all` / `fav scaffold` が揃い、
「書いていて気持ちいい」開発体験を達成した。

### 達成コンポーネント（v31.1〜v31.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| エラーメッセージ v2 | v31.1 | rustc スタイル・E0001〜E0021 全件 hint: 付与 |
| typo 候補 + URL | v31.2 | Levenshtein ≤ 2 候補提示・全エラーコード URL |
| fav explain | v31.3 | `fav explain E0001〜E0021` 説明・修正例出力 |
| REPL 品質向上 | v31.4 | :doc / :load / :history / :save / タブ補完 |
| LSP Inlay Hints | v31.5 | bind 変数の型推論結果インライン表示 |
| fav test --watch | v31.6 | ファイル変更で自動テスト再実行 |
| fav check --all | v31.7 | プロジェクト全体クロスファイルチェック |
| fav scaffold | v31.8 | stage / seq スタブを既存プロジェクトに追記 |
| ドッグフード修正 vol.2 | v31.9 | REPL 空行スキップ / check --all 空ディレクトリ警告 |

**宣言日**: 2026-07-03
**宣言バージョン**: v32.0.0
```

---

## README.md 追記内容

v31.0 行の直後に追加:

```markdown
**v32.0（2026-07-03）で、[Language Polish](./MILESTONE.md) マイルストーンを宣言しました。**
エラーメッセージが rustc スタイルに刷新され、`fav explain E0001` でエラー詳細を確認できます。REPL が `:doc` / `:history` / タブ補完を備え、`fav test --watch` / `fav check --all` / `fav scaffold` が揃いました。
```

---

## 完了条件

- `Cargo.toml` version = `"32.0.0"`
- `MILESTONE.md` に `"Language Polish"` セクションが存在すること
- `README.md` に `"v32.0"` の記述があること
- `cargo test --bin fav v320000` — 4/4 PASS
- `cargo test`（`cargo clean` 後）— 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v32.0.0]` セクション
- `benchmarks/v32.0.0.json` 存在
- `versions/current.md` を v32.0.0 に更新
- `tasks.md` が COMPLETE
