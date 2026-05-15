# Favnir v3.6.0 Implementation Plan

## Theme: 増分処理（Incremental Processing）

---

## Phase 0: バージョン更新

- `Cargo.toml` version → `"3.6.0"`
- `src/main.rs` ヘルプテキスト・バージョン表示更新
- `versions/v3.6.0/progress.md` 作成

---

## Phase 1: 型登録 + namespace

**checker.rs**
- `CheckpointMeta` を `type_defs` に pre-register（`total/valid...` と同パターン）
  - フィールド: `name: String`, `value: String`, `updated_at: String`
- `!Checkpoint` エフェクトを既存エフェクトリストに追加
- `Checkpoint` namespace を namespace チェックリストに追加
- `Checkpoint.last / save / reset / meta` のシグネチャをチェッカーに登録

**compiler.rs**
- `"Checkpoint"` を2箇所のグローバル登録ループに追加
- `"CheckpointMeta"` を type registration ループに追加

---

## Phase 2: VM プリミティブ

**vm.rs**

スレッドローカルストレージ:
```rust
thread_local! {
    static CHECKPOINT_BACKEND: RefCell<CheckpointBackend> = RefCell::new(CheckpointBackend::File {
        dir: ".fav_checkpoints".into(),
    });
}
```

`CheckpointBackend` enum:
- `File { dir: PathBuf }`
- `Sqlite { path: PathBuf, conn: ... }`

実装する VM プリミティブ:
- `Checkpoint.last(name)` → `Option<String>` (VMValue::Option)
- `Checkpoint.save(name, value)` → `Unit`
- `Checkpoint.reset(name)` → `Unit`
- `Checkpoint.meta(name)` → `VMValue::Record` (CheckpointMeta shape)
- `DB.upsert_raw(conn, type_name, row, key_field)` → `Unit`
- `IO.timestamp()` → `VMValue::Str` (ISO 8601 UTC)

File バックエンド実装:
- `last`: `.fav_checkpoints/<name>.txt` 読み取り → `Some(content)` / `None`
- `save`: ファイル書き込み（ディレクトリ自動作成）
- `reset`: ファイル削除（存在しなければ無視）
- `meta`: `<name>.meta.txt` (JSON 行形式) → CheckpointMeta

---

## Phase 3: `runes/incremental/incremental.fav`

Favnir で書かれた公開 API（VM プリミティブのラッパー）:

```
runes/
  incremental/
    incremental.fav   ← 6 関数
```

実装する関数:
1. `last(name)` → `Checkpoint.last(name)`
2. `save(name, value)` → `Checkpoint.save(name, value)`
3. `reset(name)` → `Checkpoint.reset(name)`
4. `meta(name)` → `Checkpoint.meta(name)`
5. `run_since(name, fetch_fn)` — last 取得 → fetch_fn 呼び出し → IO.timestamp で save
6. `upsert(conn, type_name, row, key_field)` → `DB.upsert_raw(...)`

---

## Phase 4: `runes/incremental/incremental.test.fav`

テスト（13 件目標）:

| # | テスト名 | 内容 |
|---|---------|------|
| 1 | `checkpoint_last_returns_none_initially` | 未設定 → Option.none |
| 2 | `checkpoint_save_and_last` | save → last で取得 |
| 3 | `checkpoint_reset_clears_value` | save → reset → last == none |
| 4 | `checkpoint_meta_name_matches` | meta.name == "test_cp" |
| 5 | `checkpoint_meta_value_after_save` | save → meta.value == saved |
| 6 | `checkpoint_meta_updated_at_nonempty` | save → meta.updated_at != "" |
| 7 | `incremental_last_wrapper` | incremental.last == Checkpoint.last |
| 8 | `incremental_save_wrapper` | incremental.save → last 確認 |
| 9 | `incremental_reset_wrapper` | incremental.reset → last none |
| 10 | `db_upsert_raw_idempotent` | 同 key で2回 upsert → count == 1 |
| 11 | `db_upsert_raw_updates_field` | upsert 後フィールド更新確認 |
| 12 | `io_timestamp_nonempty` | IO.timestamp() != "" |
| 13 | `io_timestamp_format` | String.length(IO.timestamp()) == 20 |

---

## Phase 5: driver.rs 統合テスト

`migrate_tests` モジュールに追加（6 テスト）:

1. `incremental_rune_test_file_passes` → `run_fav_test_file_with_runes("runes/incremental/incremental.test.fav")`
2. `checkpoint_last_none_in_favnir_source` — Favnir ソース直接実行
3. `checkpoint_save_and_read_in_favnir_source` — save → last
4. `db_upsert_raw_in_favnir_source` — upsert idempotency
5. `incremental_run_since_in_favnir_source` — run_since フロー
6. `fav_checkpoint_list_command` — `cmd_checkpoint_list()` の出力確認

---

## Phase 6: `fav.toml [checkpoint]` + SQLite バックエンド

**driver.rs**
- `FavToml` struct に `checkpoint: Option<CheckpointConfig>` 追加
- `CheckpointConfig { backend: String, path: String }` struct
- `load_checkpoint_config(toml_path)` → VM の `CHECKPOINT_BACKEND` を設定
- `cmd_run` / `cmd_check` 実行前に `load_checkpoint_config` 呼び出し

**vm.rs**
- `CheckpointBackend::Sqlite` 実装（rusqlite を使用、既存 DB クレート依存）
- `_fav_checkpoints` テーブル自動作成
- CRUD 実装

**main.rs**
- `fav checkpoint` サブコマンド追加
  - `list` / `show <name>` / `reset <name>` / `set <name> <value>`

---

## Phase 7: examples + docs

- `examples/incremental_demo/main.fav` — ETL パイプライン例
- `versions/v3.6.0/langspec.md`
- `versions/v3.6.0/migration-guide.md`
- `versions/v3.6.0/progress.md` 全フェーズ完了に更新

---

## 依存関係

- `rusqlite`: 既存（v3.3.0 DB rune で追加済み）
- `chrono`: 新規追加（`IO.timestamp()` の UTC フォーマット用）

## テスト目標

v3.5.0: 743 tests → v3.6.0 目標: **~790 tests**
