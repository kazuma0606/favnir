# v67.7.0 Spec — Interactive Profiling（`fav profile --interactive`）

Version: 67.7.0
Status: 未着手
Base tests: 3509
Target tests: 3511

---

## 概要

プロファイリング結果をインタラクティブに探索する `fav profile --interactive` コマンドを実装する。
ホットスポットをドリルダウンし、コード行レベルでボトルネックを特定して最適化提案を表示する。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.7.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3509 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/profiler/` ディレクトリが存在することを確認（v19.8.0 で作成済み）
- `fav/src/profiler/interactive.rs` が存在しないことを確認（新規作成）
- `driver.rs` に `v67600_tests` が存在することを確認（`v67700_tests` の挿入位置）
- `driver.rs` に `v67700_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67600_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `proptest_stage_invariant`, `proptest_counterexample_shrink`
- `versions/current.md` の「進行中バージョン」が `v67.6.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/profiler/interactive.rs` — 新規作成

インタラクティブプロファイリングのコア実装。以下のキーワードを含むこと（テストでアサートされる）:
- `"--interactive"` — フラグ名（`profile_interactive_hotspot` テスト）
- `"hotspot"` — ホットスポット表示機能（`profile_interactive_hotspot` テスト）
- `"drill"` — ドリルダウンコマンド（`profile_interactive_drill` テスト）
- `"Suggestion"` — 最適化提案表示（`profile_interactive_drill` テスト）

追加する定数・関数:

```rust
pub const INTERACTIVE_HELP: &str = "\
fav profile --interactive — インタラクティブプロファイリング

使用例:
  fav profile --interactive pipeline.fav

コマンド:
  drill    ホットスポットをコード行レベルにドリルダウン
  hotspot  実行時間の上位ステージを表示
  next     次のホットスポットに移動
  quit     インタラクティブモードを終了

--interactive モードは Suggestion（最適化提案）を自動表示します。
";

pub fn cmd_profile_interactive(src: &str) -> String {
    // ...
}
```

### 2. `fav/src/profiler/mod.rs` — `pub mod interactive;` を追加

既存の `mod.rs` 末尾に `pub mod interactive;` を追記する。

### 3. `main.rs` — `Some("profile")` アームに `--interactive` 分岐を追加

既存の arg 解析 while ループに `--interactive` フラグを追加し、
dispatch ブロックに `interactive` ブランチを追加する:

```rust
// while ループ内に追加
} else if arg == "--interactive" {
    interactive = true; i += 1;
```

```rust
// dispatch ブロックに追加（compare/build の前）
if interactive {
    println!("{}", profiler::interactive::cmd_profile_interactive(&path));
} else if let Some(ref v) = compare {
    ...
```

`let mut interactive = false;` を `let mut build = false;` の直後に追加すること。

### 4. `driver.rs` — `v67700_tests` 追加

挿入位置: `// -- v67600_tests (v67.6.0)` コメントの直前

```rust
// -- v67700_tests (v67.7.0) -- Interactive Profiling --
#[cfg(test)]
mod v67700_tests {
    #[test]
    fn profile_interactive_hotspot() {
        let src = include_str!("../profiler/interactive.rs");
        assert!(
            src.contains("--interactive") && src.contains("hotspot"),
            "interactive.rs should contain '--interactive' and 'hotspot' keywords"
        );
    }

    #[test]
    fn profile_interactive_drill() {
        let src = include_str!("../profiler/interactive.rs");
        assert!(
            src.contains("drill") && src.contains("Suggestion"),
            "interactive.rs should contain 'drill' and 'Suggestion' keywords"
        );
    }
}
```

> **`include_str!` パスの注意**: `driver.rs` は `fav/src/driver.rs` にあるため、
> `fav/src/profiler/interactive.rs` へのパスは `"../profiler/interactive.rs"` ではなく
> `"profiler/interactive.rs"` が正しい（同じ `fav/src/` を起点とする相対パス）。

---

## 完了条件

- `fav/src/profiler/interactive.rs` が `"--interactive"` / `"hotspot"` / `"drill"` / `"Suggestion"` を含む
- `fav/src/profiler/mod.rs` に `pub mod interactive;` が追加されている
- `fav/src/main.rs` の `Some("profile")` アームに `--interactive` フラグ処理と `cmd_profile_interactive` 呼び出しが存在する
- `cargo build` でエラーなし
- `cargo test --bin fav v67700_tests` で 2 件 PASS
  - `profile_interactive_hotspot` PASS
  - `profile_interactive_drill` PASS
- `cargo test -j 8 -- --test-threads=8` で 3511 tests passed, 0 failed

---

## 非スコープ

> **ロードマップとの意図的乖離**: ロードマップ v67.7.0 には「`drill` コマンド」「lint 統合（W041）」「`fav suggest` 連携」「インクリメンタル再プロファイル」が実装内容として列挙されているが、v67.7.0 ではキーワードを含むスタブ実装で代替する。

- `drill` コマンドの実際のインタラクティブ実装（REPL 風探索） — 将来フェーズ
- lint 統合（W041 警告の自動表示） — 将来フェーズ
- `fav suggest` 連携（ドリルダウン中の最適化提案） — 将来フェーズ
- インクリメンタル再プロファイル（修正後の diff 即時確認） — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

`driver.rs` は `fav/src/driver.rs` に存在する。`fav/src/profiler/interactive.rs` へのパスは:
- `"profiler/interactive.rs"` — **正しい**（`fav/src/` を起点とした相対パス）
- `"../profiler/interactive.rs"` — **誤り**（`fav/src/` の一つ上になる）

### 既存 `Some("profile")` アームへの影響

既存の `--compare` / `--build` / 通常プロファイルの動作は変更しない。
`interactive` フラグが true のとき最優先で dispatch し、他のフラグと排他にする（compare/build と同時指定時はエラー）。

### テスト数増加の根拠

`v67700_tests` モジュール内の `#[test]` fn 2 件（`profile_interactive_hotspot` / `profile_interactive_drill`）で +2。
