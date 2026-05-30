# Favnir v4.0.0 Language Specification

## Theme: 残件一括消化 — gRPC 完全実装 + pipe match + pattern guard + スタックトレース

v3.x で積み残した 4 本の柱をまとめて実装し、言語とランタイムを完成形に近づけるメジャーバージョン。

---

## 変更サマリー

| 分類 | 機能 | 由来 |
|------|------|------|
| gRPC 完全実装 | `Grpc.serve_raw` / `Grpc.serve_stream_raw` — 実 dispatch | v3.9.0 残件 |
| gRPC 完全実装 | `Grpc.call_raw` — 実 RPC 送受信 | v3.9.0 残件 |
| gRPC 完全実装 | `Grpc.call_stream_raw` — 正しいレスポンス受信 | v3.9.0 残件（バグ修正）|
| 言語機能 | `pipe match`（`|> match { ... }`） | ロードマップ v2.2.0 繰越 |
| 言語機能 | `pattern guard`（`where` 句） | ロードマップ v2.2.0 繰越 |
| ランタイム品質 | スタックトレース | ロードマップ v2.4.0 繰越 |

---

## 1. gRPC 完全実装

### 背景

v3.9.0 では tonic/tokio の骨格を導入したが、実際のリクエスト処理は未実装のままだった:

- `Grpc.serve_raw`: スレッドを `park()` するだけ（ハンドラ呼び出し不可）
- `Grpc.call_raw`: 接続成功時に code 12 スタブを返すだけ（実 RPC 未送信）
- `Grpc.call_stream_raw`: リクエストフレームを誤ってデコードするバグあり

v4.0.0 でこれらを全て本物の gRPC 実装に置き換える。

### 1-1. `Grpc.serve_raw` — VM ハンドラ dispatch 実装

tonic サーバーが受け取ったリクエストを Favnir ハンドラ関数に橋渡しするため、
`std::sync::mpsc` チャネルで VM メインスレッドと tonic スレッドを同期する。

**チャネル設計（各リクエストが自分の応答チャネルを持つ方式）**:

```rust
// リクエストメッセージ: (ハンドラ名, proto bytes, 応答用送信端)
type GrpcRequestMsg = (String, Vec<u8>, std::sync::mpsc::SyncSender<Vec<u8>>);
```

**動作フロー**:

```
tonic スレッド（tokio async）:
  1. HTTP/2 リクエスト受信
  2. decode_grpc_frame → proto bytes
  3. URL パスから method_name 抽出 → pascal_to_snake → handler_name
  4. oneshot チャネル (res_tx, res_rx) を生成
  5. req_tx.send((handler_name, proto_bytes, res_tx))  ← ブロッキング送信
  6. spawn_blocking(|| res_rx.recv()) → レスポンス待ち
  7. レスポンス bytes → encode_grpc_frame → HTTP/2 レスポンス

VM メインスレッド:
  1. req_rx.recv() でブロッキング待機
  2. handler_name で fn_idx 解決（pascal_to_snake は既存関数を再利用）
  3. proto_bytes_to_string_map → VMValue::Record
  4. invoke_function(artifact, fn_idx, [req_record])
  5. 結果 → map_to_proto_bytes or error handling
  6. res_tx.send(response_bytes)
  7. goto 1（無限ループ）
```

**`Grpc.serve_raw` はブロッキング**（返らない）:
メインスレッドが無限ループでリクエストを処理し続ける。
`Grpc.serve_raw(50051, "UserService");` は実行後プログラムを占有する。

**ハンドラシグネチャ（変更なし）**:
```favnir
// ユニタリ: Map を受け取り Result を返す
public fn handle_get_user(req: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc { ... }
```

### 1-2. `Grpc.serve_stream_raw` — ストリーミング dispatch 実装

`Grpc.serve_raw` と同じ mpsc 設計だが、ハンドラが `List<Map<String,String>>` を返す場合に
各要素を個別 gRPC フレームとして HTTP/2 で送信する。

```
ハンドラが List を返した場合:
  items.iter()
    → each item: map_to_proto_bytes → encode_grpc_frame
    → frames を連結してボディとして送信（HTTP/2 trailing DATA frames）
```

### 1-3. `Grpc.call_raw` — 実 RPC 送受信実装

v3.9.0 の stub（code 12）を取り除き、実際の gRPC Unary RPC を送受信する。

```rust
// リクエスト:
//   1. string_map_to_proto_bytes(payload) → proto_bytes
//   2. encode_grpc_frame(proto_bytes) → frame
//   3. HTTP/2 POST /<ServiceName>/<MethodName>
//      Content-Type: application/grpc
//      body: frame

// レスポンス:
//   1. HTTP/2 response body を bytes として読み取る
//   2. decode_grpc_frame(resp_bytes) → proto_bytes
//   3. proto_bytes_to_string_map(proto_bytes) → Map<String,String>
//   4. Result::Ok(VMValue::Record(map))
```

tonic の low-level HTTP/2 リクエストには `hyper` を直接使うか、
tonic の `Grpc::unary` codec API を使う（どちらでも可）。

**エラーコード体系（変更なし）**:
- 接続失敗: `RpcError { code: 14, message: "..." }`  ← 既存テスト互換
- gRPC status エラー: `RpcError { code: <grpc_status_code>, message: "..." }`

### 1-4. `Grpc.call_stream_raw` — バグ修正（レスポンス正常受信）

v3.9.0 のバグ（リクエストフレームを誤ってデコードしていた）を修正。
接続成功後にサーバーからのレスポンスボディを受信し、`decode_all_grpc_frames` で全フレームを取り出す。

```rust
// 修正後:
// 1. リクエスト送信（call_raw と同じ）
// 2. レスポンスボディを全て読み取る
// 3. decode_all_grpc_frames(response_body) → Vec<Vec<u8>>
// 4. 各 frame → proto_bytes_to_string_map → VMValue::Record
// 5. VMValue::List(records)
```

---

## 2. `pipe match`（`|> match { ... }`）

### 構文

```favnir
fetch_user(id)
  |> match {
    Ok(user) => render(user)
    Err(e)   => default_view(e)
  }
```

`|> match { arms }` は以下の脱糖と等価:

```favnir
bind __tmp <- fetch_user(id)
match __tmp {
    Ok(user) => render(user)
    Err(e)   => default_view(e)
}
```

### 型チェック

- 左辺の型 `T` から match アームの網羅性を検査する（既存 match チェックと同一ロジック）
- 全アームの戻り型が一致していること

### AST

```
Expr::PipeMatch {
    lhs: Box<Expr>,
    arms: Vec<MatchArm>,
    span: Span,
}
```

### パーサー

`|>` トークンの後に `match` キーワードが続く場合、`PipeMatch` として解析する。
`|>` の後が `match` 以外（識別子・関数呼び出し等）なら従来の `Pipe` のまま。

---

## 3. `pattern guard`（`where` 句）

### 構文

```favnir
match user {
    { role: "admin", age } where age >= 18 => grant_access(user)
    { role: "admin" }                       => deny_underage()
    _                                       => deny_unauthorized()
}
```

### セマンティクス

- アーム末尾に `where <expr>` を付加できる
- `<expr>` の型は `Bool` でなければならない（型エラー E0xxx）
- パターンがマッチした後にガード式を評価し、`false` なら次のアームへフォールスルー
- ガード式のスコープ内では、パターンでバインドした変数が使える

### AST

```
MatchArm {
    pattern: Pattern,
    guard:   Option<Expr>,   // ← 追加
    body:    Expr,
}
```

### コンパイラ

```
// ガードありアームのコンパイル:
// 1. パターンマッチ命令
// 2. ガード評価 → Bool
// 3. Bool が false なら次のアームの先頭へ jump
// 4. Bool が true ならアーム本体を実行
```

---

## 4. スタックトレース

### 出力形式

```
RuntimeError: division by zero
  at divide (math.fav:12)
  at process (pipeline.fav:34)
  at main (main.fav:5)
```

### 実装

**コンパイラ側**:
- 各関数呼び出し（`Call` opcode）に行番号・ファイル名のデバッグ情報を付加
- `DebugInfo { fn_name: String, file: String, line: u32 }` を artifact に格納

**VM 側**:
- `call_stack: Vec<CallFrame>` を VM 構造体に追加
  ```rust
  struct CallFrame {
      fn_name: String,
      file: String,
      line: u32,
  }
  ```
- 関数呼び出し時に `call_stack.push(...)` 、return 時に `call_stack.pop()`
- ランタイムエラー発生時に `call_stack` を逆順で表示

**表示制御**:
- `fav run` では常にスタックトレースを表示
- `fav test` ではテスト失敗時のみ表示（テスト名と合わせて表示）
- スタックの深さ制限: 最大 50 フレーム（それ以上は `... N more frames` で省略）

---

## 5. Breaking Changes

| 変更 | 影響 |
|------|------|
| `Grpc.serve_raw` が実際のリクエストをブロッキング処理するようになる | v3.8.0 の tiny_http サーバーとは互換なし |
| `Grpc.call_raw` 接続成功時に実際の RPC が実行される | code 12 スタブが撤廃される |
| `Grpc.call_stream_raw` の戻り値が正しくレスポンスから生成される | v3.9.0 の誤 decode 動作に依存していたコードは修正が必要 |
| `Grpc.serve_raw` / `Grpc.serve_stream_raw` がブロッキングになる | `serve_raw` の後ろに処理を書いても実行されない（v3.8.0 は即 return）|

`pipe match` / `pattern guard` / スタックトレースは既存コードへの破壊的変更なし。

---

## 6. 典型ワークフロー

### pipe match

```favnir
// v3.x（冗長）
bind result <- Grpc.call_raw("localhost:50051", "/UserService/GetUser", req)
match result {
    Ok(user) => IO.println($"Got user: {user}")
    Err(e)   => IO.println($"Error: {e.message}")
}

// v4.0.0（簡潔）
Grpc.call_raw("localhost:50051", "/UserService/GetUser", req)
  |> match {
    Ok(user) => IO.println($"Got user: {user}")
    Err(e)   => IO.println($"Error: {e.message}")
  }
```

### pattern guard

```favnir
public fn classify(score: Int) -> String {
    match score {
        s where s >= 90 => "A"
        s where s >= 80 => "B"
        s where s >= 70 => "C"
        _               => "F"
    }
}
```

### gRPC サーバー（動作する）

```favnir
import rune "grpc"

type GetUserRequest  = { id: String }
type GetUserResponse = { id: String name: String }

public fn handle_get_user(req: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc {
    bind id <- Option.unwrap_or(Map.get(req, "id"), "0")
    grpc.ok(Map.set(Map.set((), "id", id), "name", "Alice"))
}

public fn main() -> Unit !Io !Rpc {
    IO.println("gRPC server on :50051");
    grpc.serve(50051, "UserService");   // ← ここでブロック（v4.0.0 で実際に動く）
}
```

---

## 7. 新規 Cargo 依存

v3.9.0 で追加済みの `tonic` / `tokio` / `prost` を引き続き使用。
追加の依存は原則なし（hyper は tonic の transitive dependency として既に利用可能）。

---

## テスト目標

- 全テスト ~950 件以上パス
- gRPC end-to-end テスト（実際のポートにサーバーを起動してクライアントから呼び出す）
- pipe match / pattern guard の各テスト 10 件以上
- スタックトレースが正しいフレーム順で表示されることの確認テスト
