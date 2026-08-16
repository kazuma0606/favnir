# v70.4.0 タスクリスト — 構造化エラー診断

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.3.0` であることを確認
- [x] `cargo test` が全 pass（3565 tests）であることを確認
- [x] `strsim = "0.11"` が `fav/Cargo.toml` の dependencies に登録されていることを確認

---

## T1: `ErrorReport` 構造体と関数群を driver.rs に追加

- [x] driver.rs の `use` セクション（ファイル先頭付近）に `use strsim;` を追加する
  - ※ checker.rs と同様に `strsim::levenshtein(...)` 直接パス参照で OK（`use strsim;` 不要と判明）
- [x] driver.rs の `v703000_tests` の直前に以下を追加する:
  - `ErrorReport` 構造体（code / file / line / col / source_line / span_len / message / hint / suggestion / doc_url）
  - `suggest_similar_name(name: &str, candidates: &[&str]) -> Option<String>`（strsim::levenshtein 使用、距離 ≤ 3）
  - `format_error_report(r: &ErrorReport) -> String`（rustc スタイル出力）※ 既存 `format_diagnostic` と名前衝突回避のため改名
  - `build_e0374_report(file, line, col, source_line, effect_name) -> ErrorReport`（E0374 専用ビルダー、`doc_url: Some("https://favnir.dev/docs/language/ctx-migration")`）
  - `build_e0001_report(file, line, col, source_line, var_name, candidates) -> ErrorReport`（E0001 専用ビルダー）
- [x] `cargo test` で既存テスト（3565 件）が全 pass することを確認

---

## T2: `v704000_tests` モジュールを driver.rs 末尾に追加

- [x] `v703000_tests` の直後（driver.rs 末尾）に `v704000_tests` モジュールを追加する
- [x] `diagnostic_e0374_shows_migration_hint` テストを実装する:
  - `build_e0374_report` が `error[E0374]` を含む診断テキストを返すことを assert
  - ファイル名・廃止バージョン（v35.4.0）・ctx 移行ヒント・fav migrate コマンドを含むことを assert
  - `doc_url`（`favnir.dev/docs/language/ctx-migration`）を含むことを assert
- [x] `diagnostic_e0001_suggests_similar_name` テストを実装する:
  - `build_e0001_report` が `error[E0001]` / `未定義変数` を含むことを assert
  - `suggest_similar_name("ordr", &["order", "other", "data"])` が `Some("order")` を返すことを assert
- [x] `cargo test v704000` で 2 件 pass することを確認

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.3.0"` → `"70.4.0"` に変更する
- [x] driver.rs 内の `cargo_toml_version_is_70_3_0` テスト関数内の `"70.3.0"` 文字列を `replace_all: true` で `"70.4.0"` に一括更新

---

## T4: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.3.0 エントリの直前）に v70.4.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `ErrorReport` 構造体
  - Added: `suggest_similar_name` — Levenshtein 距離 ≤ 3 の候補返却
  - Added: `format_error_report` — rustc スタイルの診断テキスト生成
  - Added: `build_e0374_report` / `build_e0001_report` — 専用ビルダー
  - Added: `v704000_tests` 2 件（3565 → 3567 tests）

---

## T5: versions/current.md 更新

- [x] `versions/current.md` を開く
- [x] 「進行中バージョン」を `v70.4.0`（構造化エラー診断）に更新する
- [x] 「次に切る版」を `v70.5.0` に更新する

---

## T6: 最終確認

- [x] `cargo test v704000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3567 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.4.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### spec-reviewer 指摘（実装前）
- **[HIGH] カラー/LSP JSON 出力スコープ未定義**: spec.md に「v70.4 スコープ外」セクション追加（v70.5 以降に先送り）
- **[HIGH] 修正後コードブロック欠落**: spec.md に「v70.4 スコープ外」明記
- **[HIGH] E0374 の `doc_url: None`**: `Some("https://favnir.dev/docs/language/ctx-migration")` に変更、テスト assert 追加
- **[MED] `use strsim;` 手順欠落**: `strsim::levenshtein` は直接パス参照で動作（checker.rs と同パターン）
- **[MED] span フィールド命名不一致**: spec.md Background に注記追加
- **[MED] バージョン置換対象の不明確さ**: plan.md Step 3・tasks.md T3 を具体化
- **実装時判明**: 既存 `format_diagnostic(source, error)` と名前衝突 → 新関数を `format_error_report` に改名

### code-reviewer 指摘（実装後）
- **[MED] 行番号ガター幅が行番号桁数に追随しない**: `pad = " ".repeat(ln.len())` を導入し、全ガター行で `{pad}|` を使用するよう `format_error_report` を修正
- **[MED] `span_len` の契約が文書化されていない**: `ErrorReport` のドキュメントコメントに `span_len >= 1` の契約を明記
- **[LOW] `ErrorReport` に `#[derive(Debug)]` がない**: `#[derive(Debug)]` を追加
- **[LOW] テストが整形構造を検証していない**: `:43:62`・`43|`・`^^^` 等のアサーションを両テストに追加

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `diagnostic_e0374_shows_migration_hint` が pass
- [x] `diagnostic_e0001_suggests_similar_name` が pass
- [x] テスト総数: 3567（+2）
