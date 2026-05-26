# Favnir v6.4.0 実装計画 — Playground 改善

作成日: 2026-05-26

---

## 実装順序

```
Phase A (WASM ビルドパイプライン) → Phase B (List 型) → Phase C (Record 型) → Phase D (サンプル更新)
```

各フェーズ完了後に `cargo test` を実行してリグレッションがないことを確認する。

---

## Phase A — WASM ビルドパイプライン整備

### A-1: `fav-wasm` クレート構造の確認

`fav/` 以下の WASM エントリポイントを確認:
- `fav/src/wasm_entry.rs`（または同等ファイル）の場所と内容
- `Cargo.toml` に `wasm32-unknown-unknown` ターゲット設定があるか確認

### A-2: `scripts/build-wasm.sh` 作成

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$REPO_ROOT/fav"
wasm-pack build --target web --out-dir "$REPO_ROOT/site/public/wasm" .
```

ビルド対象が `wasm-pack` 対応クレートであるかを確認し、
`Cargo.toml` の `[lib] crate-type = ["cdylib"]` を確認・追記。

### A-3: `scripts/deploy-site.sh` に組み込み

既存の `deploy-site.sh` の冒頭（npm build の前）に:

```bash
echo "Building WASM..."
bash "$REPO_ROOT/scripts/build-wasm.sh"
```

### 判断ポイント

`wasm-pack` がインストールされていない場合は、代替として
`cargo build --target wasm32-unknown-unknown` + `wasm-bindgen-cli` を使う。
どちらの手順でも `site/public/wasm/favnir.js` が生成されればよい。

---

## Phase B — WASM バックエンド: List 型対応

### B-1: `favnir_type_to_wasm_results` / `favnir_type_to_wasm_params` の拡張

`fav/src/backend/wasm_codegen.rs` の 2 関数に `Type::List(_)` ブランチを追加:

```rust
Type::List(_) => Ok(vec![ValType::I32]),  // heap pointer
```

### B-2: `wasm_local_for_type` の拡張

List をローカル変数として保持するケース:

```rust
Type::List(_) => {
    let idx = *next;
    *next += 1;
    Ok(WasmLocal::Single(idx))  // i32 pointer
}
```

### B-3: `build_wasm_function` — `IRExpr::RecordConstruct` の List ケース対応

List リテラル（`[1, 2, 3]` などがどう IR になるか）を確認し、
- `Nil` セル: `bump_alloc(4)` → `i32.store tag=0`
- `Cons` セル: `bump_alloc(8)` → `i32.store tag=1` + `i64.store value` + `i32.store next_ptr`

のコードを生成する。

### B-4: List 操作ビルトインの WASM 対応

`List.singleton` / `List.first` / `List.rest` / `List.is_empty` の WASM 実装。
これらは `IRExpr::Call` の callee が特定の global 名になるため、
`compile_builtin_call` の中で分岐する。

### B-5: `wasm_exec.rs` にホスト関数追加

Playground 出力のため `io_println_list_int` を追加:

```rust
linker.func_wrap("fav_host", "io_println_list_int", |mut caller: Caller<'_, ()>, ptr: i32| {
    // linked list を walk して各要素を println
    ...
});
```

### B-6: テスト追加

`wasm_codegen.rs` の `#[cfg(test)]` に:
- `wasm_list_int_roundtrip`: `List<Int>` を返す関数をコンパイル・実行
- `wasm_list_singleton_and_first`: `List.singleton` / `List.first` が正しい値を返す

---

## Phase C — WASM バックエンド: Record 型対応

### C-1: `favnir_type_to_wasm_results` に `Type::Record(_)` 追加

```rust
Type::Record(_) => Ok(vec![ValType::I32]),  // heap pointer
```

### C-2: フィールドオフセット計算ヘルパー

```rust
fn record_field_offset(fields: &[(String, Type)], target: &str) -> usize {
    fields.iter().take_while(|(name, _)| name != target).count() * 8
}
```

### C-3: `IRExpr::RecordConstruct` の codegen

```rust
// bump_alloc(fields.len() * 8) → store each field
```

### C-4: `IRExpr::FieldAccess` の codegen

```rust
// load base pointer → i64.load at computed offset
```

### C-5: テスト追加

- `wasm_record_construct_and_access`: `{x: Int, y: Int}` を構築してフィールドにアクセス

---

## Phase D — Playground サンプルコード更新

### D-1: `site/app/playground/page.tsx` の `EXAMPLE_CODE` 変更

`clamp` の例を `stage`/`seq` パイプライン例に置き換える。
`Transform(3)` → `49` が出力されること。

### D-2: 「非対応」メッセージの見直し

List/Record が対応済みになるため、Playground の非対応メッセージを
`Option` / `Result` / Sum type に絞る（またはメッセージを削除）。

### D-3: 動作確認

`fav deploy-site` を実行し、ブラウザで Playground を開いて:
1. サンプルコードが表示される
2. 型チェックがエラーなしで通る
3. 実行ボタンで `49` が出力される

---

## リスクと対策

| リスク | 対策 |
|--------|------|
| bump-alloc の境界チェックなし | Playground は小さなプログラムのみ想定。GC は v7 以降に持ち越す |
| `wasm-pack` 未インストール | 手順書に install コマンドを明記。CI でも `cargo install wasm-pack` を追加 |
| String (ptr, len) と List (ptr) の混在 | `WasmLocal` enum でそれぞれ別パターンを持つ（既存の `StringPtrLen` と同様に管理） |
| Record フィールド順序の不一致 | checker の型情報からフィールド順を取得する。IRProgram に型情報が付いているか確認 |

---

## ファイル変更一覧（予定）

| ファイル | 変更内容 |
|---------|---------|
| `fav/src/backend/wasm_codegen.rs` | List/Record 型対応、テスト追加 |
| `fav/src/backend/wasm_exec.rs` | `io_println_list_int` ホスト関数追加 |
| `scripts/build-wasm.sh` | 新規作成 |
| `scripts/deploy-site.sh` | WASM ビルド呼び出し追加 |
| `site/app/playground/page.tsx` | EXAMPLE_CODE 変更、非対応メッセージ更新 |
