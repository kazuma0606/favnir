# v30.6.0 仕様書 — fav test プロジェクト統合

## 概要

`fav test`（引数なし）でプロジェクト全体の全テストを一括実行できるようにする。
具体的には `src/` に加えて `tests/` ディレクトリも走査対象に含める。

---

## 背景

v30.5.0 で `examples/csv-to-postgres/tests/pipeline_test.fav` を作成したが、
`fav test`（引数なし）は `fav.toml` の `src` ディレクトリ（`src/`）しかスキャンしない。
`tests/` ディレクトリは走査されないため、テストが実行されない。

現状コード（`driver.rs:4299-4301`）:
```rust
let src_dir = toml.src_dir(&root);
collect_test_files(&src_dir)   // src/ のみ
```

---

## スコープ

### IN SCOPE

- `cmd_test`（引数なし時）— `src/` に加えて `tests/` も走査
- `collect_test_files` — `tests/` ディレクトリが存在する場合は自動追加
- `fav test --filter <pattern>` — 既存実装の動作確認（変更なし）
- 失敗メッセージ形式の確認・必要に応じて改善
- `v306000_tests`（3 件）Rust テスト追加

### OUT OF SCOPE

- `fav test --coverage`（既存機能）の変更
- `fav test --fail-fast` の変更
- `tests/` 以外のディレクトリ（例: `spec/`）の走査
- site/ MDX 更新（`fav test` の挙動変更は CHANGELOG のみで周知。`site/content/docs/tools/` 内に `fav test` 専用ページが存在する場合は v30.9 以降で更新）

---

## 実装仕様

### `cmd_test` 引数なし時の走査対象変更（`driver.rs`）

現状: `src/` のみ
変更後: `src/` + `tests/`（`tests/` が存在する場合）

```rust
// 変更前
let src_dir = toml.src_dir(&root);
collect_test_files(&src_dir)

// 変更後
let src_dir = toml.src_dir(&root);
let tests_dir = root.join("tests");
let mut files = collect_test_files(&src_dir);
if tests_dir.is_dir() {
    files.extend(collect_test_files(&tests_dir));
    files.sort();
    files.dedup();
}
files
```

### 走査対象ファイル

`collect_test_files` は既存実装通り:
- `*.fav`
- `*.test.fav`
- `*.spec.fav`

`tests/` 配下のファイルは `TestDef` ブロックを含む `.fav` ファイルのみ実行対象（VM が `test` ブロックのみを実行する既存動作に準拠）。

> **`dedup` について**: `src/foo.fav` と `tests/foo.fav` は絶対パスが異なるため `dedup` では除去されない（両方実行される）。`dedup` が有効なのはシンボリックリンク等で同一の絶対パスが 2 回収集された場合のみ。

---

## テスト設計（v306000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|------------|
| 1 | `cargo_toml_version_is_30_6_0` | `Cargo.toml` に `version = "30.6.0"` |
| 2 | `cmd_test_scans_tests_dir` | `driver.rs` に `tests_dir` と `is_dir` の両方が含まれること |
| 3 | `benchmark_v30_6_0_exists` | `benchmarks/v30.6.0.json` に `30.6.0` |

---

## 完了条件

- `Cargo.toml` version = "30.6.0"
- `cmd_test`（引数なし）が `tests/` ディレクトリも走査する
- `fav test`（引数なし、`examples/csv-to-postgres/` 配下で実行）が `pipeline_test.fav` の 3 件を検出する
- `fav test --filter validate`（`examples/csv-to-postgres/` で実行）がフィルタ動作する（ロードマップ完了条件）
- `cargo test v306000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.6.0]` セクション
- `benchmarks/v30.6.0.json` 存在
- `versions/current.md` を v30.6.0 に更新
- `tasks.md` が COMPLETE

> **ロードマップとのカウント差**: ロードマップは「Rust テスト 1 件」と記載しているが、spec では 3 件追加する（version / 実装確認 / benchmark の 3 点が最小セット）。
