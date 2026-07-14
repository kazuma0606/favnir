# v40.5.0 実装計画

## 概要

Streaming Foundations スプリント第 5 版。`fav/src/toml.rs` に `StreamConfig` 構造体・`[stream]` セクション解析・`inject_stream_config` スタブを追加する。前バージョンまでの stream.fav スタブ追加とは異なり、**Rust コードの実質的な実装**が必要。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2826 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.4.0` であることを確認
- `v40400_tests::cargo_toml_version_is_40_4_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録

### Step 2 — toml.rs に StreamConfig 追加
`StateConfig`（行132〜138）直後には `// ── Azure config (v14.2.0)` コメントブロックがある。
`StreamConfig` は **`// ── Azure config (v14.2.0)` コメント行の直前**（行139 付近）に挿入する:
- `// ── Stream config (v40.5.0) ──` コメント見出し
- `StreamConfig` 構造体（`watermark_delay: Option<u32>` / `late_policy: Option<String>`）
- `FavToml` 構造体の `state: Option<StateConfig>` フィールド直後に `pub stream: Option<StreamConfig>` 追加

### Step 3 — parse_fav_toml に `[stream]` 解析追加
`parse_fav_toml` 関数内（`StateConfig` の解析コードと同パターン）:
1. `let mut stream_cfg: Option<StreamConfig> = None;` をアキュムレーター宣言に追加
2. セクション検出: `if trimmed == "[stream]" { section = "stream"; continue; }` を `[state]` 検出の直後に追加
3. `"stream" =>` match アームを `"state" =>` アームの直後に追加
4. FavToml 構築の `state: state_cfg,` 直後に `stream: stream_cfg,` を追加

### Step 4 — inject_stream_config スタブ追加
`parse_fav_toml_pub` 関数の近くに `pub fn inject_stream_config` スタブを追加。

### Step 5 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.4.0"` → `"40.5.0"` に変更。

### Step 6 — CHANGELOG.md 更新
`[v40.5.0]` エントリを `[v40.4.0]` の直後に追加。

### Step 7 — driver.rs 更新
1. `v40400_tests::cargo_toml_version_is_40_4_0` をスタブ化
2. `v40500_tests` モジュール（3 テスト）を末尾に追加

### Step 8 — cargo test 実行
`cargo test` で 2829 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（toml.rs — StreamConfig 構造体 + FavToml フィールド）
       └→ Step 3（toml.rs — parse_fav_toml 解析ロジック）
            └→ Step 4（toml.rs — inject_stream_config スタブ）
                 └→ Step 7（driver.rs — fav_toml_stream_section_parsed）
  └→ Step 5（Cargo.toml）
       └→ Step 7（driver.rs — cargo_toml_version_is_40_5_0）
  └→ Step 6（CHANGELOG）
       └→ Step 7（driver.rs — changelog_has_v40_5_0）
            └→ Step 8（cargo test）
```

Step 2〜6 のうち Step 5・6 は Step 2〜4 と並列実施可能。Step 3〜4 は Step 2 完了後。

---

## リスクと注意点

- `parse_fav_toml` 関数の最終 `FavToml { ... }` 構築部に `stream: stream_cfg,` を追加し忘れないこと
- `watermark_delay` は整数値（`u32`）のため `val.trim_matches('"').parse().ok()` で変換する（`"` なし前提）
- `v40400_tests` のスタブ化を忘れると version assertion が失敗する
- `v40500_tests` で `use super::*` が必要（`parse_fav_toml_pub` を直接呼ぶため）
