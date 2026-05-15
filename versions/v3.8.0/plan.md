# Favnir v3.8.0 Implementation Plan

## Theme: `grpc` rune — gRPC サービス定義 + Protobuf 出力

---

## Phase 0: バージョン更新 + 依存追加

- `Cargo.toml` version → `"3.8.0"`
- `Cargo.toml` に依存追加:
  - `prost = "0.12"` — Protobuf エンコード/デコード
  - `prost-build = "0.12"` — .proto ファイルのパース（build-dependencies）
  - `tonic = { version = "0.11", features = ["transport"] }` — gRPC サーバー/クライアント
  - `tokio = { version = "1", features = ["full"] }` — async ランタイム（tonic の依存）
- `src/main.rs` ヘルプテキスト・バージョン更新

> **注意**: tokio は重い依存。既存テストへの影響を確認する。
> `[dev-dependencies]` ではなく `[dependencies]` に追加（VM ランタイムが使用するため）。

---

## Phase 1: 型登録 + namespace

**checker.rs**
- `RpcError` を `type_defs` に pre-register（`code: Int`, `message: String`）
- `RpcRequest` を `type_defs` に pre-register（`method: String`, `payload: Map<String,String>`）
  - 注: `Map<String,String>` フィールドは `Type::Map(...)` として登録
- `!Rpc` エフェクトを既存エフェクトリストに追加（`Effect::Rpc`）
- `Grpc` namespace をチェックリストに追加
- `Grpc.serve_raw / call_raw / encode_raw / decode_raw` のシグネチャ登録

**compiler.rs**
- `"Grpc"` を2箇所のグローバル登録ループに追加
- `"RpcError"`, `"RpcRequest"` を type registration ループに追加

---

## Phase 2: VM プリミティブ

**vm.rs**

### Protobuf ヘルパー関数（非 VM、内部 Rust）

```rust
fn map_to_proto_bytes(type_name: &str, row: &IndexMap<String, VMValue>, type_metas: &...) -> Result<Vec<u8>, String>
fn proto_bytes_to_map(bytes: &[u8], type_meta: &TypeMeta) -> Result<IndexMap<String, VMValue>, String>
```

`prost` の dynamic message（`prost_types::Value`）または手動エンコードで実装。
フィールド番号は `type_meta.fields` の順序から決定（1-indexed）。

### `Grpc.serve_raw`

```rust
"Grpc.serve_raw" => {
    // 1. tokio::runtime::Builder::new_multi_thread().build() でランタイム作成
    // 2. tonic で gRPC サーバーを起動（dedicated thread）
    // 3. リクエスト受信時に crossbeam / std::sync::mpsc チャネルで VM スレッドへ転送
    // 4. VM が Favnir ハンドラを呼び出してレスポンスを返す
    // 5. レスポンスをチャネル経由で tonic ハンドラへ返す
}
```

ハンドラ登録方式:
- `service_name` に対応する Favnir 関数をアーティファクトから検索
- 命名規則: `handle_<method_name_snake_case>` の関数をハンドラとして使用

### `Grpc.call_raw`（クライアント側）

```rust
"Grpc.call_raw" => {
    // tonic::transport::Channel で接続
    // 手動 gRPC リクエスト（method + payload bytes）
    // → Result<Map<String,String>, RpcError>
}
```

### `Grpc.encode_raw` / `Grpc.decode_raw`

```rust
"Grpc.encode_raw" => {
    // type_metas からスキーマ取得 → prost エンコード → Base64 文字列
}
"Grpc.decode_raw" => {
    // Base64 → bytes → prost デコード → Map<String,String>
}
```

**vm_stdlib_tests.rs**: 6件の新テスト追加

| テスト名 | 内容 |
|---------|------|
| `grpc_encode_decode_roundtrip` | encode → decode → 元データと一致 |
| `grpc_encode_int_field` | Int フィールドが正しくエンコードされる |
| `grpc_encode_string_field` | String フィールドが正しくエンコードされる |
| `grpc_call_raw_returns_err_on_bad_host` | 不正ホスト → Err(RpcError) |
| `rpc_error_code_field_accessible` | `rpc_error.code` が Int で取得できる |
| `rpc_error_message_field_accessible` | `rpc_error.message` が String で取得できる |

> `Grpc.serve_raw` の統合テストは driver 側で実施（別スレッドでサーバーを立てて call_raw で確認）。

---

## Phase 3: `runes/grpc/grpc.fav`

```
runes/
  grpc/
    grpc.fav   ← 6 関数
```

実装する関数:
1. `serve(port, service_name)` → `Grpc.serve_raw(port, service_name)`
2. `call(host, method, payload)` → `Grpc.call_raw(host, method, payload)`
3. `encode(type_name, row)` → `Grpc.encode_raw(type_name, row)`
4. `decode(type_name, encoded)` → `Grpc.decode_raw(type_name, encoded)`
5. `ok(payload)` → `Result.ok(payload)`（純粋関数）
6. `err(code, message)` → `Result.err(RpcError { code: code, message: message })`（純粋関数）

---

## Phase 4: `runes/grpc/grpc.test.fav`

テスト（10 件目標）:

| # | テスト名 | 内容 |
|---|---------|------|
| 1 | `grpc_encode_decode_roundtrip` | encode → decode でデータ保持 |
| 2 | `grpc_encode_int_field_preserved` | Int フィールドが保持される |
| 3 | `grpc_encode_string_field_preserved` | String フィールドが保持される |
| 4 | `grpc_call_bad_host_is_err` | 不正ホスト → Err |
| 5 | `grpc_ok_is_ok_result` | grpc.ok(...) → Result.is_ok |
| 6 | `grpc_err_is_err_result` | grpc.err(...) → Result.is_err |
| 7 | `grpc_err_code_preserved` | grpc.err(2, "...").code == 2 |
| 8 | `grpc_err_message_preserved` | grpc.err(2, "msg").message == "msg" |
| 9 | `grpc_encode_float_field_preserved` | Float フィールドが保持される |
| 10 | `grpc_encode_bool_field_preserved` | Bool フィールドが保持される |

---

## Phase 5: driver 統合テスト

`migrate_tests` モジュールに追加（6 テスト）:

1. `grpc_rune_test_file_passes` → `run_fav_test_file_with_runes("runes/grpc/grpc.test.fav")`
2. `grpc_encode_decode_in_favnir_source` — inline Favnir ソース
3. `grpc_ok_helper_in_favnir_source` — `grpc.ok(payload)` が Result.is_ok
4. `grpc_err_helper_in_favnir_source` — `grpc.err(2, "msg")` の code/message
5. `grpc_call_bad_host_in_favnir_source` — Err(RpcError) を返す
6. `fav_build_proto_generates_message_block` — `cmd_build_proto` の SDL 生成確認

---

## Phase 6: `fav build --proto` + `fav infer --proto`

### `fav build --proto`（driver.rs）

`fav build` の `--proto` フラグ。`fav build --graphql` と同様に AST を静的走査。

```rust
pub fn cmd_build_proto(file: &str, out: Option<&str>) {
    // AST を parse → render_proto_schema → ファイル or stdout
}

fn render_proto_schema(program: &ast::Program) -> String {
    // TypeDef → message { field_type field_name = field_number; }
    // InterfaceDef → service { rpc MethodName(Request) returns (Response); }
    // Stream<T> → stream T
}
```

型マッピング（Favnir → proto3）:
- `Int` → `int64`
- `Float` → `double`
- `String` → `string`
- `Bool` → `bool`
- `Option<T>` → `optional <T>`
- `List<T>` → `repeated <T>`
- `Result<T, E>` → `<T>`（エラーは gRPC status で返す）
- `Stream<T>` → `stream <T>`（サーバーストリーミング）
- `Unit` → `google.protobuf.Empty`

### `fav infer --proto`（driver.rs）

既存 `.proto` ファイルを Favnir 型定義に変換。

```rust
pub fn cmd_infer_proto(proto_path: &str, out_path: Option<&str>) {
    // prost-build で .proto をパース → descriptor
    // descriptor の message → TypeDef
    // descriptor の service → InterfaceDef
    // render_fav_from_proto(descriptor) → String
}
```

proto3 → Favnir 型マッピング:
- `int32` / `int64` / `sint64` → `Int`
- `float` / `double` → `Float`
- `string` / `bytes` → `String`
- `bool` → `Bool`
- `repeated T` → `List<T>`
- `optional T` → `Option<T>`
- `stream T`（戻り型）→ `Stream<T>`
- `google.protobuf.Empty` → `Unit`

**main.rs**
- `fav build --proto <file> [--out <path>]` フラグ追加
- `fav infer --proto <file> [--out <path>]` フラグ追加

---

## Phase 7: examples + docs

- `fav/examples/grpc_server_demo/src/main.fav` — サーバー起動例
- `fav/examples/grpc_client_demo/src/main.fav` — クライアント呼び出し例
- `fav/examples/proto_roundtrip_demo/src/main.fav` — encode/decode デモ
- `versions/v3.8.0/langspec.md`
- `versions/v3.8.0/migration-guide.md`
- `versions/v3.8.0/progress.md` 全フェーズ完了に更新

---

## 依存関係

| クレート | バージョン | 用途 |
|---------|----------|------|
| `prost` | `"0.12"` | Protobuf エンコード/デコード |
| `prost-build` | `"0.12"` | .proto パース（build-dependency） |
| `tonic` | `"0.11"` + `transport` | gRPC サーバー/クライアント |
| `tokio` | `"1"` + `full` | async ランタイム |

---

## テスト目標

v3.7.0: ~840 tests → v3.8.0 目標: **~890 tests**

---

## 実装上の注意

### `Grpc.serve_raw` のスレッドモデル

```
[Favnir VM スレッド]  ←─チャネル─→  [tonic gRPC サーバー thread]
       │                                       │
  Favnir ハンドラ呼び出し                 tonic リクエスト受信
  Result<Map,RpcError> 返却              tonic レスポンス送信
```

- `std::sync::mpsc` または `crossbeam-channel` でスレッド間通信
- タイムアウト: デフォルト 30 秒（設定可能にしない）
- VM はブロッキングのまま（tokio ランタイムは gRPC スレッドが管理）

### Protobuf フィールド番号の決定

`type_meta.fields` のインデックス（1-indexed）をフィールド番号として使用。
`type Msg = { a: Int b: String }` → `a = 1, b = 2`。

### `prost` dynamic encoding

`prost` は通常コード生成を使う（`.proto` → Rust struct）が、
VM では型が動的なため、`prost::encoding::*` の低レベル API を使って手動エンコード。

### `fav infer --proto` の `prost-build` 利用

`prost-build` の `FileDescriptorSet` パースを利用してメッセージ/サービス定義を取得。
`protoc` コマンドへの依存を避けるため、`protox` クレート（pure Rust proto parser）も検討。

> **代替**: `prost-build` は `protoc` を必要とする場合がある。
> `protox = "0.5"` (pure Rust .proto パーサー) で代替することを検討する。
