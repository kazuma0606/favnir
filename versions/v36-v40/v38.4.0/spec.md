# v38.4.0 spec — LSP AI 補完（オプション）

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.4.0 |
| テーマ | LSP AI 補完 — `fav.toml` の `[lsp.ai]` セクション解析 + `enabled` フラグ制御 |
| 前提 | v38.3.0 COMPLETE — `fav generate --from csv` 実装済み |
| 完了条件 | `v38400_tests` 全テスト pass・`cargo test` 0 failures（≥ 2758 件） |

## 背景と目的

LSP AI 補完を `fav.toml` の `[lsp.ai]` セクションで制御可能にする。
`enabled = true` のとき AI 補完が有効（v38.7.0 で Llm Rune 本実装予定）、未設定または `enabled = false` の場合はフォールバック動作とする。
v38.4.0 では**設定解析部分**のみ実装し、実際の LLM rerank はスタブ。

**想定動作**:
```toml
# fav.toml
[lsp.ai]
enabled = true
```

```rust
let cfg = parse_lsp_ai_config(toml_str);
// cfg.enabled == true  → AI 補完有効（v38.7.0 で本実装）
// cfg.enabled == false → 通常 LSP 補完にフォールバック
```

## 実装スコープ

### 1. `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加

既存 `toml.rs` のファイル末尾（最終 `pub fn` の後）に追加:

```rust
// ── v38.4.0 — [lsp.ai] 設定解析 ──────────────────────────────────────────────

pub struct LspAiConfig {
    pub enabled: bool,
}

pub fn parse_lsp_ai_config(toml: &str) -> LspAiConfig {
    LspAiConfig { enabled: parse_lsp_ai_enabled(toml) }
}

fn parse_lsp_ai_enabled(toml: &str) -> bool {
    let mut in_lsp_ai = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[lsp.ai]" {
            in_lsp_ai = true;
        } else if trimmed.starts_with('[') {
            in_lsp_ai = false;
        } else if in_lsp_ai && trimmed == "enabled = true" {
            return true;
        }
    }
    false
}
```

**エクスポート**:
- `pub struct LspAiConfig { pub enabled: bool }` — 設定値コンテナ
- `pub fn parse_lsp_ai_config(toml: &str) -> LspAiConfig` — TOML 文字列から解析

**動作仕様**:
| 入力 | `cfg.enabled` |
|---|---|
| `[lsp.ai]\nenabled = true` | `true` |
| `[lsp.ai]\nenabled = false` | `false` |
| `[lsp.ai]` のみ（`enabled` なし） | `false` |
| `[lsp.ai]` セクション自体なし | `false` |
| `enabled = true # comment`（インラインコメント付き） | `false`（意図的制限・`trim()` 後の完全一致が失敗するため） |

### 2. `driver.rs` — テストモジュール追加

#### `v38300_tests::cargo_toml_version_is_38_3_0` のスタブ化

```rust
// Stubbed: version bumped to 38.4.0 — assertion intentionally removed
```

#### `v38400_tests` モジュール新規追加（4 テスト）

```rust
// ── v38400_tests (v38.4.0) — LSP AI 補完設定解析 ─────────────────────────────
#[cfg(test)]
mod v38400_tests {
    #[test]
    fn cargo_toml_version_is_38_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.4.0"), "Cargo.toml must contain version 38.4.0");
    }

    #[test]
    fn changelog_has_v38_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.4.0]"), "CHANGELOG.md must contain [v38.4.0]");
    }

    #[test]
    fn lsp_ai_enabled_when_configured() {
        let toml = "[lsp.ai]\nenabled = true\n";
        let cfg = crate::toml::parse_lsp_ai_config(toml);
        assert!(cfg.enabled, "lsp.ai.enabled should be true when set in [lsp.ai]");
    }

    #[test]
    fn lsp_ai_disabled_by_default() {
        let toml = "[project]\nname = \"my-pipeline\"\n";
        let cfg = crate::toml::parse_lsp_ai_config(toml);
        assert!(!cfg.enabled, "lsp.ai.enabled should default to false when [lsp.ai] absent");
    }
}
```

### 3. `CHANGELOG.md` — `[v38.4.0]` エントリ追加

```
## [v38.4.0] — 2026-07-10

### Added
- `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加
- `[lsp.ai] enabled = true` で LSP AI 補完を有効化（v38.7.0 で本実装）
- `v38400_tests` 4 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 4. その他ドキュメント更新

- `fav/Cargo.toml`: `38.3.0` → `38.4.0`
- `versions/current.md`: 最新安定版 → v38.4.0、次バージョン → v38.5.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.4.0 を ✅ 完了済みにマーク・テスト件数を 4 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.3.0 | 2754 |
| v38.4.0 追加分 | +4 |
| v38.4.0 期待値 | 2758 |

## 注意事項

### `[lsp.ai]` セクション解析の境界判定

別のセクション（例: `[lsp.formatting]`）に入ったら `in_lsp_ai` を `false` に戻す必要がある。
`trimmed.starts_with('[')` の条件で正しくリセットされる。

### `enabled = true` の厳密一致

`"enabled = true"` は `trim()` 後に完全一致で判定する。
`enabled=true`（スペースなし）は v38.4.0 では非対応（単純実装優先）。

### `gen` 予約語（Rust 2024）

変数名には `in_lsp_ai`・`trimmed` 等を使用する — `gen` は使わないこと。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `toml.rs` に `pub fn parse_lsp_ai_config` が含まれる | `lsp_ai_enabled_when_configured` テスト |
| 2 | `[lsp.ai] enabled = true` で `cfg.enabled == true` | `lsp_ai_enabled_when_configured` テスト |
| 3 | `[lsp.ai]` 非設定時に `cfg.enabled == false` | `lsp_ai_disabled_by_default` テスト |
| 4 | `CHANGELOG.md` に `[v38.4.0]` が含まれる | `changelog_has_v38_4_0` テスト |
| 5 | `Cargo.toml` バージョンが `38.4.0` | `cargo_toml_version_is_38_4_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2758） | `cargo test` 実行結果 |
| 7 | `roadmap-v38.1-v39.0.md` の v38.4.0 が ✅ かつテスト件数が 4 件 | T9 後に目視確認 |
| 8 | `versions/current.md` が v38.4.0（最新安定版）・v38.5.0（次バージョン）に更新されている | T7 完了後に目視確認 |
