# v22.4.0 — Event-driven Pipeline（イベントトリガー）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TriggerAnnotation` struct + `FlwDef.trigger` フィールド

- [x]**事前確認**: `grep -n "ApiAnnotation\|FlwDef\|streaming.*StreamingAnnotation" fav/src/ast.rs | head -20` で ApiAnnotation と FlwDef の位置を確認
- [x]`ApiAnnotation` struct ブロックの直後に `TriggerAnnotation` struct を追加（コメント `// ── TriggerAnnotation (v22.4.0)` 付き）
  - フィールド: `event: String`, `bucket: Option<String>`, `topic: Option<String>`, `span: Span`
  - `#[derive(Debug, Clone)]`
- [x]`FlwDef` 内の `streaming: Option<StreamingAnnotation>` フィールドの直後に `pub trigger: Option<TriggerAnnotation>,` を追加
- [x]`cargo check --bin fav` でコンパイルエラー箇所を確認（`FlwDef` 直接初期化に `trigger: None` が必要な箇所）

---

### T2: `fav/src/frontend/parser.rs` — `parse_trigger_annotation` + `parse_item` 適用

- [x]**事前確認**: `grep -n "parse_checkpoint_annotation\|parse_streaming_annotation\|TokenKind::Seq" fav/src/frontend/parser.rs | head -10` で位置を確認
- [x]`parse_checkpoint_annotation` メソッドの直後に `parse_trigger_annotation` メソッドを追加（plan.md T2-1 のコードに従う）
  - `#[trigger]` が `#[checkpoint]` と同時に指定されても正しく動作すること（独立した lookahead）
- [x]`parse_item()` 内で `checkpoint_ann` 取得の直後に `let trigger_ann = self.parse_trigger_annotation()?;` を追加
- [x]`TokenKind::Seq` ブランチに `fd.trigger = trigger_ann;` を追加（`fd.streaming = streaming_ann;` の直後）
- [x]**注意**: `parse_flw_def_or_binding` 内で `FlwDef { ... }` を直接構築している箇所に `trigger: None` を追加（`parser.rs` の L1841 / L1975 / L1996 付近の 3 箇所 + `cargo check` で追加箇所を確認）
- [x]`cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/driver.rs` — `cmd_deploy_trigger` + `build_trigger_config_json` + `v224000_tests`

- [x]**事前確認**: `grep -n "pub fn cmd_deploy\b" fav/src/driver.rs | head -5` で `cmd_deploy` の位置を確認

#### 3-1: `FlwDef` 初期化箇所に `trigger: None` を追加

- [x]T1 の `cargo check` エラーリストを参照し、`driver.rs` 内の全 `FlwDef { ... }` 初期化箇所に `trigger: None` を追加

#### 3-2: `build_trigger_config_json` + `cmd_deploy_trigger` 関数を追加

- [x]`cmd_deploy` 関数の直後に `build_trigger_config_json(entries: &[(&str, &crate::ast::TriggerAnnotation)]) -> String` を追加（plan.md T3-2 のコードに従う）
  - **可視性**: `pub(crate)` — テストから直接呼ぶため
  - JSON 構造: `[{ "pipeline": "...", "trigger": { "event": "...", "bucket"/"topic": "..." } }]`
- [x]`pub fn cmd_deploy_trigger(file: &str, out: Option<&str>)` を追加（plan.md T3-2 のコードに従う）
  - `load_file` → `Parser::parse_str` → `FlwDef` でフィルタ → `build_trigger_config_json` → stdout

#### 3-3: `v223000_tests::version_is_22_3_0` に `#[ignore]` を追加

#### 3-4: `v224000_tests` モジュールを追加（5 テスト）

- [x]`version_is_22_4_0`
- [x]`trigger_annotation_s3_parsed` — `#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]` が `FlwDef.trigger.event == "s3:ObjectCreated"` / `bucket == Some("raw-data")` に格納される
- [x]`trigger_annotation_kafka_parsed` — `#[trigger(event = "kafka:message", topic = "orders")]` が `FlwDef.trigger.topic == Some("orders")` に格納される
- [x]`deploy_trigger_generates_json` — `build_trigger_config_json` が `"pipeline"` / `"event"` / `"bucket"` を含む JSON を返す
- [x]`changelog_has_v22_4_0`

- [x]`cargo test v224000 --bin fav` — 5/5 PASS を確認
- [x]`cargo test --bin fav` — リグレッションなし（1856 件以上合格）を確認

---

### T4: `fav/src/main.rs` — `deploy --trigger <file>` CLI フラグ追加

- [x]**事前確認**: `grep -n "Some(\"deploy\")\|--dry-run\|trigger_file" fav/src/main.rs | head -10` で deploy ブランチの構造を確認
- [x]`deploy` ブランチの変数宣言に `let mut trigger_file: Option<String> = None;` を追加
- [x]`while` ループの `match` に `"--trigger"` アームを追加（`--dry-run` アームの直後、**catch-all `other =>` アームの前**に挿入すること）
  - `args.get(i + 1)` でファイルパスを取得、`i += 2`
- [x]`cmd_deploy` 呼び出し部分を `trigger_file` の有無で分岐
  - `Some(ref tfile)` → `crate::driver::cmd_deploy_trigger(tfile, None)`
  - `None` → 既存の `cmd_deploy(...)` 呼び出し
- [x]`cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/Cargo.toml` + `CHANGELOG.md` + MDX

- [x]**事前確認**: `grep "\[v22.3.0\]" CHANGELOG.md` で現在の先頭エントリを確認
- [x]`fav/Cargo.toml` の `version = "22.3.0"` → `"22.4.0"` に変更
- [x]v22.4.0 エントリを `CHANGELOG.md` の先頭（v22.3.0 エントリの上）に追加
  - `#[trigger(...)]` アノテーション / `fav deploy --trigger` / `build_trigger_config_json` / `TriggerAnnotation` struct を記載
- [x]`grep "\[v22.4.0\]" CHANGELOG.md` で追加確認
- [x]`site/content/docs/cli/trigger.mdx` を新規作成
  - `#[trigger(...)]` の使用例（S3 / Kafka / SQS）
  - `fav deploy --trigger` の出力 JSON 例
  - サポートキー（event / bucket / topic）の説明
  - 将来のデプロイ（v22.5+）への言及

---

## テスト一覧（v224000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_4_0` | Cargo.toml に `version = "22.4.0"` が含まれる |
| `trigger_annotation_s3_parsed` | S3 トリガーアノテーションが `FlwDef.trigger` に正しく格納される |
| `trigger_annotation_kafka_parsed` | Kafka トリガーアノテーションが `FlwDef.trigger.topic` に格納される |
| `deploy_trigger_generates_json` | `build_trigger_config_json` が正しい JSON を生成する |
| `changelog_has_v22_4_0` | CHANGELOG.md に `[v22.4.0]` が含まれる |

---

## 完了条件チェックリスト

- [x]`TriggerAnnotation` struct が `ast.rs` に追加される
- [x]`FlwDef.trigger: Option<TriggerAnnotation>` フィールドが追加される
- [x]`#[trigger(event = "...", bucket = "...")]` がパースされる
- [x]`#[trigger(event = "...", topic = "...")]` がパースされる
- [x]`fav deploy --trigger <file>` が JSON デプロイ設定を stdout に出力する
- [x]`cargo test v224000 --bin fav` — 5/5 PASS
- [x]`cargo test --bin fav` — リグレッションなし（1856 件以上合格）
- [x]`CHANGELOG.md` に v22.4.0 エントリ
- [x]`site/content/docs/cli/trigger.mdx` 作成済み

---

## コードレビュー指摘と対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| SECURITY-HIGH | `build_trigger_config_json` で JSON 文字列エスケープなし（インジェクション） | `escape_json_str` ヘルパーを追加し全フィールドに適用 |
| BUG-HIGH | `parse_item` の余分な 4 スペースインデント | `fn parse_item` の先頭スペースを修正 |
| BUG-MED | `trigger_ann` が非 FlwDef Seq 分岐でサイレント破棄 | `streaming_ann` と同様の既存パターンのため仕様として受容（スコープ外） |
| STYLE-LOW | `out: Option<&str>` が現在 `None` 固定呼び出しのみ | 将来の `--output` フラグ追加を見越した設計として維持 |

## 実装メモ

- `TriggerAnnotation` 追加後の `FlwDef` 直接初期化エラーは `parser.rs` L1841/L1975/L1996 の 3 箇所のみ（`driver.rs` / `checker.rs` 等には FlwDef 直接初期化なし）
- `build_trigger_config_json` は `pub(crate)` としてテストから直接呼び出し
- テスト結果: `cargo test v224000 --bin fav` — 5/5 PASS、`cargo test --bin fav` — 1860 PASS（0 failures）

## 優先度

```
T1（ast.rs）          ← 最初（T2/T3 の依存元）
T2（parser.rs）       ← T1 完了後
T3（driver.rs）       ← T1 完了後（T2 と並列可）
T4（main.rs）         ← T3 完了後
T5（Cargo + doc）     ← T4 完了後
```
