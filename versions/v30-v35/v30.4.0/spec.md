# v30.4.0 仕様書 — Rune import マルチファイル動作検証

## 概要

複数の `.fav` ファイルが同一の Rune（`runes/postgres`）を import した場合に、
`fav check` が正しく動作することをフィクスチャで検証する。

---

## 背景

v30.2.0 の `postgres-etl` スキャフォールドでは `stages.fav` のみが `import runes/postgres` を持つ。
実際のプロジェクトでは:
- `stages.fav` — Postgres に書き込む（`Postgres.execute`）
- `main.fav` — スキーマ初期化など Postgres を直接操作する

と複数ファイルが同一 Rune を import する。この場合に:
- 型チェックが二重定義エラーを出さないか
- エフェクト型（`!Postgres`）が正しく伝播するか

を確認し、発見したバグを修正する。

---

## スコープ

### IN SCOPE
- `fav/tests/fixtures/multifile_rune_import/` フィクスチャ作成
  - `fav.toml`
  - `src/types.fav` — Rune import なし（型定義のみ）
  - `src/validators.fav` — Rune import なし（純粋バリデーション）
  - `src/stages.fav` — `import runes/postgres`（関数呼び出し）
  - `src/main.fav` — `import runes/postgres`（同一 Rune の 2 回目 import）
- `fav check` で各ファイルが通ること（手動検証）
- `v304000_tests`（7 件）Rust テスト追加
- CHANGELOG / benchmark / current.md 更新

### OUT OF SCOPE
- `fav run` / `fav test` の実行（実 DB 不要のため `fav check` のみ）
- Rune import の実行時二重初期化バグ修正（検出した場合は v30.9.0 に defer）
- site/ MDX 更新

---

## フィクスチャ設計

### ファイル構成

```
fav/tests/fixtures/multifile_rune_import/
├── fav.toml
└── src/
    ├── types.fav        Rune import なし — RawRow / ValidRow / RowError 型定義
    ├── validators.fav   Rune import なし — validate_row（Some/None パターン）
    ├── stages.fav       import runes/postgres — LoadCsv / ValidateRows / WriteToDb / EtlPipeline
    └── main.fav         import runes/postgres — Main stage（EtlPipeline 呼び出し）
```

### 検証シナリオ

`stages.fav` と `main.fav` の両方が `import runes/postgres` を持つ。
`fav check` がこれを正常に解決し、エラーなく通ること。

---

## Favnir 言語制約（v30.3.0 調査結果）

| 制約 | 正しい書き方 |
|------|------------|
| `let` キーワード不可 | `bind x <- expr` または直接式 |
| `String.to_int` は `Option<Int>` 返し | `Some(n)` / `None` でパターンマッチ |
| `String.to_float` は `Option<Float>` 返し | `Some(f)` / `None` でパターンマッチ |
| インラインレコードは型名プレフィックス必須 | `ValidRow { id: id, name: ..., amount: ..., date: ... }` |
| `Postgres.execute(sql, params_json)` — 2 引数 | `Postgres.execute_raw` 経由の環境変数ベース API |
| `seq` 型合成の制約 | `seq` は同一 Result エラー型を要求する → `bind` 連鎖で代替 |
| エラー型の統一 | `bind` 連鎖内では全 Result のエラー型を `String` に統一する |

---

## テスト設計（v304000_tests — 7 件）

| # | テスト名 | 確認内容 |
|---|---------|---------|
| 1 | `cargo_toml_version_is_30_4_0` | `Cargo.toml` に `version = "30.4.0"` |
| 2 | `multifile_rune_import_fav_toml_exists` | `fav.toml` に `multifile_rune_import` |
| 3 | `multifile_rune_import_types_fav_exists` | `types.fav` に `RawRow` |
| 4 | `multifile_rune_import_stages_imports_postgres` | `stages.fav` に `import runes/postgres` |
| 5 | `multifile_rune_import_main_imports_postgres` | `main.fav` に `import runes/postgres` |
| 6 | `multifile_rune_import_validators_no_rune_import` | `validators.fav` に `import runes/` が**ない** |
| 7 | `benchmark_v30_4_0_exists` | `benchmarks/v30.4.0.json` に `30.4.0` |
| 8 | `changelog_has_v30_4_0` | `CHANGELOG.md` に `[v30.4.0]` |

---

## 完了条件

- `Cargo.toml` version = "30.4.0"
- `fav/tests/fixtures/multifile_rune_import/` — 5 ファイル（fav.toml + 4 .fav）
- `fav check` が各 `.fav` ファイルで通ること（手動検証）
- `cargo test v304000` — 7/7 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.4.0]` セクション
- `benchmarks/v30.4.0.json` 存在
- `versions/current.md` を v30.4.0 に更新
- `tasks.md` が COMPLETE
