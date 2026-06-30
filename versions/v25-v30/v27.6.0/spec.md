# v27.6.0 仕様書 — jsonl Rune 追加

## 概要

JSONL（JSON Lines）Rune を新規追加する。
`import rune "jsonl"` → `JSONL.*` 名前空間に 4 関数を追加（read / write / stream / append）。

---

## 背景

ロードマップ v27.6「jsonl Rune 追加」より。Data Lakehouse フェーズの第 6 コンポーネント。
JSON Lines は LLM ファインチューニングデータ・構造化ログ・イベントストアの現代的標準。
ストリーミング処理との親和性が高く、`Stream.*` と組み合わせて大容量データを扱える。

ファイル I/O ベースのため `!Io` エフェクト（DeltaLake / Iceberg と統一）を採用。
ロードマップに `JSONL.read[T]` / `JSONL.stream[T]` のジェネリック API が記載されているが、
stub 段階では VM レベルでジェネリック型引数の評価ができないため v28.x 以降に延期する。

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `JSONL.read` | `(path: String) -> Result<String, String> !Io` | 全件読み込み。JSON 配列文字列を返す |
| `JSONL.write` | `(path: String, rows: String) -> Result<Unit, String> !Io` | JSON 配列文字列を JSONL 形式で書き込み（上書き） |
| `JSONL.stream` | `(path: String) -> Result<String, String> !Io` | ストリーミング読み込み stub。JSON 配列文字列を返す（コールバック機構は v28.x） |
| `JSONL.append` | `(path: String, row: String) -> Result<Unit, String> !Io` | JSON オブジェクト文字列を 1 行追記 |

> **エフェクト**: `!Io`（DeltaLake / Iceberg / fs Rune と統一。ファイル I/O ベース）
>
> **`stream` のスコープ**: ロードマップの `JSONL.stream[T](path, fn)` ジェネリック＋コールバック形式は v28.x に延期。v27.6.0 は `stream(path: String) -> Result<String, String>` として stub 実装し、1 行ずつのコールバック機構を提供しない。

---

## VM Primitive（vm.rs に追加）

| primitive 名 | シグネチャ（引数） | 実装方針 |
|---|---|---|
| `JSONL.read_raw` | `(path: String)` | stub: 引数検証、`ok_vm(VMValue::Str("[]".into()))` 返却 |
| `JSONL.write_raw` | `(path: String, rows: String)` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `JSONL.stream_raw` | `(path: String)` | stub: 引数検証、`ok_vm(VMValue::Str("[]".into()))` 返却 |
| `JSONL.append_raw` | `(path: String, row: String)` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |

> **挿入位置**: Redshift ブロック末尾（`"Redshift.unload_to_s3_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17942 付近）・Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）直前。
>
> **wasm32 ガード**: `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アームで追加する。wasm32 アームは `err_vm("JSONL not supported on wasm32")` を返す。

---

## runes/jsonl/jsonl.fav（新規作成）

```favnir
// runes/jsonl/jsonl.fav — JSONL Rune (v27.6.0)
// JSON Lines（1 行 1 JSON オブジェクト）の読み書き・ストリーミング処理。
// ジェネリック型付き API（JSONL.read[T] / JSONL.stream[T]）は v28.x 以降。
// TODO(v28.x): JSONL.read[T] / JSONL.stream[T] ジェネリック API + コールバック機構に移行予定。
public fn read(path: String) -> Result<String, String> !Io {
    JSONL.read_raw(path)
}
public fn write(path: String, rows: String) -> Result<Unit, String> !Io {
    JSONL.write_raw(path, rows)
}
public fn stream(path: String) -> Result<String, String> !Io {
    JSONL.stream_raw(path)
}
public fn append(path: String, row: String) -> Result<Unit, String> !Io {
    JSONL.append_raw(path, row)
}
```

---

## examples/jsonl_etl.fav

```favnir
// examples/jsonl_etl.fav — JSONL ETL デモ (v27.6.0)
import rune "jsonl"

stage ReadData: Unit -> Result<String, String> !Io = |_| {
    JSONL.read("data/events.jsonl")
}

// seq pipeline は前ステージの成功値（String）を次ステージの引数として渡す
stage WriteProcessed: String -> Result<Unit, String> !Io = |rows| {
    JSONL.write("data/processed.jsonl", rows)
}

seq JsonlEtlPipeline = ReadData |> WriteProcessed
```

---

## テスト

### driver.rs v276000_tests（10 件）

| テスト名 | 内容 |
|---|---|
| `jsonl_rune_has_read_fn` | `jsonl.fav` に `"fn read("` が含まれること |
| `jsonl_rune_has_write_fn` | `jsonl.fav` に `"fn write("` が含まれること |
| `jsonl_rune_has_stream_fn` | `jsonl.fav` に `"fn stream("` が含まれること |
| `jsonl_rune_has_append_fn` | `jsonl.fav` に `"fn append("` が含まれること |
| `jsonl_rune_vm_has_read_raw` | `vm.rs` に `"JSONL.read_raw"` が含まれること |
| `jsonl_rune_vm_has_write_raw` | `vm.rs` に `"JSONL.write_raw"` が含まれること |
| `jsonl_rune_vm_has_stream_raw` | `vm.rs` に `"JSONL.stream_raw"` が含まれること |
| `jsonl_rune_vm_has_append_raw` | `vm.rs` に `"JSONL.append_raw"` が含まれること |
| `jsonl_example_has_pipeline` | `examples/jsonl_etl.fav` に `"JsonlEtlPipeline"` が含まれること |
| `changelog_has_v27_6_0` | `CHANGELOG.md` に `"[v27.6.0]"` が含まれること |

### `cargo test jsonl` 期待値

- `v276000_tests::jsonl_rune_has_*` 4 件
- `v276000_tests::jsonl_rune_vm_has_*` 4 件
- `v276000_tests::jsonl_example_has_pipeline` 1 件
- 合計 9 件（`changelog_has_v27_6_0` は `jsonl` を含まないため除外）（ロードマップ要件「3 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.6.0"` であること
- [ ] `runes/jsonl/jsonl.fav` に `public fn read(` が含まれること
- [ ] `runes/jsonl/jsonl.fav` に `public fn write(` が含まれること
- [ ] `runes/jsonl/jsonl.fav` に `public fn stream(` が含まれること
- [ ] `runes/jsonl/jsonl.fav` に `public fn append(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `JSONL.read_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `JSONL.write_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `JSONL.stream_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `JSONL.append_raw` が含まれること
- [ ] `examples/jsonl_etl.fav` に `JsonlEtlPipeline` が含まれること
- [ ] `site/content/docs/runes/jsonl.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.6.0]` エントリが存在すること
- [ ] `benchmarks/v27.6.0.json` が存在すること（test_count: 2186）
- [ ] `v276000_tests` 10 件すべて PASS
- [ ] `cargo test jsonl --bin fav` で 9 件 PASS（`changelog_has_v27_6_0` は `jsonl` を含まないため除外）
- [ ] 総テスト数 ≥ 2186 件
- [ ] `fav/self/checker.fav` の `ns_to_effect` に `"JSONL" => "IO"` が登録されていること

---

## スコープ外（v28.x 以降）

- `JSONL.write` の append / overwrite モードオプション引数
  - **延期根拠**: ロードマップでは「書き込み（追記 / 上書きオプション）」と記載されているが、v27.6.0 では上書き専用に簡略化し、追記は `JSONL.append` 関数で代替する。オプション引数の設計（`mode: String` か別フラグか）は v28.x で確定する。
- `JSONL.read[T]` ジェネリック API（型安全な行取得）
  - **延期根拠**: stub 段階のため VM レベルでジェネリック型引数の評価ができない
- `JSONL.stream[T](path, fn)` コールバック機構（1 行ずつ変換・フィルタ）
  - **延期根拠**: 高階関数のコールバックは VM レベルの関数値渡し機構が必要（v28.x で対応）
- `Stream.*` との統合（ストリーミングパイプライン）
- JSONL ファイルの行数バリデーション・エラーライン特定
- gzip 圧縮 JSONL（`.jsonl.gz`）サポート
