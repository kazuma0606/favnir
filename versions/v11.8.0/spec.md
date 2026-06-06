# Favnir v11.8.0 仕様書

作成日: 2026-06-06
テーマ: `fav transpile` CLI 完成 + checker 統合

---

## 背景と目的

v11.7.0 で uv 統合が完了した。
v11.8.0 では `fav transpile` の品質を高める 2 つの機能を追加する:

1. **型チェック統合** — `checker.fav` による型チェックを transpile 前に実行し、
   型エラーがあれば Python 生成をブロックする
2. **リネージコメント付与** — `--lineage` フラグで生成 Python 関数に
   エフェクト・ソース・シンク情報をコメントとして付与する

---

## 型チェック統合

### フロー

```
fav transpile --target python <file.fav>
    ↓
1. Parser::parse_str (既存)
    ↓
2. check_source_str → checker.fav 実行 (NEW)
    ↓ エラーがあれば exit 1 (Python 生成ブロック)
3. emit_python (既存)
    ↓
4. ファイル書き込み (既存)
```

### `--no-check` フラグ

```
fav transpile --target python <file.fav> --no-check
```

型チェックをスキップする（既存コードが型エラーを含む場合のエスケープハッチ）。

### エラー表示

`check_source_str(src)` がエラーを返した場合、`format_diagnostic` で表示して exit 1:

```
error[E0315]: Postgres.* call requires `!Postgres` effect on enclosing fn/stage
  --> <file.fav>:3:3
  |
3 |   Postgres.execute_raw(sql, "[]")
  |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

---

## リネージコメント付与（`--lineage` フラグ）

### フラグ

```
fav transpile --target python <file.fav> --lineage
```

### 出力例

```python
# [lineage] effects: !Postgres | sources: orders | sinks: summary
def fetch_orders(sql: str) -> Any:
    ...

# [lineage] effects: !IO | sources: - | sinks: -
def print_result(msg: str) -> None:
    ...
```

### 実装方針

`emit_python.rs` の `Emitter` に `lineage_comments: HashMap<String, String>` を追加。
`emit_fn_def` / `emit_trf_def` の冒頭でコメントを挿入する。

コメント文字列は `cmd_transpile` 側で `lineage_analysis(prog)` から構築し
`emit_python_with_lineage(prog, source_path, comments)` に渡す。

### コメント生成ロジック

```rust
fn build_lineage_comments(report: &LineageReport) -> HashMap<String, String> {
    report.transformations.iter().map(|entry| {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.join(", ")
        };
        let sources = if entry.sources.is_empty() { "-".to_string() }
                      else { entry.sources.join(", ") };
        let sinks   = if entry.sinks.is_empty()   { "-".to_string() }
                      else { entry.sinks.join(", ") };
        let comment = format!(
            "# [lineage] effects: {} | sources: {} | sinks: {}",
            effects, sources, sinks
        );
        (entry.name.clone(), comment)
    }).collect()
}
```

---

## emit_python.rs の変更

### Emitter 新フィールド

```rust
lineage_comments: std::collections::HashMap<String, String>,
```

### 新 API

```rust
pub fn emit_python_with_lineage(
    prog: &Program,
    source_path: &str,
    comments: std::collections::HashMap<String, String>,
) -> String {
    let mut e = Emitter::new();
    e.lineage_comments = comments;
    e.emit_program(prog, source_path)
}
```

### `emit_fn_def` / `emit_trf_def` コメント挿入

```rust
fn emit_fn_def(&mut self, fd: &FnDef) {
    if let Some(comment) = self.lineage_comments.get(&fd.name) {
        self.line(comment);
    }
    // ...既存コード...
}
```

---

## テスト設計（v11800_tests）

| テスト名 | 検証内容 |
|---|---|
| `transpile_blocks_on_type_error` | 型エラーありの Fav コードを check_source_str が検出する |
| `transpile_type_check_passes_valid` | 正常な Fav コードで check_source_str がエラーなし |
| `transpile_lineage_comment_effects` | `--lineage` で `# [lineage] effects:` コメントが付く |
| `transpile_lineage_comment_pure_fn` | エフェクトなし fn で `# [lineage] effects: Pure` が付く |
| `transpile_no_check_skips_error` | `--no-check` フラグで型エラーコードも Python 生成される |
| `transpile_lineage_postgres_fn` | `!Postgres` fn に `!Postgres` エフェクトコメントが付く |

---

## バージョン更新

- `fav/Cargo.toml`: `version = "11.8.0"`
