# v26.0.0 タスクリスト — Rune Foundation マイルストーン宣言

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-26

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `25.9.0`、テスト数 2035 件、`v259000_tests` に `version_is_` テストがないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.0.0"` に bump | [x] |
| T2 | `MILESTONE.md` に「Rune Foundation」セクション追記（`"Rune Foundation"` を含む） | [x] |
| T3 | `examples/postgres_etl.fav` 新規作成（postgres Rune デモ） | [x] |
| T4 | `examples/s3_csv_to_parquet.fav` 新規作成（s3 Rune デモ） | [x] |
| T5 | `examples/full_etl.fav` 新規作成（`"postgres"` を含む、postgres → s3 → kafka 統合デモ） | [x] |
| T6 | `README.md` 更新: `"v26.0"` を含む「Rune Foundation」セクション追記 | [x] |
| T7 | `site/content/docs/rune-foundation.mdx` 新規作成（`"Rune Foundation"` を含む） | [x] |
| T8 | `versions/roadmap/roadmap-v25.1-v26.0.md` 更新: v25.1〜v25.9 を COMPLETE、v26.0.0 を宣言済みに変更（ロードマップ内の `CallFn` → `CallNamed(0x56)` の名称修正も含む） | [x] |
| T9 | `fav/src/driver.rs` 更新: `v260000_tests`（5 件）を `v259000_tests` の直後に追加 | [x] |
| T9.5 | `cargo test v260000 --bin fav` — 5/5 PASS 確認 | [x] |
| T10 | `CHANGELOG.md` 更新: 先頭に `[v26.0.0]` エントリ追加 | [x] |
| T11 | `benchmarks/v26.0.0.json` 新規作成（test_count: 2040） | [x] |
| T12 | `cargo test --bin fav` — 2040 件 PASS 確認（リグレッションなし） | [x] |
| T13 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.0.0"` であること
- [x] `MILESTONE.md` に `"Rune Foundation"` が含まれること
- [x] `examples/postgres_etl.fav` が存在すること
- [x] `examples/s3_csv_to_parquet.fav` が存在すること
- [x] `examples/full_etl.fav` が存在し `"postgres"` を含むこと
- [x] `README.md` に `"v26.0"` が含まれること
- [x] `site/content/docs/rune-foundation.mdx` が存在し `"Rune Foundation"` を含むこと
- [x] `versions/roadmap/roadmap-v25.1-v26.0.md` が v25.1〜v26.0 ステータスを反映していること
- [x] `v260000_tests` 5 件すべて PASS
- [x] 総テスト数 ≥ 2040 件
- [x] `CHANGELOG.md` に `[v26.0.0]` エントリが存在すること
- [x] `benchmarks/v26.0.0.json` が存在すること（test_count: 2040）

> 注: ロードマップ記載の Docker E2E 実行（`fav run examples/full_etl.fav` in Docker）は v26.x に延期（spec.md §スコープ外 参照）。デモファイルの存在確認のみ本バージョンで実施する。

---

## メモ

### `version_is_` テスト確認

v259000_tests に `version_is_` テストは存在しない（T0 で確認済み）。
目標テスト数: **2035 + 5 = 2040 件**（固定）。

### `include_str!` パス（fav/src/driver.rs から）

| ファイル | パス |
|---|---|
| `MILESTONE.md` | `"../../MILESTONE.md"` |
| `examples/full_etl.fav` | `"../../examples/full_etl.fav"` |
| `site/content/docs/rune-foundation.mdx` | `"../../site/content/docs/rune-foundation.mdx"` |
| `CHANGELOG.md` | `"../../CHANGELOG.md"` |
| `README.md` | `"../../README.md"` |

### `examples/` ディレクトリについて

`examples/` ディレクトリが未存在の場合は新規作成。
Favnir コードとして文法的に正しいことが必要（ただし Docker 実行は必須ではない）。

### MILESTONE.md 追記内容（追記、上書き不可）

```markdown
## v26.0.0 — Rune Foundation（2026-06-XX）

コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が
「動く Rune の 5 条件（connect / read / write / error / test）」をすべてクリアした。

また vm.fav Phase 6（CallNamed opcode, 0x56）が完成し、
multi-function Favnir プログラムを vm.fav インタープリター上で実行できるようになった。

> 「Favnir で書いたパイプラインが実際の本番データを動かせる」
> — `fav run examples/full_etl.fav`（postgres → 集計 → s3 → kafka 通知）
```

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] `full_etl.fav` の `Bytes.from_string` は存在しない（正: `Bytes.from_str`） | `Bytes.from_str` に修正 |
| [HIGH] `full_etl.fav` の `Summarize` stage で `let` を使用（Favnir 構文違反） | `bind x <- Result.ok(...)` パターンに修正 |
| [MED] `MILESTONE.md` 63行目に `CallFn` の旧記述が残存 | `CallNamed` opcode として確定した旨の記述に修正 |
| [MED] ロードマップの完了条件表に `CallFn` が残存 | `CallNamed(0x56)` に統一修正 |
| [MED] `full_etl.fav` の `config.postgres` が未定義（デモ用途として許容） | コメントに「`config` は環境変数または fav.toml から注入」の旨が記載済み（対応不要） |
| [LOW] `v260000_tests` に benchmark JSON の test_count 確認テストがない | `benchmark_v26_0_0_exists` テストを追加（合計 6件、総テスト数 2041件） |
| [LOW] `rune-foundation.mdx` の相対リンクが Next.js ビルドで 404 になる可能性 | デモ用ドキュメントとして許容（既存 MDX の慣習に合わせた参照形式） |
