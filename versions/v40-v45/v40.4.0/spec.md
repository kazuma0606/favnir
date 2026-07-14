# v40.4.0 仕様書 — Out-of-order イベント処理

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.4.0 |
| テーマ | Streaming Foundations — Out-of-order イベント処理（`with_late_policy`） |
| 前バージョン | v40.3.0（2823 tests） |
| 目標テスト数 | 2826（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.4.0 |

---

## 背景と目的

v40.3.0 で `Event` 型に `timestamp` フィールドを追加した。
リアルタイムストリームでは、ネットワーク遅延等により **イベントが順序通りに到着しない**
（out-of-order）ことがある。v40.4.0 では遅延イベントへの対処ポリシーを宣言的に記述できる
`with_late_policy` 関数をスタブとして追加する。

```favnir
stage FilterLate {
  bind valid <- Stream.with_late_policy(events,
    tolerance: 5,
    policy: "drop")
  // 5秒超の遅延イベントを drop する
}
```

`tolerance`（秒）を超えて遅延したイベントを `policy`（`"drop"` または `"reprocess"`）で処理する。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `runes/stream/stream.fav` | `with_late_policy(stream, tolerance, policy)` スタブ追加 + ヘッダー更新 |
| `runes/stream/rune.toml` | version: `40.3.0` → `40.4.0`、description に `with_late_policy` 追記 |
| `fav/Cargo.toml` | version: `40.3.0` → `40.4.0` |
| `CHANGELOG.md` | `[v40.4.0]` エントリ追加（`[v40.3.0]` の直後） |
| `fav/src/driver.rs` | v40300_tests stub 化 + v40400_tests 追加 |

---

## `with_late_policy` 設計

```favnir
// v40.4.0 — Out-of-order イベント処理（スタブ実装）
// tolerance: 許容遅延秒数
// policy: "drop" | "reprocess"
// TODO: v40.4.0 stub — 実際の遅延検出ロジックは v40.5〜v40.6 の Watermark 基盤で実装
public fn with_late_policy(stream, tolerance, policy) {
    Stream.filter(stream, fn(e) { true })
}
```

**注**: v40.4.0 はスタブ実装（全イベントをそのまま通過）。実際の遅延検出には
`Event.timestamp` と Watermark の比較が必要であり、v40.5〜v40.6 で実装する。

---

## テスト設計（v40400_tests）

```rust
#[cfg(test)]
mod v40400_tests {
    #[test]
    fn cargo_toml_version_is_40_4_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.4.0"), "Cargo.toml must contain version 40.4.0");
    }

    #[test]
    fn changelog_has_v40_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.4.0]"), "CHANGELOG.md must contain [v40.4.0]");
    }

    #[test]
    fn stream_fav_has_late_policy() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("with_late_policy"), "stream.fav must contain with_late_policy");
    }
}
```

テスト数: 2823 + 3 = **2826**

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 |
|---|------|
| 1 | `runes/stream/stream.fav` に `with_late_policy` 関数が存在する |
| 2 | `Cargo.toml` の version が `40.4.0` |
| 3 | `CHANGELOG.md` に `[v40.4.0]` エントリが存在する |
| 4 | `cargo test` 全通過（failures=0、テスト数 ≥ 2826） |
| 5 | `v40400_tests` 3 件すべて pass |

**手動確認（テスト化対象外 — ロードマップが「3 件」と明示）:**

| M-1 | `runes/stream/rune.toml` の version が `40.4.0`、description に `with_late_policy` が反映されている |
|-----|-----|

**用語注記:** ロードマップ §v40.4.0 本文の `late_tolerance` は説明上の名称。関数引数名は `tolerance` で確定。

---

## ロードマップとの差異

特記事項なし。ロードマップ §v40.4.0 の仕様を完全に反映。
