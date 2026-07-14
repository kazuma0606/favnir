# v40.2.0 仕様書 — Session Window

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.2.0 |
| テーマ | Streaming Foundations — `session_window` |
| 前バージョン | v40.1.0（2817 tests） |
| 目標テスト数 | 2820（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.2.0 |

---

## 背景と目的

v40.1.0 で tumbling / sliding window のスタブを追加した。
v40.2.0 では、ユーザーアクティビティのまとまりを捉えるのに適した **session window** を追加する。

session window はアイドル時間（gap）でウィンドウを区切る。たとえば `gap: 30` なら、
30 秒間イベントがなければウィンドウを閉じて新しいウィンドウを開始する。

```favnir
stage SessionAggregate {
  bind sessions <- Stream.session_window(events, gap: 30)
  // 30秒アイドルでウィンドウを閉じる
}
```

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `runes/stream/stream.fav` | `session_window(stream, gap)` 関数スタブ追加 |
| `runes/stream/rune.toml` | version: `40.1.0` → `40.2.0` |
| `fav/Cargo.toml` | version: `40.1.0` → `40.2.0` |
| `CHANGELOG.md` | `[v40.2.0]` エントリ追加（`[v40.1.0]` の直後） |
| `fav/src/driver.rs` | v40100_tests stub 化 + v40200_tests 追加 |

---

## session_window 設計

```favnir
// session_window — gap 秒間イベントがなければウィンドウを閉じる
public fn session_window(stream, gap) {
    // TODO: v40.2.0 stub — session window に分割
    Stream.window(stream, gap, fn(w) { w })
}
```

**注**: v40.2.0 はスタブ実装。実際の gap ベースのウィンドウロジックは
v40.3〜v40.6 の `Event<T>` + timestamp 統合時に具体化する。

---

## テスト設計（v40200_tests）

```rust
#[cfg(test)]
mod v40200_tests {
    #[test]
    fn cargo_toml_version_is_40_2_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.2.0"), "Cargo.toml must contain version 40.2.0");
    }

    #[test]
    fn changelog_has_v40_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.2.0]"), "CHANGELOG.md must contain [v40.2.0]");
    }

    #[test]
    fn stream_rune_has_session_window() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("session_window"), "stream.fav must contain session_window");
    }
}
```

テスト数: 2817 + 3 = **2820**

---

## 完了条件

| # | 条件 |
|---|------|
| 1 | `runes/stream/stream.fav` に `session_window` が存在する |
| 2 | `runes/stream/rune.toml` の version が `40.2.0`（手動確認項目 — テスト化対象外、ロードマップが「3 件」と明示） |
| 3 | `Cargo.toml` の version が `40.2.0` |
| 4 | `CHANGELOG.md` に `[v40.2.0]` エントリが存在する |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2820） |
| 6 | `v40200_tests` 3 件すべて pass |

---

## ロードマップとの差異

特記事項なし。ロードマップ §v40.2.0 の仕様を完全に反映。
