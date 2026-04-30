# Favnir v1.0.0 仕様書 — 安定版

作成日: 2026-04-30（Codex レビュー反映）

> **テーマ**: 仕様・ツールチェイン・エコシステムの入口を揃えた安定版
>
> **成功の鍵はスコープ管理。** 機能追加より完成度と絞り込みを優先する。

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 + 仕様書骨格 | `v1.0.0` がビルドされ、langspec.md の章立てが存在する |
| 1 | LSP 最小実装 | hover と diagnostics が動く |
| 2 | WASM String 戻り値 | `fn greet(name: String) -> String` がビルド・実行できる |
| 3 | WASM クロージャ | `let f <- \|x\| x + 1; f(5)` がビルド・実行できる |
| 4 | rune 依存管理 | `fav install` が path/local 依存を解決できる |
| 5 | ドキュメント整備 | 3 本の examples + README + langspec polish |

---

## 2. Phase 0 — バージョン更新 + 仕様書骨格

### 変更内容

- `Cargo.toml`: `version = "1.0.0"`
- `main.rs`: HELP テキスト `v1.0.0`
- `versions/v1.0.0/langspec.md`: **章立てのみ** 作成

### langspec.md の章立て (Phase 0 の範囲)

```
1. 基本型
2. 関数・trf・flw
3. effect system
4. パターンマッチング
5. モジュールシステム
6. 標準ライブラリ
7. CLI リファレンス
8. エラーコード一覧
9. 後方互換ポリシー
```

各章の **内容は空欄 or 箇条書きのみ**。polish は Phase 5 で行う。

---

## 3. Phase 1 — LSP 最小実装

### コマンド

```
fav lsp
```

stdin/stdout JSON-RPC。`--port` オプションは v1.1.0 以降。

### 対応機能（厳守）

| LSP メソッド | 対応内容 |
|---|---|
| `initialize` | capabilities 応答 |
| `textDocument/didOpen` | パース + 型チェック → diagnostics 送信 |
| `textDocument/didChange` | 同上 |
| `textDocument/publishDiagnostics` | TypeError → Diagnostic 変換 |
| `textDocument/hover` | カーソル位置の型を表示 |
| `textDocument/definition` | **null を返すスタブ** |
| `shutdown` / `exit` | 正常終了 |

**completion・rename・references は v1.1.0 以降。絶対に追加しない。**

### hover レスポンス

```markdown
```
x: Int
```
```

型のみ。effect は v1.1.0 以降。

### Checker の変更

`Checker` に `pub type_at: HashMap<Span, Type>` を追加。
記録対象: `Expr::Ident` と `Expr::Call` の span のみ（最小限）。

### アーキテクチャ

```
src/lsp/
  mod.rs            — run_lsp_server() / read_message / write_message
  protocol.rs       — JSON-RPC + LSP 型定義 (serde_json)
  document_store.rs — URI → (source, errors, type_at) キャッシュ
  hover.rs          — handle_hover
  diagnostics.rs    — errors_to_diagnostics
```

---

## 4. Phase 2 — WASM String 戻り値

### Done definition

```fav
public fn greet(name: String) -> String {
    name
}
public fn main() -> Unit !Io {
    IO.println(greet("Favnir"))
}
```

これが `fav build --target wasm` + `fav exec` で動けば完了。

### 変更方針

`favnir_type_to_wasm_results(Type::String)` を `Ok(vec![I32, I32])` に変更し、W001 を解除。

`slot_map` を `HashMap<u16, WasmLocal>` に変更:

```rust
enum WasmLocal {
    Single(u32),
    StringPtrLen(u32, u32),  // (ptr_local, len_local)
}
```

### 制約（広げない）

- String を返す `if/else` 式は **W002 のまま** (multi-value block type は未対応)
- `Debug.show` は **W002 のまま**
- `main` は `() -> Unit !Io` のみ (W003 維持)

---

## 5. Phase 3 — WASM クロージャ

### Done definition

```fav
public fn main() -> Unit !Io {
    let f <- |x| x + 1
    IO.println_int(f(5))
}
```

これが動けば完了。`List.map(f, xs)` 等の高階ビルトインは **v1.1.0 以降**。

### 変更方針

クロージャ = `(fn_table_idx: i32, env_ptr: i32)` の 2 値ペア。

- **TableSection**: クロージャ由来合成関数を funcref で登録
- **ElementSection**: offset 0 から合成関数を配置
- **GlobalSection**: `$heap_ptr: (mut i32) = 65536` (bump allocator)
- 合成関数 `$closure_N`: `(env_ptr: i32, ...params) -> return_ty`
- 直接呼び出しのみ対応: `call_indirect`

### 制約（広げない）

- 高階ビルトイン (`List.map` 等) への渡し → **W002 のまま**
- `trf` / `flw` の WASM 対応 → **v1.1.0**
- クロージャの再帰 → **v1.1.0**

---

## 6. Phase 4 — rune 依存管理

### Done definition

```toml
# fav.toml
[dependencies]
csv_helper = { path = "../csv_helper" }
```

`fav install` が `csv_helper` の `src/` を解決して `fav.lock` を生成できれば完了。

### fav.toml 拡張

```toml
[dependencies]
csv_helper  = { path = "../csv_helper" }
data_utils  = { version = "0.2.0", registry = "local" }
```

### registry の種類（v1.0.0 のみ）

| 種別 | 記法 |
|---|---|
| ローカルパス | `{ path = "..." }` |
| ローカルレジストリ | `{ version = "...", registry = "local" }` |

**HTTP レジストリは v1.1.0。追加しない。**

### fav.lock（最小 TOML）

```toml
[[package]]
name = "csv_helper"
version = "1.0.0"
source = "path:../csv_helper"
```

hash フィールドは v1.0.0 では省略可。

### コマンド

```
fav install       # fav.toml の全依存を解決
fav publish       # ~/.fav/registry/ に登録（ローカルのみ）
```

---

## 7. Phase 5 — ドキュメント整備

### 必須 examples（3 本のみ）

| ファイル | 内容 |
|---|---|
| `examples/hello.fav` | Hello World (既存) |
| `examples/string_wasm.fav` | WASM String 戻り値デモ |
| `examples/closures_wasm.fav` | WASM クロージャデモ |

`examples/multi_rune/` は **オプション**（時間があれば）。

### README.md

- インストール・クイックスタート・CLI リファレンス・WASM・LSP 設定例
- 長すぎない。各セクション 10 行以内を目安。

### langspec.md の polish

Phase 0 で作った骨格に内容を埋める。全部は埋めなくていい。
**最低限**: 基本型・effect system・エラーコード一覧を完成させる。

---

## 8. 完了条件

- [ ] `cargo build` 警告ゼロ
- [ ] `cargo test` 全通過 (目標 330+)
- [ ] `fav lsp` が hover + diagnostics を返す
- [ ] `fn greet(name: String) -> String { name }` が WASM でビルド・実行できる
- [ ] `let f <- |x| x + 1; f(5)` が WASM でビルド・実行できる
- [ ] `fav install` が path 依存を解決して `fav.lock` を生成する
- [ ] `examples/string_wasm.fav` と `examples/closures_wasm.fav` が動く
- [ ] `Cargo.toml` バージョンが `"1.0.0"`

---

## 9. 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| LSP: completion / rename | v1.1.0 |
| LSP: `--port` TCP モード | v1.1.0 |
| WASM: String の `if/else` result | v1.1.0 |
| WASM: `List<T>` / `Map<V>` | v1.1.0 |
| WASM: 高階ビルトイン (`List.map(f, xs)`) | v1.1.0 |
| WASM: `trf` / `flw` | v1.1.0 |
| rune: HTTP レジストリ | v1.1.0 |
| rune: `fav.lock` hash 検証 | v1.1.0 |
| `examples/multi_rune/` | オプション |
| セルフホスティング | v2.0.0 |
