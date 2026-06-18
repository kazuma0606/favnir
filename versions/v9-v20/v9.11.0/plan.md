# Favnir v9.11.0 Plan

Date: 2026-06-02
Theme: LSP 補完 + go-to-definition 強化

---

## 作業フェーズ

```
Phase A: ビルトイン関数テーブル定義
Phase B: モジュール補完（completion.rs 拡張）
Phase C: Rune 補完（completion.rs 拡張）
Phase D: Signature help（signature.rs 新規 + mod.rs 拡張）
Phase E: 定義ジャンプ改善（definition.rs 拡張 or mod.rs）
Phase F: テスト + self-check + commit
```

---

## Phase A: ビルトイン関数テーブル

### A-1: `BuiltinFn` 構造体定義

`fav/src/lsp/completion.rs` に追加:
```rust
struct BuiltinFn {
    namespace: &'static str,
    name: &'static str,
    signature: &'static str,
    params: &'static [&'static str],
}
```

### A-2: テーブル定義（`BUILTIN_FNS`）

主要 namespace の関数を列挙:

**List**: `map` / `filter` / `reduce` / `first` / `last` / `length` / `append` / `concat` / `contains` / `find` / `partition` / `zip_with` / `flat_map` / `chunk` / `take_while` / `drop_while` / `unique` / `group_by` / `count` / `sum` / `min` / `max` / `sort_by` / `reverse`

**String**: `length` / `split` / `join` / `contains` / `trim` / `trim_start` / `trim_end` / `starts_with` / `ends_with` / `replace` / `to_upper` / `to_lower` / `pad_left` / `pad_right` / `truncate` / `repeat` / `slice` / `index_of`

**Map**: `empty` / `insert` / `get` / `remove` / `keys` / `values` / `contains_key` / `size` / `map_values` / `filter` / `merge_with` / `from_list` / `to_list`

**Result**: `ok` / `err` / `map` / `map_err` / `and_then` / `is_ok` / `is_err` / `unwrap_or` / `all`

**Option**: `some` / `none` / `map` / `and_then` / `unwrap_or` / `is_some` / `is_none`

**IO**: `println` / `print` / `read_line` / `read_file` / `write_file` / `append_file` / `file_exists` / `now_ms`

**Json / Csv / Gen / Http / Llm / DB / AWS / Env / Debug / Schema / T / Float / Int**: 各 namespace の主要関数

---

## Phase B: モジュール補完

### B-1: `module_completions(ns: &str) -> Vec<CompletionItem>`

- `BUILTIN_FNS` を ns でフィルタ
- 各関数を `CompletionItem { label: name, detail: signature, kind: Function }` に変換

### B-2: `get_completions` 関数の拡張

現状: `.` トリガー時に `field_completions` を呼ぶ。
追加: カーソル前のトークンがビルトイン namespace と一致する場合 `module_completions` を追加。

判定ロジック:
```
1. カーソル位置の前のトークン列を取得
2. `.` の直前にある識別子を取り出す
3. その識別子が BUILTIN_NAMESPACES に含まれれば module_completions を使う
4. それ以外は既存の field_completions
```

---

## Phase C: Rune 補完

### C-1: `KNOWN_RUNES` 定数

```rust
const KNOWN_RUNES: &[(&str, &str)] = &[
    ("aws",      "AWS S3/SQS/DynamoDB !AWS"),
    ("cache",    "Cache operations !Cache"),
    ("csv",      "CSV read/write !Io"),
    ("db",       "SQL database !Db"),
    ("email",    "Email sending !Io"),
    ("fs",       "Filesystem operations !Io"),
    ("gen",      "UUID/NanoId generation !Gen"),
    ("graphql",  "GraphQL client !Http"),
    ("grpc",     "gRPC client !Http"),
    ("http",     "HTTP client !Http"),
    ("json",     "JSON encode/decode"),
    ("llm",      "LLM (Claude/OpenAI) !Llm"),
    ("queue",    "Message queue !Queue"),
    ("slack",    "Slack messaging !Io"),
    ("sql",      "SQL query builder !Db"),
];
```

### C-2: `rune_completions() -> Vec<CompletionItem>`

各 Rune を `CompletionItem { label, detail, kind: Module }` に変換。

### C-3: トリガー判定

`get_completions` でカーソル前テキストが `import rune "` パターンにマッチする場合、
`rune_completions()` を返す。

---

## Phase D: Signature Help

### D-1: `fav/src/lsp/signature.rs` 新規作成

```rust
pub fn get_signature_help(
    src: &str,
    position: Position,
    program: &Program,
) -> Option<SignatureHelpResult>
```

ロジック:
1. カーソル位置からソースを逆スキャンして開いている `(` を探す
2. `(` の前の識別子を取り出す（`foo(` → `foo`、`List.map(` → `List` + `map`）
3. ユーザー定義: `program` の fn/stage テーブルからシグネチャを取得
4. ビルトイン: `BUILTIN_FNS` テーブルから取得
5. `,` の数をカウントして `activeParameter` を決定

### D-2: `mod.rs` — `textDocument/signatureHelp` ハンドラ追加

```rust
"textDocument/signatureHelp" => {
    let params: TextDocumentPositionParams = serde_json::from_value(params)?;
    handle_signature_help(&state, params)
}
```

### D-3: `mod.rs` — `initialize` に `signatureHelpProvider` 追加

```json
"signatureHelpProvider": {
  "triggerCharacters": ["(", ","]
}
```

---

## Phase E: 定義ジャンプ改善

### E-1: Rune 関数ジャンプ

現状の `textDocument/definition` は `program.fn_defs` のみを参照。

追加:
- カーソル下が `<ident>.<fn>` パターンかつ `ident` が KNOWN_RUNES のキーなら
  `runes/<ident>/<ident>.fav` を検索して該当 `fn <fn>` の行を返す

### E-2: seq 内 stage 名ジャンプ

`seq` 定義内の `|>` チェーン中にある識別子を stage 定義へジャンプ。
現状の定義ジャンプロジックを `stage_defs` にも拡張。

---

## Phase F: テスト + self-check + commit

### F-1: `v9110_tests` モジュール（`driver.rs` または `lsp/mod.rs`）

5 件以上のユニットテスト:
- `F-1a: module_completion_list` — `List.` → `map` / `filter` 等が含まれる
- `F-1b: module_completion_string` — `String.` → `split` / `trim` 等が含まれる
- `F-1c: rune_completion_returns_known_runes` — `import rune "` → http / csv 等が含まれる
- `F-1d: signature_help_builtin` — `List.map(` → `activeParameter: 0` のシグネチャ
- `F-1e: signature_help_comma` — `List.map(f,` → `activeParameter: 1`

### F-2: `cargo test v9110` — 5 件通過確認

### F-3: `cargo test checker_fav_wire_self_check` — 通過確認

### F-4: `cargo test bootstrap` — 通過確認

### F-5: `cargo test` — 全件通過確認

### F-6: `fav/Cargo.toml` version → `"9.11.0"`

### F-7: `fav/self/cli.fav` の `run_version` → `"9.11.0"`

### F-8: `versions/v9.11.0/tasks.md` 完了チェック

### F-9: `memory/MEMORY.md` に v9.11.0 完了を記録

### F-10: commit

---

## 依存関係

```
A → B → F
A → C → F
A → D (D-1) → D (D-2, D-3) → F
E → F
```

Phase A, E は並行可能。B, C, D は A 完了後に並行実装可能。

---

## ファイル変更一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/lsp/completion.rs` | 変更（モジュール補完・Rune 補完追加） |
| `fav/src/lsp/mod.rs` | 変更（signatureHelp ハンドラ + capability 追加） |
| `fav/src/lsp/signature.rs` | 新規（Signature help ロジック） |
| `fav/src/driver.rs` | 変更（v9110_tests 追加） |
| `fav/self/cli.fav` | 変更（run_version 更新） |
| `fav/Cargo.toml` | 変更（version 更新） |
