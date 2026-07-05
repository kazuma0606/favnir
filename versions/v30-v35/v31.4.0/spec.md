# v31.4.0 仕様書 — REPL 品質向上（プロンプト / 履歴上限 / セッション補完）

## 概要

`fav repl` をデータ探索ツールとして実用レベルにする。
`:doc` / `:load` / `:history` / `:save` / タブ補完のコマンド群は v17.5.0 で実装済みだが、
以下の品質ギャップを埋める。

---

## 背景

ロードマップ v31.4 より:

```
favnir> :doc List.group_by
  List.group_by : List<T> -> (T -> String) -> Map<String, List<T>>

favnir> :load src/pipeline.fav
  loaded: LoadCsv, ValidateRows, WriteToDb, ...

favnir> :history
  1: List.length([1, 2, 3])
  2: String.split("a,b,c", ",")

favnir> List.g<Tab>
  List.get    List.group_by
```

---

## 既存実装の確認事項

ロードマップ v31.4 は「上記 5 コマンドが動作する」を完了条件に掲げているが、
`:doc` / `:load` / `:history` / `:save` / タブ補完は v17.5.0 で実装済み（`CHANGELOG.md [v17.5.0]` に記録）。
v31.4.0 の目的はこれらの実装済み機能の **品質ギャップを埋める** ことである。

| 項目 | 状態 |
|---|---|
| `:doc <fn>` — BUILTIN_DOCS を検索 | **実装済み** (`handle_doc_cmd` / `repl_doc_str`, v17.5.0) |
| `:load <file>` | **実装済み** (`handle_load_cmd`, v17.5.0) |
| `:history` — セッション内入力履歴を表示 | **実装済み** (`print_history`, v17.5.0) |
| `:save <file>` | **実装済み** (`handle_save_cmd`, v17.5.0) |
| タブ補完 — `:` コマンド + BUILTIN_DOCS | **実装済み** (`repl_complete_prefix`, v17.5.0) |
| REPL プロンプト | **`> `のまま** — ロードマップは `favnir> ` を期待 |
| 履歴上限 | **上限なし** — ロードマップは最大 100 件を指定 |
| セッション定義名の補完 | **未対応** — `:load` で読み込んだ関数名が補完されない |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.3.0` → `31.4.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_3_0` をスタブ化
- `fav/src/driver.rs` — REPL プロンプトを `> ` → `favnir> ` に変更
- `fav/src/driver.rs` — `add_history()` に 100 件上限を追加（超えた場合は先頭を削除）
- `fav/src/driver.rs` — `repl_complete_with_defs(prefix, def_names)` 関数を追加（セッション定義名を補完に含める）
- `fav/src/driver.rs` — `v314000_tests`（3 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.4.0]` セクション追加
- `benchmarks/v31.4.0.json` 新規作成
- `versions/current.md` — v31.4.0 に更新

### OUT OF SCOPE

- `rustyline` 等外部クレートによるリアルタイムタブ補完の実装（インタラクティブ補完は v32.x 以降）
- `:doc` への user-defined 関数のドキュメント表示（docstring 構文が未定義）
- `cmd_repl()` のタブキーハンドリング（stdin がライン単位のため）
- `CHANGELOG.md` への `:doc` / `:load` / `:history` / `:save` の追記（v17.5.0 で記録済み、再掲しない）

---

## 実装詳細

### REPL プロンプト変更

```rust
// before
let _ = write!(out, "> ");

// after
let _ = write!(out, "favnir> ");
```

### 履歴上限（100 件）

push 後のサイズが 101 件になった時点で先頭エントリを削除し、最大 100 件を維持する。

```rust
fn add_history(&mut self, line: &str) {
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

### セッション定義名補完

`repl_complete_prefix` はシグネチャを変えず、新関数を追加する:

```rust
pub fn repl_complete_with_defs(prefix: &str, def_names: &[String]) -> Vec<String> {
    let mut result = repl_complete_prefix(prefix);
    for name in def_names {
        if name.starts_with(prefix) && !result.contains(name) {
            result.push(name.clone());
        }
    }
    result.sort();
    result
}
```

---

## テスト設計（v314000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_4_0` | `Cargo.toml` に `version = "31.4.0"` |
| 2 | `benchmark_v31_4_0_exists` | `benchmarks/v31.4.0.json` に `"31.4.0"` |
| 3 | `repl_complete_with_defs_delegates_to_prefix` | `repl_complete_with_defs("List.", &[])` が `"List.map"` を含む（既存 BUILTIN_DOCS への委譲パスを検証）|
| 4 | `repl_complete_with_defs_returns_session_defs` | `repl_complete_with_defs("my", &["my_fn".to_string()])` が `"my_fn"` を含み `"other_fn"` を含まない |

> `v314000_tests` は `use super::*` あり。

---

## 完了条件

- `Cargo.toml` version = `"31.4.0"`
- REPL プロンプトが `favnir> ` になっている
- `add_history()` が 100 件を超えたとき先頭エントリを削除する
- `repl_complete_with_defs("List.", &[])` が `"List.map"` 等を含む（BUILTIN_DOCS への委譲が機能していること）
- `repl_complete_with_defs("my", &["my_fn".to_string()])` が `["my_fn"]` を含む
- `cargo test v314000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.4.0]` セクション
- `benchmarks/v31.4.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v31.4.0 に更新
- `tasks.md` が COMPLETE
