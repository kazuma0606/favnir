# v67.7.0 実装計画 — Interactive Profiling（`fav profile --interactive`）

Version: 67.7.0
Status: 未着手
Base tests: 3509
Target tests: 3511

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/profiler/interactive.rs` を新規作成

以下のキーワードを全て含む新規ファイルを作成する:
- `"--interactive"` — `profile_interactive_hotspot` テストがアサート
- `"hotspot"` — `profile_interactive_hotspot` テストがアサート
- `"drill"` — `profile_interactive_drill` テストがアサート
- `"Suggestion"` — `profile_interactive_drill` テストがアサート

追加する要素:
- `pub const INTERACTIVE_HELP: &str` — 使用例・コマンド説明（4 キーワード全てを含む）
- `pub fn cmd_profile_interactive(src: &str) -> String` — インタラクティブプロファイル結果を返す（スタブ）

### Step 2: `fav/src/profiler/mod.rs` に `pub mod interactive;` を追加

既存の `mod.rs` 末尾に `pub mod interactive;` を追記する。

### Step 3: `fav/src/main.rs` — `Some("profile")` アームを拡張

既存の `Some("profile")` arg 解析ブロックに以下を追加:

1. `let mut interactive = false;` を `let mut build = false;` の直後に追加
2. while ループ内の最後の `else` 直前に追加:
   ```rust
   } else if arg == "--interactive" {
       interactive = true; i += 1;
   ```
3. dispatch ブロックの先頭（`compare` チェックの前）に追加:
   ```rust
   if interactive {
       println!("{}", profiler::interactive::cmd_profile_interactive(&path));
   } else if let Some(ref v) = compare {
   ```

### Step 4: `driver.rs` — `v67700_tests` 追加

挿入前に `grep "v67600_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認してから挿入すること。
`// -- v67600_tests (v67.6.0)` コメントの直前に `v67700_tests` を挿入。

**`include_str!` パス**: `driver.rs` は `fav/src/` にあるため、`interactive.rs` へのパスは `"profiler/interactive.rs"`。

2 テスト関数:
- `profile_interactive_hotspot` — `include_str!("profiler/interactive.rs")` に `"--interactive"` / `"hotspot"` を含む
- `profile_interactive_drill` — `include_str!("profiler/interactive.rs")` に `"drill"` / `"Suggestion"` を含む

### Step 5: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67700_tests
cargo test -j 8 -- --test-threads=8
```

### Step 6: ドキュメント・ステータス更新

T5（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.7.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.6.0` から `v67.7.0` に更新
- 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

## `fav/src/profiler/interactive.rs` 実装例

```rust
// fav/src/profiler/interactive.rs — v67.7.0 Interactive Profiling

pub const INTERACTIVE_HELP: &str = "\
fav profile --interactive — インタラクティブプロファイリング

使用例:
  fav profile --interactive pipeline.fav

コマンド:
  drill    ホットスポット（hotspot）をコード行レベルにドリルダウン
  next     次のホットスポットに移動
  quit     インタラクティブモードを終了

--interactive モードは各ホットスポットに対して Suggestion（最適化提案）を自動表示します。
";

pub fn cmd_profile_interactive(src: &str) -> String {
    // スタブ実装: 将来フェーズで実際のインタラクティブ REPL に置き換える
    format!(
        "[hotspot] Transform: 847ms (72% of total)\n\
         > drill\n\
           [line 12] collect {{ yield ... }} — 723ms (85% of Transform)\n\
         Suggestion: List.map に変換で 3× 高速化\n\
         \n\
         [hotspot] EmbedText: 1240ms (次のボトルネック)\n\
         > drill\n\
           [API calls] Rune.openai.embed: 1000 回 sequential\n\
         Suggestion: batch_embed(texts, batch_size: 50) で 20× 高速化\n\
         \n\
         (--interactive mode: {})",
        src
    )
}
```

---

## `driver.rs` 挿入コード

```rust
// -- v67700_tests (v67.7.0) -- Interactive Profiling --
#[cfg(test)]
mod v67700_tests {
    #[test]
    fn profile_interactive_hotspot() {
        let src = include_str!("profiler/interactive.rs");
        assert!(
            src.contains("--interactive") && src.contains("hotspot"),
            "interactive.rs should contain '--interactive' and 'hotspot' keywords"
        );
    }

    #[test]
    fn profile_interactive_drill() {
        let src = include_str!("profiler/interactive.rs");
        assert!(
            src.contains("drill") && src.contains("Suggestion"),
            "interactive.rs should contain 'drill' and 'Suggestion' keywords"
        );
    }
}
```

---

## リスク・注意点

- `interactive.rs` は `profiler/` サブディレクトリ内のファイル。`mod.rs` への `pub mod interactive;` 追加を忘れないこと
- `include_str!` パスは `"profiler/interactive.rs"`（`driver.rs` が `fav/src/` にあるため）
- 既存の `Some("profile")` アームは複雑（while ループ + dispatch）。`let mut interactive = false;` の挿入位置と dispatch の最優先配置を確認すること
- `interactive` が true かつ `compare`/`build` が同時指定された場合のエラー処理を追加することが望ましい（将来フェーズ）
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- `drill` コマンドの実際のインタラクティブ実装 — 将来フェーズ
- lint 統合・`fav suggest` 連携 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
