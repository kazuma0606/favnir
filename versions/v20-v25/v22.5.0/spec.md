# v22.5.0 仕様書 — Pipeline Orchestration（DAG スケジューリング）

## 概要

Airflow / Prefect 等の外部ツールを使わず、Favnir 自体でパイプライン間の依存を管理・実行できる
`pipeline` ブロックを導入する。`after` キーワードで step 間の依存を宣言し、
`fav orchestrate run` で依存順に自動実行する。

v22.5.0 は **構文解析・DAG トポロジカルソート・逐次実行・ステータス JSON 出力**を実装する。
Lambda / ECS へのリモートデプロイは v22.8+ で対応する。

**テーマ**: 「パイプライン依存管理を型システムに統合する」

---

## ロードマップ完了条件との対応

v22.5.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第五弾。
v23.0 の完了条件⑤「`fav orchestrate` で multi-step DAG が依存順に実行できる」を実装する。

---

## 機能仕様

### `pipeline` ブロック構文

`seq` 宣言に名前を付け、`after` で実行順序の依存を宣言する。

```favnir
pipeline DailyETL {
  step "load_raw"   = seq LoadRaw
  step "transform"  = seq Transform  after "load_raw"
  step "enrich"     = seq Enrich     after "transform"
  step "write"      = seq Write      after "enrich", "load_meta"
  step "load_meta"  = seq LoadMeta
}
```

#### 構文ルール

- `pipeline <Name> { ... }` — `pipeline` は予約語（`TokenKind::Pipeline`）
- `step "<name>" = seq <SeqName>` — step 名は文字列リテラル、seq 名は識別子
- `after "<dep1>", "<dep2>"` — `after` はソフトキーワード（識別子として lexing）
- 依存なし step は任意の順序で実行開始できる（並列化は v22.6+ 対応予定）
- v22.5.0 では逐次実行（トポロジカル順）のみ

### `fav orchestrate` コマンド

> **注意**: ロードマップの `fav orchestrate status` / `fav orchestrate retry "enrich"` は引数省略の概略形。spec.md の以下のシグネチャが正式仕様。

```bash
# pipeline を依存順に実行
fav orchestrate run DailyETL pipeline.fav
fav orchestrate run DailyETL pipeline.fav --dry-run

# 最後の実行ステータスを表示
fav orchestrate status DailyETL

# 特定 step のみ再実行（前後の依存は skip）
fav orchestrate retry "enrich" DailyETL pipeline.fav
```

#### `orchestrate run` の動作

1. `.fav` ファイルを解析して `pipeline <Name>` を探す
2. DAG のトポロジカルソートを実行（循環依存を検出したらエラー）
3. step を依存順に逐次実行（`cmd_run` を step ごとに呼ぶ）
4. 実行結果を `.fav_orchestrate/<Name>_<timestamp>.json` に保存

#### ステータス JSON フォーマット

```json
{
  "pipeline": "DailyETL",
  "run_at": "2026-06-21T12:00:00Z",
  "steps": [
    { "name": "load_raw",  "seq": "LoadRaw",    "status": "ok",     "elapsed_ms": 120 },
    { "name": "transform", "seq": "Transform",  "status": "ok",     "elapsed_ms": 340 },
    { "name": "enrich",    "seq": "Enrich",     "status": "failed", "elapsed_ms": 50  },
    { "name": "write",     "seq": "Write",      "status": "skip",   "elapsed_ms": 0   }
  ]
}
```

status 値: `"ok"` / `"failed"` / `"skip"`（依存 step が失敗した場合 skip）/ `"pending"`

---

## アーキテクチャ

### `PipelineStep` / `PipelineDef` struct（`ast.rs`）

```rust
/// v22.5.0: A single step in a pipeline DAG.
#[derive(Debug, Clone)]
pub struct PipelineStep {
    pub name: String,        // "load_raw"
    pub seq_name: String,    // "LoadRaw" — seq 宣言名
    pub after: Vec<String>,  // 依存 step 名のリスト
    pub span: Span,
}

/// v22.5.0: `pipeline Name { step ... }` block.
#[derive(Debug, Clone)]
pub struct PipelineDef {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    pub span: Span,
}
```

`Item::PipelineDef(PipelineDef)` を `Item` enum に追加する。

### レキサー変更（`frontend/lexer.rs`）

`TokenKind::Pipeline` を追加し、`"pipeline"` キーワードをマップする。

```rust
// TokenKind enum に追加
Pipeline,

// keyword match に追加
"pipeline" => TokenKind::Pipeline,
```

### パーサー変更（`frontend/parser.rs`）

`parse_pipeline_def()` メソッドを追加:

```rust
fn parse_pipeline_def(&mut self) -> Result<PipelineDef, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Pipeline)?;
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::LBrace)?;
    let mut steps = Vec::new();
    while self.peek() != &TokenKind::RBrace && !self.at_end() {
        steps.push(self.parse_pipeline_step()?);
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(PipelineDef { name, steps, span: self.span_from(&start) })
}

fn parse_pipeline_step(&mut self) -> Result<PipelineStep, ParseError> {
    let start = self.peek_span().clone();
    // step "<name>"
    self.expect_ident_name("step")?;
    let name = self.expect_str()?;
    self.expect(&TokenKind::Eq)?;
    // seq <SeqName>
    self.expect(&TokenKind::Seq)?;
    let (seq_name, _) = self.expect_ident()?;
    // optional: after "<dep1>", "<dep2>"
    let mut after = Vec::new();
    if self.peek_ident_text("after") {
        self.advance(); // consume "after"
        let dep = self.expect_str()?;
        after.push(dep);
        while self.peek() == &TokenKind::Comma {
            self.advance();
            if !matches!(self.peek(), TokenKind::Str(_)) { break; }
            after.push(self.expect_str()?);
        }
    }
    Ok(PipelineStep { name, seq_name, after, span: self.span_from(&start) })
}
```

`parse_item()` に `TokenKind::Pipeline` ブランチを追加:

```rust
TokenKind::Pipeline => Ok(Item::PipelineDef(self.parse_pipeline_def()?)),
```

### `build_topo_order` + `cmd_orchestrate_*`（`driver.rs`）

```rust
/// Kahn's algorithm でトポロジカルソート順を返す。循環依存がある場合 Err。
pub(crate) fn build_topo_order(steps: &[crate::ast::PipelineStep]) -> Result<Vec<usize>, String> {
    // name → index map
    // in-degree と adjacency list を構築
    // BFS (Kahn) でソート
    // 処理済み数 != steps.len() なら cycle detected
}

pub fn cmd_orchestrate_run(file: &str, pipeline_name: &str, dry_run: bool) {
    // load_file → Parser::parse_str → find PipelineDef by name
    // build_topo_order → 依存順に step を実行
    // dry_run: 実行せず実行順を表示のみ
    // ステータス JSON を .fav_orchestrate/ に保存
}

pub fn cmd_orchestrate_status(pipeline_name: &str) {
    // .fav_orchestrate/<name>_*.json を最新 1 件読み込んで表示
}

pub fn cmd_orchestrate_retry(step_name: &str, file: &str, pipeline_name: &str) {
    // 指定 step のみ実行（依存チェックをスキップ）
}
```

### CLI（`main.rs`）

```
fav orchestrate run   <PipelineName> <file> [--dry-run]
fav orchestrate status <PipelineName>
fav orchestrate retry  <StepName> <PipelineName> <file>
```

```rust
Some("orchestrate") => match args.get(2).map(|s| s.as_str()) {
    Some("run")    => { /* parse args → cmd_orchestrate_run */ }
    Some("status") => { /* parse args → cmd_orchestrate_status */ }
    Some("retry")  => { /* parse args → cmd_orchestrate_retry */ }
    _ => { eprintln!("..."); process::exit(1); }
}
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/frontend/lexer.rs` | 更新 | `TokenKind::Pipeline` 追加 |
| `fav/src/ast.rs` | 更新 | `PipelineStep` / `PipelineDef` struct + `Item::PipelineDef` + `Item::span()` への `PipelineDef` アーム追加 |
| `fav/src/fmt.rs` | 更新 | `fmt_item` の exhaustive match に `Item::PipelineDef` アーム追加（スタブ） |
| `fav/src/middle/checker.rs` | 更新 | `check_item` の exhaustive match に `Item::PipelineDef => {}` 追加 |
| `fav/src/middle/compiler.rs` | 更新 | Item ループに `Item::PipelineDef` アーム追加（スタブ） |
| `fav/src/frontend/parser.rs` | 更新 | `parse_pipeline_def` / `parse_pipeline_step` + `parse_item` 適用 |
| `fav/src/driver.rs` | 更新 | `build_topo_order` / `cmd_orchestrate_run` / `cmd_orchestrate_status` / `cmd_orchestrate_retry` / `v225000_tests` 5 件 |
| `fav/src/main.rs` | 更新 | `fav orchestrate run/status/retry` CLI 追加 |
| `fav/Cargo.toml` | 更新 | `version = "22.4.0"` → `"22.5.0"` |
| `CHANGELOG.md` | 更新 | v22.5.0 エントリ追加 |
| `site/content/docs/cli/orchestrate.mdx` | 新規 | `fav orchestrate` ドキュメント |

---

## テスト一覧（v225000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_5_0` | Cargo.toml に `version = "22.5.0"` が含まれる |
| `pipeline_def_parsed` | `pipeline DailyETL { step "load" = seq Load \n step "transform" = seq Transform after "load" }` が正しくパースされる |
| `pipeline_dag_topo_order` | 3 step DAG（A→B→C）で `build_topo_order` が `[0, 1, 2]` を返す |
| `pipeline_dag_cycle_detected` | A depends on B、B depends on A で `build_topo_order` が `Err` を返す |
| `changelog_has_v22_5_0` | CHANGELOG.md に `[v22.5.0]` が含まれる |

---

## スコープ外（v22.5.0 では実装しない）

- step の並列実行（トポロジカルレイヤー内の step を同時実行）
- Lambda / ECS へのリモートデプロイ
- `fav.toml` への pipeline 定義埋め込み
- step タイムアウト / リトライ（v22.6.0 で実装）
- Web UI / ダッシュボード

---

## 完了条件

- [ ] `TokenKind::Pipeline` が lexer に追加される
- [ ] `PipelineStep` / `PipelineDef` struct が `ast.rs` に追加される
- [ ] `Item::PipelineDef` が `Item` enum に追加される
- [ ] `pipeline Name { step "..." = seq X after "..." }` がパースされる
- [ ] `build_topo_order` が Kahn's algorithm で正しい実行順を返す
- [ ] 循環依存を `Err` として検出する
- [ ] `fav orchestrate run <name> <file>` が依存順に step を実行する（seq の実際の実行統合は v22.6+。v22.5.0 は stdout への step 表示のみ）
- [ ] `fav orchestrate status <name>` が最新ステータスを表示する
- [ ] `fav orchestrate retry <step> <name> <file>` が指定 step を単独実行する
- [ ] `cargo test v225000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1860 件以上合格）
- [ ] `CHANGELOG.md` に v22.5.0 エントリ
- [ ] `site/content/docs/cli/orchestrate.mdx` 作成済み
