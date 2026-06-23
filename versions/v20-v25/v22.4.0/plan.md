# v22.4.0 実装計画 — Event-driven Pipeline（イベントトリガー）

## 実装順序

```
T1（ast.rs）          ← 最初（T2/T3/T5 の依存元）
T2（parser.rs）       ← T1 完了後
T3（driver.rs）       ← T1 完了後（T2 と並列可）
T4（main.rs）         ← T3 完了後
T5（Cargo + doc）     ← T4 完了後
```

---

## T1: `fav/src/ast.rs` — `TriggerAnnotation` + `FlwDef.trigger`

### 事前確認コマンド

```bash
grep -n "ApiAnnotation\|FlwDef\|streaming.*StreamingAnnotation\|pub span" fav/src/ast.rs | head -20
```

### 1-1: `TriggerAnnotation` struct 追加

`ApiAnnotation` struct の直後（L570 付近）に追加:

```rust
// ── TriggerAnnotation (v22.4.0) ───────────────────────────────────────────────

/// `#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]` annotation on seq definitions.
#[derive(Debug, Clone)]
pub struct TriggerAnnotation {
    /// イベント種別: "s3:ObjectCreated", "kafka:message", "sqs:message" など
    pub event: String,
    /// S3 バケット名（s3:* イベント用）
    pub bucket: Option<String>,
    /// Kafka / SQS トピック / キュー名（kafka:* / sqs:* イベント用）
    pub topic: Option<String>,
    pub span: Span,
}
```

### 1-2: `FlwDef.trigger` フィールド追加

`FlwDef` 内の `streaming` フィールドの直後に追加:

```rust
    /// v19.1.0: `#[streaming]` annotation — enables chunk-based evaluation.
    pub streaming: Option<StreamingAnnotation>,
    /// v22.4.0: `#[trigger(...)]` annotation — event-driven pipeline trigger.
    pub trigger: Option<TriggerAnnotation>,
    pub span: Span,
```

### 確認

```bash
cargo check --bin fav
# FlwDef を直接初期化している箇所に trigger: None を追加するよう cargo check で確認
```

---

## T2: `fav/src/frontend/parser.rs` — `parse_trigger_annotation` + `parse_item` 適用

### 事前確認コマンド

```bash
grep -n "parse_checkpoint_annotation\|parse_streaming_annotation\|streaming_ann\|TokenKind::Seq" fav/src/frontend/parser.rs | head -10
```

### 2-1: `parse_trigger_annotation` メソッドを追加

`parse_checkpoint_annotation` の直後に追加:

```rust
/// v22.4.0: parse optional `#[trigger(event = "...", bucket/topic = "...")]` annotation on seq.
fn parse_trigger_annotation(&mut self) -> Result<Option<crate::ast::TriggerAnnotation>, ParseError> {
    // Lookahead: # [ trigger
    let is_trigger = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "trigger"));
    if !is_trigger {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance(); // #
    self.expect(&TokenKind::LBracket)?;
    self.expect_ident_name("trigger")?;
    self.expect(&TokenKind::LParen)?;
    // event = "..."
    self.expect_ident_name("event")?;
    self.expect(&TokenKind::Eq)?;
    let event = self.expect_str()?;
    // optional: , bucket = "..." OR , topic = "..."
    let mut bucket: Option<String> = None;
    let mut topic: Option<String> = None;
    while self.peek() == &TokenKind::Comma {
        self.advance(); // ,
        if self.peek() == &TokenKind::RParen { break; } // trailing comma
        let (key, _) = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let val = self.expect_str()?;
        match key.as_str() {
            "bucket" => bucket = Some(val),
            "topic"  => topic  = Some(val),
            other    => return Err(ParseError::new(
                format!("unknown trigger key `{}`; expected `bucket` or `topic`", other),
                self.peek_span().clone(),
            )),
        }
    }
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(crate::ast::TriggerAnnotation {
        event,
        bucket,
        topic,
        span: self.span_from(&start),
    }))
}
```

### 2-2: `parse_item()` で呼び出し

`checkpoint_ann` の直後に追加:

```rust
        // v22.1.0: parse optional #[checkpoint] annotation
        let checkpoint_ann = self.parse_checkpoint_annotation()?;
        // v22.4.0: parse optional #[trigger(...)] annotation
        let trigger_ann = self.parse_trigger_annotation()?;
```

`TokenKind::Seq` ブランチに `trigger_ann` の適用を追加:

```rust
            TokenKind::Seq => {
                let item = self.parse_flw_def_or_binding(vis)?;
                Ok(match item {
                    Item::FlwDef(mut fd) => {
                        fd.streaming = streaming_ann;
                        fd.trigger = trigger_ann;  // v22.4.0
                        Item::FlwDef(fd)
                    }
                    other => other,
                })
            }
```

### 確認

```bash
cargo check --bin fav
```

---

## T3: `fav/src/driver.rs` — `cmd_deploy_trigger` + `build_trigger_config_json` + `v224000_tests`

### 事前確認コマンド

```bash
grep -n "pub fn cmd_deploy\b\|fn build_trigger\|v223000_tests" fav/src/driver.rs | head -10
```

### 3-1: `FlwDef` 直接初期化に `trigger: None` 追加

`cargo check` でエラーが出た箇所に `trigger: None` を追加する。

**主な修正箇所（parser.rs）**: `parse_flw_def_or_binding` 内の `FlwDef { ... }` 直接構築箇所が 3 箇所ある:
- `parser.rs` L1841 付近（通常の FlwDef 構築）
- `parser.rs` L1975 付近（binding 側の FlwDef）
- `parser.rs` L1996 付近（別パス）

これら 3 箇所すべてに `trigger: None,` を追加する。`cargo check` でその他の箇所も確認すること。

### 3-2: `build_trigger_config_json` 関数を追加

`cmd_deploy` 関数の直後に追加:

```rust
// ── v22.4.0: Event-driven trigger deployment ──────────────────────────────────

fn build_trigger_config_json(entries: &[(&str, &crate::ast::TriggerAnnotation)]) -> String {
    let items: Vec<String> = entries.iter().map(|(name, t)| {
        let extra = match (t.bucket.as_deref(), t.topic.as_deref()) {
            (Some(b), _) => format!(",\n      \"bucket\": \"{}\"", b),
            (_, Some(tp)) => format!(",\n      \"topic\": \"{}\"", tp),
            _ => String::new(),
        };
        format!(
            "  {{\n    \"pipeline\": \"{}\",\n    \"trigger\": {{\n      \"event\": \"{}\"{}    }}\n  }}",
            name, t.event, extra
        )
    }).collect();
    format!("[\n{}\n]", items.join(",\n"))
}

pub fn cmd_deploy_trigger(file: &str, out: Option<&str>) {
    let src = load_file(file);
    let prog = Parser::parse_str(&src, file).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let entries: Vec<(&str, &crate::ast::TriggerAnnotation)> = prog.items.iter()
        .filter_map(|item| {
            if let crate::ast::Item::FlwDef(fd) = item {
                fd.trigger.as_ref().map(|t| (fd.name.as_str(), t))
            } else {
                None
            }
        })
        .collect();
    if entries.is_empty() {
        eprintln!("warning: no #[trigger(...)] annotations found in {}", file);
        return;
    }
    let json = build_trigger_config_json(&entries);
    match out {
        Some(path) => std::fs::write(path, &json).unwrap_or_else(|e| {
            eprintln!("error: failed to write {}: {}", path, e);
            process::exit(1);
        }),
        None => println!("{}", json),
    }
}
```

### 3-3: `v223000_tests::version_is_22_3_0` に `#[ignore]` を追加

### 3-4: `v224000_tests` モジュールを追加

`v223000_tests` の直後に追加:

```rust
// ── v224000_tests (v22.4.0) — Event-driven Pipeline ──────────────────────────
#[cfg(test)]
mod v224000_tests {
    use super::*;

    #[test]
    fn version_is_22_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.4.0\""), "Cargo.toml should have version 22.4.0");
    }

    #[test]
    fn trigger_annotation_s3_parsed() {
        let src = r#"
#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]
seq ProcessUpload = ParseCsv |> Save
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
            let t = fd.trigger.as_ref().expect("trigger should be Some");
            assert_eq!(t.event, "s3:ObjectCreated");
            assert_eq!(t.bucket.as_deref(), Some("raw-data"));
            assert!(t.topic.is_none());
        } else {
            panic!("expected FlwDef item");
        }
    }

    #[test]
    fn trigger_annotation_kafka_parsed() {
        let src = r#"
#[trigger(event = "kafka:message", topic = "orders")]
seq ProcessOrder = Deserialize |> Save
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
            let t = fd.trigger.as_ref().expect("trigger should be Some");
            assert_eq!(t.event, "kafka:message");
            assert_eq!(t.topic.as_deref(), Some("orders"));
            assert!(t.bucket.is_none());
        } else {
            panic!("expected FlwDef item");
        }
    }

    #[test]
    fn deploy_trigger_generates_json() {
        // cmd_deploy_trigger の結果を StringIO 的にキャプチャするため、
        // build_trigger_config_json を直接テストする
        use crate::ast::TriggerAnnotation;
        let span = crate::ast::Span::new("test", 0, 0, 1, 1);
        let t = TriggerAnnotation {
            event: "s3:ObjectCreated".to_string(),
            bucket: Some("my-bucket".to_string()),
            topic: None,
            span,
        };
        let entries: Vec<(&str, &TriggerAnnotation)> = vec![("MyPipeline", &t)];
        let json = crate::driver::build_trigger_config_json(&entries);
        assert!(json.contains("\"pipeline\": \"MyPipeline\""), "json={}", json);
        assert!(json.contains("\"event\": \"s3:ObjectCreated\""), "json={}", json);
        assert!(json.contains("\"bucket\": \"my-bucket\""), "json={}", json);
    }

    #[test]
    fn changelog_has_v22_4_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.4.0]"), "CHANGELOG should have v22.4.0 entry");
    }
}
```

**注意**: `build_trigger_config_json` は `pub(crate)` または `pub` にする必要がある（テストから呼ぶため）。

### 確認

```bash
cargo test v224000 --bin fav   # 5/5 PASS を確認
cargo test --bin fav           # リグレッションなし（1856 件以上）確認
```

---

## T4: `fav/src/main.rs` — `deploy --trigger <file>` CLI フラグ追加

### 事前確認コマンド

```bash
grep -n "Some(\"deploy\")\|--dry-run\|cmd_deploy\b" fav/src/main.rs | head -10
```

### 4-1: `deploy` ブランチに `--trigger` フラグを追加

**注意**: `deploy` ブランチの `while` ループには末尾に `other => { eprintln!(...); process::exit(1); }` という catch-all アームがある。`--trigger` アームはこの catch-all の**前**（`--dry-run` アームの直後）に挿入すること。

既存の `deploy` ブランチの変数宣言に追加:

```rust
let mut trigger_file: Option<String> = None;
```

`while` ループの `match` アームに追加（`--dry-run` アームの直後）:

```rust
"--trigger" => {
    trigger_file = Some(
        args.get(i + 1)
            .unwrap_or_else(|| {
                eprintln!("error: --trigger requires a file path");
                process::exit(1);
            })
            .clone(),
    );
    i += 2;
}
```

`cmd_deploy` 呼び出しの前で分岐:

```rust
if let Some(ref tfile) = trigger_file {
    crate::driver::cmd_deploy_trigger(tfile, None);
} else {
    cmd_deploy(
        env.as_deref(),
        function_name.as_deref(),
        region.as_deref(),
        dry_run,
    );
}
```

### 確認

```bash
cargo check --bin fav
```

**重要**: `deploy` ブランチの `while` ループ末尾には `other => { eprintln!(...); process::exit(1); }` という catch-all アームが存在する。`--trigger` アームはこの catch-all の**前**に挿入すること（後ろに追加すると到達不能コードになりコンパイルエラー）。

---

## T5: Cargo.toml + CHANGELOG.md + MDX

### 5-1: `fav/Cargo.toml` バージョン更新

```
version = "22.3.0" → "22.4.0"
```

### 5-2: `CHANGELOG.md` に v22.4.0 エントリを先頭に追加

```markdown
## [v22.4.0] — 2026-06-21 — Event-driven Pipeline（イベントトリガー）

...
```

### 5-3: `site/content/docs/cli/trigger.mdx` を新規作成

内容:
- `#[trigger(...)]` アノテーションの説明と使用例
- `fav deploy --trigger` コマンド
- サポートイベント種別（s3 / kafka / sqs）
- 出力 JSON フォーマット
- 将来の実際のデプロイ（v22.5+）への言及

---

## 主要な落とし穴・注意事項

1. **`FlwDef` 直接初期化の漏れ**: `parser.rs` 内の `parse_flw_def_or_binding` で `FlwDef { ... }` を構築している箇所に `trigger: None` を追加する必要がある。`cargo check` で確認する。

2. **`parse_trigger_annotation` の呼び出しタイミング**: `parse_item()` で `checkpoint_ann` の直後に呼ぶ。`streaming_ann` と同様に `seq` ブランチでのみ適用する（`stage` / `fn` には適用しない）。

3. **`build_trigger_config_json` の可視性**: テストから直接呼ぶため `pub(crate)` にする。`cmd_deploy_trigger` は `main.rs` から呼ぶため `pub` にする。

4. **JSON の特殊文字エスケープ**: `build_trigger_config_json` 内のパイプライン名・イベント名・バケット名に `"` が含まれる場合は未対応（v22.4.0 スコープ外）。入力は識別子・文字列リテラルなので問題ない想定。

5. **`FlwDef.trigger` の `parse_flw_def_or_binding` 内初期化**: `parse_flw_def` 内で直接 `FlwDef { ..., trigger: None, ... }` と書く必要がある（`parse_item` 側で後から設定するため）。
