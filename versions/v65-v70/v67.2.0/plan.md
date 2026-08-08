# v67.2.0 実装計画 — Time-Travel Debugging（記録 & リプレイ）

Version: 67.2.0
Status: 未着手
Base tests: 3499
Target tests: 3501

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/debug.rs` に Time-Travel Debugging を追加

v67.1.0 で作成済みの `debug.rs` を編集し、以下のキーワードを追加する:

- `"--record"` と `"--replay"` — `debug_record_replay` テストがアサート
- `"rewind"` / `"forward"` / `".fav-trace"` — `debug_rewind_to_step` テストがアサート

`TIME_TRAVEL_HELP` 定数と `cmd_debug_replay` 関数を追加する。
既存の `cmd_debug` / `DEBUG_HELP` は変更しない。

### Step 2: `driver.rs` テスト追加

`// -- v67100_tests (v67.1.0)` コメントの直前に `v67200_tests` を挿入。

2 テスト関数:
- `debug_record_replay` — `include_str!("debug.rs")` に `"--record"` と `"--replay"` を含む
- `debug_rewind_to_step` — `include_str!("debug.rs")` に `"rewind"` / `"forward"` / `".fav-trace"` を含む

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67200_tests
cargo test -j 8 -- --test-threads=8
```

### Step 4: ドキュメント・ステータス更新

T3（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.2.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.1.0` から `v67.2.0` に更新
- 本 `tasks.md` を COMPLETE に更新

---

## `driver.rs` 挿入コード

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

## リスク・注意点

- `debug.rs` は編集（上書きではなく追記）。既存の `cmd_debug` / `DEBUG_HELP` を削除・変更しないこと（v67100_tests が依存）
- `"--record"` の `-` はハイフン。文字列リテラル中に含まれれば OK（`include_str!` テキスト検索）
- `".fav-trace"` はドットを含む拡張子。`TIME_TRAVEL_HELP` 定数に `session.fav-trace` と書けばドットが自然に含まれ、アサートが通る。`fav-trace` のみでドットを省くと `debug_rewind_to_step` が FAIL するため注意
- `cmd_debug_replay` は `pub` 関数だが main.rs から未接続のため、Rust の `dead_code` 警告が出る可能性がある。`#[allow(dead_code)]` を付けるか、`pub` を外して `pub(crate)` にすることを検討すること
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- 実際のトレースファイル書き込み・読み込み実装 — 将来フェーズ
- `--record` / `--replay` フラグの main.rs 登録 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
