# v40.5.0 仕様書 — `fav.toml [stream]` セクション

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.5.0 |
| テーマ | Streaming Foundations — `fav.toml [stream]` セクション解析 |
| 前バージョン | v40.4.0（2826 tests） |
| 目標テスト数 | 2829（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.5.0 |

---

## 背景と目的

v40.1〜v40.4 でウィンドウ関数・Event 型・遅延ポリシーのスタブを追加した。
これらの設定値（`watermark_delay`・`late_policy`）をコード中にハードコードするのではなく、
`fav.toml` の `[stream]` セクションで一元管理できるようにする。

```toml
[stream]
watermark_delay = 5     # Watermark の遅延許容（秒）
late_policy = "drop"    # drop | reprocess
```

`fav.toml` パーサー（`toml.rs`）に `StreamConfig` 構造体と `[stream]` セクション解析を追加し、
`inject_stream_config` スタブ関数でパイプラインへの伝播口を用意する。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `fav/src/toml.rs` | `StreamConfig` 構造体 + `FavToml` への `stream` フィールド追加 + `[stream]` セクション解析 + `inject_stream_config` スタブ |
| `fav/Cargo.toml` | version: `40.4.0` → `40.5.0` |
| `CHANGELOG.md` | `[v40.5.0]` エントリ追加（`[v40.4.0]` の直後） |
| `fav/src/driver.rs` | v40400_tests stub 化 + v40500_tests 追加 |

---

## `StreamConfig` 設計

既存の `StateConfig`（v22.3.0）と同一パターンで追加する。

```rust
// ── Stream config (v40.5.0) ───────────────────────────────────────────────────

/// `[stream]` section of fav.toml (v40.5.0).
#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    /// Watermark 遅延許容秒数（秒）。デフォルト 0。
    pub watermark_delay: Option<u32>,
    /// 遅延イベントのポリシー: "drop" | "reprocess"。
    pub late_policy: Option<String>,
}
```

`FavToml` に `pub stream: Option<StreamConfig>` フィールドを追加する（`state` フィールドの直後）。

---

## `inject_stream_config` 設計

```rust
/// パイプライン実行コンテキストに `[stream]` 設定を注入するスタブ。
/// 実際の伝播ロジックは v40.6〜v40.9 で実装。
pub fn inject_stream_config(_cfg: &StreamConfig) {
    // TODO: v40.5.0 stub
}
```

---

## `[stream]` セクション解析設計

```rust
// parse_fav_toml 内の追加コード（StateConfig と同パターン）

// アキュムレーター
let mut stream_cfg: Option<StreamConfig> = None;

// セクション検出
if trimmed == "[stream]" { section = "stream"; continue; }

// キー解析
"stream" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        let mut current = stream_cfg.take().unwrap_or_default();
        match key {
            "watermark_delay" => {
                // parse_kv は既に trim_matches('"') 済みのため val.parse().ok() でも同じだが
                // 冗長な trim は無害のため慣例に合わせて記述
                current.watermark_delay = val.trim_matches('"').parse().ok();
            }
            "late_policy" => {
                current.late_policy = Some(val.trim_matches('"').to_string());
            }
            _ => {}
        }
        stream_cfg = Some(current);
    }
}

// FavToml 構築（state フィールドの直後）
stream: stream_cfg,
```

---

## テスト設計（v40500_tests）

```rust
#[cfg(test)]
mod v40500_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_40_5_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.5.0"), "Cargo.toml must contain version 40.5.0");
    }

    #[test]
    fn changelog_has_v40_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.5.0]"), "CHANGELOG.md must contain [v40.5.0]");
    }

    #[test]
    fn fav_toml_stream_section_parsed() {
        let toml = "[package]\nname = \"test\"\nversion = \"1.0\"\n\n[stream]\nwatermark_delay = 5\nlate_policy = \"drop\"\n";
        let parsed = parse_fav_toml_pub(toml);
        let stream = parsed.stream.expect("stream config should be parsed");
        assert_eq!(stream.watermark_delay, Some(5));
        assert_eq!(stream.late_policy.as_deref(), Some("drop"));
    }
}
```

**注**: `fav_toml_stream_section_parsed` は `parse_fav_toml_pub` を直接呼ぶため、
`use super::*` が必要（`changelog_has_v40_5_0` でも `include_str!` のみ使用だが、
`fav_toml_stream_section_parsed` のために `use super::*` を付ける）。

テスト数: 2826 + 3 = **2829**

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 |
|---|------|
| 1 | `toml.rs` に `StreamConfig` 構造体が存在し、`FavToml.stream` フィールドが追加されている |
| 2 | `[stream]` セクションが正しく解析される（`fav_toml_stream_section_parsed` テスト通過） |
| 3 | `Cargo.toml` の version が `40.5.0` |
| 4 | `CHANGELOG.md` に `[v40.5.0]` エントリが存在する |
| 5 | `inject_stream_config` 関数が `toml.rs` に存在する（コンパイル通過で確認） |
| 6 | `cargo test` 全通過（failures=0、テスト数 ≥ 2829） |
| 7 | `v40500_tests` 3 件すべて pass |

---

## ロードマップとの差異

特記事項なし。ロードマップ §v40.5.0 の仕様を完全に反映。
`inject_stream_config` はスタブとして追加し、実際の伝播は v40.6 以降で実装する。
