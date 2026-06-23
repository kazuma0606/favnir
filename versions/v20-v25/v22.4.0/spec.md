# v22.4.0 仕様書 — Event-driven Pipeline（イベントトリガー）

## 概要

S3 / SQS / Kafka をトリガーとするパイプラインを Favnir で定義・デプロイできるようにする。
`#[trigger(...)]` アノテーションを `seq` 宣言に付与することでイベント駆動パイプラインを記述し、
`fav deploy --trigger` でデプロイ設定 JSON を生成する。

v22.4.0 は**コンパイル時の構造抽出とデプロイ設定生成のスタブ実装**。
Lambda + EventBridge への実際のデプロイは v22.5+ で対応する。

**テーマ**: 「イベント駆動パイプラインを型システムに統合する」

---

## ロードマップ完了条件との対応

v22.4.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第四弾。
v23.0 の完了条件③「`#[trigger(event = "s3:...")]` で S3 イベント駆動パイプラインがデプロイできる」の
構文・型システム・デプロイ設定生成部分を実装する。

**注意**: v22.4.0 は条件③の**部分実装**（構文解析・JSON 生成のみ）。
Lambda + EventBridge への実際のデプロイ（AWS API 呼び出し）は v22.5+ で実装し、v23.0 で完了条件③を満たす。

---

## 機能仕様

### `#[trigger(...)]` アノテーション

`seq` 宣言の直前に付与できる。

```favnir
// S3 ファイルアップロードをトリガーに
#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]
seq ProcessUpload = ParseCsv |> Validate |> WriteToWarehouse

// Kafka メッセージをトリガーに
#[trigger(event = "kafka:message", topic = "orders")]
seq ProcessOrder = DeserializeOrder |> EnrichOrder |> SaveOrder

// SQS メッセージをトリガーに
#[trigger(event = "sqs:message", topic = "data-queue")]
seq ProcessQueue = Deserialize |> Process |> Save
```

#### キー一覧

| キー | 型 | 必須 | 説明 |
|---|---|---|---|
| `event` | String | ✓ | イベント種別（`"s3:ObjectCreated"` / `"kafka:message"` / `"sqs:message"` など） |
| `bucket` | String | △ | S3 イベントのバケット名 |
| `topic` | String | △ | Kafka / SQS のトピック / キュー名 |

`bucket` と `topic` はどちらか一方のみ指定する（イベント種別に応じて）。

### `fav deploy --trigger <file>`

```bash
fav deploy --trigger src/pipeline.fav
# → stdout に JSON デプロイ設定を出力
```

出力 JSON の形式:

```json
[
  {
    "pipeline": "ProcessUpload",
    "trigger": {
      "event": "s3:ObjectCreated",
      "bucket": "raw-data"
    }
  },
  {
    "pipeline": "ProcessOrder",
    "trigger": {
      "event": "kafka:message",
      "topic": "orders"
    }
  }
]
```

`--trigger` が指定された場合、既存の `--env` / `--function` / `--region` フラグは無視され、
`cmd_deploy_trigger` が呼ばれる。

---

## アーキテクチャ

### `TriggerAnnotation` struct（`ast.rs`）

```rust
/// v22.4.0: `#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]`
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

### `FlwDef.trigger` フィールド（`ast.rs`）

```rust
pub struct FlwDef {
    // ... 既存フィールド ...
    pub streaming: Option<StreamingAnnotation>,
    /// v22.4.0: `#[trigger(...)]` annotation — event-driven pipeline trigger.
    pub trigger: Option<TriggerAnnotation>,
    pub span: Span,
}
```

### パーサー変更（`frontend/parser.rs`）

`parse_trigger_annotation()` メソッドを追加:

```rust
fn parse_trigger_annotation(&mut self) -> Result<Option<TriggerAnnotation>, ParseError> {
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
    Ok(Some(TriggerAnnotation { event, bucket, topic, span: self.span_from(&start) }))
}
```

`parse_item()` で `checkpoint_ann` の直後に呼び出し:

```rust
// v22.4.0: parse optional #[trigger(...)] annotation
let trigger_ann = self.parse_trigger_annotation()?;
```

`seq` ブランチで適用:

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

### `cmd_deploy_trigger`（`driver.rs`）

```rust
pub fn cmd_deploy_trigger(file: &str, out: Option<&str>) {
    let src = load_file(file);
    let prog = Parser::parse_str(&src, file).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let entries: Vec<_> = prog.items.iter()
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

pub(crate) fn build_trigger_config_json(entries: &[(&str, &crate::ast::TriggerAnnotation)]) -> String {
    let items: Vec<String> = entries.iter().map(|(name, t)| {
        let event_src = match (t.bucket.as_deref(), t.topic.as_deref()) {
            (Some(b), _) => format!("\"bucket\": \"{}\"", b),
            (_, Some(tp)) => format!("\"topic\": \"{}\"", tp),
            _ => String::new(),
        };
        let src_field = if event_src.is_empty() {
            String::new()
        } else {
            format!(",\n      {}", event_src)
        };
        format!(
            "  {{\n    \"pipeline\": \"{}\",\n    \"trigger\": {{\n      \"event\": \"{}\"{}    }}\n  }}",
            name, t.event, src_field
        )
    }).collect();
    format!("[\n{}\n]", items.join(",\n"))
}
```

### `fav deploy --trigger` CLI（`main.rs`）

既存の `deploy` コマンド内に `--trigger` フラグを追加:

```rust
"--trigger" => {
    trigger_file = Some(args.get(i + 1).unwrap_or_else(|| {
        eprintln!("error: --trigger requires a file path");
        process::exit(1);
    }).clone());
    i += 2;
}
```

`--trigger` が指定された場合は `cmd_deploy_trigger` を呼ぶ（他フラグは無視）:

```rust
if let Some(ref tfile) = trigger_file {
    cmd_deploy_trigger(tfile, out.as_deref());
} else {
    cmd_deploy(env.as_deref(), function_name.as_deref(), region.as_deref(), dry_run);
}
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `TriggerAnnotation` struct 追加 / `FlwDef.trigger` フィールド追加 |
| `fav/src/frontend/parser.rs` | 更新 | `parse_trigger_annotation()` / `parse_item()` 適用 |
| `fav/src/driver.rs` | 更新 | `cmd_deploy_trigger` / `build_trigger_config_json` / `v224000_tests`（5 件） |
| `fav/src/main.rs` | 更新 | `deploy --trigger <file>` CLI フラグ追加 |
| `fav/Cargo.toml` | 更新 | `version = "22.3.0"` → `"22.4.0"` |
| `CHANGELOG.md` | 更新 | v22.4.0 エントリ追加 |
| `site/content/docs/cli/trigger.mdx` | 新規 | `fav deploy --trigger` ドキュメント |

---

## テスト一覧（v224000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_4_0` | Cargo.toml に `version = "22.4.0"` が含まれる |
| `trigger_annotation_s3_parsed` | `#[trigger(event = "s3:ObjectCreated", bucket = "data")]` が `FlwDef.trigger` に格納される |
| `trigger_annotation_kafka_parsed` | `#[trigger(event = "kafka:message", topic = "orders")]` が `FlwDef.trigger.topic` に格納される |
| `deploy_trigger_generates_json` | `cmd_deploy_trigger` がパイプライン名と event を含む JSON を stdout に出力する |
| `changelog_has_v22_4_0` | CHANGELOG.md に `[v22.4.0]` が含まれる |

---

## スコープ外（v22.4.0 では実装しない）

- Lambda + EventBridge への実際のデプロイ（AWS API 呼び出し）
- SQS / Kafka Lambda Trigger のプロビジョニング
- `#[trigger]` が付いた seq の型チェック強化（エフェクト検証等）
- デプロイ設定の Terraform ファイル生成（JSON のみ）
- 複数ファイルのプロジェクトへの対応

---

## 完了条件

- [ ] `TriggerAnnotation` struct が `ast.rs` に追加される
- [ ] `FlwDef.trigger: Option<TriggerAnnotation>` フィールドが追加される
- [ ] `#[trigger(event = "...", bucket = "...")]` がパースされる
- [ ] `#[trigger(event = "...", topic = "...")]` がパースされる
- [ ] `fav deploy --trigger <file>` が JSON デプロイ設定を stdout に出力する
- [ ] `cargo test v224000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1856 件以上合格）
- [ ] `CHANGELOG.md` に v22.4.0 エントリ
- [ ] `site/content/docs/cli/trigger.mdx` 作成済み
