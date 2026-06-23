# v22.5.0 — Pipeline Orchestration（DAG スケジューリング）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/frontend/lexer.rs` — `TokenKind::Pipeline` 追加

- [x] **事前確認**: `grep -n "Seq\|Bench\|Alias\|As," fav/src/frontend/lexer.rs | head -10` で `TokenKind::As` の位置を確認
- [x] `TokenKind` enum の `As,` の直後（`// Effect keywords` コメントの前）に `Pipeline,  // v22.5.0` を追加
- [x] lexer の keyword match 内の `"as" => TokenKind::As,` の直後に `"pipeline" => TokenKind::Pipeline,` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/ast.rs` — `PipelineStep` / `PipelineDef` + `Item::PipelineDef`

- [x] **事前確認**: `grep -n "TriggerAnnotation\|pub enum Item\|FlwDef(FlwDef)" fav/src/ast.rs | head -10` で挿入位置を確認
- [x] `TriggerAnnotation` ブロックの直後（`// ── FnDef` コメントの前）に `PipelineStep` / `PipelineDef` struct を追加（plan.md T2-1 のコードに従う）
- [x] `Item` enum の `FlwDef(FlwDef),` の直後に `PipelineDef(PipelineDef),  // v22.5.0` を追加
- [x] `ast.rs` の `Item::span()` メソッド（`UseAlias` アームの直後）に `Item::PipelineDef(pd) => &pd.span,` を追加（**exhaustive match のため必須**）
- [x] `fav/src/fmt.rs` の `fmt_item` 末尾に `Item::PipelineDef(_) => None,` を追加（**exhaustive match のため必須**）
- [x] `fav/src/middle/checker.rs` の `check_item` 末尾に `Item::PipelineDef(_) => {}` を追加（**exhaustive match のため必須**）
- [x] `fav/src/middle/compiler.rs` の Item ループに `Item::PipelineDef(_) => {}` を追加（`cargo check` でエラー位置を確認してから追加）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/frontend/parser.rs` — `parse_pipeline_step` / `parse_pipeline_def` + `parse_item` 適用

- [x] **事前確認**: `grep -n "fn parse_flw_def\b\|TokenKind::Seq\b\|parse_flw_def_or_binding" fav/src/frontend/parser.rs | head -10` で挿入位置を確認
- [x] `parse_flw_def` メソッドの直前に `parse_pipeline_step` / `parse_pipeline_def` を追加（plan.md T3-1 のコードに従う）
  - `step` / `after` はソフトキーワード（`expect_ident_name` / `peek_ident_text` を使用）
  - `after` の trailing comma 対応: `matches!(self.peek(), TokenKind::Str(_))` で次がない場合は break
- [x] `parse_item()` の `TokenKind::Seq` ブランチの**直前**に `TokenKind::Pipeline => Ok(Item::PipelineDef(self.parse_pipeline_def()?)),` を追加
- [x] `expect_ident()` に `TokenKind::Pipeline` を受け入れる arm を追加（後方互換性確保）
- [x] `parse_primary()` に `TokenKind::Pipeline` を識別子式として受け入れる arm を追加（後方互換性確保）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/driver.rs` — `build_topo_order` + `cmd_orchestrate_*` + `v225000_tests`

- [x] **事前確認**: `grep -n "pub fn cmd_deploy_trigger\|// ── v22.4.0\|v224000_tests" fav/src/driver.rs | head -5` で挿入位置を確認

#### 4-1: 関数群を追加

- [x] `cmd_deploy_trigger` の直後に `// ── v22.5.0: Pipeline Orchestration` ブロックを追加（plan.md T4-1 のコードに従う）
  - `build_topo_order` (pub(crate)) — Kahn's algorithm
  - `find_pipeline_def` (private) — prog から名前で PipelineDef を検索
  - `cmd_orchestrate_run` (pub) — dry-run サポート + ステータス JSON 保存
  - `cmd_orchestrate_status` (pub) — `.fav_orchestrate/` から最新 JSON を表示
  - `cmd_orchestrate_retry` (pub) — 単独 step 再実行（v22.5.0 はスタブ）
- [x] `chrono` が `Cargo.toml` に含まれていることを確認（`cmd_deploy` で使用済みのため追加不要）
- [x] **`HashSet<String>` を使用**: `failed: HashSet<String>` にし、`failed.insert(step.name.clone())` を使うこと（`&str` のライフタイム問題回避）

#### 4-2: `v224000_tests::version_is_22_4_0` に `#[ignore]` を追加

- [x] 完了

#### 4-3: `v225000_tests` モジュールを追加（5 テスト）

- [x] `version_is_22_5_0`
- [x] `pipeline_def_parsed` — pipeline ブロックが正しくパースされる
- [x] `pipeline_dag_topo_order` — A→B→C で `[0, 1, 2]` が返る
- [x] `pipeline_dag_cycle_detected` — A⇄B で `Err("circular...")` が返る
- [x] `changelog_has_v22_5_0`

- [x] `cargo test v225000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1864 件合格）を確認

---

### T5: `fav/src/main.rs` — `fav orchestrate` CLI 追加

- [x] **事前確認**: `grep -n "Some(\"deploy\")\|Some(\"mcp\")" fav/src/main.rs | head -5` で `Some("deploy")` ブランチの後ろを確認
- [x] `Some("deploy")` ブランチの直後に `Some("orchestrate")` ブランチを追加（plan.md T5-1 のコードに従う）
  - `orchestrate run <PipelineName> <file> [--dry-run]`
  - `orchestrate status <PipelineName>`
  - `orchestrate retry <StepName> <PipelineName> <file>`
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/Cargo.toml` + `CHANGELOG.md` + MDX

- [x] **事前確認**: `grep "\[v22.4.0\]" CHANGELOG.md` で現在の先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "22.4.0"` → `"22.5.0"` に変更
- [x] v22.5.0 エントリを `CHANGELOG.md` の先頭（v22.4.0 エントリの上）に追加
  - `pipeline` ブロック / `fav orchestrate run/status/retry` / `build_topo_order` / Kahn's algorithm を記載
- [x] `grep "\[v22.5.0\]" CHANGELOG.md` で追加確認
- [x] `site/content/docs/cli/orchestrate.mdx` を新規作成
  - `pipeline` ブロック構文と `step` / `after` の説明
  - `fav orchestrate run/status/retry` の使用例
  - ステータス JSON フォーマット
  - 将来の拡張（並列実行・リモートデプロイ v22.6+）への言及

---

## テスト一覧（v225000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_5_0` | Cargo.toml に `version = "22.5.0"` が含まれる |
| `pipeline_def_parsed` | `pipeline DailyETL { step "load" = seq Load \n step "transform" = seq Transform after "load" }` が正しくパースされる |
| `pipeline_dag_topo_order` | A→B→C で `build_topo_order` が `[0, 1, 2]` を返す |
| `pipeline_dag_cycle_detected` | A⇄B（循環）で `build_topo_order` が `Err("circular...")` を返す |
| `changelog_has_v22_5_0` | CHANGELOG.md に `[v22.5.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `TokenKind::Pipeline` が lexer に追加される
- [x] `PipelineStep` / `PipelineDef` struct が `ast.rs` に追加される
- [x] `Item::PipelineDef` が `Item` enum に追加される
- [x] `pipeline Name { step "..." = seq X after "..." }` がパースされる
- [x] `build_topo_order` が Kahn's algorithm で正しい実行順を返す
- [x] 循環依存を `Err` として検出する
- [x] `fav orchestrate run <name> <file>` が依存順に step を実行する（seq の実際の実行統合は v22.6+。v22.5.0 は stdout への step 表示のみ）（v22.5.0 はスタブ）
- [x] `fav orchestrate status <name>` が最新ステータスを表示する
- [x] `fav orchestrate retry <step> <name> <file>` が指定 step を単独実行する（スタブ）
- [x] `cargo test v225000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1864 件合格）
- [x] `CHANGELOG.md` に v22.5.0 エントリ
- [x] `site/content/docs/cli/orchestrate.mdx` 作成済み

---

## コードレビュー指摘と対応

- 後方互換性: `pipeline` キーワードが既存 Favnir コード（`use pipeline.{...}`、`seq pipeline = ...`、`5 |> pipeline`）で識別子として使用されていた → `expect_ident()` および `parse_primary()` に `TokenKind::Pipeline` を受け入れる arm を追加して解決

---

## 優先度

```
T1（lexer.rs）         ← 最初（T2/T3 の依存元）
T2（ast.rs）           ← T1 完了後
T3（parser.rs）        ← T2 完了後
T4（driver.rs）        ← T3 完了後
T5（main.rs）          ← T4 完了後
T6（Cargo + doc）      ← T5 完了後
```
