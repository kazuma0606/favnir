# v40.5.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2829（前バージョン 2826 + 3）
**実績テスト数**: 2829 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2826 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.4.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.5.0 を確認
- [x] `v40400_tests::cargo_toml_version_is_40_4_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44373
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40400_tests` の閉じ `}` の行番号を確認し記録: 行44389
- [x] `driver.rs` に `v40500_tests` モジュールが存在しないことを確認（今回新規作成）

---

## T1 — toml.rs に StreamConfig 構造体追加

- [x] `StateConfig`（行135〜138）直後に `StreamConfig` 構造体を追加
  - `watermark_delay: Option<u32>`
  - `late_policy: Option<String>`
  - `#[derive(Debug, Clone, Default)]`
- [x] `FavToml` 構造体の `state: Option<StateConfig>` 直後に `pub stream: Option<StreamConfig>` を追加

---

## T2 — parse_fav_toml に [stream] 解析追加

- [x] アキュムレーター `let mut stream_cfg: Option<StreamConfig> = None;` を `state_cfg` 宣言の直後に追加
- [x] セクション検出 `if trimmed == "[stream]" { section = "stream"; continue; }` を `[state]` 検出の直後に追加
- [x] `"stream" =>` match アームを `"state" =>` アームの直後に追加（`watermark_delay` / `late_policy` を解析）
- [x] `FavToml { ... }` 構築部の `state: state_cfg,` 直後に `stream: stream_cfg,` を追加

---

## T3 — inject_stream_config スタブ追加

- [x] `pub fn inject_stream_config(_cfg: &StreamConfig)` スタブを `parse_fav_toml_pub` の近くに追加

---

## T4 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.4.0"` → `"40.5.0"` に変更

---

## T5 — CHANGELOG.md 更新

- [x] `[v40.5.0]` エントリを `[v40.4.0]` の直後に追加

---

## T6 — driver.rs 更新

- [x] `v40400_tests::cargo_toml_version_is_40_4_0` をスタブ化（`#[test]` アトリビュートは維持したまま本体を空にする）
- [x] `v40500_tests` モジュール（3 テスト）を追加（`crate::toml::parse_fav_toml_pub` 直接参照）
  - `cargo_toml_version_is_40_5_0`（NOTE コメント付き）
  - `changelog_has_v40_5_0`
  - `fav_toml_stream_section_parsed`（`crate::toml::parse_fav_toml_pub` を呼ぶ実装テスト）
- [x] `resolver.rs` / `checker.rs` / `driver.rs` の `FavToml` 初期化部（5 箇所）に `stream: None` を追加（コンパイルエラー修正）

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2829 を確認（実績: 2829）
- [x] `v40500_tests` 3 件すべて pass を確認（特に `fav_toml_stream_section_parsed`）

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.5.0（最新安定版）・v40.6.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.5.0 を完了済みにマーク
  （`roadmap-v40.1-v45.0.md` はマスター概要のため個別バージョン完了マーク不要）
- [x] `versions/v40-v45/v40.5.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**実装時判明事項**:
- `v40500_tests` で `use super::*` を使っても `parse_fav_toml_pub` は `crate::toml` 経由でしか見えない
  → `crate::toml::parse_fav_toml_pub(toml)` に修正（spec.md の `use super::*` 記述は参考例として維持）
- `FavToml` に `stream` フィールド追加により `resolver.rs`（3 箇所）・`checker.rs`（2 箇所）・`driver.rs`（1 箇所）の初期化部も更新が必要

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（実装前レビュー通過）
- [x] code-reviewer 指摘対応済み（実装後確認待ち）
