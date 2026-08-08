# v60.2.0 Spec — `fav check --fix` 自動修正 Phase 1

Date: 2026-07-30
Status: 計画中

---

## 概要

`fav check --fix` コマンドを実装する。typo 修正と未使用 bind 削除の 2 種類の自動修正を提供し、
`--dry-run` フラグでプレビューのみ表示するモードも追加する。

---

## 実装コード一覧とロードマップとの対応

| ロードマップ記述 | 実際のコード | 備考 |
|---|---|---|
| `E0001`（typo 候補 1 件のみ） | `E0102`（`checker.rs` の未定義変数エラー） | E0001 は存在しない；未定義変数は E0102 |
| `W001`（未使用 bind） | `L002`（`lint.rs` の unused bind lint） | W001 はチェッカーの「型未解決」；未使用 bind は L002 |

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_check_fix` / `cmd_check_fix_src` / `extract_backtick_ident` 追加、`v60200_tests` 追加 |

**`main.rs` の `--fix` フラグ追加はスコープ外。**
ロードマップには記載があるが、実際のファイル書き換え（span ベースのトークン置換・行削除）が完成してから追加する。
v60.2.0 では `cmd_check_fix_src` が修正サマリー文字列を返す動作のみ実装し、テストで検証する。

---

## 機能仕様

**チェッカーについて**: `cmd_check_fix_src` は Rust checker（`Checker::check_program`）を使用する。
v8.5.0 以降デフォルトはセルフホスト checker（checker.fav）だが、`cmd_check_fix_src` は
Rust checker を直接呼ぶ（v60.1.0 の `cmd_check_span_output` と同じ設計方針）。
セルフホスト checker との整合は v60.x 以降の課題とする。

### `fav check --fix <file>` （v60.2.0 実装範囲）

1. `Parser::parse_str` でソースをパース
2. `Checker::check_program`（Rust checker）で型チェック（E0102 候補を収集）
3. `lint::lint_program` で lint（L002 候補を収集）
4. 修正サマリー文字列を返す（**ファイルへの書き戻しは今バージョンのスコープ外**）

修正ルール:
- **E0102 typo fix**: `error.hints` に `"did you mean \`X\`?"` が **1 件のみ** 存在する場合、span の位置のトークンを `X` に置換
- **L002 unused bind**: `bind x <- ...` の行ごと削除

### `fav check --fix --dry-run <file>`

ファイルを変更せず、適用予定の変更を標準出力に表示する。

### 出力形式

```
[auto-fixed] E0102: `user_id` → `userId` (pipeline.fav:12)
[auto-fixed] L002: unused bind `tmp` removed (pipeline.fav:8)
2 fixes applied.

# dry-run 時
[would fix] E0102: `user_id` → `userId` (pipeline.fav:12)
[would fix] L002: unused bind `tmp` removed (pipeline.fav:8)
2 fixes would be applied (dry-run, no changes made).
```

---

## テスト仕様

### `check_fix_typo_single_candidate`

- ソース `"fn go(userId: Int) -> Int { user_id }"` を `cmd_check_fix_src` に渡す
- `user_id` vs `userId` の Levenshtein 距離 = 2（`_` 削除 + `i→I` 置換）で閾値以内
  → did-you-mean hint が 1 件生成されることが確認済み
- 出力に `"[auto-fixed]"` と `"E0102"` が含まれることを assert
- line 番号は `Span.line`（1-indexed）を使用

### `check_fix_unused_bind`

- L002 未使用 bind を含むソースを `cmd_check_fix_src` に渡す（dry_run=true）
- 出力に `"[would fix]"` と `"L002"` が含まれることを assert

---

## 完了条件

- `cargo test` 全通過（3332 → **3334** tests passed, 0 failed）
- 以下の 2 テストが pass:
  - `v60200_tests::check_fix_typo_single_candidate`
  - `v60200_tests::check_fix_unused_bind`

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v60.1-v61.0.md`（v60.2.0 セクション）
- 次バージョン: v60.3.0 — LSP Code Action
