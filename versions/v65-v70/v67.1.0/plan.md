# v67.1.0 実装計画 — `fav debug`（ステップ実行デバッガ）

Version: 67.1.0
Status: 未着手
Base tests: 3497
Target tests: 3499

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/debug.rs` 作成 + `main.rs` に `mod debug;` 追加

`fav/src/debug.rs` を新規作成する。

必須要件:
- `"step"` を含む（`debug_step_execution` テストがアサート）
- `"inspect"` を含む（`debug_step_execution` テストがアサート）
- `"breakpoint"` を含む（`debug_breakpoint_stage` テストがアサート）
- `pub fn cmd_debug(src: &str, _args: &[String]) -> String` を実装

次に `fav/src/main.rs` に `mod debug;` を追加する（追加しないと `debug.rs` が型チェックされない）。

### Step 2: `site/content/docs/tools/debug.mdx` 作成

`site/content/docs/tools/` ディレクトリに `debug.mdx` を新規作成する。
MDX 先頭に `import` 文を置かない（acorn パースエラー回避）。
`fav debug pipeline.fav` の使用例・`step` / `breakpoint` / `inspect` の説明を記述する。

### Step 3: `driver.rs` テスト追加

`// -- v67000_tests (v67.0.0)` コメントの直前に `v67100_tests` を挿入。

2 テスト関数:
- `debug_step_execution` — `include_str!("debug.rs")` に `"step"` と `"inspect"` を含む
- `debug_breakpoint_stage` — `include_str!("debug.rs")` に `"breakpoint"` を含む

### Step 4: ビルド・テスト確認

```bash
# 以下は順番に実行すること
cargo build
cargo test --bin fav v67100_tests
cargo test -j 8 -- --test-threads=8
```

---

## `driver.rs` 挿入コード

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

## リスク・注意点

- `debug.rs` は `fav/src/` に配置する（`include_str!("debug.rs")` のパスは driver.rs と同じディレクトリ）
- **`fav/src/main.rs` に `mod debug;` を追加すること**。未追加だと `debug.rs` がコンパイルされず型エラーが潜伏する
- `cmd_debug` 関数のシグネチャは将来の main.rs 統合を想定して `pub` にする
- MDX 先頭に `import` 文を置くと acorn パースエラーになる（過去実績）

### Step 5: ドキュメント・ステータス更新

T4（全テスト通過）を確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.1.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を v67.1.0 に更新
- 本 `tasks.md` を COMPLETE に更新

## 非スコープ

- 実際のインタラクティブ REPL 実装 — 将来フェーズ
- main.rs への `fav debug` コマンド登録 — 将来フェーズ
- VM への実ステップ実行フック — 将来フェーズ
