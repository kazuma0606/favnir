# Favnir v1.0.0 仕様書 — 安定版

作成日: 2026-04-30

> **テーマ**: 仕様・ツールチェイン・エコシステムの入口を揃えた安定版リリース
>
> v0.9.0 までで言語コア・VM・WASM backend の基盤が揃った。
> v1.0.0 では「他者が使い始められる状態」を目標に、
> LSP・rune 依存管理・WASM String/クロージャ対応・ドキュメント整備を行う。

---

## 1. スコープ概要

| Phase | テーマ | 主な追加 |
|---|---|---|
| 0 | 仕様安定化 | 言語仕様書 (langspec.md)、後方互換保証ポリシー |
| 1 | LSP 最小実装 | `fav lsp`、hover、diagnostics |
| 2 | WASM String 戻り値 | multi-value return、W001 解除 |
| 3 | WASM クロージャ | function table、`call_indirect` |
| 4 | rune 依存管理 | `fav.toml [dependencies]`、`fav install` (ローカル) |
| 5 | ドキュメント + リリース | README 完全版、examples、リリースノート |

---

## 2. Phase 0 — 言語仕様書

### 目的

v1.0.0 以降、言語の構文・型システム・effect システムを安定させる。
`versions/v1.0.0/langspec.md` として仕様を文書化する。

### 内容

- 基本型・複合型・型推論ルール
- `fn` / `trf` / `flw` / `type` / `cap` / `impl` の構文
- `bind` / `chain` / `yield` / `collect` の意味論
- effect system: `!Io`, `!Db`, `!Network`, `!File`, `!Trace`, `!Emit<T>`
- パターンマッチング (ガード含む)
- モジュールシステム (`namespace`, `use`, `pub`)
- エラーコード一覧 (E001–E036, W001–W004)
- 後方互換保証: v1.0.0 以降は破壊的変更を行わない

### 後方互換ポリシー

- 構文・型推論規則・エラーコードは **マイナーバージョン** では変更しない
- 新機能追加のみ許可
- 破壊的変更は **メジャーバージョン** (v2.0.0) まで持ち越す

---

## 3. Phase 1 — LSP 最小実装

### コマンド

```
fav lsp [--port <n>]
```

- デフォルト: stdin/stdout で JSON-RPC (エディタ統合の標準方式)
- `--port`: TCP モード (デバッグ用)

### 対応機能

| LSP メソッド | 機能 |
|---|---|
| `initialize` | capabilites 応答 |
| `textDocument/didOpen` | ドキュメントをメモリ上でパース+型チェック |
| `textDocument/didChange` | 変更差分を受け取り再チェック |
| `textDocument/publishDiagnostics` | 型エラーを diagnostics として送信 |
| `textDocument/hover` | カーソル位置の型・effect を表示 |
| `textDocument/definition` | 定義元へのジャンプ (同一ファイル内) |

### hover レスポンス形式

```
fn greet(name: String) -> String
effects: !Io
```

```
x: Int
```

### diagnostics 形式

```json
{
  "range": { "start": {"line": 4, "character": 10}, "end": {"line": 4, "character": 15} },
  "severity": 1,
  "code": "E001",
  "message": "Type mismatch: expected Int, got String"
}
```

### アーキテクチャ

```
src/lsp/
  mod.rs          — LSP サーバーエントリ (ループ)
  protocol.rs     — JSON-RPC + LSP 型定義 (serde_json)
  document_store.rs — ドキュメント (URI → source) のインメモリ管理
  hover.rs        — hover ハンドラ (AST span → 型情報)
  diagnostics.rs  — diagnostics 生成 (TypeError → LSP Diagnostic)
```

### hover の仕組み

1. `textDocument/hover` で `(uri, position)` を受け取る
2. `DocumentStore` からソースと AST (+ 型情報) を取得
3. `position` を span に変換して最も近い AST ノードを検索
4. `Checker` が記録した型情報から型・effect を文字列化して返す

### Checker の変更

`Checker` に型情報マップ `type_at: HashMap<Span, Type>` を追加し、
各式チェック時に記録する。hover 時はこのマップを参照する。

---

## 4. Phase 2 — WASM String 戻り値

### 現状

`fn greet() -> String` は W001 エラー。String は戻り値に使えない。

### 変更方針

WASM multi-value return を使って String を `(i32, i32)` (ptr, len) として返す。

| 型 | WASM パラメータ | WASM 戻り値 |
|---|---|---|
| `Int` | `[I64]` | `[I64]` |
| `Float` | `[F64]` | `[F64]` |
| `Bool` | `[I32]` | `[I32]` |
| `Unit` | `[]` | `[]` |
| `String` | `[I32, I32]` | `[I32, I32]` ← 変更 |

### String ローカル変数

String 型のローカルは **2つの WASM ローカル** に分解する:
- `slot_ptr` (i32) — data section のポインタ
- `slot_len` (i32) — バイト長

`slot_map: HashMap<u16, (u32, Option<u32>)>` に変更し、
String は `(ptr_local, Some(len_local))`, 他は `(local, None)` で表現。

### String ローカルへの bind

```
IRStmt::Bind(slot, expr)  where expr.ty() == String
→ emit_expr(expr)  // pushes (ptr, len)
→ LocalSet(len_local)
→ LocalSet(ptr_local)
```

### String ローカルの読み取り

```
IRExpr::Local(slot, String)
→ LocalGet(ptr_local)
→ LocalGet(len_local)
```

### W001 変更後の制約

- String を戻り値にできる (W001 解除)
- String を **ローカル変数** として bind できる
- ただし String は **算術演算・比較** 不可 (W002 として残す)
- `Debug.show` の呼び出しは W002 のまま (String + Unit 連鎖が複雑なため)

### W003 変更

`main` は引き続き `() -> Unit !Io` のみ許可 (W003 は維持)。

---

## 5. Phase 3 — WASM クロージャ

### 現状

`IRExpr::Closure` は W002。高階関数・`trf` が WASM で使えない。

### 変更方針

**WASM function table** + **環境ポインタ** によるクロージャ実装。

#### クロージャの表現

WASM では以下の 2 値でクロージャを表現:
- `fn_table_idx` (i32) — function table のインデックス
- `env_ptr` (i32) — 線形メモリ上のキャプチャ環境

`IRExpr::Closure(fn_idx, captures, _)` を:
1. キャプチャ値を線形メモリに書き込む (bump allocator)
2. `fn_table_idx = fn_idx` として `I32Const(fn_table_idx)` を push
3. `env_ptr` を push

#### 高階関数呼び出し

クロージャを引数に取る関数 (`List.map(f, list)` 等) の WASM 版は現フェーズでは非対応。
クロージャを **直接呼び出す** ケース (`let f = |x| x + 1; f(5)`) のみ対応:
- スタック上の `(fn_table_idx, env_ptr)` から `call_indirect` で呼び出す

#### bump allocator

```wasm
(global $heap_ptr (mut i32) (i32.const 65536))  ;; 1ページ目の終端から開始
```

`fn bump_alloc(size: i32) -> i32`:
- `heap_ptr` を `size` バイト進めてアドレスを返す
- 境界チェックなし (WASM MVP の制約)

#### TableSection

```
(table (export "fn_table") N funcref)
(elem (i32.const 0) $closure_0 $closure_1 ...)
```

- クロージャ用合成関数を事前に table に登録
- 呼び出し元は `call_indirect (type $closure_sig) (local.get $fn_idx) (local.get $env_ptr)`

#### W002 残存制約

以下は引き続き W002:
- 高階ビルトイン呼び出し (`List.map(f, xs)`) — function table 経由の間接呼び出し + Iterator が必要
- `trf` / `flw` — 上記と同様
- `chain` / `collect` — List 型依存

---

## 6. Phase 4 — rune 依存管理

### fav.toml の拡張

```toml
[rune]
name = "my_app"
version = "1.0.0"
src = "src"

[dependencies]
data_utils = { version = "0.2.0", registry = "local" }
csv_helper = { path = "../csv_helper" }
```

### registry の種類 (v1.0.0)

| 種別 | 記法 | 説明 |
|---|---|---|
| ローカルパス | `{ path = "..." }` | 相対パスで指定 |
| ローカルレジストリ | `{ version = "...", registry = "local" }` | `~/.fav/registry/` を参照 |

HTTP レジストリは v1.1.0 以降。

### `fav install` コマンド

```
fav install              # fav.toml の全依存を解決
fav install <rune>       # 指定 rune を追加してインストール
```

動作:
1. `fav.toml` の `[dependencies]` を読む
2. `{ path }` は相対パス解決してシンボリックリンク or コピー
3. `{ registry = "local" }` は `~/.fav/registry/<name>/<version>/` から読む
4. `~/.fav/cache/<name>@<version>/` に展開
5. `fav.lock` を生成 (name, version, source_path, hash)

### `fav publish` コマンド (ローカルのみ)

```
fav publish              # ~/.fav/registry/ にパッケージを登録
```

動作:
1. `fav.toml` を読む
2. `src/` 配下を tar.gz に圧縮
3. `~/.fav/registry/<name>/<version>/` に展開

### fav.lock 形式

```toml
[[package]]
name = "data_utils"
version = "0.2.0"
source = "local:data_utils@0.2.0"
hash = "sha256:..."

[[package]]
name = "csv_helper"
version = "1.0.0"
source = "path:../csv_helper"
hash = "sha256:..."
```

### `fav check` / `fav run` の依存解決

- 実行前に `fav.lock` を読み、依存 rune の `src/` を `load_all_items` に追加

---

## 7. Phase 5 — ドキュメント + リリース

### README.md の完全版

- インストール方法 (`cargo install fav`)
- クイックスタート (hello.fav → run → build → exec)
- 言語概要 (型、effect、trf、chain)
- CLI リファレンス (全コマンド)
- WASM backend の使い方
- LSP 設定例 (VS Code, Neovim)

### examples/ の整備

| ファイル | 内容 |
|---|---|
| `examples/hello.fav` | Hello World (既存) |
| `examples/math_wasm.fav` | WASM 算術 (既存) |
| `examples/string_wasm.fav` | WASM String 戻り値 (新規) |
| `examples/closures_wasm.fav` | WASM クロージャ (新規) |
| `examples/multi_file/` | マルチファイル (既存) |
| `examples/pipeline.fav` | trf + flw (既存) |

### LSP 設定ドキュメント (docs/lsp.md)

- VS Code: `.vscode/extensions.json` + 設定例
- Neovim: `nvim-lspconfig` 設定例
- 汎用: Language Client の設定方法

### リリースノートの作成

`versions/v1.0.0/RELEASE_NOTES.md` として:
- v0.x からの変更点
- Breaking changes (なし)
- 既知の制限事項
- 次のバージョン (v1.1.0) の予告

---

## 8. エラーコード追加

| コード | 意味 |
|---|---|
| E037 | LSP: ドキュメントが見つからない (内部エラー) |
| E038 | rune 依存解決エラー: rune not found |
| E039 | rune 依存解決エラー: version conflict |
| E040 | fav.lock の整合性エラー |

---

## 9. 完了条件

- [ ] `cargo build` 警告ゼロ
- [ ] `cargo test` 全テスト通過 (目標: 330+)
- [ ] `fav lsp` が VS Code または Neovim で動作する
- [ ] `fn greet() -> String { "hello" }` が WASM でビルド・実行できる
- [ ] クロージャを含む関数が WASM でビルド・実行できる
- [ ] `fav install` が ローカルパス依存を解決できる
- [ ] `langspec.md` が完成している
- [ ] `README.md` が完全版になっている

---

## 10. 既知の制約 (v1.0.0 でも先送り)

| 制約 | 対応バージョン |
|---|---|
| `List<T>` / `Map<V>` の WASM 対応 | v1.1.0 (WasmGC) |
| `trf` / `flw` の WASM 対応 | v1.1.0 |
| `chain` / `collect` の WASM 対応 | v1.1.0 |
| `Db`/`Network`/`File` の WASM 対応 | v1.1.0 |
| HTTP レジストリ | v1.1.0 |
| LSP: 補完 (completion) | v1.1.0 |
| LSP: リネーム (rename) | v1.1.0 |
| セルフホスティング | v2.0.0 |
