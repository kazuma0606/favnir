# v67.2.0 Spec — Time-Travel Debugging（記録 & リプレイ）

Version: 67.2.0
Status: 未着手
Base tests: 3499
Target tests: 3501

---

## 概要

パイプライン実行を `.fav-trace` ファイルに記録し、任意のステップに巻き戻す Time-Travel Debugging を実装する。
本番障害の再現に威力を発揮し、「再現性のある調査」を実現する。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.2.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3499 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/debug.rs` が存在することを確認（v67.1.0 で作成済み。本バージョンで拡張する）
- `driver.rs` に `v67100_tests` が存在することを確認（`v67200_tests` の挿入位置）
- `driver.rs` に `v67200_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67100_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `debug_step_execution`, `debug_breakpoint_stage`
- `versions/current.md` の「進行中バージョン」が `v67.1.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/debug.rs` — Time-Travel Debugging キーワードを追加

v67.1.0 で作成済みの `debug.rs` を拡張する。
以下のキーワードを追加する（テストでアサートされる）:
- `"--record"` — 実行トレース記録フラグ（`debug_record_replay` テスト）
- `"--replay"` — トレースファイルからのリプレイフラグ（`debug_record_replay` テスト）
- `"rewind"` — 任意ステップへの巻き戻しコマンド（`debug_rewind_to_step` テスト）
- `"forward"` — 1 ステップ進むコマンド（`debug_rewind_to_step` テスト）
- `".fav-trace"` — トレースファイルの拡張子（`debug_rewind_to_step` テスト）

追加する定数・関数の例:

```rust
pub const TIME_TRAVEL_HELP: &str = "\
Time-Travel Debugging:
  fav run pipeline.fav --record session.fav-trace   実行を .fav-trace に記録
  fav debug --replay session.fav-trace              .fav-trace からリプレイ

リプレイコマンド:
  rewind <step>    指定ステップに巻き戻す
  forward          1 ステップ進む
  inspect <expr>   現在ステップのデータを確認
  quit             リプレイを終了
";

pub fn cmd_debug_replay(trace_path: &str, _args: &[String]) -> String {
    format!(
        "[fav debug --replay] .fav-trace ファイル: {}\n\
         rewind / forward / inspect / quit が利用可能です。",
        trace_path
    )
}
```

### 2. `driver.rs` — `v67200_tests` 追加

挿入位置: `// -- v67100_tests (v67.1.0)` コメントの直前

```rust
// -- v67200_tests (v67.2.0) -- Time-Travel Debugging --
#[cfg(test)]
mod v67200_tests {
    #[test]
    fn debug_record_replay() {
        let src = include_str!("debug.rs");
        assert!(
            src.contains("--record") && src.contains("--replay"),
            "debug.rs should contain '--record' and '--replay' flag descriptions"
        );
    }

    #[test]
    fn debug_rewind_to_step() {
        let src = include_str!("debug.rs");
        assert!(
            src.contains("rewind") && src.contains("forward") && src.contains(".fav-trace"),
            "debug.rs should contain 'rewind', 'forward', and '.fav-trace' keywords"
        );
    }
}
```

---

## 完了条件

- `fav/src/debug.rs` が `"--record"` / `"--replay"` / `"rewind"` / `"forward"` / `".fav-trace"` を含む
- `cargo build` でエラーなし
- `cargo test --bin fav v67200_tests` で 2 件 PASS
  - `debug_record_replay` PASS
  - `debug_rewind_to_step` PASS
- `cargo test -j 8 -- --test-threads=8` で 3501 tests passed, 0 failed

---

## 非スコープ

- 実際のトレースファイル書き込み・読み込み実装 — 将来フェーズ
- `.fav-trace` バイナリフォーマット実装（バイナリ、gzip 圧縮） — 将来フェーズ
- ラージデータのメモリ効率実装（参照のみ記録、コピーしない） — 将来フェーズ
- `--record` / `--replay` フラグの main.rs 登録 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"debug.rs"` → `fav/src/debug.rs`（同じ `fav/src/` ディレクトリ）

### テスト数増加の根拠

`v67200_tests` モジュール内の `#[test]` fn 2 件（`debug_record_replay` / `debug_rewind_to_step`）で +2。
