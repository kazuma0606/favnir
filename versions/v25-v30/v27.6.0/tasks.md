# v27.6.0 タスクリスト — jsonl Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T11 | spec-reviewer レビュー実施（実装前・T0 開始前に完了） | [x] |
| T0 | 事前確認: `Cargo.toml` が `27.5.0`、テスト数 2176 件、`runes/jsonl/` が存在しない、`vm.rs` に `JSONL.read_raw` がないことを確認。`cargo test jsonl --bin fav` のベースライン件数（0 件）を記録 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.6.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` に JSONL primitive 4 件追加（Redshift ブロック末尾・Azure Blob 直前。`#[cfg]` ガード付き） | [x] |
| T3 | `runes/jsonl/jsonl.fav` 新規作成（4 関数: read / write / stream / append） | [x] |
| T4 | `examples/jsonl_etl.fav` 新規作成（ReadData \|> WriteProcessed） | [x] |
| T5 | `site/content/docs/runes/jsonl.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.6.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.6.0.json` 新規作成（test_count: 2186） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v276000_tests`（10 件）を `v275000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v276000 --bin fav` — 10/10 PASS 確認 | [x] |
| T8.6 | `cargo test jsonl --bin fav` — 9 件 PASS 確認（`changelog_has_v27_6_0` はテスト名に `jsonl` を含まないため除外） | [x] |
| T10 | `fav/self/checker.fav` 更新: `ns_to_effect` に `"JSONL" => "IO"` を追加（v27.5.0 BUG の教訓。T9 より前に完了すること） | [x] |
| T9 | `cargo test --bin fav` — 2186 件 PASS 確認（リグレッションなし） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.6.0"` であること
- [x] `runes/jsonl/jsonl.fav` に `public fn read(` が含まれること
- [x] `runes/jsonl/jsonl.fav` に `public fn write(` が含まれること
- [x] `runes/jsonl/jsonl.fav` に `public fn stream(` が含まれること
- [x] `runes/jsonl/jsonl.fav` に `public fn append(` が含まれること
- [x] `fav/src/backend/vm.rs` に `JSONL.read_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `JSONL.write_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `JSONL.stream_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `JSONL.append_raw` が含まれること
- [x] `examples/jsonl_etl.fav` に `JsonlEtlPipeline` が含まれること
- [x] `site/content/docs/runes/jsonl.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.6.0]` エントリが存在すること
- [x] `benchmarks/v27.6.0.json` が存在すること（test_count: 2186）
- [x] `v276000_tests` 10 件すべて PASS
- [x] `cargo test jsonl --bin fav` で 9 件 PASS
- [x] 総テスト数 ≥ 2186 件
- [x] `fav/self/checker.fav` `ns_to_effect` に `"JSONL"` が登録されていること

---

## メモ

### vm.rs 挿入位置

Redshift ブロック末尾（`"Redshift.unload_to_s3_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17942 付近）に挿入。
Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。
wasm32 ガードは Redshift / ClickHouse と同パターン（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アーム）。

### `stream_raw` の設計

ロードマップの `JSONL.stream[T](path, fn)` はコールバック関数を引数に取るが、stub 段階では高階関数渡しが不要なため `path: String` のみを引数とする。コールバック機構は v28.x に延期。

### `read_raw` と `stream_raw` の違い

どちらも `path: String` → `"[]"` を返す stub だが、別 primitive として分離することで v28.x で独立して実装できるようにしておく。`stream_raw` は将来的にコールバックや非同期イテレータ API に変化する予定。

### checker.fav 更新（T10）

v27.5.0 でコードレビュー [BUG] として指摘された `ns_to_effect` 漏れを防ぐため、T10 として明示的にタスク化。`"JSONL"` → `"IO"` を `"Debug"` ブランチの直後・デフォルト `""` の前に追加する。

### テスト数計算

2176（v27.5.0 完了後）+ 10（v276000_tests）= 2186

### example の型連鎖

```
ReadData:      Unit   -> Result<String, String> !Io
WriteProcessed: String -> Result<Unit, String>  !Io
```
`ReadData` の成功値（String = JSON 配列）→ `WriteProcessed` の引数（String）: 接続 OK

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/jsonl/jsonl.fav` | `favnir/runes/jsonl/jsonl.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/jsonl_etl.fav` | `favnir/examples/jsonl_etl.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [STYLE] checker.fav の `ns_to_effect` ネストが ClickHouse → Redshift → JSONL で 3 段に肥大化 | 既存パターン踏襲のため今バージョンでは対応なし。v28.x で ns 追加時に平坦化を検討する |
