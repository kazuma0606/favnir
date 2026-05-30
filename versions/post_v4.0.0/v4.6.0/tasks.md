# Favnir v4.6.0 タスクリスト — Log Rune

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新 ✅

- [x] `fav/Cargo.toml` の version を `"4.6.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.6.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）✅

### 1-A: `LogConfig` thread_local + `set_log_config`

- [x] `LogConfig` 構造体を定義（`level`, `format`, `output`, `service` フィールド）
- [x] `Default` 実装（`level = "info"`, `format = "text"`, `output = "stdout"`, `service = ""`）
- [x] `thread_local! { static LOG_CONFIG, static LOG_CODES }` を追加
- [x] `pub fn set_log_config(cfg: LogConfig)` / `pub fn set_log_codes(...)` を追加

### 1-B〜H: ヘルパーとプリミティブ ✅

- [x] `log_level_passes`, `log_timestamp_text`, `log_timestamp_iso`, `log_timestamp_millis`
- [x] `log_ctx_to_text`, `log_format_text`, `log_format_json`, `log_metric_emf`, `log_auto_emit`
- [x] `"Log.emit_raw"`, `"Log.metric_raw"`, `"Log.map_to_json_raw"` アーム追加

---

## Phase 2: `fav.toml` 拡張（`fav/src/toml.rs`）✅

- [x] `LogConfig` 構造体追加（`level`, `format`, `output`, `service` + `Default`）
- [x] `FavToml` に `pub log: Option<LogConfig>` 追加
- [x] `parse_fav_toml` に `[log]` セクションパース追加
- [x] `FavToml` リテラル初期化箇所に `log: None` 追加

---

## Phase 3: checker.rs への変更 ✅

- [x] `("Log", "emit_raw")` → `Unit`
- [x] `("Log", "metric_raw")` → `Unit`
- [x] `("Log", "map_to_json_raw")` → `String`
- [x] `("Log", _)` フォールバック → `Unit`

---

## Phase 4: compiler.rs への変更 ✅

- [x] namespace リストに `"Log"` 追加（2箇所）

---

## Phase 5: `logs/*.yaml` ロード ✅

- [x] `fn load_log_codes(root: &Path)` 追加
- [x] `thread_local! { static LOG_CODES }` / `set_log_codes` 追加（vm.rs）
- [x] `cmd_run` で呼び出し

---

## Phase 6: `driver.rs` 設定反映 ✅

- [x] `cmd_run` に `set_log_config` 呼び出し追加

---

## Phase 7: VM プリミティブのエラー自動ログ

- [ ] Phase 7 は将来バージョンで実装（テスト副作用のリスクあり）

---

## Phase 8: rune ファイル作成（`runes/log/`）✅

- [x] `runes/log/codes.fav` — 12 件のコード定数
- [x] `runes/log/emitter.fav`（元 `emit.fav`; `emit` はキーワードのためリネーム）
- [x] `runes/log/metric.fav`
- [x] `runes/log/log.fav`（barrel: `use emitter.*`, `use metric.*`, `use codes.*`）
- [x] `runes/log/log.test.fav` — 14 件のテスト

> **注意**: `emit` は Favnir のキーワード（`TokenKind::Emit`）。モジュール名として使えないため `emitter.fav` にリネーム。

---

## Phase 9: テスト追加 ✅

### 9-A: `fav/src/backend/vm_stdlib_tests.rs`（8 件）✅

- [x] `log_emit_text_format_runs`
- [x] `log_emit_json_format_runs`
- [x] `log_emit_level_filter_suppresses`
- [x] `log_emit_level_filter_passes`
- [x] `log_metric_text_runs`
- [x] `log_metric_json_runs`
- [x] `log_emit_ctx_json_runs`
- [x] `log_map_to_json_raw_returns_json`

### 9-B: `fav/src/driver.rs` 統合テスト（5 件）✅

- [x] `log_info_in_favnir_source`
- [x] `log_error_ctx_in_favnir_source`
- [x] `log_metric_in_favnir_source`
- [x] `log_metric_with_unit_in_favnir_source`
- [x] `log_codes_in_favnir_source`

---

## Phase 10: examples 追加 ✅

- [x] `examples/log_demo/fav.toml`
- [x] `examples/log_demo/src/main.fav`

---

## 完了条件 ✅

- [x] `cargo build` が通る
- [x] 既存 848 件が全て pass
- [x] 新規テスト 13 件以上が pass（Rust 8 件 + 統合 5 件）
- [x] Favnir rune テスト 14 件が log.test.fav でカバー
- [x] 861 件全て pass（2026-05-17 確認）

---

## 実装メモ（次バージョンへの引き継ぎ）

- **`emit` はキーワード** — `TokenKind::Emit` のため rune モジュール名に使えない。`emitter.fav` を使用。
- **`!Log` エフェクトは存在しない** — ログ関数は `!Io` を宣言。`Effect` enum に変更なし。
- **`Log.*` は effect check なし** — `check_builtin_apply` で `("Log", _)` は `require_auth_effect` 等を呼ばない。
- **`Log.map_to_json_raw`** — `Map<String, String>` を JSON 文字列に変換する VM primitive。emitter.fav の `*_ctx` 関数から呼ぶ。
- **自動エラーログ（Phase 7）** — 将来バージョンで実装。テストへの副作用注意。
- **`main() -> Unit !Io`** — ログ呼び出しを含む `main` は `Bool` ではなく `Unit !Io` を返す必要あり（ブロック内に複数式を置けないため）。
