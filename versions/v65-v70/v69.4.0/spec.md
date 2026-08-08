# v69.4.0 仕様書 — `fav migrate --ai`（旧 ETL → AI ETL 自動変換）

Status: DRAFT
Version: 69.4.0
Date: 2026-08-08

---

## 概要

既存の Favnir ETL パイプラインを AI ETL パターンに変換する提案を生成するコマンド `fav migrate --ai` を実装する。
静的解析でパターンを検出し、変換提案を出力するシンプルな実装とする。

---

## スコープ

### IN（本バージョンで実施）

- `pub fn cmd_migrate_ai(src: &str, output: Option<&str>, dry_run: bool, interactive: bool)` を `driver.rs` に追加
- `main.rs` の `Some("migrate")` アームに `--ai` / `--output` / `--interactive` フラグ解析を追加
- `cmd_migrate_ai` を `main.rs` の use 宣言に追加
- `--dry-run` は既存の `dry_run` 変数を流用（新規フラグ追加不要）

### OUT（本バージョンでは実施しない）

- 実際の LLM 呼び出し（静的解析のみ。ロードマップには「Claude API が分析」と記載があるが、本 sub-version での実装は見送り。将来フェーズで対応）
- 変換後の型チェック呼び出し（ロードマップには「型エラーなくコンパイルできることを確認」と記載があるが、本 sub-version での実装は見送り。将来フェーズで対応）
- AST / IR 変更なし
- Rust テスト追加なし（ロードマップ方針「提案生成の品質は手動検証」）
- Cargo.toml / CHANGELOG.md 変更なし（sub-version ポリシー）

---

## CLI インターフェース

```sh
fav migrate --ai src/old-pipeline.fav --output src/ai-pipeline.fav
fav migrate --ai src/old-pipeline.fav --dry-run
fav migrate --ai src/old-pipeline.fav --interactive
```

出力例（`--dry-run` なし）:

```
Analyzing old-pipeline.fav...

Suggestions:
[1] LoadCsv → LoadCsv（変更なし）
[2] Transform → EmbedAndTransform
    + Rune.embed.openai を追加（text フィールドから埋め込み生成）
[3] InsertDB → InsertDB + StoreToVectorDB
    + ベクトルを Pinecone に並行保存
[4] SendReport → SemanticEnrich（LLM で要約を追加）

Generated: src/ai-pipeline.fav
```

`--dry-run` 時は提案のみ出力し、ファイルは書き出さない。

---

## 実装詳細

### `driver.rs` — `pub fn cmd_migrate_ai`

```rust
pub fn cmd_migrate_ai(src: &str, output: Option<&str>, dry_run: bool, interactive: bool)
```

- `interactive` は将来用。現バージョンでは `let _ = interactive;` で無視
- 静的解析で以下のパターンを検出して提案を生成:
  - `stage` 宣言に `String` 型フィールドがある → `Rune.embed.openai` 追加を提案
  - `Rune.pg.insert` / `Rune.mysql.insert` → `Rune.pinecone.upsert` 並行保存を提案
  - `Rune.slack.send` / `Rune.email.send` → `Rune.llm.summarize` 追加を提案
- 提案がない場合: `[INFO] AI ETL への変換候補が見つかりませんでした` を出力
- `dry_run` が false かつ `output` が `Some(path)` の場合: ヘッダーコメントを付けて書き出す
- `output` が `None` の場合: 変換後ソースを stdout に出力

ヘッダーコメント（変換後ファイル先頭）:

```
// AI ETL pipeline — migrated by `fav migrate --ai`
// Set ANTHROPIC_API_KEY and OPENAI_API_KEY in your environment.
```

### `main.rs` — フラグ解析追加

`Some("migrate")` アームの変数宣言に追加:

```rust
let mut ai_mode = false;
let mut interactive = false;
let mut output_path: Option<String> = None;
```

フラグ解析ループに追加（既存フラグの前に挿入）:

```rust
"--ai" => { ai_mode = true; i += 1; }
"--interactive" => { interactive = true; i += 1; }
"--output" => {
    output_path = Some(args.get(i + 1).unwrap_or_else(|| {
        eprintln!("error: --output requires a file path"); process::exit(1);
    }).clone());
    i += 2;
}
```

ディスパッチ（`if to_version.as_deref() == Some("enterprise")` の直前）:

```rust
if ai_mode {
    let src_path = file.as_deref().unwrap_or("src/pipeline.fav");
    let src = std::fs::read_to_string(src_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {}", src_path, e);
        process::exit(1);
    });
    cmd_migrate_ai(&src, output_path.as_deref(), dry_run, interactive);
    return;
}
```

### `main.rs` — use 宣言への追加

`cmd_migrate_dry_run, migrate_enterprise_import` のある行に `cmd_migrate_ai,` を追加。

---

## ヘルパー関数

`driver.rs` に private ヘルパーを追加:

```rust
fn extract_stage_name(line: &str) -> Option<&str>
```

`"stage Foo:"` / `"public stage Foo:"` からステージ名を切り出す。

---

## テスト仕様

**テスト追加なし**（ロードマップ方針）

提案生成の品質は手動検証で確認する。
本バージョンでは `cargo test` のテスト数は 3545 のまま変化しない。

---

## 完了条件

- 以下の入力で `fav migrate --ai` が `Suggestions:` を含む出力を返すこと（手動確認）:
  ```sh
  echo 'public stage Transform: String -> String = |s| { s }' > /tmp/test.fav
  fav migrate --ai /tmp/test.fav --dry-run
  # 期待: "[AI] stage Transform: String フィールドを検出 — Rune.embed.openai で埋め込みベクトル生成を推奨" 相当の行が出力される
  ```
- `--dry-run` 時に `--output` 指定ファイルが生成されないこと（手動確認）
- `--output` 未指定時に変換後ソースが stdout に出力されること（手動確認）
- `cargo test --bin fav -- --test-threads=8` で **3545 tests passed, 0 failed**（テスト数変化なし）
- `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.4.0 状態列が「完了」になっていること
- `versions/current.md` の「進行中バージョン」が `v69.4.0` に更新されていること
