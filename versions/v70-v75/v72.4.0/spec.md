# v72.4.0 仕様 — REPL 2.0

Date: 2026-08-12
Status: 計画中

---

## Background

既存の `fav repl`（`cmd_repl` / `ReplSession`）は v9.10.0 で実装され、`:history` / `:load` / `:save` / マルチライン継続（`needs_continuation`）が実装済み。
v72.4.0 では REPL 体験の大幅強化として次を追加する:

1. `:timing` モード — 式評価の実行時間を ms 単位で表示
2. TAB 補完ヘルパー関数 — スコープ内変数・関数名からプレフィックス補完

ロードマップ記載の `rustyline` 統合・Rune メソッド補完・`~/.fav_history` 永続化は
外部クレート追加コストとユニットテスト困難性を考慮し v72.5.0 以降に延期した。
本バージョンでは `:timing` モードとテスト可能な補完ロジック（`repl_tab_complete`）を先行実装する。

---

## Goals

1. `repl_tab_complete(prefix: &str, scope: &[&str]) -> Vec<String>` を `pub fn` で `driver.rs` に追加する
   - `prefix` に前方一致する要素を `scope` から返す
   - 大文字小文字区別あり
2. `ReplSession` に `timing_enabled: bool` フィールドを追加する
3. `:timing on` / `:timing off` コマンドを `cmd_repl` に追加する
4. `handle_expression` 内でタイミング計測（`std::time::Instant`）を追加し、`timing_enabled` 時に ms 表示する
5. テスト 2 件を `v724000_tests` モジュールとして `driver.rs` に追加する

---

## API 例

### TAB 補完

```rust
let completions = repl_tab_complete("Li", &["List", "Csv", "linq"]);
// → ["List"]  (前方一致、入力順)
```

### `:timing` モード

```
fav> :timing on
Timing enabled.
fav> List.map([1,2,3], |x| x * x)
[1, 4, 9] : List<Int>  (0.1ms)
fav> :timing off
Timing disabled.
```

---

## 実装詳細

### `repl_tab_complete`

```rust
pub fn repl_tab_complete(prefix: &str, scope: &[&str]) -> Vec<String> {
    scope.iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| s.to_string())
        .collect()
}
```

### `ReplSession::timing_enabled`

```rust
pub struct ReplSession {
    // 既存フィールド...
    pub timing_enabled: bool,
}
```

`ReplSession::new()` で `timing_enabled: false` を追加する。

### `:timing` ハンドラ

`cmd_repl` の `match line` ブロックに追加:

```rust
":timing on"  => { session.timing_enabled = true;  println!("Timing enabled."); }
":timing off" => { session.timing_enabled = false; println!("Timing disabled."); }
```

### タイミング計測

`handle_expression` に `session` の参照を渡す（または `timing_enabled` フラグを渡す）:

```rust
let start = std::time::Instant::now();
// ... 既存の評価ロジック ...
if session.timing_enabled {
    println!("({}ms)", start.elapsed().as_millis());
}
```

**注意**: `handle_expression` のシグネチャ変更が必要な場合は最小限の変更に留める。
`session` を参照渡しできない場合は `timing_enabled: bool` を引数として渡す。

---

## 成功条件

- `repl2_tab_completion`: `repl_tab_complete("Li", &["List", "Csv", "linq"])` が `["List"]` を返す
- `repl2_multiline_input`: `needs_continuation("fn main() {")` が `true` を返す（`pub fn` 化して外部からテスト可能にする）
- `cargo test v724000` で 2 件 pass
- `cargo test` 全体で 3622 tests pass（v72.3.0 完了時点 3620 + 2）

**WASM への影響**: なし（`std::time::Instant` は WASM でも使用可能、`rustyline` は追加しない）。

---

## エラーコード

新規エラーコードなし。

---

## 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `repl_tab_complete` / `needs_continuation` pub 化 / `ReplSession.timing_enabled` / `:timing` ハンドラ / `v724000_tests` 追加 |
| `fav/Cargo.toml` | `version = "72.3.0"` → `"72.4.0"` |
| `CHANGELOG.md` | `## [v72.4.0]` エントリ追加 |
| `versions/current.md` | 進行中バージョンを v72.4.0 に更新 |

---

## スコープ外（明示的除外）

- `rustyline` クレートによる readline 編集・TAB キー統合（v72.5.0 以降）
- `:import rune` REPL コマンド（現状 REPL はインポート非対応）
- `:debug` / `:type` コマンドの拡張（既存で対応済み）
- REPL の WASM 対応（web UI 統合は別タスク）
- `site/content/docs/cli/repl.mdx` 更新（v72.5.0 以降）
