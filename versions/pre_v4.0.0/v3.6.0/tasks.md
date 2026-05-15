# Favnir v3.6.0 Tasks

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"3.6.0"` に更新
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を更新

## Phase 1: 型登録 + namespace

- [x] `checker.rs`: `CheckpointMeta` を `type_defs` に pre-register（name/value/updated_at フィールド）
- [x] `checker.rs`: `!Checkpoint` をエフェクトリストに追加（`Effect::Checkpoint` + 呼び出し箇所チェック）
- [x] `checker.rs`: `Checkpoint` namespace をチェックリストに追加
- [x] `checker.rs`: `Checkpoint.last/save/reset/meta` のシグネチャ登録
- [x] `compiler.rs`: `"Checkpoint"` を2箇所のグローバル登録ループに追加
- [x] `compiler.rs`: `"CheckpointMeta"` を type registration ループに追加

## Phase 2: VM プリミティブ

- [x] `vm.rs`: `CheckpointBackend` enum 定義（File / Sqlite variants）
- [x] `vm.rs`: `CHECKPOINT_BACKEND` thread-local 定義
- [x] `vm.rs`: `Checkpoint.last(name)` 実装（file バックエンド）
- [x] `vm.rs`: `Checkpoint.save(name, value)` 実装（file バックエンド）
- [x] `vm.rs`: `Checkpoint.reset(name)` 実装（file バックエンド）
- [x] `vm.rs`: `Checkpoint.meta(name)` 実装 → `VMValue::Record` (CheckpointMeta)
- [x] `vm.rs`: `DB.upsert_raw(conn, type_name, row, key_field)` 実装
- [x] `vm.rs`: `IO.timestamp()` 実装（chrono UTC ISO 8601）
- [x] `Cargo.toml`: `chrono = { version = "0.4", default-features = false, features = ["clock"] }` 追加
- [x] `vm_stdlib_tests.rs`: 6件の新テスト追加（計画8件 → 実装6件、主要シナリオはすべてカバー）
  - `checkpoint_last_returns_none_initially`
  - `checkpoint_save_and_meta_roundtrip`
  - `checkpoint_reset_clears_saved_value`
  - `io_timestamp_returns_iso_utc_length`
  - `db_upsert_raw_is_idempotent`
  - `db_upsert_raw_updates_existing_row`

## Phase 3: `runes/incremental/incremental.fav`

- [x] `runes/incremental/incremental.fav` 作成
  - [x] `last(name)` 実装
  - [x] `save(name, value)` 実装
  - [x] `reset(name)` 実装
  - [x] `meta(name)` 実装
  - [x] `run_since(name, fetch_fn)` 実装
  - [x] `upsert(conn, type_name, row, key_field)` 実装

## Phase 4: `runes/incremental/incremental.test.fav`

- [x] `runes/incremental/incremental.test.fav` 作成（13 テスト）
  - [x] `checkpoint_last_returns_none_initially`
  - [x] `checkpoint_save_and_last`
  - [x] `checkpoint_reset_clears_value`
  - [x] `checkpoint_meta_name_matches`
  - [x] `checkpoint_meta_value_after_save`
  - [x] `checkpoint_meta_updated_at_nonempty`
  - [x] `incremental_last_wrapper`
  - [x] `incremental_save_wrapper`
  - [x] `incremental_reset_wrapper`
  - [x] `db_upsert_raw_idempotent`
  - [x] `db_upsert_raw_updates_field`
  - [x] `io_timestamp_nonempty`
  - [x] `io_timestamp_format`

## Phase 5: driver.rs 統合テスト

- [x] `driver.rs`: `incremental_rune_test_file_passes` テスト追加
- [x] `driver.rs`: `checkpoint_last_none_in_favnir_source` テスト追加
- [x] `driver.rs`: `checkpoint_save_and_read_in_favnir_source` テスト追加
- [x] `driver.rs`: `db_upsert_raw_in_favnir_source` テスト追加
- [x] `driver.rs`: `incremental_run_since_in_favnir_source` テスト追加
- [x] `driver.rs`: `fav_checkpoint_list_command` テスト追加

## Phase 6: `fav.toml [checkpoint]` + SQLite バックエンド + CLI

- [x] `toml.rs`: `CheckpointConfig` struct 追加
- [x] `toml.rs`: `FavToml` に `checkpoint: Option<CheckpointConfig>` フィールド追加
- [x] `driver.rs`: `checkpoint_backend_from_config` / `load_checkpoint_config_for_file` 実装
- [x] `driver.rs`: `cmd_run` / `cmd_check` 前に `load_checkpoint_config_for_file` 呼び出し
- [x] `vm.rs`: `CheckpointBackend::Sqlite` 実装（`_fav_checkpoints` テーブル自動作成）
- [x] `vm.rs`: Sqlite バックエンドの last/save/reset/meta 実装
- [x] `main.rs`: `fav checkpoint list` サブコマンド実装
- [x] `main.rs`: `fav checkpoint show <name>` サブコマンド実装
- [x] `main.rs`: `fav checkpoint reset <name>` サブコマンド実装
- [x] `main.rs`: `fav checkpoint set <name> <value>` サブコマンド実装
- [x] `driver.rs`: `cmd_checkpoint_{list,show,reset,set}` 関数実装

## Phase 7: examples + docs

- [x] `fav/examples/incremental_demo/src/main.fav` 作成
- [x] `versions/v3.6.0/langspec.md` 作成
- [x] `versions/v3.6.0/migration-guide.md` 作成
- [x] `versions/v3.6.0/progress.md` 全フェーズ完了に更新
- [ ] `memory/MEMORY.md` を v3.6.0 完了状態に更新
