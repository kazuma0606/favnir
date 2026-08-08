# v69.4.0 タスクリスト

Status: COMPLETE
Version: 69.4.0
Note: `fav migrate --ai` — driver.rs に cmd_migrate_ai 追加 + main.rs フラグ解析追加（テスト追加なし）
Base tests: 3545
Target tests: 3545（変化なし）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3545 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.3.0` であることを確認
- [x] `driver.rs` に `cmd_migrate_ai` が存在しないことを確認（今回新規追加）
- [x] `main.rs` の `Some("migrate")` アームに `--ai` フラグが存在しないことを確認

---

## T1: `driver.rs` — `cmd_migrate_ai` 実装

- [x] `fn extract_stage_name(line: &str) -> Option<&str>` を追加（private ヘルパー）
  - [x] `"stage Foo:"` / `"public stage Foo:"` からステージ名を切り出す
- [x] `pub fn cmd_migrate_ai(src: &str, output: Option<&str>, dry_run: bool, interactive: bool)` を追加
  - [x] `interactive` は `let _ = interactive;` で無視（将来用）
  - [x] `stage` + `String` 型 → `Rune.embed.openai` 追加提案を生成
  - [x] `Rune.pg.insert` / `Rune.mysql.insert` → `Rune.pinecone.upsert` 追加提案を生成
  - [x] `Rune.slack.send` / `Rune.email.send` → `Rune.llm.summarize` 追加提案を生成
  - [x] 提案なし時は `[INFO]` メッセージを出力
  - [x] `dry_run == true` 時はファイルを書き出さず stdout に提案のみ出力
  - [x] `dry_run == false && output.is_some()` 時はヘッダーコメント付きで書き出す
  - [x] `dry_run == false && output.is_none()` 時は変換後ソースを stdout に出力

---

## T2: `main.rs` — フラグ解析追加

- [x] use 宣言行に `cmd_migrate_ai,` を追加
- [x] `Some("migrate")` アームの変数宣言に `ai_mode / interactive / output_path` を追加
- [x] フラグ解析ループに `"--ai"` / `"--interactive"` / `"--output"` アームを追加
- [x] `if to_version == Some("enterprise")` の直前に `if ai_mode { ... return; }` を追加

---

## T3: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3545 tests passed, 0 failed** を確認（テスト数変化なし）

---

## T4: 手動動作確認

- [x] `echo 'public stage Transform: String -> String = |s| { s }' > /tmp/test.fav && fav migrate --ai /tmp/test.fav --dry-run` が `Suggestions:` を出力することを確認
- [x] `--dry-run` 時に `--output` で指定したファイルが生成されないことを確認
- [x] `--output` 指定時にヘッダーコメントが付いたファイルが生成されることを確認
- [x] `--output` 未指定時（`--dry-run` なし）に変換後ソースが stdout に出力されることを確認

---

## T5: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.4.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を `v69.3.0` から `v69.4.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

---

## 設計上の意図的省略

- 実際の LLM 呼び出し: 将来フェーズ（現バージョンは静的解析のみ。ロードマップ記載あるが本 sub-version では見送り）
- 変換後の型チェック呼び出し: 将来フェーズ（ロードマップ記載あるが本 sub-version では見送り）
- Rust テスト: ロードマップ明示的方針「提案生成の品質は手動検証」
- `--interactive` フラグの完全実装: 将来フェーズ（フラグ解析のみ実装し動作は無視）
