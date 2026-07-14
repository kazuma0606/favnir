# v40.1.0 仕様書 — Tumbling / Sliding Window

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.1.0 |
| テーマ | Streaming Foundations — Window 基盤（tumbling / sliding） |
| 前バージョン | v40.0.0（2814 tests） |
| 目標テスト数 | 2817（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.1.0 |

---

## 背景と目的

v40.0.0「Enterprise Governance」でポリシー管理・RBAC 等のガバナンス機能を整備した。
次スプリント v40.1〜v41.0「Streaming Foundations」では、リアルタイムデータパイプラインの
基盤となるウィンドウ演算子を段階的に実装する。

v40.1.0 は最初のステップとして、最も基本的なウィンドウ型である **tumbling window** と
**sliding window** の Rune スタブを `runes/stream/` に追加する。
これにより、後続バージョン（v40.2〜v40.9）のウィンドウ実装の土台を確立する。

---

## 実装スコープ

### 新規ファイル

| ファイル | 内容 |
|----------|------|
| `runes/stream/stream.fav` | Stream Rune — `tumbling_window` / `sliding_window` 関数スタブ |
| `runes/stream/rune.toml` | Rune メタデータ（name=stream, version=0.1.0） |

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `fav/Cargo.toml` | version: `40.0.0` → `40.1.0` |
| `CHANGELOG.md` | `[v40.1.0]` エントリ追加（`[v40.0.0]` の直後） |
| `fav/src/driver.rs` | v40000_tests stub 化 + v40100_tests 追加 |

---

## stream.fav 設計

```favnir
// runes/stream/stream.fav
// Stream Rune — ウィンドウ演算基盤

fn tumbling_window(stream: List<A>, size: Int) -> List<List<A>> {
  // TODO: v40.1.0 stub — tumbling window に分割
  List.chunk(stream, size)
}

fn sliding_window(stream: List<A>, size: Int, step: Int) -> List<List<A>> {
  // TODO: v40.1.0 stub — sliding window に分割
  List.sliding(stream, size, step)
}
```

**注**: v40.1.0 はスタブ実装。実際のストリーミング VM 統合は v40.3〜v40.6 で行う。

**引数名について**: ロードマップでは `Stream.tumbling_window(stream, seconds)` と `seconds`（時間単位）が示されているが、
v40.1.0 はスタブのため汎用名 `size: Int` を採用する。v40.3 の `Event<T>` + timestamp 統合時に
`seconds` 相当のセマンティクスを付与する。

**ジェネリクス `A` について**: もし `A` がパーサーエラーになる場合は `List<Int>` に型を固定し、
コメントで `// TODO: ジェネリクス化は v40.3 で行う` と明記する。

**rune.toml フォーマット**: 実装時に既存 Rune（例: `runes/kafka/rune.toml`）のフォーマットを確認し、
`author` / `license` フィールドを他 Rune と統一すること。

---

## テスト設計（v40100_tests）

```rust
#[cfg(test)]
mod v40100_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_40_1_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.1.0"), "Cargo.toml must contain version 40.1.0");
    }

    #[test]
    fn changelog_has_v40_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.1.0]"), "CHANGELOG.md must contain [v40.1.0]");
    }

    #[test]
    fn stream_rune_has_window_functions() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("tumbling_window"), "stream.fav must contain tumbling_window");
        assert!(src.contains("sliding_window"), "stream.fav must contain sliding_window");
    }
}
```

テスト数: 2814 + 3 = **2817**

---

## 完了条件

| # | 条件 |
|---|------|
| 1 | `runes/stream/stream.fav` に `tumbling_window` / `sliding_window` が存在する |
| 2 | `runes/stream/rune.toml` が存在する |
| 3 | `Cargo.toml` の version が `40.1.0` |
| 4 | `CHANGELOG.md` に `[v40.1.0]` エントリが存在する |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2817） |
| 6 | `v40100_tests` 3 件すべて pass |

---

## ロードマップとの差異

特記事項なし。ロードマップ §v40.1.0 の仕様を完全に反映。
