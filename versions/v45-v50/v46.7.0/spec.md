# Spec: v46.7.0 — `fav explain --lineage` 2.0

Date: 2026-07-17
Status: 計画中

---

## 概要

`lineage.rs` の `LineageEntry` に `is_dead: bool` フラグを追加し、
`return` 早期脱出パスを持つステージ／fn を dead path としてマークする。
`render_lineage_mermaid_with_opts(report, show_dead)` で表示切替を可能にし、
`fav explain --lineage --show-dead` CLI フラグから呼び出せるようにする。

---

## スコープ

### 1. `LineageEntry` に `is_dead: bool` 追加（`lineage.rs`）

```rust
pub struct LineageEntry {
    pub name: String,
    pub kind: String,
    pub capability: Option<String>,
    pub effects: Vec<String>,
    pub sources: Vec<String>,
    pub sinks: Vec<String>,
    pub is_dead: bool,  // v46.7.0: true = トップレベルに Stmt::Return が存在する
}
```

**JSON 後方互換性注意**: `LineageEntry` は `#[derive(Serialize)]` を持つため、`is_dead: false` が全エントリのJSON出力に追加される。strict パーサーを使う既存ユーザーへの breaking change に注意。

### 2. ヘルパー `has_early_return(stmts: &[ast::Stmt]) -> bool`（`lineage.rs`）

トップレベル `stmts` を走査し `Stmt::Return` が 1 件以上あれば `true`。Phase 1 スコープ: ネストした if/match/for 内は対象外。

### 3. `lineage_analysis` の更新（`lineage.rs`）

- `TrfDef` ブランチの `LineageEntry` 構築に `is_dead: has_early_return(&trf.body.stmts)` を追加
- `FnDef` ブランチの `LineageEntry` 構築に `is_dead: has_early_return(&fndef.body.stmts)` を追加

### 4. `render_lineage_mermaid_with_opts`（`lineage.rs`）

```rust
pub fn render_lineage_mermaid_with_opts(report: &LineageReport, show_dead: bool) -> String
```

- `show_dead = false`: 既存 `render_lineage_mermaid` と同等
- `show_dead = true`: dead エントリに `classDef deadEntry stroke-dasharray:5 5` + `class <id> deadEntry` を付与

既存の `render_lineage_mermaid` は `render_lineage_mermaid_with_opts(report, false)` に委譲。
`sanitize_mermaid_id` は `lineage.rs` 内に既存定義あり（line 1230 付近）— 同モジュール内で直接使用可。

### 5. `main.rs` の `--show-dead` CLI フラグ追加

`main.rs` の `--lineage` パースループ（line 777）に `--show-dead` アームを追加:

```rust
"--show-dead" => { show_dead = true; i += 1; }
```

`cmd_explain_lineage(file, &format, show_dead)` に更新。

### 6. `driver.rs` の更新

- `pub use` ブロックに `render_lineage_mermaid_with_opts` を追加
- `cmd_explain_lineage(file, format)` → `cmd_explain_lineage(file, format, show_dead: bool)` に拡張
- format = `"mermaid"` かつ `show_dead = true` のとき `render_lineage_mermaid_with_opts(&report, true)` を呼ぶ
- 呼び出し元は `main.rs:800` の 1 箇所のみ

---

## 実装ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/lineage.rs` | `is_dead` フィールド追加、`has_early_return` 追加、`lineage_analysis` 更新、`render_lineage_mermaid_with_opts` 追加 |
| `fav/src/driver.rs` | 既存 `LineageEntry` リテラル（5 箇所）に `is_dead: false` 追加、`pub use` 更新、`cmd_explain_lineage` シグネチャ更新、`v467000_tests` 追加 |
| `fav/src/main.rs` | `--show-dead` フラグ追加（line 782 の match アーム）、`cmd_explain_lineage` 呼び出し更新 |
| `fav/Cargo.toml` | version → `46.7.0` |
| `CHANGELOG.md` | v46.7.0 エントリ追加 |
| `versions/current.md` | v46.7.0（3007 tests）に更新 |

---

## テスト

| テスト名 | 内容 |
|---|---|
| `lineage_return_path_is_dead` | `fn Validate(ctx: LoadCtx, ...) { return rows }` をパースし `transformations[0].is_dead == true` を確認 |
| `lineage_happy_path_active` | `fn Transform(ctx: WriteCtx, ...) { rows }` をパースし `is_dead == false` を確認。`render_lineage_mermaid_with_opts(report, true)` が `"class Transform deadEntry"` を含まないことを確認 |

**テスト数**: 3005 + 2 = **3007 tests**

---

## 完了条件

- `cargo test` 3007 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version = `46.7.0`
- `CHANGELOG.md` に v46.7.0 エントリ
- `versions/current.md` を v46.7.0（3007 tests）に更新
