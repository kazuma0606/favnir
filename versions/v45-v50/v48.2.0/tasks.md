# Tasks: v48.2.0 — import 構文刷新（ローカルファイル）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3047 passed, 0 failed を確認
- [x] `ast.rs` に `ImportKind::Local` バリアントが存在することを確認（v48.1.0 で追加済み）
- [x] `import "./src/helpers"` が現状 `ImportKind::Legacy` になることを確認（まだ `Local` になっていない）

## T1 — Parser 変更

- [x] `parser.rs`: `parse_import_decl` の `Str` ブランチに `./` / `../` prefix 判定を追加
  - [x] `path.starts_with("./") || path.starts_with("../")` → `kind = ImportKind::Local`
  - [x] それ以外の文字列 → `kind = ImportKind::Legacy`（デフォルト、変更不要）
- [x] `cargo build` でコンパイルエラーなしを確認（ast.rs 変更なしのため新規エラーは発生しないはず）

## T2 — `driver.rs` テスト追加・バージョン更新・完了

- [x] `v482000_tests` モジュールを `v481000_tests` の直前に追加（2テスト）
  - [x] `import_local_parses`: `import "./src/helpers" as helpers` → `ImportKind::Local`、`path == "./src/helpers"`、`alias == Some("helpers")`
  - [x] `import_local_relative_path`: `import "../utils/common"` → `ImportKind::Local`、`path == "../utils/common"`、`alias == None`
- [x] `fav/Cargo.toml` version → `"48.2.0"`
- [x] `CHANGELOG.md` に v48.2.0 エントリ追加
- [x] `cargo test` 3049 passed, 0 failed（3047 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.2.0（3049 tests）に更新、進行中バージョンを `v48.3.0` に更新
- [x] `versions/current.md` のサブスプリントリンクを `roadmap-v48.1-v49.0.md` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.2.0 テスト数を実績値 3049 に更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全 `[x]`）

---

> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v49.0.0 マイルストーン宣言時に実施
> **注記**: site/ MDX（`module-system.mdx` 等）の更新は v48.9.0 のスコープ。本バージョンでは不要。

---

## コードレビュー指摘と対応（code-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `collect_project_sources` が `ImportKind::Local` を考慮しない（`../` traversal 含む） | v48.3.0 スコープ（spec.md に「parser のみ」と明記済み） |
| [HIGH] | `../` traversal 検証なし | v48.3.0 スコープ |
| [MED] | `fmt.rs` が `ImportKind::Package` を `import "kafka"` に書き換えてラウンドトリップ破壊 | `fmt.rs` を `kind` で分岐: `Package` → `import {path}`、それ以外 → `import "{path}"` |
| [LOW] | `.hidden` エッジケース（`.` ありスラッシュなし） | 実害なし・スキップ |

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `is_rune` と `ImportKind::Local` の相互作用が spec 未記述 | spec.md 注意事項に「`./` prefix は `.` を含むため `is_rune = false` が保証される」を追記 |
| [HIGH] | ロードマップ v48.2.0 テスト推定値が 3044（ずれ 5 件） | `roadmap-v48.1-v49.0.md` の推定値を 3044 → 3049 に修正（spec 作成時） |
| [MED] | spec.md テストコードが raw string 形式（plan.md と不統一） | spec.md をエスケープ文字列 `"..."` 形式に統一 |
| [MED] | `import_local_relative_path` に `alias: None` 検証なし | spec.md + tasks.md + driver.rs テストに `alias: None` を追加 |
| [MED] | tasks.md の「確認」止まり（ロードマップ更新が読み取りのみに見える） | 「実績値 3049 に**更新**」に修正 |
| [LOW] | `current.md` サブスプリントリンクが古い（v46.1-v47.0） | tasks.md T2 に更新項目追加 + 実装時に修正 |
| [LOW] | site/ MDX スコープが未明示 | spec.md 注意事項 + tasks.md 注記に「v48.9.0 スコープ」を明記 |
