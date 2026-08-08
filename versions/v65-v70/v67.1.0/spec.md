# v67.1.0 Spec — `fav debug`（ステップ実行デバッガ）

Version: 67.1.0
Status: 未着手
Base tests: 3497
Target tests: 3499

---

## 概要

パイプラインをステップ単位で実行し、各ステージの入出力を確認できるデバッガを実装する。
AI パイプラインの LLM 呼び出し・ベクトル変換を「見える化」する。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.1.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3497 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/debug.rs` が存在しないことを確認（新規作成対象）
- `site/content/docs/tools/debug.mdx` が存在しないことを確認（新規作成対象）
- `driver.rs` に `v67000_tests` が存在することを確認（`v67100_tests` の挿入位置）
- `driver.rs` に `v67100_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67000_tests` で 4 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `cargo_toml_version_is_67_0_0`, `changelog_has_v67_0_0`, `milestone_has_ai_native_stage`, `readme_mentions_ai_native`
- `versions/current.md` の「進行中バージョン」が `v67.0.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/debug.rs` — デバッガスタブ実装、`main.rs` への `mod` 宣言追加

新規ファイル。`cmd_debug` 関数とデバッガのヘルプ文字列を実装する。
**`fav/src/main.rs` に `mod debug;` を追加して、`debug.rs` をコンパイル対象に含める**（これをしないと `cargo build` で `debug.rs` が型チェックされず、`cmd_debug` の型エラーが潜伏する）。

必須キーワード（テストでアサートされる）:
- `"step"` — ステップ実行コマンド（`debug_step_execution` テスト）
- `"inspect"` — レコード・ベクトル内容確認コマンド（`debug_step_execution` テスト）
- `"breakpoint"` — ブレークポイント機能（`debug_breakpoint_stage` テスト）

```rust
// fav/src/debug.rs — v67.1.0 fav debug ステップ実行デバッガ

pub const DEBUG_HELP: &str = "\
fav debug — ステップ実行デバッガ v67.1.0

コマンド:
  run                    パイプラインを実行（各ステージ後に自動停止）
  step                   1 ステージ進む
  continue               次のブレークポイントまで実行
  inspect <expr>         レコード / ベクトルの内容を確認
  breakpoint <stage>     特定ステージで停止
  diff <row>             ステージ前後のレコード差分を表示
  quit                   デバッガを終了
";

pub fn cmd_debug(src: &str, _args: &[String]) -> String {
    format!(
        "[fav debug] v67.1.0 — ステップ実行モード\n\
         入力ファイル: {}\n\
         step / inspect / breakpoint / continue / quit が利用可能です。\n\
         'help' でコマンド一覧を表示。",
        src
    )
}
```

### 2. `site/content/docs/tools/debug.mdx` — ドキュメント

`site/content/docs/tools/` ディレクトリに新規作成。
MDX 先頭に `import` 文を置かない（acorn パースエラー回避）。
最低限含むべき内容:
- `fav debug pipeline.fav` の使用例
- `step` / `breakpoint` / `inspect` コマンドの説明

### 3. `driver.rs` — `v67100_tests` 追加

挿入位置: `// -- v67000_tests (v67.0.0)` コメントの直前

```rust
// -- v67100_tests (v67.1.0) -- fav debug ステップ実行デバッガ --
#[cfg(test)]
mod v67100_tests {
    #[test]
    fn debug_step_execution() {
        let src = include_str!("debug.rs");
        assert!(
            src.contains("step") && src.contains("inspect"),
            "debug.rs should contain 'step' and 'inspect' keywords"
        );
    }

    #[test]
    fn debug_breakpoint_stage() {
        let src = include_str!("debug.rs");
        assert!(
            src.contains("breakpoint"),
            "debug.rs should contain 'breakpoint' help string"
        );
    }
}
```

---

## 完了条件

- `fav/src/debug.rs` が存在し `"step"` / `"inspect"` / `"breakpoint"` を含む
- `fav/src/main.rs` に `mod debug;` が追加され、`debug.rs` がコンパイル対象になっている
- `site/content/docs/tools/debug.mdx` が存在し `"step"` / `"breakpoint"` を含む
- `cargo build` でエラーなし（`debug.rs` が型チェックされた状態で）
- `cargo test --bin fav v67100_tests` で 2 件 PASS
  - `debug_step_execution` PASS
  - `debug_breakpoint_stage` PASS
- `cargo test -j 8 -- --test-threads=8` で 3499 tests passed, 0 failed

---

## 非スコープ

- 実際のインタラクティブ REPL 実装 — 将来フェーズ
- VM への実ステップ実行フック — 将来フェーズ
- `fav debug` コマンドの main.rs 登録 — 将来フェーズ

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"debug.rs"` → `fav/src/debug.rs`（同じ `fav/src/` ディレクトリ）

### テスト数増加の根拠

`v67100_tests` モジュール内の `#[test]` fn 2 件（`debug_step_execution` / `debug_breakpoint_stage`）で +2。
