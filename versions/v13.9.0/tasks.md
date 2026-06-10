# v13.9.0 Tasks — 型状態パターン統合 + lineage 更新

Date: 2026-06-11
Branch: feat/v13-capability-context
Completed: 2026-06-11

---

## Phase A — E0024 エラーカタログ追加

- [x] A-1: `fav/src/error_catalog.rs` — E0024 エントリを追加（E0023 の直後）
- [x] A-2: `fav/src/driver.rs` `get_help_text` に E0024 ヘルプテキストを追加

---

## Phase B — lint.rs: E0024 型状態チェック実装

- [x] B-1: `collect_type_state_edges(program)` を実装
  - `fn f(d: A) -> Result<B, _>` の形から `(A, B)` エッジを収集
  - 同一ファイル内に `type A(...)` 宣言があるもののみ対象
- [x] B-2: `check_type_state_errors(program)` を実装（E0024 を返す）
  - `expected_map: "B" → "A"` を構築
  - 全 FnDef ボディをスキャンして型状態違反を検出
- [x] B-3: `collect_type_state_in_block` / `collect_type_state_in_expr` ヘルパーを実装
  - `Expr::Apply(callee, args)` で callee が型状態シーケンスの関数名 → 引数型を検査
  - ローカル変数の型を束縛追跡（`bind x <- f(...)` で x の型を記録）
- [x] B-4: `--legacy` モードでは E0024 を出さない（`cmd_check` 側で制御）

---

## Phase C — driver.rs: cmd_check への統合

- [x] C-1: `cmd_check` の E0023 ブロック後に E0024 チェックブロックを追加
  - `if !legacy_check && !json { ... }` 条件で呼び出す
  - `parsed_prog` を E0023 と共有（二重パース回避）
- [x] C-2: legacy モード（`--legacy`）では E0024 は実行しない
- [x] C-3: JSON 出力モードでは E0024 チェックをスキップ

---

## Phase D — lineage.rs 更新

- [x] D-1: `LineageEntry` 構造体に `capability: Option<String>` フィールドを追加（`kind` は新分類に変更）
- [x] D-2: `classify_capability_kind(params, effects)` ヘルパーを実装
  - `DbWrite` / `WriteCtx` / `MigrateCtx` → `("write", "DbWrite")`
  - `StorageWrite` → `("sink", "StorageWrite")`
  - `DbRead` / `LoadCtx` → `("read", "DbRead")`
  - `AppCtx` → `("read", "DbRead")`（保守的分類）
  - `Io` / `CommonCtx` / `File` → `("io", "Io")`
  - `Http` / `Llm` / `Rpc` / `Network` → `("io", "HttpClient")`
  - それ以外 → `("transform", null)`
- [x] D-3: `LineageEntry` 構築箇所（TrfDef・FnDef 2 か所）に `classify_capability_kind` を適用
- [x] D-4: テキスト出力を新形式に更新（`name [kind] capability`）
- [x] D-5: 既存テスト 4 件を新形式に合わせて更新
  - `snowflake_lineage_shows_effect` → `kind == "read"` チェックに変更
  - `postgres_lineage_shows_effect` → 同上
  - `lineage_http_effect_in_sources` → `kind == "io"` チェックに変更
  - `lineage_llm_effect_in_sources` → エントリ存在確認に変更

---

## Phase E — fav doc --builtins --format json 更新

- [x] E-1: `BuiltinPrimitive` 構造体に `capability: Option<&'static str>` と `impls: Vec<&'static str>` フィールドを追加
  - 既存エントリはマクロで `capability: None, impls: vec![]` にデフォルト設定
- [x] E-2: `DbRead` / `DbWrite` / `StorageWrite` / `StorageRead` / `Io` の capability interface エントリを追加
- [x] E-3: `builtin_primitives_for_test()` テスト用公開ヘルパーを追加

---

## Phase F — テスト追加

- [x] F-1: `fav/src/driver.rs` に `v139000_tests` モジュールを追加
- [x] F-2: 以下のテストを実装:
  - [x] `version_is_13_9_0` — `CARGO_PKG_VERSION == "13.9.0"`
  - [x] `e0024_type_state_skip_phase` — `Loaded` を `transform(d: Validated)` に渡す → E0024
  - [x] `e0024_correct_sequence_no_error` — `Loaded → Validated → Transformed` の正しい順序 → E0024 なし
  - [x] `e0024_pure_fn_not_affected` — Int 引数の純粋関数 → E0024 なし
  - [x] `e0024_legacy_mode_no_error` — フェーズ違反なしのケース → E0024 なし
  - [x] `lineage_db_read_node` — `fn load(ctx: LoadCtx, ...)` → `kind: "read"`, `capability: "DbRead"`
  - [x] `lineage_pure_transform_node` — capability なし stage → read/write でないことを確認
  - [x] `lineage_storage_write_sink` — `fn save(ctx: WriteCtx, ...)` → `kind: "write"`
  - [x] `doc_builtins_capability_field` — DbRead capability エントリの存在確認
- [x] F-3: `cargo test v139000` で全件パス確認（9/9）

---

## Phase G — バージョンバンプ + コミット

- [x] G-1: `fav/Cargo.toml` → `version = "13.9.0"`
- [x] G-2: `cargo test` 全件パス確認（1493 passed, 0 failed）
- [ ] G-3: `git commit -m "feat: v13.9.0 — 型状態パターン統合 + lineage 更新 (E0024)"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| E0024 が error_catalog に追加された | ✓ |
| `check_type_state_errors` が E0024 を返す | ✓ |
| 標準 `fav check` が E0024 を出力（非 legacy） | ✓ |
| `--legacy` では E0024 は発生しない | ✓ |
| `lineage.rs` が `kind` + `capability` フィールドを出力する | ✓ |
| `fav doc --builtins --format json` に `capability` フィールドあり | ✓ |
| `cargo test v139000` 全件パス（9/9） | ✓ |
| `cargo test` 全件パス（1493 passed） | ✓ |
| `CARGO_PKG_VERSION == "13.9.0"` | ✓ |

---

## 実装ノート

- **`collect_type_state_edges`**: `type X(...)` 宣言を同一ファイルから収集し、`fn f(d: A) -> Result<B, _>` の形で A・B 両方が `type_state_names` に含まれる場合のみエッジとして記録。
- **型追跡**: `bind x <- f(args)` で x の型を fn_output から記録。`chain x <- f(args)` も同様。params も seeding する。
- **lineage `kind` フィールド変更**: v13.9.0 で "stage"/"fn" から "read"/"write"/"transform"/"sink"/"io" に変更。既存テスト 4 件を更新。`capability` フィールドは新規追加。
- **`BuiltinPrimitive` 拡張**: マクロ経由の既存エントリは `capability: None, impls: vec![]` でデフォルト設定。capability interface 5 種（DbRead/DbWrite/StorageWrite/StorageRead/Io）を直接構造体リテラルで追加。
